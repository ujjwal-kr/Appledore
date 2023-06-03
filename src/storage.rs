use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use crate::encoder::*;

#[derive(Clone, Debug)]
enum Value {
    String(Vec<u8>),
    Vector(Vec<String>),
    Hash(HashMap<String, Vec<u8>>),
}

#[derive(Clone, Debug)]
struct Unit {
    expireat: Option<Instant>,
    value: Value,
}

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    BadType,
    BadCommand,
}

pub enum PopReply {
    String(String),
    Usize(usize),
}

#[derive(Clone)]
pub struct Storage(HashMap<String, Unit>);

impl Storage {
    pub fn new() -> Self {
        Storage(HashMap::new())
    }

    pub fn set_string(&mut self, key: String, value: String) {
        self.0.insert(
            key,
            Unit {
                expireat: None,
                value: Value::String(encode_resp_bulk_string(value)),
            },
        );
    }

    pub fn set_string_ex(&mut self, key: String, value: String, time: u64) {
        let total_time = Instant::now() + Duration::from_millis(time);
        self.0.insert(
            key,
            Unit {
                expireat: Some(total_time),
                value: Value::String(encode_resp_bulk_string(value)),
            },
        );
    }

    pub fn get_string(&mut self, key: &str) -> Result<Vec<u8>, StorageError> {
        match self.0.get(key) {
            Some(s) => match s.expireat {
                Some(v) => {
                    if v < Instant::now() {
                        self.0.remove(key);
                        return Err(StorageError::NotFound);
                    } else {
                        match &s.value {
                            Value::String(v) => Ok(v.clone()),
                            _ => Err(StorageError::BadType),
                        }
                    }
                }
                None => match &s.value {
                    Value::String(v) => Ok(v.clone()),
                    _ => Err(StorageError::BadType),
                },
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn delete(&mut self, keys: Vec<String>) -> usize {
        let mut len = 0;
        for key in keys {
            match self.0.remove(&key) {
                Some(_) => len += 1,
                None => (),
            }
        }
        len
    }

    pub fn set_array(
        &mut self,
        key: String,
        arr: Vec<String>,
        cmd: &str,
    ) -> Result<usize, StorageError> {
        match self.get_array(&key, [0, 0].to_vec()) {
            Ok(_) => match self.0.get(&key) {
                None => Err(StorageError::NotFound),
                Some(v) => match &v.value {
                    Value::Vector(vec) => {
                        let mut temp_vec = vec.clone();
                        if cmd == "rpush" {
                            temp_vec.extend(arr)
                        } else {
                            temp_vec.splice(0..0, arr);
                        }
                        self.0.insert(
                            key,
                            Unit {
                                expireat: None,
                                value: Value::Vector(temp_vec.clone()),
                            },
                        );
                        return Ok(temp_vec.len());
                    }
                    _ => Err(StorageError::BadType),
                },
            },
            Err(_) => {
                self.0.insert(
                    key,
                    Unit {
                        expireat: None,
                        value: Value::Vector(arr.clone()),
                    },
                );
                Ok(arr.len())
            }
        }
    }

    pub fn get_array(&mut self, key: &str, bound: Vec<usize>) -> Result<Vec<String>, StorageError> {
        match self.0.get(key) {
            Some(s) => match &s.value {
                Value::Vector(v) => Ok(v.clone()[bound[0]..bound[1]].to_vec()),
                _ => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn get_array_len(&mut self, key: &str) -> Result<usize, StorageError> {
        match self.0.get(key) {
            Some(s) => match &s.value {
                Value::Vector(v) => Ok(v.len()),
                _ => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn pop_array(&mut self, cmd: Vec<String>) -> Result<PopReply, StorageError> {
        let key = cmd[1].as_str();
        match self.0.get_mut(key) {
            Some(u) => match &mut u.value {
                Value::Vector(v) => {
                    if cmd.len() > 2 {
                        let mut total = 0usize;
                        for item in cmd.iter().skip(2) {
                            let i = v.iter().position(|x| x == item);
                            if let Some(idx) = i {
                                v.remove(idx);
                                total += 1;
                            }
                        }
                        return Ok(PopReply::Usize(total));
                    } else {
                        return Ok(PopReply::String(v.pop().unwrap()));
                    }
                }
                _ => return Err(StorageError::BadType),
            },
            _ => return Err(StorageError::NotFound),
        };
    }

    pub fn array_get(&mut self, key: &str, mut index: i32) -> Result<String, StorageError> {
        match self.0.get(key) {
            Some(u) => match &u.value {
                Value::Vector(v) => {
                    if index < 0 {
                        index = v.len() as i32 - (index * -1);
                    }
                    if v.len() < 1 || index >= v.len() as i32 || index < 0 {
                        return Err(StorageError::NotFound);
                    }
                    return Ok(v[index as usize].clone());
                }
                _ => Err(StorageError::BadType),
            },
            None => Err(StorageError::NotFound),
        }
    }

    pub fn hash_set(&mut self, cmd: Vec<String>) -> Result<usize, StorageError> {
        if cmd.len() % 2 != 0 {
            return Err(StorageError::BadCommand);
        }
        let key = &cmd[1];
        match self.0.get_mut(key) {
            Some(u) => match &mut u.value {
                Value::Hash(map) => {
                    let mut i = 0usize;
                    for item in cmd.chunks(2).skip(1) {
                        i += 1;
                        map.insert(
                            item[0].to_owned(),
                            encode_resp_bulk_string(item[1].to_owned()),
                        );
                    }
                    return Ok(i);
                }
                _ => return Err(StorageError::BadType),
            },
            _ => {
                let mut i = 0usize;
                let mut map: HashMap<String, Vec<u8>> = HashMap::new();
                for item in cmd.chunks(2).skip(1) {
                    i += 1;
                    map.insert(
                        item[0].to_owned(),
                        encode_resp_bulk_string(item[1].to_owned()),
                    );
                }
                self.0.insert(
                    key.to_owned(),
                    Unit {
                        expireat: None,
                        value: Value::Hash(map),
                    },
                );
                return Ok(i);
            }
        };
    }
}

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

#[derive(Clone, Debug)]
enum Value {
    String(String),
    Vector(Vec<String>),
}

#[derive(Clone, Debug)]
struct Unit {
    expireat: u128, // add a bool if expirable to avoid comparison
    value: Value,
}

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    BadType,
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
                expireat: u128::MAX,
                value: Value::String(value),
            },
        );
    }

    pub fn set_string_ex(&mut self, key: String, value: String, time: u64) {
        let total_time: Duration = Duration::from_millis(time) + Instant::now().elapsed();
        self.0.insert(
            key,
            Unit {
                expireat: total_time.as_millis(),
                value: Value::String(value),
            },
        );
    }

    pub fn get_string(&mut self, key: &str) -> Result<String, StorageError> {
        match self.0.get(key) {
            Some(s) => {
                if s.expireat >= Instant::now().elapsed().as_millis() {
                    self.delete(key.to_string()).unwrap();
                    return Err(StorageError::NotFound)
                } else {
                    match &s.value {
                        Value::String(v) => Ok(v.clone()),
                        Value::Vector(_) => Err(StorageError::BadType),
                    }
                }
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn delete(&mut self, key: String) -> Result<(), StorageError> {
        match self.0.remove(&key) {
            Some(_) => Ok(()),
            None => Err(StorageError::NotFound)
        }
    }

    pub fn set_array(
        &mut self,
        key: String,
        arr: Vec<String>,
        cmd: &str,
    ) -> Result<usize, StorageError> {
        match self.get_array(&key, [0, arr.len()].to_vec()) {
            Ok(_) => match self.0.get_mut(&key) {
                Some(v) => match v.value.clone() {
                    Value::Vector(mut value) => {
                        if cmd == "lpush" {
                            value.splice(0..0, arr);
                            Ok(value.len())
                        } else if cmd == "rpush" {
                            value.extend(arr);
                            Ok(value.len())
                        } else {
                            panic!("nani?")
                        }
                    }
                    Value::String(_) => return Err(StorageError::BadType),
                },
                None => {
                    panic!("nani?")
                }
            },
            Err(_) => {
                self.0.insert(
                    key,
                    Unit {
                        expireat: u128::MAX,
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
                Value::String(_) => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn get_array_len(&mut self, key: &str) -> Result<usize, StorageError> {
        match self.0.get(key) {
            Some(s) => match &s.value {
                Value::Vector(v) => Ok(v.len()),
                Value::String(_) => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }
}

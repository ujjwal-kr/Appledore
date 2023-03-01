use std::collections::HashMap;

#[derive(Clone, Debug)]
enum Unit {
    String(String),
    Vector(Vec<String>),
}

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
        self.0.insert(key, Unit::String(value));
    }

    pub fn get_string(&mut self, key: &str) -> Result<String, StorageError> {
        match self.0.get(key) {
            Some(s) => match s {
                Unit::String(v) => Ok(v.clone()),
                Unit::Vector(_) => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
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
                Some(v) => match v {
                    Unit::Vector(value) => {
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
                    Unit::String(_) => return Err(StorageError::BadType),
                },
                None => {
                    panic!("nani?")
                }
            },
            Err(_) => {
                self.0.insert(key, Unit::Vector(arr.clone()));
                Ok(arr.len())
            }
        }
    }

    pub fn get_array(&mut self, key: &str, bound: Vec<usize>) -> Result<Vec<String>, StorageError> {
        match self.0.get(key) {
            Some(s) => match s {
                Unit::Vector(v) => Ok(v.clone()[bound[0]..bound[1]].to_vec()),
                Unit::String(_) => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn get_array_len(&mut self, key: &str) -> Result<usize, StorageError> {
        match self.0.get(key) {
            Some(s) => match s {
                Unit::Vector(v) => Ok(v.len()),
                Unit::String(_) => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }
}

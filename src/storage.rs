use std::collections::HashMap;

#[derive(Clone)]
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

    pub fn set_array(&mut self, key: String, arr: Vec<String>) {
        self.0.insert(key, Unit::Vector(arr));
    }

    pub fn get_array(&mut self, key: &str) -> Result<Vec<String>, StorageError> {
        match self.0.get(key) {
            Some(s) => match s {
                Unit::Vector(v) => Ok(v.clone()),
                Unit::String(_)=> Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }
}

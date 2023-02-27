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

    pub fn get_string(&self, key: &str) -> Result<String, StorageError> {
        match self.0.get(key) {
            Some(s) => match s {
                Unit::String(v) => Ok(v.clone()),
                Unit::Vector(_) => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }
}

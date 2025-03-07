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
    Queue(Queue),
}

#[derive(Clone, Debug)]
struct Unit {
    expireat: Option<Instant>,
    value: Value,
}

#[derive(Clone, Debug)]
struct Queue {
    queue: Vec<String>
}

#[derive(Debug)]
pub enum StorageError {
    NotFound,
    BadType,
    BadCommand,
    OutOfRange,
}

pub enum PopReply {
    String(String),
    Vector(Vec<String>),
}

#[derive(Clone)]
pub struct Storage(HashMap<String, Unit>);

impl Queue {
    fn new() -> Self {
        Queue {
            queue: Vec::new(),
        }
    }

    fn append(&mut self, items: Vec<String>) {
        self.queue.extend(items);
    }

    fn dequeue(&mut self) -> Option<String> {
        if self.queue.is_empty() {
            None
        } else {
            Some(self.queue.remove(0))
        }
    }

    fn size(&mut self) -> usize {
        self.queue.len()
    }
}

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

    pub fn set_string_px(&mut self, key: String, value: String, time: u64) {
        let total_time = Instant::now() + Duration::from_millis(time);
        self.0.insert(
            key,
            Unit {
                expireat: Some(total_time),
                value: Value::String(encode_resp_bulk_string(value)),
            },
        );
    }

    pub fn set_string_ex(&mut self, key: String, value: String, time: u64) {
        let total_time = Instant::now() + Duration::from_secs(time);
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
                        Err(StorageError::NotFound)
                    } else {
                        match &s.value {
                            Value::String(v) => Ok(v.to_vec()),
                            _ => Err(StorageError::BadType),
                        }
                    }
                }
                None => match &s.value {
                    Value::String(v) => Ok(v.to_vec()),
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
            Ok(_) => match self.0.get_mut(&key) {
                None => Err(StorageError::NotFound),
                Some(v) => match &mut v.value {
                    Value::Vector(vec) => {
                        if cmd == "rpush" {
                            vec.extend(arr)
                        } else {
                            vec.splice(0..0, arr);
                        }
                        Ok(vec.len())
                    }
                    _ => Err(StorageError::BadType),
                },
            },
            Err(_) => {
                let len = arr.len();
                self.0.insert(
                    key,
                    Unit {
                        expireat: None,
                        value: Value::Vector(arr),
                    },
                );
                Ok(len)
            }
        }
    }

    pub fn get_array(&mut self, key: &str, bound: Vec<usize>) -> Result<Vec<String>, StorageError> {
        match self.0.get(key) {
            Some(s) => match &s.value {
                Value::Vector(v) => {
                    if bound[1] < bound[0] {
                        return Err(StorageError::BadCommand);
                    }
                    let length = bound[1] - bound[0];
                    if length > v.len() {
                        return Err(StorageError::BadCommand);
                    }
                    Ok(v[bound[0]..bound[1]].to_vec())
                }
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
                    if cmd.len() == 2 {
                        return Ok(PopReply::String(v.pop().unwrap()));
                    }
                    match cmd[2].parse::<u64>() {
                        Ok(mut n) => {
                            let mut final_vec: Vec<String> = vec![];
                            if n > v.len() as u64 {
                                n = v.len() as u64;
                            }
                            for _ in 0..n {
                                final_vec.push(v.pop().unwrap())
                            }
                            Ok(PopReply::Vector(final_vec))
                        }
                        Err(_) => Err(StorageError::BadCommand),
                    }
                }
                _ => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn remove_array(
        &mut self,
        key: &str,
        mut count: i64,
        element: String,
    ) -> Result<i64, StorageError> {
        match self.0.get_mut(key) {
            Some(u) => match &mut u.value {
                Value::Vector(v) => {
                    let mut idxs: Vec<usize> = vec![];
                    if v.is_empty() {
                        return Ok(0);
                    }
                    if count < 0 {
                        count = -count;
                        let mut idx = v.len().checked_sub(1);
                        while let Some(i) = idx {
                            if idxs.len() as i64 != count {
                                if v[i] == element {
                                    idxs.push(i);
                                }
                            } else {
                                break;
                            }
                            idx = i.checked_sub(1)
                        }
                    } else {
                        for (i, item) in v.iter().enumerate() {
                            if idxs.len() as i64 != count {
                                if item == &element {
                                    idxs.push(i);
                                }
                            } else {
                                break;
                            }
                        }
                    }
                    for i in &idxs {
                        v.remove(*i);
                    }
                    Ok(idxs.len() as i64)
                }
                _ => Err(StorageError::BadCommand),
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn array_get(&mut self, key: &str, mut index: i64) -> Result<String, StorageError> {
        match self.0.get(key) {
            Some(u) => match &u.value {
                Value::Vector(v) => {
                    if index < 0 {
                        index = v.len() as i64 - -index;
                    }
                    if v.is_empty() || index >= v.len() as i64 || index < 0 {
                        return Err(StorageError::NotFound);
                    }
                    Ok(v[index as usize].to_owned())
                }
                _ => Err(StorageError::BadType),
            },
            None => Err(StorageError::NotFound),
        }
    }

    pub fn array_set(
        &mut self,
        key: &str,
        mut index: i64,
        element: String,
    ) -> Result<(), StorageError> {
        match self.0.get_mut(key) {
            Some(u) => match &mut u.value {
                Value::Vector(v) => {
                    if index < 0 {
                        index = v.len() as i64 - -index
                    }
                    if index >= v.len() as i64 {
                        return Err(StorageError::OutOfRange);
                    }
                    v[index as usize] = element;
                    Ok(())
                }
                _ => Err(StorageError::BadType),
            },
            _ => Err(StorageError::NotFound),
        }
    }

    pub fn queue_add(&mut self, cmd: Vec<String>) -> Result<(), StorageError> {
        if cmd.len() < 3 {
            return Err(StorageError::BadCommand);
        }
        let key = &cmd[1];
        match self.0.get_mut(key) {
            Some(u) => match &mut u.value {
                Value::Queue(q) => {
                    let items: Vec<_> = cmd[2..].to_vec().into();
                    q.append(items);
                    return Ok(());
                }
                _ => Err(StorageError::BadType),
            },
            _ => {
                let mut new_queue: Queue = Queue::new();
                let items: Vec<_> = cmd[2..].to_owned().into();
                new_queue.append(items);
                self.0.insert(
                    cmd[1].to_owned(),
                    Unit {
                        expireat: None,
                        value: Value::Queue(new_queue),
                    },
                );
                return Ok(());
            }
        }
    }

    pub fn dequeue(&mut self, cmd: Vec<String>) -> Result<String, StorageError> {
        if cmd.len() < 2 {
            return Err(StorageError::BadCommand);
        }
        match self.0.get_mut(&cmd[1]) {
            Some(u) => match &mut u.value {
                Value::Queue(q) => {
                    if q.size() == 0 {
                        self.0.remove(&cmd[1]);
                        return Err(StorageError::OutOfRange);
                    }
                    return Ok(q.dequeue().unwrap());
                }
                _ => Err(StorageError::BadType),
            },
            None => Err(StorageError::NotFound),
        }
    }

    pub fn qlen(&mut self, cmd: Vec<String>) -> Result<usize, StorageError> {
        if cmd.len() < 2 {
            return Err(StorageError::BadCommand);
        }
        match self.0.get(&cmd[1]) {
            Some(u) => match &u.value {
                Value::Queue(q) => Ok(q.clone().size()),
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
                    Ok(i)
                }
                _ => Err(StorageError::BadType),
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
                Ok(i)
            }
        }
    }
}

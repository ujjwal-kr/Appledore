use std::sync::{Arc, Mutex};

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    decoder::*,
    encoder::*,
    storage::{Storage, StorageError},
};

pub async fn push(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    if pure_cmd.len() < 3 {
        stream
            .write(&encode_resp_error_string(
                format!("Invalid args for {}", pure_cmd[0]).trim(),
            ))
            .await
            .unwrap();
    } else {
        let items = pure_cmd[2..pure_cmd.len()].to_vec();
        let clock = client_store.lock().unwrap().set_array(
            pure_cmd[1].clone(),
            items.clone(),
            pure_cmd[0].trim(),
        );
        match clock {
            Ok(len) => {
                let str_len = len.to_string();
                stream
                    .write(&encode_resp_integer(str_len.trim()))
                    .await
                    .unwrap();
            }
            Err(_) => {
                stream
                    .write(&encode_resp_error_string(
                        "WRONGTYPE Operation against a key holding the wrong kind of value",
                    ))
                    .await
                    .unwrap();
            }
        }
    }
}

pub async fn lrange(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    if pure_cmd.len() < 4 {
        stream
            .write(&encode_resp_error_string("Invalid args for lrange"))
            .await
            .unwrap();
    } else {
        let key = pure_cmd[1].clone();
        let len_clock = client_store.lock().unwrap().get_array_len(&key);
        let mut len: usize = 0;
        match len_clock {
            Ok(v) => len = v,
            Err(e) => match e {
                StorageError::BadType => {
                    stream
                        .write(&encode_resp_error_string(
                            "WRONGTYPE Operation against a key holding the wrong kind of value",
                        ))
                        .await
                        .unwrap();
                }
                _ => {
                    stream.write(&encode_resp_empty_array()).await.unwrap();
                }
            },
        }
        if len > 0 {
            match decode_array_indices(pure_cmd[2].trim(), pure_cmd[3].trim(), len) {
                Ok(bound) => {
                    let array_clock = client_store.lock().unwrap().get_array(&key, bound);
                    match array_clock {
                        Ok(array) => {
                            stream.write(&encode_resp_arrays(array)).await.unwrap();
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {
                    stream
                        .write(&encode_resp_error_string("Invalid range"))
                        .await
                        .unwrap();
                }
            }
        }
    }
}

pub async fn llen(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    if pure_cmd.len() != 2 {
        stream
            .write(&encode_resp_error_string("Invalid args for 'llen'"))
            .await
            .unwrap();
        return;
    }
    let key = pure_cmd[1].as_str();
    let clock = client_store.lock().unwrap().get_array_len(key);
    match clock {
        Ok(len) => {
            stream
                .write(&encode_resp_integer(len.to_string().as_str()))
                .await
                .unwrap();
        }
        Err(_) => {
            stream.write(&encode_resp_integer("0")).await.unwrap();
        }
    }
}

pub async fn lpop(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    if pure_cmd.len() < 2 {
        stream
            .write(&encode_resp_error_string("Invalid args for lpop"))
            .await
            .unwrap();
        return;
    }
    let clock = client_store.lock().unwrap().pop_array(pure_cmd);
    match clock {
        Ok(reply) => match reply {
            crate::storage::PopReply::String(s) => {
                stream.write(&encode_resp_bulk_string(s)).await.unwrap();
            }
            crate::storage::PopReply::Vector(v) => {
                stream.write(&encode_resp_arrays(v)).await.unwrap();
            }
        },
        Err(e) => match e {
            StorageError::BadType => {
                stream
                    .write(&encode_resp_error_string(
                        "WRONGTYPE operation against a key holding the wrong kind of value",
                    ))
                    .await
                    .unwrap();
            }
            _ => {
                stream.write(&empty_bulk_string()).await.unwrap();
            }
        },
    };
}

pub async fn lindex(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    if pure_cmd.len() != 3 {
        stream
            .write(&encode_resp_error_string("Invalid arguments for linex"))
            .await
            .unwrap();
        return;
    }

    let index: i32 = match pure_cmd[2].parse::<i32>() {
        Ok(i) => i,
        _ => {
            stream
                .write(&encode_resp_error_string("Invalid arguments for linex"))
                .await
                .unwrap();
            return;
        }
    };
    let clock = client_store
        .lock()
        .unwrap()
        .array_get(pure_cmd[1].trim(), index);
    match clock {
        Ok(s) => {
            stream.write(&encode_resp_bulk_string(s)).await.unwrap();
        }
        Err(e) => match e {
            StorageError::BadType => {
                stream
                    .write(&encode_resp_error_string(
                        "WRONGTYPE operation against a key holding the wrong kind of value",
                    ))
                    .await
                    .unwrap();
            }
            _ => {
                stream.write(&empty_bulk_string()).await.unwrap();
            }
        },
    }
}

pub async fn lrem(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    if pure_cmd.len() != 4 {
        stream
            .write(&encode_resp_error_string("Invalid arguments for 'lrem'"))
            .await
            .unwrap();
    }
    if let Ok(n) = pure_cmd[2].parse::<i32>() {
        let clock =
            client_store
                .lock()
                .unwrap()
                .remove_array(pure_cmd[1].as_str(), n, pure_cmd[3].clone());
        match clock {
            Ok(count) => {
                stream
                    .write(&encode_resp_integer(count.to_string().as_str()))
                    .await
                    .unwrap();
            }
            Err(e) => {
                match e {
                    StorageError::BadType => {
                        stream.write(&encode_resp_error_string("WRONGTYPE operation against the key holding the wrong kind of value")).await.unwrap();
                    }
                    _ => {
                        stream.write(&empty_bulk_string()).await.unwrap();
                    }
                }
            }
        }
    } else {
        stream
            .write(&encode_resp_error_string("Invalid arguments for 'lrem'"))
            .await
            .unwrap();
    }
}

pub async fn lset(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    if pure_cmd.len() < 4 {
        stream
            .write(&encode_resp_error_string("Invalid arguments for 'lset'"))
            .await
            .unwrap();
    }
    if let Ok(n) = pure_cmd[2].parse::<i32>() {
        let clock =
            client_store
                .lock()
                .unwrap()
                .array_set(pure_cmd[1].trim(), n, pure_cmd[3].clone());
        match clock {
            Ok(()) => {
                stream.write(&encode_resp_simple_string("OK")).await.unwrap();
            },
            Err(e) => {
                match e {
                    StorageError::BadType => {
                        stream.write(&encode_resp_error_string("WRONGTYPE operation against the key holding the wrong kind of value")).await.unwrap();
                    }
                    StorageError::OutOfRange => {
                        stream
                            .write(&encode_resp_error_string("index out of range"))
                            .await
                            .unwrap();
                    }
                    _ => {
                        stream.write(&encode_resp_empty_array()).await.unwrap();
                    }
                }
            }
        }
    } else {
        stream
            .write(&encode_resp_error_string("Invalid arguments for 'lset'"))
            .await
            .unwrap();
    }
}

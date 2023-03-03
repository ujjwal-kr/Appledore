use std::{sync::{Arc, Mutex}, num::ParseIntError};

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    decoder::*,
    encoder::*,
    storage::{Storage, StorageError},
};

pub async fn ping(stream: &mut TcpStream) {
    stream
        .write(&encode_resp_simple_string("PONG"))
        .await
        .unwrap();
}

pub async fn echo(stream: &mut TcpStream, pure_cmd: Vec<String>) {
    if pure_cmd.len() < 2 {
        stream
            .write(&encode_resp_error_string("Invalid args for ECHO"))
            .await
            .unwrap();
    } else {
        stream
            .write(&encode_resp_bulk_string(pure_cmd[1].clone()))
            .await
            .unwrap();
    }
}

pub async fn set(stream: &mut TcpStream, pure_cmd: Vec<String>, client_store: Arc<Mutex<Storage>>) {
    if pure_cmd.len() < 3 {
        stream
            .write(&encode_resp_error_string("Invalid args for SET"))
            .await
            .unwrap();
    } else if pure_cmd.len() == 3 {
        let k = pure_cmd[1].clone();
        let v = pure_cmd[2].clone();
        client_store.lock().unwrap().set_string(k, v);
        stream
            .write(&encode_resp_simple_string("OK"))
            .await
            .unwrap();
    } else if pure_cmd.len() == 5 {
        if pure_cmd[3].to_lowercase() == "px" {
            let key = pure_cmd[1].clone();
            let millis: u64;
            match parse_u64(pure_cmd[4].as_str()) {
                Ok(v) => {
                    millis = v;
                    client_store.lock().unwrap().set_string_ex(key, pure_cmd[2].clone(), millis);
                    stream
                    .write(&encode_resp_simple_string("OK"))
                    .await
                    .unwrap();
                },
                _e => {
                    stream
                    .write(&encode_resp_error_string("Invalid args for GET"))
                    .await
                    .unwrap();
                }
            }
        } else {
            stream
            .write(&encode_resp_error_string("Invalid args for GET"))
            .await
            .unwrap();
        }
    }
}

pub async fn get(stream: &mut TcpStream, pure_cmd: Vec<String>, client_store: Arc<Mutex<Storage>>) {
    if pure_cmd.len() < 2 {
        stream
            .write(&encode_resp_error_string("Invalid args for GET"))
            .await
            .unwrap();
    } else {
        let key = pure_cmd[1].clone();
        let clock = client_store.lock().unwrap().get_string(&key);
        match clock {
            Ok(value) => {
                stream.write(&encode_resp_bulk_string(value)).await.unwrap();
            }
            Err(e) => match e {
                StorageError::BadType => {
                    stream
                        .write(&encode_resp_error_string(
                            "WRONGTYPE Operation against a key holding the wrong kind of value",
                        ))
                        .await
                        .unwrap();
                }
                StorageError::NotFound => {
                    stream.write(&empty_bulk_string()).await.unwrap();
                }
            },
        }
    }
}

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
                StorageError::NotFound => {
                    stream.write(&empty_bulk_string()).await.unwrap();
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

pub async fn undefined(stream: &mut TcpStream) {
    stream
        .write(&encode_resp_error_string("Command not recognised"))
        .await
        .unwrap();
}

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


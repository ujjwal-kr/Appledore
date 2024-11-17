use std::sync::{Arc, Mutex};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    encoder::*,
    storage::{Storage, StorageError},
};

pub async fn queue_add(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    let clock = client_store.lock().unwrap().queue_add(pure_cmd);
    match clock {
        Ok(()) => {
            stream
                .write(&encode_resp_simple_string("OK"))
                .await
                .unwrap();
        }
        Err(StorageError::BadCommand) => {
            stream
                .write(&encode_resp_error_string(
                    "wrong number of arguments for 'qadd' command",
                ))
                .await
                .unwrap();
        }
        _ => {
            stream
                .write(&encode_resp_error_string(
                    "WRONGTYPE Operation against a key holding the wrong kind of value",
                ))
                .await
                .unwrap();
        }
    }
}

pub async fn dequeue(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    let clock = client_store.lock().unwrap().dequeue(pure_cmd);
    match clock {
        Ok(s) => {
            stream.write(&encode_resp_bulk_string(s)).await.unwrap();
        }
        Err(StorageError::OutOfRange) => {
            stream.write(&empty_bulk_string()).await.unwrap();
        }
        Err(StorageError::NotFound) => {
            stream.write(&empty_bulk_string()).await.unwrap();
        }
        _ => {
            stream
                .write(&encode_resp_error_string(
                    "WRONGTYPE Operation against a key holding the wrong kind of value",
                ))
                .await
                .unwrap();
        }
    }
}

pub async fn qlen(stream: &mut TcpStream, pure_cmd: Vec<String>, client_store: Arc<Mutex<Storage>>) {
    let clock = client_store.lock().unwrap().qlen(pure_cmd);
    match clock {
        Ok(s) => {
            stream.write(&encode_resp_integer(s.to_string().as_str())).await.unwrap();
        }
        Err(StorageError::NotFound) => {
            stream.write(&encode_resp_integer(0.to_string().as_str())).await.unwrap();
        }
        _ => {
            stream
                .write(&encode_resp_error_string(
                    "WRONGTYPE Operation against a key holding the wrong kind of value",
                ))
                .await
                .unwrap();
        }
    }
}

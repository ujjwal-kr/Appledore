pub mod hash;
pub mod array;

use std::sync::{Arc, Mutex};

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
                    client_store
                        .lock()
                        .unwrap()
                        .set_string_ex(key, pure_cmd[2].clone(), millis);
                    stream
                        .write(&encode_resp_simple_string("OK"))
                        .await
                        .unwrap();
                }
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
                stream.write(&value).await.unwrap();
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
                _ => {
                    stream.write(&empty_bulk_string()).await.unwrap();
                }
            },
        }
    }
}

pub async fn del(stream: &mut TcpStream, pure_cmd: Vec<String>, client_store: Arc<Mutex<Storage>>) {
    if pure_cmd.len() < 2 {
        stream
            .write(&encode_resp_error_string("Invalid args for DEL"))
            .await
            .unwrap();
    } else {
        let keys = pure_cmd[1..pure_cmd.len()].to_vec();
        let len = client_store.lock().unwrap().delete(keys);
        stream
            .write(&encode_resp_integer(len.to_string().as_str()))
            .await
            .unwrap();
    }
}



pub async fn undefined(stream: &mut TcpStream) {
    stream
        .write(&encode_resp_error_string("Command not recognised"))
        .await
        .unwrap();
}

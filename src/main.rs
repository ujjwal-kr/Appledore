use std::sync::{Arc, Mutex};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

mod decoder;
mod encoder;
mod storage;

use decoder::*;
use encoder::*;
use storage::{Storage, StorageError};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    let storage_engine = Arc::new(Mutex::new(Storage::new()));
    println!("Listening on ::6379");
    loop {
        let incoming = listener.accept().await;
        let cloned_storage = Arc::clone(&storage_engine);
        match incoming {
            Ok((mut stream, _)) => {
                println!("New Connection");
                tokio::spawn(async move {
                    handle_connection(&mut stream, cloned_storage).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_connection(stream: &mut TcpStream, client_store: Arc<Mutex<Storage>>) {
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf).await.unwrap();
        // break the loop if no bytes recieved
        if bytes_read == 0 {
            println!("Client closed the connection");
            break;
        }
        let str_cmd = String::from_utf8_lossy(&buf);
        let cmd: Vec<&str> = str_cmd.split("\r\n").collect::<Vec<&str>>();
        if cmd[0].len() != 2 {
            stream
                .write(&encode_resp_error_string("(error) Cannot Process"))
                .await
                .unwrap();
        }
        let cmd_len: usize = cmd[0][1..2].parse::<usize>().unwrap() * 2;
        let pure_cmd = decode_get_pure_command(cmd[0..cmd_len + 1].to_vec());
        match pure_cmd[0].to_ascii_lowercase().trim() {
            "ping" => {
                stream
                    .write(&encode_resp_simple_string("PONG"))
                    .await
                    .unwrap();
            }
            "echo" => {
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
            "set" => {
                if pure_cmd.len() < 3 {
                    stream
                        .write(&encode_resp_error_string("Invalid args for SET"))
                        .await
                        .unwrap();
                }
                let k = pure_cmd[1].clone();
                let v = pure_cmd[2].clone();
                client_store.lock().unwrap().set_string(k, v);
                stream
                    .write(&encode_resp_simple_string("OK"))
                    .await
                    .unwrap();
            }
            "get" => {
                if pure_cmd.len() < 2 {
                    stream
                        .write(&encode_resp_error_string("Invalid args for GET"))
                        .await
                        .unwrap();
                }
                let key = pure_cmd[1].clone();
                let clock = client_store.lock().unwrap().get_string(&key);
                match clock {
                    Ok(value) => {
                        stream.write(&encode_resp_bulk_string(value)).await.unwrap();
                    }
                    Err(e) => match e {
                        StorageError::BadType => {
                            stream.write(&encode_resp_error_string(
                                "WRONGTYPE Operation against a key holding the wrong kind of value",
                            )).await.unwrap();
                        }
                        StorageError::NotFound => {
                            stream.write(&empty_bulk_string()).await.unwrap();
                        }
                    },
                }
            }
            "lpush" | "rpush" => {
                if pure_cmd.len() < 3 {
                    stream
                        .write(&encode_resp_error_string(
                            format!("Invalid args for {}", pure_cmd[0]).trim(),
                        ))
                        .await
                        .unwrap();
                }
                let items = pure_cmd[2..pure_cmd.len()].to_vec();
                let clock = client_store.lock().unwrap().set_array(
                    pure_cmd[1].clone(),
                    items.clone(),
                    pure_cmd[0].trim(),
                );
                match clock {
                    Ok(_) => {
                        stream
                            .write(&encode_resp_integer(items.len().to_string().trim()))
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
            "lrange" => {
                if pure_cmd.len() < 4 {
                    stream
                        .write(&encode_resp_error_string("Invalid args for lrange"))
                        .await
                        .unwrap();
                }
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
            _ => {
                stream
                    .write(&encode_resp_error_string("Command not recognised"))
                    .await
                    .unwrap();
            }
        };
    }
}

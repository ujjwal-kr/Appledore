use std::sync::{Arc, Mutex};

use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    encoder::*,
    storage::{Storage, StorageError},
};

pub async fn hash_set(
    stream: &mut TcpStream,
    pure_cmd: Vec<String>,
    client_store: Arc<Mutex<Storage>>,
) {
    let clock = client_store.lock().unwrap().hash_set(pure_cmd);
    match clock {
        Ok(size) => {
            stream
                .write(&encode_resp_integer(size.to_string().as_str()))
                .await
                .unwrap();
        }
        Err(StorageError::BadType) => {
            stream
                .write(&encode_resp_error_string(
                    "WRONGTYPE Operation against a key holding the wrong kind of value",
                ))
                .await
                .unwrap();
        }
        _ => {
            stream
                .write(&encode_resp_error_string(
                    "wrong number of arguments for 'hset' command",
                ))
                .await
                .unwrap();
        }
    };
}

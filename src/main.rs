use tokio::{net::{TcpListener, TcpStream}, io::{AsyncReadExt, AsyncWriteExt}};

mod encoder;
mod decoder;

use encoder::*;
use decoder::*;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("Listening on ::6379");
    loop {
        let incoming = listener.accept().await;
        match incoming {
            Ok((mut stream, _)) => {
                println!("New Connection");
                tokio::spawn(async move {
                    handle_connection(&mut stream).await;
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

async fn handle_connection(stream: &mut TcpStream) {
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
        
        println!("{:?}", cmd);
        
        if cmd[0].len() != 2 {
           stream.write(&encode_resp_error_string("(error) Cannot Process")).await.unwrap();
        }
        let cmd_len: usize = cmd[0][1..2].parse::<usize>().unwrap() * 2;
        let pure_cmd = decode_get_pure_command(cmd[0..cmd_len + 1].to_vec());
        match pure_cmd[0].to_ascii_lowercase().trim() {
            "ping" => stream.write(&encode_resp_simple_string("PONG")).await.unwrap(),
            _ => stream.write(&encode_resp_error_string("Command not recognised")).await.unwrap(),
        };
    }
}

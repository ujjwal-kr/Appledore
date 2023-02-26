use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

mod encoder;
use encoder::*;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Listening on ::6379");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                handle_connection(s);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf).unwrap();
        // break the loop if no bytes recieved
        if bytes_read == 0 {
            println!("Client closed the connection");
            break;
        }
        stream.write(&encode_resp_simple_string("PONG")).unwrap();
    }
}

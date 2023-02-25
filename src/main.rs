use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn encode_resp_simple_string(s: &str) -> Vec<u8> {
    let mut encoded: Vec<u8> = vec![];
    encoded.push(b'+');
    encoded.extend(s.as_bytes());
    encoded.extend(&[b'\r', b'\n']);
    encoded
}

fn main() {
    println!("Logs from your program will appear here!");
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                handle_connection(s)
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
        let mut buf = [0; 512];
        stream.read(&mut buf).unwrap();
        println!("{:?}", buf);
        stream.write(&encode_resp_simple_string("PONG")).unwrap();   
}
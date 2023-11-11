use std::net::TcpListener;
use std::io::{Read, Write};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    println!("Listening for connections on 127.0.0.1:4221");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut buffer: [u8; 128] = [0; 128];
                let read_size = stream.read(&mut buffer).unwrap();

                println!("read {} bytes", read_size);

                write!(&mut stream, "HTTP/1.1 200 OK\r\n\r\n").unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

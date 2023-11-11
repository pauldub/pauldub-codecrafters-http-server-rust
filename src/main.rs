use std::net::TcpListener;
use std::io::{Read, Write};

pub mod http {
    use nom::{IResult, bytes::complete::{tag, take_till1}, character::complete::{space1, line_ending}};

    #[derive(Debug)]
    pub struct Request {
        pub method: String,
        pub path: String,
        pub http_version: String,
    }
    
    
    pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Request> {
        let (input, method) = tag("GET")(bytes)?;
        let (input, _) = space1(input)?;
        let (input, path) = take_till1(|c| c == b' ')(input)?;
        let (input, _) = space1(input)?;
        let (input, http_version) = tag("HTTP/1.1")(input)?;
        let (input, _) = line_ending(input)?;

        let request = Request{
            method: String::from_utf8_lossy(method).to_string(),
            path: String::from_utf8_lossy(path).to_string(),
            http_version: String::from_utf8_lossy(http_version).to_string(),
        };
        
        Ok((input, request))
    }
    
}

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
                
                let (_, request) = http::from_bytes(&buffer).unwrap();
                println!("request: {:?}", request);
                
                if request.path == "/" {
                    write!(&mut stream, "HTTP/1.1 200 OK\r\n\r\n").unwrap();
                } else {
                    write!(&mut stream, "HTTP/1.1 404 Not Found\r\n\r\n").unwrap();
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

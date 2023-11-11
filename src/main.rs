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
    
    impl Request {
        pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
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
    
    pub struct Header {
        pub name: String,
        pub value: String,
    }
    
    impl Header { 
        pub fn from_bytes(bytes: &[u8]) -> IResult<&[u8], Self> {
            let (input, name) = take_till1(|c| c == b':')(bytes)?;
            let (input, _) = tag(": ")(input)?;
            let (input, value) = take_till1(|c| c == b'\r')(input)?;
            let (input, _) = line_ending(input)?;
    
            let header = Header{
                name: String::from_utf8_lossy(name).to_string(),
                value: String::from_utf8_lossy(value).to_string(),
            };
            
            Ok((input, header))
        }
        
        pub fn parse_all(bytes: &[u8]) -> IResult<&[u8], Vec<Self>> {
            let mut headers = Vec::new();
            let mut input = bytes;
    
            loop {
                let (leftover, header) = Header::from_bytes(input)?;
                input = leftover;
                headers.push(header);
    
                // Stop when there are two lines separating the headers from the body
                if input.len() >= 4 && &input[0..4] == b"\r\n\r\n" {
                    input = &input[4..];
                    break;
                }
            }
    
            Ok((input, headers))
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    println!("Listening for connections on 127.0.0.1:4221");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let mut buffer = [0 as u8; 1024];
                let read_size = stream.read(&mut buffer).unwrap();

                println!("read {} bytes", read_size);
                
                let (input, request) = http::Request::from_bytes(&buffer).unwrap();
                println!("request: {:?}", request);
                
                if request.path.starts_with("/echo/") {
                    match request.path.split_once("/echo/") {
                        Some((_, message)) => {
                            write!(&mut stream, "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", message.len(), message).unwrap();
                        },
                        None => {
                            write!(&mut stream, "HTTP/1.1 400 Bad Request\r\n\r\n").unwrap();
                        }
                    }
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

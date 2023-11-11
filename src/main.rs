use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub mod http {
    use nom::{
        bytes::complete::{tag, take_till1},
        character::complete::{line_ending, space1},
        IResult,
    };

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

            let request = Request {
                method: String::from_utf8_lossy(method).to_string(),
                path: String::from_utf8_lossy(path).to_string(),
                http_version: String::from_utf8_lossy(http_version).to_string(),
            };

            Ok((input, request))
        }
    }

    #[derive(Debug)]
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

            let header = Header {
                name: String::from_utf8_lossy(name).to_string(),
                value: String::from_utf8_lossy(value).to_string(),
            };

            Ok((input, header))
        }

        pub fn parse_all(bytes: &[u8]) -> IResult<&[u8], Vec<Self>> {
            let mut headers = Vec::new();
            let mut input = bytes;

            loop {
                // Stop when there are two lines separating the headers from the body
                if input.len() >= 4 && &input[0..4] == b"\r\n\r\n" {
                    input = &input[4..];
                    break;
                }

                if input.len() == 0 {
                    break;
                }

                if input.len() == 2 && &input[0..2] == b"\r\n" {
                    break;
                }

                let (leftover, header) = Header::from_bytes(input)?;
                input = leftover;
                headers.push(header);
            }

            Ok((input, headers))
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    println!("Listening for connections on 127.0.0.1:4221");

    loop {
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        // connection was closed
                        break;
                    }
                    Ok(n) => {
                        println!("read {} bytes", n);

                        let raw_request = &buf[0..n];
                        let (input, request) = http::Request::from_bytes(raw_request).unwrap();
                        println!("request: {:?}", request);

                        let (_input, headers) = http::Header::parse_all(input).unwrap();
                        println!("request headers: {:?}", headers);

                        if request.path == "/" {
                            socket.write(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();
                        } else if request.path.starts_with("/echo/") {
                            match request.path.split_once("/echo/") {
                                Some((_, message)) => {
                                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", message.len(), message);
                                    socket.write_all(response.as_bytes()).await.unwrap();
                                }
                                None => {
                                    socket.write(b"HTTP/1.1 400 Bad Request\r\n\r\n").await.unwrap();
                                }
                            }
                        } else if request.path == "/user-agent" {
                            match headers.iter().find(|h| h.name == "User-Agent") {
                                Some(header) => {
                                    let response = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", header.value.len(), header.value);
                                    socket.write_all(response.as_bytes()).await.unwrap();
                                }
                                None => {
                                    socket.write(b"HTTP/1.1 400 Bad Request\r\n\r\n").await.unwrap();
                                }
                            }
                        } else {
                            socket.write(b"HTTP/1.1 404 Not Found\r\n\r\n").await.unwrap();
                        }
                        break;
                    }
                    Err(_) => {
                        // an error occurred, we can handle it here or just break
                        break;
                    }
                }
            }
        });
    }
}

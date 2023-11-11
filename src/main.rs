pub mod http;
pub mod handlers;

use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    // Read the --directory <directory> argument
    let args: Vec<String> = std::env::args().collect();
    let root_directory = match args.get(2) {
        Some(dir) => dir.to_owned(),
        None => ".".to_owned(),
    };
    let directory = Arc::new(root_directory);
    
    let listener = TcpListener::bind("127.0.0.1:4221").await.unwrap();

    println!("Listening for connections on 127.0.0.1:4221");

    loop {
        let directory = directory.clone();
        let (mut socket, _) = listener.accept().await.unwrap();

        tokio::spawn(async move {
            let directory = directory.clone();
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

                        let (input, headers) = http::Header::parse_all(input).unwrap();
                        println!("request headers: {:?}", headers);

                        if request.path == "/" {
                            socket.write(b"HTTP/1.1 200 OK\r\n\r\n").await.unwrap();
                        } else if request.path.starts_with("/echo/") {
                            handlers::echo(&mut socket, &request, &headers).await.unwrap();
                        } else if request.path == "/user-agent" {
                            handlers::user_agent(&mut socket, &request, &headers).await.unwrap();
                        } else if request.path.starts_with("/files/") {
                            handlers::files(&mut socket, &request, &headers, &input, &directory).await.unwrap();
                        } else {
                            socket
                                .write(b"HTTP/1.1 404 Not Found\r\n\r\n")
                                .await
                                .unwrap();
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

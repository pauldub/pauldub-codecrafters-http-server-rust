use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::http;

pub async fn echo(
    socket: &mut TcpStream,
    request: &http::Request,
    _headers: &Vec<http::Header>,
) -> Result<()> {
    match request.path.split_once("/echo/") {
        Some((_, message)) => {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            socket.write_all(response.as_bytes()).await?;
        }
        None => {
            socket.write(b"HTTP/1.1 400 Bad Request\r\n\r\n").await?;
        }
    }

    Ok(())
}

pub async fn user_agent(
    socket: &mut TcpStream,
    _request: &http::Request,
    headers: &Vec<http::Header>,
) -> Result<()> {
    match headers.iter().find(|h| h.name == "User-Agent") {
        Some(header) => {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                header.value.len(),
                header.value
            );
            socket.write_all(response.as_bytes()).await?;
        }
        None => {
            socket.write(b"HTTP/1.1 400 Bad Request\r\n\r\n").await?;
        }
    }

    Ok(())
}

pub async fn files(
    socket: &mut TcpStream,
    request: &http::Request,
    _headers: &Vec<http::Header>,
    directory: &str,
) -> Result<()> {
    match request.path.split_once("/files/") {
        Some((_, path)) => {
            let path = format!("{}/{}", directory, path);
            let contents = tokio::fs::read(path).await.unwrap();

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
                contents.len(),
            );
            socket.write_all(response.as_bytes()).await?;
            socket.write_all(&contents).await?;
        }
        None => {
            socket.write(b"HTTP/1.1 400 Bad Request\r\n\r\n").await?;
        }
    }

    Ok(())
}

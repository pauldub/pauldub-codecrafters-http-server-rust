use anyhow::Result;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::http;

pub async fn echo(
    socket: &mut TcpStream,
    request: &http::Request,
    _headers: &[http::Header],
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
            socket
                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                .await?;
        }
    }

    Ok(())
}

pub async fn user_agent(
    socket: &mut TcpStream,
    _request: &http::Request,
    headers: &[http::Header],
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
            socket
                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                .await?;
        }
    }

    Ok(())
}

async fn read_file(
    socket: &mut TcpStream,
    _request: &http::Request,
    _headers: &[http::Header],
    directory: &str,
    path: &str,
) -> Result<()> {
    let path = format!("{}/{}", directory, path);
    match tokio::fs::try_exists(path.clone()).await {
        Ok(true) => {}
        _ => {
            socket.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n").await?;
            return Ok(());
        }
    }

    let contents = tokio::fs::read(path).await.unwrap();

    let response = format!(
        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n",
        contents.len(),
    );
    socket.write_all(response.as_bytes()).await?;
    socket.write_all(&contents).await?;

    Ok(())
}

pub async fn write_file(
    socket: &mut TcpStream,
    _request: &http::Request,
    headers: &[http::Header],
    input: &[u8],
    directory: &str,
    path: &str,
) -> Result<()> {
    let path = format!("{}/{}", directory, path);
    let content_length = headers.iter().find(|h| h.name == "Content-Length").unwrap();
    let request_body = input[0..content_length.value.parse::<usize>().unwrap()].to_vec();

    tokio::fs::write(path, request_body).await.unwrap();

    socket.write_all(b"HTTP/1.1 201 Created\r\n\r\n").await?;

    Ok(())
}

pub async fn files(
    socket: &mut TcpStream,
    request: &http::Request,
    headers: &[http::Header],
    input: &[u8],
    directory: &str,
) -> Result<()> {
    match request.path.split_once("/files/") {
        Some((_, path)) => {
            if request.method == "GET" {
                read_file(socket, request, headers, directory, path).await?
            } else if request.method == "POST" {
                write_file(socket, request, headers, input, directory, path).await?
            } else {
                socket
                    .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                    .await?;
            }
        }
        None => {
            socket
                .write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n")
                .await?;
        }
    }

    Ok(())
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Transport connect and JSON-RPC over Unix, TCP, abstract sockets, and HTTP `/jsonrpc`.

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UnixStream};
use tokio::time::timeout;
use tracing::trace;

use super::atomic_rpc::send_jsonrpc_line;

/// Connect to a Linux abstract socket, returning a tokio-ready `UnixStream`.
#[cfg(target_os = "linux")]
pub(crate) fn connect_abstract(name: &str) -> Result<UnixStream> {
    use std::os::linux::net::SocketAddrExt;
    use std::os::unix::net::SocketAddr;

    let addr = SocketAddr::from_abstract_name(name)
        .context(format!("Invalid abstract socket name: {name}"))?;
    let std_stream = std::os::unix::net::UnixStream::connect_addr(&addr)
        .context(format!("Failed to connect to abstract socket: @{name}"))?;
    std_stream.set_nonblocking(true)?;
    Ok(UnixStream::from_std(std_stream)?)
}

pub(crate) async fn jsonrpc_unix(path: &Path, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
    let stream = UnixStream::connect(path).await.context(format!(
        "Failed to connect to Unix socket: {}",
        path.display()
    ))?;

    send_jsonrpc_line(stream, request).await
}

/// Connect to a Unix socket, perform a BTSP client handshake via BearDog
/// delegation, then send a JSON-RPC request over the authenticated channel.
pub(crate) async fn jsonrpc_unix_btsp(
    path: &Path,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let stream = UnixStream::connect(path).await.context(format!(
        "BTSP: failed to connect to Unix socket: {}",
        path.display()
    ))?;

    let reader = crate::btsp_client::perform_client_handshake(stream)
        .await
        .map_err(|e| anyhow::anyhow!("BTSP client handshake failed for {}: {e}", path.display()))?;

    let inner = reader.into_inner();
    send_jsonrpc_line(inner, request).await
}

pub(crate) async fn jsonrpc_tcp(
    host: &str,
    port: u16,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let addr = format!("{host}:{port}");
    let stream = TcpStream::connect(&addr)
        .await
        .context(format!("Failed to connect to TCP: {addr}"))?;

    send_jsonrpc_line(stream, request).await
}

/// Minimal HTTP/1.1 POST to `POST /jsonrpc` over a raw `TcpStream`.
pub(crate) async fn jsonrpc_http(
    host: &str,
    port: u16,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let addr = format!("{host}:{port}");
    let mut stream = TcpStream::connect(&addr)
        .await
        .context(format!("Failed to connect to HTTP endpoint: {addr}"))?;

    let body = serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

    let http_request = format!(
        "POST /jsonrpc HTTP/1.1\r\n\
         Host: {}\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        addr,
        body.len(),
        body
    );

    trace!("Sending HTTP JSON-RPC request to {}", addr);

    stream.write_all(http_request.as_bytes()).await?;
    stream.flush().await?;

    let mut response_buf = Vec::new();
    let mut reader = BufReader::new(stream);
    loop {
        let mut line = String::new();
        match reader.read_line(&mut line).await {
            Ok(0) => break,
            Ok(_) => response_buf.push(line),
            Err(e) => {
                if response_buf.is_empty() {
                    return Err(e).context("Failed to read HTTP response");
                }
                break;
            }
        }
    }

    let full_response = response_buf.join("");
    let body_start = full_response
        .find("\r\n\r\n")
        .or_else(|| full_response.find("\n\n"))
        .map(|pos| {
            if full_response[pos..].starts_with("\r\n\r\n") {
                pos + 4
            } else {
                pos + 2
            }
        })
        .ok_or_else(|| anyhow::anyhow!("Malformed HTTP response: no body separator"))?;

    let json_body = full_response[body_start..].trim();

    trace!("Received HTTP JSON-RPC response: {}", json_body);

    let response: JsonRpcResponse = serde_json::from_str(json_body).context(format!(
        "Failed to parse JSON-RPC response from HTTP body: {}",
        &json_body[..json_body.len().min(200)]
    ))?;

    Ok(response)
}

/// Send JSON-RPC over an abstract socket (Linux/Android).
#[cfg(target_os = "linux")]
pub(crate) async fn jsonrpc_abstract(
    name: &str,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let stream = connect_abstract(name)?;
    send_jsonrpc_line(stream, request).await
}

#[cfg(not(target_os = "linux"))]
pub(crate) async fn jsonrpc_abstract(
    name: &str,
    _request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    anyhow::bail!(
        "Abstract sockets not supported on this platform (only Linux/Android). \
         Socket name: @{}",
        name
    )
}

pub(crate) async fn connect_unix_timed(path: &Path, timeout_dur: Duration) -> Result<UnixStream> {
    let stream = timeout(timeout_dur, UnixStream::connect(path))
        .await
        .context("Unix connect timeout")?
        .context(format!("Unix connect: {}", path.display()))?;
    Ok(stream)
}

pub(crate) async fn connect_tcp_timed(
    host: &str,
    port: u16,
    timeout_dur: Duration,
) -> Result<TcpStream> {
    let addr = format!("{host}:{port}");
    let stream = timeout(timeout_dur, TcpStream::connect(&addr))
        .await
        .context("TCP connect timeout")?
        .context(format!("TCP connect: {addr}"))?;
    Ok(stream)
}

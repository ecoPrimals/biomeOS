// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Transport connect and JSON-RPC over Unix, TCP, abstract sockets, and HTTP `/jsonrpc`.

use anyhow::{Context, Result};
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
#[cfg(unix)]
use tokio::net::UnixStream;
use tokio::time::timeout;
use tracing::trace;

use crate::ipc::connect_transport;
use crate::TransportEndpoint;

use super::atomic_rpc::send_jsonrpc_line;

/// Connect via [`connect_transport`] and send one newline-delimited JSON-RPC exchange.
///
/// Handles `UnixSocket`, `TcpSocket`, and `AbstractSocket` endpoints (including
/// Windows port-file fallback for Unix paths). HTTP framing uses [`jsonrpc_http`].
pub(crate) async fn jsonrpc_via_transport(
    endpoint: &TransportEndpoint,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let stream = connect_transport(endpoint)
        .await
        .with_context(|| format!("Failed to connect to {endpoint}"))?;

    send_jsonrpc_line(stream, request).await
}

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

/// Legacy per-path entry point; prefer [`jsonrpc_via_transport`].
#[expect(dead_code, reason = "retained for callers that pass a path directly")]
pub(crate) async fn jsonrpc_unix(path: &Path, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
    jsonrpc_via_transport(
        &TransportEndpoint::UnixSocket {
            path: path.to_path_buf(),
        },
        request,
    )
    .await
}

/// Connect to a Unix socket, perform a BTSP client handshake + Phase 3 negotiate,
/// then send a JSON-RPC request over the encrypted (or plaintext fallback) channel.
#[cfg(unix)]
pub(crate) async fn jsonrpc_unix_btsp(
    path: &Path,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    let stream = UnixStream::connect(path).await.context(format!(
        "BTSP: failed to connect to Unix socket: {}",
        path.display()
    ))?;

    let outcome = crate::btsp_client_phase3::perform_client_handshake_phase3(stream)
        .await
        .with_context(|| format!("BTSP handshake failed for {}", path.display()))?;

    match outcome {
        crate::btsp_client_phase3::ClientPhase3Outcome::Encrypted { keys, stream } => {
            send_encrypted_jsonrpc(stream, request, &keys).await
        }
        crate::btsp_client_phase3::ClientPhase3Outcome::Plaintext { stream } => {
            send_jsonrpc_line(stream, request).await
        }
    }
}

#[cfg(windows)]
pub(crate) async fn jsonrpc_unix_btsp(
    path: &Path,
    _request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    anyhow::bail!(
        "Unix domain sockets unavailable on Windows: {}",
        path.display()
    )
}

/// Send a JSON-RPC request and read one response using Phase 3 encrypted framing.
///
/// Client encrypts with `client_to_server`, decrypts server reply with `server_to_client`.
#[cfg(unix)]
async fn send_encrypted_jsonrpc(
    stream: UnixStream,
    request: JsonRpcRequest,
    keys: &crate::btsp_crypto::SessionKeys,
) -> Result<JsonRpcResponse> {
    use tokio::io::AsyncReadExt;

    let request_bytes =
        serde_json::to_vec(&request).context("Failed to serialize JSON-RPC request")?;

    let frame = crate::btsp_crypto::encrypt_frame(&keys.client_to_server, &request_bytes)
        .context("encrypt_frame failed")?;

    let (mut read_half, mut write_half) = stream.into_split();

    trace!("Sending encrypted JSON-RPC frame ({} bytes)", frame.len());
    write_half.write_all(&frame).await?;
    write_half.flush().await?;

    let mut len_buf = [0u8; 4];
    let read_result = timeout(
        std::time::Duration::from_secs(30),
        read_half.read_exact(&mut len_buf),
    )
    .await;

    match read_result {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return Err(e).context("Failed to read response frame length"),
        Err(_) => anyhow::bail!("Timeout reading encrypted response"),
    }

    let payload_len = u32::from_be_bytes(len_buf) as usize;
    if payload_len > 16 * 1024 * 1024 {
        anyhow::bail!("Response frame too large: {payload_len} bytes");
    }

    let mut payload = vec![0u8; payload_len];
    read_half
        .read_exact(&mut payload)
        .await
        .context("Failed to read response frame payload")?;

    let plaintext = crate::btsp_crypto::decrypt_frame(&keys.server_to_client, &payload)
        .context("decrypt_frame failed")?;

    trace!(
        "Received encrypted JSON-RPC response ({} bytes plaintext)",
        plaintext.len()
    );

    let response: JsonRpcResponse = serde_json::from_slice(&plaintext)
        .context("Failed to parse decrypted JSON-RPC response")?;

    Ok(response)
}

/// Legacy per-host entry point; prefer [`jsonrpc_via_transport`].
#[expect(dead_code, reason = "retained for callers that pass host/port directly")]
pub(crate) async fn jsonrpc_tcp(
    host: &str,
    port: u16,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    jsonrpc_via_transport(
        &TransportEndpoint::TcpSocket {
            host: Arc::from(host),
            port,
        },
        request,
    )
    .await
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
///
/// Legacy entry point; prefer [`jsonrpc_via_transport`].
#[expect(dead_code, reason = "retained for callers that pass an abstract name directly")]
pub(crate) async fn jsonrpc_abstract(
    name: &str,
    request: JsonRpcRequest,
) -> Result<JsonRpcResponse> {
    jsonrpc_via_transport(
        &TransportEndpoint::AbstractSocket {
            name: Arc::from(name),
        },
        request,
    )
    .await
}

#[cfg(unix)]
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

#[cfg(all(test, unix))]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use crate::btsp_crypto;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    /// Simulate a server that reads an encrypted frame and responds with an encrypted frame.
    async fn encrypted_echo_server(listener: UnixListener, keys: btsp_crypto::SessionKeys) {
        let (stream, _) = listener.accept().await.unwrap();
        let (mut read_half, mut write_half) = stream.into_split();

        let mut len_buf = [0u8; 4];
        read_half.read_exact(&mut len_buf).await.unwrap();
        let payload_len = u32::from_be_bytes(len_buf) as usize;

        let mut payload = vec![0u8; payload_len];
        read_half.read_exact(&mut payload).await.unwrap();

        let plaintext = btsp_crypto::decrypt_frame(&keys.client_to_server, &payload).unwrap();

        let req: serde_json::Value = serde_json::from_slice(&plaintext).unwrap();
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": { "echo": req["params"] },
            "id": req["id"],
        });

        let response_bytes = serde_json::to_vec(&response).unwrap();
        let frame = btsp_crypto::encrypt_frame(&keys.server_to_client, &response_bytes).unwrap();
        write_half.write_all(&frame).await.unwrap();
        write_half.flush().await.unwrap();
    }

    #[tokio::test]
    async fn send_encrypted_jsonrpc_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        let keys = btsp_crypto::derive_session_keys(&[0xAA; 32], &[1u8; 12], &[2u8; 12]);
        let server_keys = keys.clone();

        let server = tokio::spawn(async move {
            encrypted_echo_server(listener, server_keys).await;
        });

        let stream = UnixStream::connect(&sock_path).await.unwrap();
        let request = biomeos_types::JsonRpcRequest::new(
            "health.liveness",
            serde_json::json!({"probe": true}),
        );

        let response = send_encrypted_jsonrpc(stream, request, &keys)
            .await
            .unwrap();

        assert!(response.error.is_none());
        let result = response.result.unwrap();
        assert_eq!(result["echo"]["probe"], true);

        server.await.unwrap();
    }

    #[tokio::test]
    async fn send_encrypted_jsonrpc_wrong_key_fails() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("test_wrong.sock");

        let listener = UnixListener::bind(&sock_path).unwrap();

        let client_keys = btsp_crypto::derive_session_keys(&[0xAA; 32], &[1u8; 12], &[2u8; 12]);
        let wrong_keys = btsp_crypto::derive_session_keys(&[0xBB; 32], &[3u8; 12], &[4u8; 12]);

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (mut read_half, mut write_half) = stream.into_split();

            let mut len_buf = [0u8; 4];
            read_half.read_exact(&mut len_buf).await.unwrap();
            let payload_len = u32::from_be_bytes(len_buf) as usize;
            let mut payload = vec![0u8; payload_len];
            read_half.read_exact(&mut payload).await.unwrap();

            let response_bytes = b"{\"jsonrpc\":\"2.0\",\"result\":\"ok\",\"id\":1}";
            let frame =
                btsp_crypto::encrypt_frame(&wrong_keys.server_to_client, response_bytes).unwrap();
            write_half.write_all(&frame).await.unwrap();
            write_half.flush().await.unwrap();
        });

        let stream = UnixStream::connect(&sock_path).await.unwrap();
        let request = biomeos_types::JsonRpcRequest::new("test", serde_json::json!({}));

        let result = send_encrypted_jsonrpc(stream, request, &client_keys).await;
        assert!(result.is_err(), "wrong key should fail decryption");

        server.await.unwrap();
    }
}

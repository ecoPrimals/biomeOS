// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::expect_used, reason = "test assertions")]

mod serve_errors;
mod serve_health;
mod serve_listeners;

use crate::neural_api_server::NeuralApiServer;
use biomeos_types::constants::ribocipher;
use biomeos_types::env_config::vars;
use serde_json::Value;
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream, UnixStream};

const RIBOCIPHER_CLEAR_SIGNAL: [u8; ribocipher::SIGNAL_LEN] =
    [ribocipher::SIGNAL_CLEAR, ribocipher::VERSION_1];

pub(super) async fn write_ribocipher_json_rpc(stream: &mut (impl AsyncWriteExt + Unpin), request: &str) {
    stream
        .write_all(&RIBOCIPHER_CLEAR_SIGNAL)
        .await
        .expect("write riboCipher signal");
    stream
        .write_all(format!("{request}\n").as_bytes())
        .await
        .expect("write request");
    stream.flush().await.expect("flush request");
}

pub(super) async fn reserve_tcp_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("reserve tcp port");
    listener.local_addr().expect("local addr").port()
}

pub(super) async fn wait_for_tcp_json_rpc(port: u16, request: &str, timeout: Duration) -> Value {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        match TcpStream::connect(format!("127.0.0.1:{port}")).await {
            Ok(mut stream) => {
                write_ribocipher_json_rpc(&mut stream, request).await;
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
                    return serde_json::from_str(&line).expect("parse response");
                }
            }
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            Err(e) => panic!("TCP health probe failed: {e}"),
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for TCP JSON-RPC on port {port}"
        );
    }
}

pub(super) async fn wait_for_uds_json_rpc(socket_path: &Path, request: &str, timeout: Duration) -> Value {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        match UnixStream::connect(socket_path).await {
            Ok(mut stream) => {
                write_ribocipher_json_rpc(&mut stream, request).await;
                let mut reader = BufReader::new(stream);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() && !line.is_empty() {
                    return serde_json::from_str(&line).expect("parse response");
                }
            }
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(25)).await;
            }
            Err(e) => panic!("UDS health probe failed: {e}"),
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for UDS JSON-RPC on {}",
            socket_path.display()
        );
    }
}

pub(super) fn runtime_env(base: &Path) -> &str {
    base.to_str().expect("utf8 runtime dir")
}

pub(super) fn coordinated_env<'a>(
    runtime_str: &'a str,
    bind: Option<&'a str>,
) -> [(&'a str, Option<&'a str>); 5] {
    [
        ("BIOMEOS_SOCKET_DIR", Some(runtime_str)),
        ("XDG_RUNTIME_DIR", Some(runtime_str)),
        (vars::MODE, Some("coordinated")),
        (vars::BIND_ADDRESS, bind),
        (vars::FAMILY_ID, None),
    ]
}

/// Run `serve()` concurrently with a health probe; dropping the returned future
/// cancels the accept loop (shutdown handling).
pub(super) async fn run_serve_until_tcp_response(
    server: NeuralApiServer,
    port: u16,
    request: &str,
    timeout: Duration,
) -> Value {
    let mut serve = Box::pin(server.serve());
    let mut probe = Box::pin(wait_for_tcp_json_rpc(port, request, timeout));

    tokio::select! {
        response = &mut probe => response,
        result = &mut serve => {
            panic!("serve exited before TCP probe succeeded: {result:?}");
        }
    }
}

pub(super) async fn run_serve_until_uds_response(
    server: NeuralApiServer,
    socket_path: &Path,
    request: &str,
    timeout: Duration,
) -> Value {
    let mut serve = Box::pin(server.serve());
    let mut probe = Box::pin(wait_for_uds_json_rpc(socket_path, request, timeout));

    tokio::select! {
        response = &mut probe => response,
        result = &mut serve => {
            panic!("serve exited before UDS probe succeeded: {result:?}");
        }
    }
}

pub(super) const STARTUP_TIMEOUT: Duration = Duration::from_secs(20);

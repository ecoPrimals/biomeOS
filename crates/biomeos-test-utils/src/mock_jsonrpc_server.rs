// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Zero-sleep mock JSON-RPC server over Unix sockets.
//!
//! Spawns a `tokio::net::UnixListener`, signals readiness via [`ReadySender`],
//! then serves a caller-provided handler for each connection. Tests await the
//! readiness signal before connecting — no `sleep()` required.

use crate::ready_signal::{ReadyReceiver, ReadySender, ready_signal};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::task::JoinHandle;

/// A running mock JSON-RPC server bound to a Unix socket.
pub struct MockJsonRpcServer {
    /// Path to the Unix socket.
    pub socket_path: PathBuf,
    handle: JoinHandle<()>,
}

impl MockJsonRpcServer {
    /// Spawn a server that responds to each JSON-RPC line with the given handler.
    ///
    /// `handler` receives the raw request line and returns the raw response line.
    /// The server signals readiness as soon as the socket is bound.
    pub async fn spawn<F>(socket_path: impl AsRef<Path>, handler: F) -> Self
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        let socket_path = socket_path.as_ref().to_path_buf();
        let (mut ready_tx, ready_rx) = ready_signal();
        let path_clone = socket_path.clone();

        let handle = tokio::spawn(async move {
            let listener =
                UnixListener::bind(&path_clone).expect("MockJsonRpcServer: failed to bind socket");
            ready_tx.signal();

            loop {
                let Ok((stream, _)) = listener.accept().await else {
                    break;
                };
                let (reader, mut writer) = stream.into_split();
                let mut buf_reader = BufReader::new(reader);
                let mut line = String::new();
                while buf_reader.read_line(&mut line).await.unwrap_or(0) > 0 {
                    let response = (handler)(line.trim());
                    let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
                    line.clear();
                }
            }
        });

        ready_rx
            .wait()
            .await
            .expect("server should signal readiness");
        Self {
            socket_path,
            handle,
        }
    }

    /// Spawn a server that echoes a fixed JSON-RPC success response for any request.
    pub async fn spawn_echo_success(
        socket_path: impl AsRef<Path>,
        result: serde_json::Value,
    ) -> Self {
        let result_str = serde_json::to_string(&result).expect("serialize result");
        Self::spawn(socket_path, move |req| {
            let id = serde_json::from_str::<serde_json::Value>(req)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(serde_json::Value::Null);
            format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{result_str}}}"#)
        })
        .await
    }

    /// Spawn a server that echoes a fixed JSON-RPC error response for any request.
    pub async fn spawn_echo_error(socket_path: impl AsRef<Path>, code: i64, message: &str) -> Self {
        let msg = message.to_string();
        Self::spawn(socket_path, move |req| {
            let id = serde_json::from_str::<serde_json::Value>(req)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(serde_json::Value::Null);
            format!(
                r#"{{"jsonrpc":"2.0","id":{id},"error":{{"code":{code},"message":"{msg}"}}}}"#
            )
        })
        .await
    }

    /// Abort the server task.
    pub fn abort(&self) {
        self.handle.abort();
    }
}

impl Drop for MockJsonRpcServer {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Convenience: create a `(ReadySender, ReadyReceiver)` pair for custom server setups.
#[must_use] 
pub fn server_ready_signal() -> (ReadySender, ReadyReceiver) {
    ready_signal()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn echo_success_server() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("test.sock");

        let server = MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!("hello")).await;

        let stream = UnixStream::connect(&server.socket_path).await.unwrap();
        let (reader, mut writer) = stream.into_split();
        writer
            .write_all(b"{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"test\"}\n")
            .await
            .unwrap();

        let mut buf = BufReader::new(reader);
        let mut line = String::new();
        buf.read_line(&mut line).await.unwrap();
        let resp: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(resp["result"], "hello");
        assert_eq!(resp["id"], 1);
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

//! Semantic Layer Integration Tests
//!
//! Tests for capability translation, runtime discovery, and semantic method routing
//!
//! **Concurrency-First Design**: All tests use proper synchronization (oneshot channels)
//! instead of arbitrary sleep() calls. Test issues will be production issues!

use biomeos_atomic_deploy::capability_translation::CapabilityTranslationRegistry;
use serde_json::json;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::sync::oneshot;

/// Mock primal server for testing semantic translation
///
/// **Concurrency**: Uses oneshot channel to signal when server is ready
struct MockPrimalServer {
    socket_path: String,
    expected_method: String,
    response: serde_json::Value,
}

impl MockPrimalServer {
    fn new(socket_path: &str, expected_method: &str, response: serde_json::Value) -> Self {
        let _ = std::fs::remove_file(socket_path);
        Self {
            socket_path: socket_path.to_string(),
            expected_method: expected_method.to_string(),
            response,
        }
    }

    /// Start server and return (handle, ready_receiver)
    /// **Concurrency**: Caller awaits ready_receiver instead of sleeping
    async fn start_with_ready(self) -> (tokio::task::JoinHandle<()>, oneshot::Receiver<()>) {
        let (ready_tx, ready_rx) = oneshot::channel();

        let handle = tokio::spawn(async move {
            let listener = UnixListener::bind(&self.socket_path).unwrap();

            // Signal ready AFTER bind succeeds
            let _ = ready_tx.send(());

            loop {
                if let Ok((mut socket, _)) = listener.accept().await {
                    let expected_method = self.expected_method.clone();
                    let response = self.response.clone();

                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 4096];
                        if let Ok(n) = socket.read(&mut buf).await {
                            let request = String::from_utf8_lossy(&buf[..n]);
                            println!("Mock server received: {request}");

                            // Parse request
                            if let Ok(req) = serde_json::from_str::<serde_json::Value>(&request) {
                                // Verify method name
                                if let Some(method) = req.get("method").and_then(|m| m.as_str()) {
                                    assert_eq!(
                                        method, expected_method,
                                        "Expected method {expected_method}, got {method}"
                                    );
                                }

                                // Send response
                                let id = req
                                    .get("id")
                                    .and_then(serde_json::Value::as_u64)
                                    .unwrap_or(1);
                                let rpc_response = json!({
                                    "jsonrpc": "2.0",
                                    "result": response,
                                    "id": id
                                });

                                let response_str = serde_json::to_string(&rpc_response).unwrap();
                                let _ = socket.write_all(response_str.as_bytes()).await;
                                let _ = socket.flush().await;
                            }
                        }
                    });
                }
            }
        });

        (handle, ready_rx)
    }
}

/// Cleanup helper for socket paths
struct SocketCleanup(String);

impl Drop for SocketCleanup {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

mod registry;
mod translation;

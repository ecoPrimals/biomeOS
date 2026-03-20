// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unix socket JSON-RPC client for BearDog
//!
//! Implements a simple JSON-RPC 2.0 client over Unix sockets.

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, error};

// Re-export JSON-RPC types from biomeos-types for backwards compatibility
pub use biomeos_types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

/// Unix socket client for JSON-RPC communication
pub struct UnixSocketClient {
    socket_path: PathBuf,
}

impl UnixSocketClient {
    /// Create a new Unix socket client
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Check if the socket exists
    pub fn is_available(&self) -> bool {
        self.socket_path.exists()
    }

    /// Send a JSON-RPC request and receive response
    pub async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Connecting to Unix socket: {}", self.socket_path.display());

        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context(format!(
                "Failed to connect to Unix socket: {}",
                self.socket_path.display()
            ))?;

        // Serialize request
        let request_str =
            serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

        debug!("Sending JSON-RPC request: {}", request_str);

        // Send request (newline-delimited)
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Read response (newline-delimited)
        let (reader, _writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .context("Failed to read JSON-RPC response")?;

        debug!("Received JSON-RPC response: {}", line);

        // Parse response
        let response: JsonRpcResponse =
            serde_json::from_str(&line).context("Failed to parse JSON-RPC response")?;

        // Check for errors
        if let Some(error) = &response.error {
            error!(
                "JSON-RPC error: code={}, message={}",
                error.code, error.message
            );
            return Err(anyhow::anyhow!(
                "JSON-RPC error: {} (code {})",
                error.message,
                error.code
            ));
        }

        Ok(response)
    }

    /// Helper to call a method and extract result
    pub async fn call_method(
        &self,
        method: impl AsRef<str>,
        params: serde_json::Value,
    ) -> Result<Value> {
        let request = JsonRpcRequest::new(method, params);
        let response = self.call(request).await?;

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("JSON-RPC response missing result field"))
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_request_creation() {
        let request = JsonRpcRequest::new("test.method", json!({"key": "value"}));
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method.as_ref(), "test.method");
        assert_eq!(request.params.as_ref().unwrap()["key"], "value");
        assert!(
            request
                .id
                .as_ref()
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0)
                > 0
        );
    }

    #[test]
    fn test_unix_socket_client_creation() {
        let client = UnixSocketClient::new("/tmp/test.sock");
        assert_eq!(client.socket_path, PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_socket_availability() {
        let client = UnixSocketClient::new("/tmp/nonexistent.sock");
        assert!(!client.is_available());
    }
}

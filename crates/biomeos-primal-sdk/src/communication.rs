// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Inter-Primal Communication Helpers
//!
//! Standard patterns for secure communication between primals using BearDog BTSP.
//!
//! # Design Principles
//!
//! - **Secure by Default**: All communication via BearDog BTSP
//! - **Discover, Don't Hardcode**: Find primals at runtime
//! - **Handle Absence**: Graceful degradation if primal unavailable
//! - **Type-Safe**: Serialize/deserialize with serde
//!
//! # Example
//!
//! ```rust,no_run
//! use biomeos_primal_sdk::communication::PrimalClient;
//! use biomeos_primal_sdk::PrimalCapability;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Connect to security provider
//! let client = PrimalClient::connect_to_capability(
//!     PrimalCapability::encryption()
//! ).await?;
//!
//! // Send request
//! let response = client.request("method_name", serde_json::json!({
//!     "param": "value"
//! })).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result, anyhow};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{Duration, timeout};

use biomeos_types::{JsonRpcRequest, JsonRpcResponse};

use crate::PrimalCapability;
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery};

/// Client for communicating with other primals
pub struct PrimalClient {
    /// Target primal info
    primal: DiscoveredPrimal,
    /// Timeout for requests
    timeout: Duration,
}

impl PrimalClient {
    /// Connect to a primal by capability
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::communication::PrimalClient;
    /// # use biomeos_primal_sdk::PrimalCapability;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = PrimalClient::connect_to_capability(
    ///     PrimalCapability::new("discovery", "mdns", "1.0")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_to_capability(capability: PrimalCapability) -> Result<Self> {
        let primal = PrimalDiscovery::find_by_capability(capability).await?;
        Ok(Self::new(primal))
    }

    /// Create client for a discovered primal
    #[must_use]
    pub const fn new(primal: DiscoveredPrimal) -> Self {
        Self {
            primal,
            timeout: Duration::from_secs(30),
        }
    }

    /// Set request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a request to the primal
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::communication::PrimalClient;
    /// # async fn example(client: PrimalClient) -> anyhow::Result<()> {
    /// let response = client.request("get_status", serde_json::json!({})).await?;
    /// println!("Status: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request(&self, method: impl AsRef<str>, params: Value) -> Result<Value> {
        let request = JsonRpcRequest::new(method, params);

        let response = timeout(self.timeout, self.send_request(request))
            .await
            .context("Request timeout")?
            .context("Request failed")?;

        if let Some(error) = response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }

        response
            .result
            .ok_or_else(|| anyhow!("No result in response"))
    }

    /// Send request over Unix socket
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Connect to primal's socket
        let mut stream = UnixStream::connect(&self.primal.socket_path)
            .await
            .context(format!("Failed to connect to {}", self.primal.name))?;

        // Serialize request
        let request_json = serde_json::to_vec(&request)?;

        // Send request
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?; // Line-delimited JSON
        stream.flush().await?;

        // Read response
        let mut response_buf = Vec::new();
        stream.read_to_end(&mut response_buf).await?;

        // Deserialize response
        let response: JsonRpcResponse =
            serde_json::from_slice(&response_buf).context("Failed to parse JSON-RPC response")?;

        Ok(response)
    }

    /// Get primal information
    #[must_use]
    pub const fn primal(&self) -> &DiscoveredPrimal {
        &self.primal
    }
}

/// Helper for creating secure tunnels via `BearDog` BTSP
pub struct SecureTunnel;

impl SecureTunnel {
    /// Establish secure tunnel to another primal via `BearDog`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::communication::SecureTunnel;
    /// # async fn example() -> anyhow::Result<()> {
    /// let tunnel = SecureTunnel::establish("target-primal-id").await?;
    /// // Use tunnel for encrypted communication
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish(target_primal_id: impl Into<String>) -> Result<PathBuf> {
        // Discover BearDog (security provider) by security capability
        let beardog = PrimalDiscovery::find_by_capability(PrimalCapability::new(
            "security",
            "encryption",
            "1.0",
        ))
        .await?;

        // Request tunnel establishment via BearDog
        let client = PrimalClient::new(beardog);
        let response = client
            .request(
                "establish_tunnel",
                serde_json::json!({
                    "target": target_primal_id.into(),
                }),
            )
            .await?;

        // Extract tunnel socket path from response
        let tunnel_path = response["tunnel_socket"]
            .as_str()
            .ok_or_else(|| anyhow!("No tunnel socket in response"))?;

        Ok(PathBuf::from(tunnel_path))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, clippy::expect_used, reason = "test")]
mod tests {
    use super::*;
    use crate::discovery::{DiscoveredPrimal, DiscoveryMethod};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test_method\""));
    }

    #[test]
    fn test_jsonrpc_request_params_types() {
        let request =
            JsonRpcRequest::new("echo", serde_json::json!({"nested": {"a": 1, "b": [2, 3]}}));
        let json = serde_json::to_string(&request).unwrap();
        assert!(
            request
                .id
                .as_ref()
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0)
                > 0
        );
        assert!(json.contains("nested"));
    }

    #[test]
    fn test_jsonrpc_response_deserialization_success() {
        let json = r#"{"jsonrpc":"2.0","result":{"ok":true},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
        assert_eq!(response.id.as_u64().unwrap(), 1);
    }

    #[test]
    fn test_jsonrpc_response_deserialization_error() {
        let json =
            r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        assert!(response.result.is_none());
        let err = response.error.unwrap();
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "Invalid Request");
        assert_eq!(response.id.as_u64().unwrap(), 1);
    }

    #[test]
    fn test_jsonrpc_response_error_with_data() {
        let json = r#"{"jsonrpc":"2.0","error":{"code":-32000,"message":"Server error","data":{"detail":"oom"}},"id":2}"#;
        let response: JsonRpcResponse = serde_json::from_str(json).unwrap();
        let err = response.error.unwrap();
        assert_eq!(err.data, Some(serde_json::json!({"detail":"oom"})));
    }

    #[test]
    fn test_primal_client_new() {
        let primal = DiscoveredPrimal {
            name: "test-primal".to_string(),
            socket_path: PathBuf::from("/tmp/test.sock"),
            capability: PrimalCapability::encryption(),
            discovered_via: DiscoveryMethod::TmpFallback,
            is_healthy: true,
        };
        let client = PrimalClient::new(primal);
        assert_eq!(client.primal().name, "test-primal");
        assert_eq!(client.primal().socket_path, PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_primal_client_with_timeout() {
        let primal = DiscoveredPrimal {
            name: "x".to_string(),
            socket_path: PathBuf::from("/tmp/x.sock"),
            capability: PrimalCapability::encryption(),
            discovered_via: DiscoveryMethod::XdgRuntime,
            is_healthy: false,
        };
        let client = PrimalClient::new(primal).with_timeout(Duration::from_secs(5));
        assert_eq!(client.primal().name, "x");
    }

    #[tokio::test]
    async fn primal_client_request_returns_rpc_error_string() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("rpc.sock");
        let listener = UnixListener::bind(&sock_path).expect("bind unix listener");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(&mut stream);
            let mut line = Vec::new();
            reader
                .read_until(b'\n', &mut line)
                .await
                .expect("read request line");
            assert!(!line.is_empty());
            let stream = reader.into_inner();
            let resp =
                br#"{"jsonrpc":"2.0","error":{"code":-32603,"message":"server boom"},"id":1}"#;
            stream.write_all(resp).await.expect("write");
            stream.shutdown().await.expect("shutdown");
        });

        let primal = DiscoveredPrimal {
            name: "local".to_string(),
            socket_path: sock_path.clone(),
            capability: PrimalCapability::encryption(),
            discovered_via: DiscoveryMethod::TmpFallback,
            is_healthy: true,
        };
        let client = PrimalClient::new(primal).with_timeout(Duration::from_secs(5));
        let err = client
            .request("ping", serde_json::json!({}))
            .await
            .expect_err("rpc error expected");
        let msg = err.to_string();
        assert!(
            msg.contains("RPC error") && msg.contains("-32603"),
            "unexpected: {msg}"
        );
        server.await.expect("server task");
    }

    #[tokio::test]
    async fn primal_client_request_err_when_no_result_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("rpc2.sock");
        let listener = UnixListener::bind(&sock_path).expect("bind");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(&mut stream);
            let mut line = Vec::new();
            reader
                .read_until(b'\n', &mut line)
                .await
                .expect("read request line");
            assert!(!line.is_empty());
            let stream = reader.into_inner();
            let resp = br#"{"jsonrpc":"2.0","id":1}"#;
            stream.write_all(resp).await.expect("write");
            stream.shutdown().await.expect("shutdown");
        });

        let primal = DiscoveredPrimal {
            name: "local".to_string(),
            socket_path: sock_path,
            capability: PrimalCapability::encryption(),
            discovered_via: DiscoveryMethod::TmpFallback,
            is_healthy: true,
        };
        let client = PrimalClient::new(primal).with_timeout(Duration::from_secs(5));
        let err = client
            .request("x", serde_json::json!({}))
            .await
            .expect_err("missing result");
        assert!(err.to_string().contains("No result"), "unexpected: {err}");
        server.await.expect("server");
    }

    #[tokio::test]
    async fn primal_client_send_request_fails_on_invalid_json_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock_path = dir.path().join("rpc3.sock");
        let listener = UnixListener::bind(&sock_path).expect("bind");
        let server = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(&mut stream);
            let mut line = Vec::new();
            reader
                .read_until(b'\n', &mut line)
                .await
                .expect("read request line");
            assert!(!line.is_empty());
            let stream = reader.into_inner();
            stream.write_all(b"not-json").await.expect("write");
            stream.shutdown().await.expect("shutdown");
        });

        let primal = DiscoveredPrimal {
            name: "local".to_string(),
            socket_path: sock_path,
            capability: PrimalCapability::encryption(),
            discovered_via: DiscoveryMethod::TmpFallback,
            is_healthy: true,
        };
        let client = PrimalClient::new(primal).with_timeout(Duration::from_secs(5));
        let err = client
            .request("x", serde_json::json!({}))
            .await
            .expect_err("parse fail");
        let chain = format!("{err:#}");
        assert!(
            chain.contains("parse") || chain.contains("JSON"),
            "unexpected: {chain}"
        );
        server.await.expect("server");
    }
}

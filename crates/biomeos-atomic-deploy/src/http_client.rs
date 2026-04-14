// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! HTTP client for Tower Atomic integration.
//!
//! Provides HTTP/HTTPS via the discovery provider (network delegate): fetching binaries, update checks,
//! niche deployment (git clone), and remote health checks.

use anyhow::{Context, Result};
use biomeos_types::primal_names;
use bytes::Bytes;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::timeout;
use tracing::{debug, info};

/// HTTP request builder for biomeOS
#[derive(Debug, Clone)]
pub struct BiomeOsHttpClient {
    /// Discovery-provider Unix socket path (Tower Atomic networking / HTTP delegate)
    discovery_socket: String,
}

/// HTTP response from the discovery delegate
#[derive(Debug, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code (200, 404, etc.)
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body (parsed JSON)
    pub body: Value,
}

impl BiomeOsHttpClient {
    /// Create HTTP client with explicit socket path (for testing / dependency injection)
    #[cfg(test)]
    pub fn with_socket(socket: impl Into<String>) -> Self {
        Self {
            discovery_socket: socket.into(),
        }
    }

    /// Create new HTTP client
    ///
    /// This delegates all HTTP requests to the discovery provider (Tower Atomic)
    /// Uses `SocketNucleation` for deterministic paths (no hardcoding)
    pub fn new() -> Self {
        use crate::nucleation::SocketNucleation;

        let discovery_provider = std::env::var("DISCOVERY_PROVIDER")
            .or_else(|_| std::env::var("BIOMEOS_NETWORK_PROVIDER"))
            .unwrap_or_else(|_| primal_names::SONGBIRD.to_string());
        let discovery_socket = std::env::var("BIOMEOS_DISCOVERY_SOCKET")
            .or_else(|_| {
                std::env::var(biomeos_types::defaults::env_vars::socket_env_key(
                    &discovery_provider,
                ))
            })
            .or_else(|_| std::env::var("DISCOVERY_SOCKET"))
            .unwrap_or_else(|_| {
                let family_id = biomeos_core::family_discovery::get_family_id();
                let mut nucleation = SocketNucleation::default();
                nucleation
                    .assign_socket(&discovery_provider, &family_id)
                    .to_string_lossy()
                    .into_owned()
            });

        info!("🌐 biomeOS HTTP client initialized (via Tower Atomic)");
        debug!("   Discovery socket: {}", discovery_socket);

        Self { discovery_socket }
    }

    /// Perform HTTP GET request
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use biomeos_atomic_deploy::http_client::BiomeOsHttpClient;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = BiomeOsHttpClient::new();
    /// let body = client.get("http://example.com").await?;
    /// println!("Response: {}", body);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, url: &str) -> Result<String> {
        info!("🌐 HTTP GET: {}", url);
        let response = self.request("GET", url, HashMap::new(), None).await?;

        let body = response
            .body
            .as_str()
            .context("Response body is not a string")?
            .to_string();

        Ok(body)
    }

    /// Perform HTTP POST request
    pub async fn post(&self, url: &str, body: Value) -> Result<String> {
        info!("🌐 HTTP POST: {}", url);
        let response = self
            .request("POST", url, HashMap::new(), Some(body))
            .await?;

        let body = response
            .body
            .as_str()
            .context("Response body is not a string")?
            .to_string();

        Ok(body)
    }

    /// Fetch binary data (e.g., download ecoBin from HTTP server)
    ///
    /// # Use Case
    ///
    /// biomeOS can fetch primal binaries from HTTP servers for niche deployment:
    /// ```no_run
    /// # use biomeos_atomic_deploy::http_client::BiomeOsHttpClient;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = BiomeOsHttpClient::new();
    /// let binary = client.fetch_binary("http://releases.example.com/beardog-v0.9.0").await?;
    /// std::fs::write("/tmp/beardog", binary)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_binary(&self, url: &str) -> Result<Bytes> {
        info!("📦 Fetching binary: {}", url);
        let body = self.get(url).await?;

        // For now, HTTP returns string body
        // Note: discovery delegate returns string body; binary would need base64 encoding
        Ok(Bytes::from(body.into_bytes()))
    }

    /// Check if a URL is reachable (health check)
    ///
    /// Returns true if status code is 2xx
    pub async fn is_reachable(&self, url: &str) -> bool {
        match self.request("GET", url, HashMap::new(), None).await {
            Ok(response) => response.status >= 200 && response.status < 300,
            Err(e) => {
                debug!("URL not reachable: {} - {}", url, e);
                false
            }
        }
    }

    /// Perform raw HTTP request (internal)
    async fn request(
        &self,
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        body: Option<Value>,
    ) -> Result<HttpResponse> {
        // Build JSON-RPC request
        let mut params = serde_json::json!({
            "method": method,
            "url": url,
            "headers": headers,
        });

        if let Some(body_value) = body {
            params["body"] = body_value;
        }

        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "http.request",
            "params": params,
            "id": 1
        });

        // Connect to discovery delegate
        let mut stream = UnixStream::connect(&self.discovery_socket)
            .await
            .context(format!(
                "Failed to connect to discovery socket at {}",
                self.discovery_socket
            ))?;

        // Send request
        let request_json = serde_json::to_string(&rpc_request)?;
        debug!("→ discovery: {}", request_json);
        stream.write_all(request_json.as_bytes()).await?;
        stream.shutdown().await?;

        // Read response with timeout to prevent hangs (60s for HTTP responses)
        let mut response_buf = String::new();
        timeout(
            Duration::from_secs(60),
            stream.read_to_string(&mut response_buf),
        )
        .await
        .context("Socket read timeout (60s)")?
        .context("Failed to read response from discovery delegate")?;
        debug!("← discovery: {}", response_buf);

        // Parse JSON-RPC response
        let rpc_response: Value =
            serde_json::from_str(&response_buf).context("Failed to parse JSON-RPC response")?;

        // Check for errors
        if let Some(error) = rpc_response.get("error") {
            anyhow::bail!("Discovery RPC error: {error}");
        }

        // Extract result
        let result = rpc_response
            .get("result")
            .context("No result in RPC response")?;

        let http_response: HttpResponse =
            serde_json::from_value(result.clone()).context("Failed to parse HTTP response")?;

        info!(
            "✅ HTTP {} {} → Status: {}",
            method, url, http_response.status
        );

        Ok(http_response)
    }
}

impl Default for BiomeOsHttpClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test assertions")]

    use super::*;
    use tempfile::tempdir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    async fn spawn_mock_server(response: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempdir().unwrap();
        let socket_path = dir.path().join("mock.sock");
        let path = socket_path.clone();
        let resp = response.to_string();
        tokio::spawn(async move {
            let listener = UnixListener::bind(&path).unwrap();
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            stream.write_all(resp.as_bytes()).await.unwrap();
        });
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        (dir, socket_path)
    }

    #[test]
    fn test_client_creation() {
        let client = BiomeOsHttpClient::new();
        assert!(
            client.discovery_socket.contains("songbird"),
            "Default socket should contain discovery primal name: {}",
            client.discovery_socket
        );
    }

    #[test]
    fn test_default() {
        let client = BiomeOsHttpClient::default();
        assert!(
            client.discovery_socket.contains("songbird"),
            "Default should match new(): {}",
            client.discovery_socket
        );
    }

    #[test]
    fn test_with_socket_for_testing() {
        let client = BiomeOsHttpClient::with_socket("/tmp/test-songbird.sock");
        assert_eq!(client.discovery_socket, "/tmp/test-songbird.sock");
    }

    #[test]
    fn test_http_response_deserialization() {
        let json = r#"{
            "status": 200,
            "headers": {"Content-Type": "text/plain"},
            "body": "hello world"
        }"#;
        let response: HttpResponse = serde_json::from_str(json).expect("parse HttpResponse");
        assert_eq!(response.status, 200);
        assert_eq!(
            response.headers.get("Content-Type"),
            Some(&"text/plain".to_string())
        );
        assert_eq!(response.body.as_str(), Some("hello world"));
    }

    #[test]
    fn test_http_response_deserialization_body_as_object() {
        let json = r#"{
            "status": 200,
            "headers": {},
            "body": {"key": "value"}
        }"#;
        let response: HttpResponse = serde_json::from_str(json).expect("parse HttpResponse");
        assert_eq!(response.status, 200);
        assert!(response.body.get("key").is_some());
    }

    #[tokio::test]
    async fn test_get_fails_when_discovery_unavailable() {
        let client = BiomeOsHttpClient::with_socket("/tmp/nonexistent-discovery-xyz.sock");

        let result = client.get("http://example.com").await;

        let err = result.expect_err("Should fail when discovery socket does not exist");
        assert!(
            err.to_string().contains("Failed to connect")
                || err.to_string().contains("discovery")
                || err.to_string().contains("Connection refused")
                || err.to_string().contains("No such file"),
            "Error should mention connection failure: {err}"
        );
    }

    #[tokio::test]
    async fn test_post_fails_when_discovery_unavailable() {
        let client = BiomeOsHttpClient::with_socket("/tmp/nonexistent-discovery-post.sock");

        let result = client
            .post("http://example.com", serde_json::json!({"test": true}))
            .await;

        let err = result.expect_err("Should fail when discovery socket does not exist");
        assert!(
            err.to_string().contains("Failed to connect")
                || err.to_string().contains("discovery")
                || err.to_string().contains("Connection refused")
                || err.to_string().contains("No such file"),
            "Error should mention connection failure: {err}"
        );
    }

    #[tokio::test]
    async fn test_fetch_binary_fails_when_discovery_unavailable() {
        let client = BiomeOsHttpClient::with_socket("/tmp/nonexistent-discovery-xyz.sock");

        let result = client.fetch_binary("http://example.com/binary").await;

        let err = result.expect_err("Should fail when discovery socket does not exist");
        assert!(
            err.to_string().contains("Failed to connect")
                || err.to_string().contains("discovery")
                || err.to_string().contains("Connection refused")
                || err.to_string().contains("No such file"),
            "Error should mention connection failure: {err}"
        );
    }

    #[tokio::test]
    async fn test_is_reachable_returns_false_when_unavailable() {
        let client = BiomeOsHttpClient::with_socket("/tmp/nonexistent-discovery-xyz.sock");

        let reachable = client.is_reachable("http://example.com").await;

        assert!(
            !reachable,
            "is_reachable should return false when discovery delegate unavailable"
        );
    }

    #[test]
    fn test_http_response_empty_headers() {
        let json = r#"{"status": 404, "headers": {}, "body": "not found"}"#;
        let response: HttpResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(response.status, 404);
        assert!(response.headers.is_empty());
    }

    #[test]
    fn test_http_response_body_as_number() {
        let json = r#"{"status": 200, "headers": {}, "body": 42}"#;
        let response: HttpResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(response.status, 200);
        assert_eq!(response.body.as_i64(), Some(42));
    }

    #[test]
    fn test_http_response_body_as_array() {
        let json = r#"{"status": 200, "headers": {}, "body": [1, 2, 3]}"#;
        let response: HttpResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(response.status, 200);
        assert!(response.body.is_array());
    }

    #[test]
    fn test_http_response_body_as_null() {
        let json = r#"{"status": 204, "headers": {}, "body": null}"#;
        let response: HttpResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(response.status, 204);
        assert!(response.body.is_null());
    }

    #[test]
    fn test_http_response_multiple_headers() {
        let json = r#"{
            "status": 200,
            "headers": {
                "Content-Type": "application/json",
                "X-Custom": "value"
            },
            "body": "{}"
        }"#;
        let response: HttpResponse = serde_json::from_str(json).expect("parse");
        assert_eq!(response.headers.len(), 2);
        assert_eq!(
            response.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(response.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[tokio::test]
    async fn test_is_reachable_returns_false_on_4xx() {
        // When discovery delegate returns 4xx, is_reachable should be false.
        // We use nonexistent socket - request will fail before we get status,
        // so we're testing the Err branch. For 4xx we'd need a mock.
        // This test documents the contract: Err => false
        let client = BiomeOsHttpClient::with_socket("/tmp/nonexistent-http-test.sock");
        let reachable = client.is_reachable("http://example.com/404").await;
        assert!(!reachable);
    }

    #[tokio::test]
    async fn test_get_rpc_error_returns_err() {
        let rpc_error =
            r#"{"jsonrpc":"2.0","error":{"code":-1,"message":"Discovery unavailable"},"id":1}"#;
        let (_dir, socket_path) = spawn_mock_server(rpc_error).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let result = client.get("http://example.com").await;

        let err = result.expect_err("get should fail on RPC error");
        assert!(err.to_string().contains("Discovery RPC error"));
        assert!(err.to_string().contains("Discovery unavailable"));
    }

    #[tokio::test]
    async fn test_get_no_result_returns_err() {
        let no_result = r#"{"jsonrpc":"2.0","id":1}"#;
        let (_dir, socket_path) = spawn_mock_server(no_result).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let result = client.get("http://example.com").await;

        let err = result.expect_err("get should fail when no result");
        assert!(err.to_string().contains("No result"));
    }

    #[tokio::test]
    async fn test_get_body_not_string_returns_err() {
        let body_object = r#"{"jsonrpc":"2.0","result":{"status":200,"headers":{},"body":{"key":"value"}},"id":1}"#;
        let (_dir, socket_path) = spawn_mock_server(body_object).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let result = client.get("http://example.com").await;

        let err = result.expect_err("get should fail when body is not string");
        assert!(err.to_string().contains("Response body is not a string"));
    }

    #[tokio::test]
    async fn test_get_invalid_json_response_returns_err() {
        let invalid_json = "not valid json";
        let (_dir, socket_path) = spawn_mock_server(invalid_json).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let result = client.get("http://example.com").await;

        let err = result.expect_err("get should fail on invalid JSON");
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("JSON"),
            "Error should mention parse/JSON: {err}"
        );
    }

    #[tokio::test]
    async fn test_get_success_returns_body() {
        let success =
            r#"{"jsonrpc":"2.0","result":{"status":200,"headers":{},"body":"hello world"},"id":1}"#;
        let (_dir, socket_path) = spawn_mock_server(success).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let result = client.get("http://example.com").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello world");
    }

    #[tokio::test]
    async fn test_post_body_not_string_returns_err() {
        let body_object =
            r#"{"jsonrpc":"2.0","result":{"status":200,"headers":{},"body":[1,2,3]},"id":1}"#;
        let (_dir, socket_path) = spawn_mock_server(body_object).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let result = client
            .post("http://example.com", serde_json::json!({"x": 1}))
            .await;

        let err = result.expect_err("post should fail when body is not string");
        assert!(err.to_string().contains("Response body is not a string"));
    }

    #[tokio::test]
    async fn test_is_reachable_returns_true_on_2xx() {
        let success =
            r#"{"jsonrpc":"2.0","result":{"status":200,"headers":{},"body":"ok"},"id":1}"#;
        let (_dir, socket_path) = spawn_mock_server(success).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let reachable = client.is_reachable("http://example.com").await;

        assert!(reachable);
    }

    #[tokio::test]
    async fn test_is_reachable_returns_false_on_4xx_status() {
        let four_oh_four =
            r#"{"jsonrpc":"2.0","result":{"status":404,"headers":{},"body":"not found"},"id":1}"#;
        let (_dir, socket_path) = spawn_mock_server(four_oh_four).await;

        let client = BiomeOsHttpClient::with_socket(socket_path.to_string_lossy().as_ref());
        let reachable = client.is_reachable("http://example.com/404").await;

        assert!(!reachable);
    }
}

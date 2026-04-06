// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Neural API Client - Pure Rust capability-based routing client

use anyhow::Context;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::Duration;

use crate::connection::json_rpc_call;
use crate::retry_config::NeuralApiRetryConfig;
use crate::types::{CapabilityInfo, HttpResponse, RoutingMetrics};

/// Neural API Client - Pure Rust capability-based routing
#[derive(Debug)]
pub struct NeuralApiClient {
    /// Path to Neural API Unix socket
    pub socket_path: PathBuf,
    /// Request timeout
    pub request_timeout: Duration,
    /// Connection timeout
    pub connection_timeout: Duration,
    /// Backoff between connection retries (used when `max_connect_attempts` is greater than 1)
    pub retry_config: NeuralApiRetryConfig,
}

impl NeuralApiClient {
    /// Create a new client with explicit socket path
    pub fn new(socket_path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        Ok(Self {
            socket_path: socket_path.into(),
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(5),
            retry_config: NeuralApiRetryConfig::default(),
        })
    }

    /// Discover Neural API socket by family ID
    pub fn discover(family_id: &str) -> anyhow::Result<Self> {
        let socket_path = Self::discover_socket(family_id);

        if !socket_path.exists() {
            anyhow::bail!(
                "Neural API not found: {} does not exist. Is Neural API running?",
                socket_path.display()
            );
        }

        Self::new(socket_path)
    }

    /// Discover socket path from family ID
    #[must_use]
    pub fn discover_socket(family_id: &str) -> PathBuf {
        if let Ok(paths) = biomeos_types::SystemPaths::new() {
            paths.primal_socket(&format!("neural-api-{family_id}"))
        } else {
            biomeos_types::defaults::socket_path(&format!("neural-api-{family_id}")).unwrap_or_else(
                |_| std::env::temp_dir().join(format!("neural-api-{family_id}.sock")),
            )
        }
    }

    /// Set request timeout
    #[must_use]
    pub const fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set connection timeout
    #[must_use]
    pub const fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    /// Set retry policy for transient connection failures
    #[must_use]
    pub const fn with_retry_config(mut self, retry_config: NeuralApiRetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Proxy HTTP request through Tower Atomic
    pub async fn proxy_http(
        &self,
        method: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<Value>,
    ) -> anyhow::Result<HttpResponse> {
        let params = serde_json::json!({
            "method": method,
            "url": url,
            "headers": headers.unwrap_or_default(),
            "body": body
        });

        let result = self.call("neural_api.proxy_http", &params).await?;
        serde_json::from_value(result).context("Failed to parse HTTP response")
    }

    /// Discover primal(s) providing a capability
    pub async fn discover_capability(&self, capability: &str) -> anyhow::Result<CapabilityInfo> {
        let params = serde_json::json!({
            "capability": capability
        });

        let result = self.call("neural_api.discover_capability", &params).await?;
        serde_json::from_value(result).context("Failed to parse capability info")
    }

    /// Route generic JSON-RPC request to primal by capability
    pub async fn route_to_primal(
        &self,
        capability: &str,
        method: &str,
        params: Value,
    ) -> anyhow::Result<Value> {
        let request_params = serde_json::json!({
            "capability": capability,
            "method": method,
            "params": params
        });

        self.call("neural_api.route_to_primal", &request_params)
            .await
    }

    /// Get routing metrics
    pub async fn get_metrics(&self) -> anyhow::Result<RoutingMetrics> {
        let result = self
            .call("neural_api.get_routing_metrics", &Value::Null)
            .await?;

        serde_json::from_value(result).context("Failed to parse routing metrics")
    }

    /// Internal: Make JSON-RPC call to Neural API
    async fn call(&self, method: &str, params: &Value) -> anyhow::Result<Value> {
        json_rpc_call(
            &self.socket_path,
            method,
            params,
            self.request_timeout,
            self.connection_timeout,
            &self.retry_config,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::NeuralApiRetryConfig;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;
    use tokio::time::Duration;

    #[test]
    fn test_new_succeeds() {
        let client = NeuralApiClient::new("/tmp/neural.sock").unwrap();
        assert_eq!(
            client.socket_path,
            std::path::PathBuf::from("/tmp/neural.sock")
        );
        assert_eq!(client.request_timeout, tokio::time::Duration::from_secs(30));
        assert_eq!(
            client.connection_timeout,
            tokio::time::Duration::from_secs(5)
        );
        assert_eq!(client.retry_config, NeuralApiRetryConfig::default());
    }

    #[test]
    fn test_new_with_pathbuf() {
        let path = std::path::PathBuf::from("/var/run/neural.sock");
        let client = NeuralApiClient::new(path.clone()).unwrap();
        assert_eq!(client.socket_path, path);
    }

    #[test]
    fn test_discover_socket_format() {
        let path = NeuralApiClient::discover_socket("fam123");
        assert!(path.to_string_lossy().contains("neural-api"));
        assert!(path.to_string_lossy().contains("fam123"));
    }

    #[test]
    fn test_discover_fails_when_socket_missing() {
        let result = NeuralApiClient::discover("nonexistent_family_xyz_12345");
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("not exist") || msg.contains("not found") || msg.contains("Neural API"),
            "Expected socket missing error, got: {msg}"
        );
    }

    #[test]
    fn test_with_request_timeout() {
        let client = NeuralApiClient::new("/tmp/sock").unwrap();
        let client = client.with_request_timeout(tokio::time::Duration::from_secs(10));
        assert_eq!(client.request_timeout, tokio::time::Duration::from_secs(10));
    }

    #[test]
    fn test_with_connection_timeout() {
        let client = NeuralApiClient::new("/tmp/sock").unwrap();
        let client = client.with_connection_timeout(tokio::time::Duration::from_millis(100));
        assert_eq!(
            client.connection_timeout,
            tokio::time::Duration::from_millis(100)
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_proxy_http_parse_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("proxy_parse.sock");
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let listener = UnixListener::bind(&socket_path_clone).expect("bind");
            ready_tx.send(()).ok();
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let bad_result = r#"{"jsonrpc":"2.0","result":"not an object","id":1}"#;
            stream
                .write_all(bad_result.as_bytes())
                .await
                .expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        ready_rx.await.expect("listener bound");

        let client = NeuralApiClient::new(socket_path)
            .unwrap()
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let result = client
            .proxy_http("GET", "http://example.com", None, None)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("Failed"),
            "Expected parse error: {}",
            err
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_discover_capability_parse_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("discover_parse.sock");
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let listener = UnixListener::bind(&socket_path_clone).expect("bind");
            ready_tx.send(()).ok();
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let bad_result = r#"{"jsonrpc":"2.0","result":{"wrong":"structure"},"id":1}"#;
            stream
                .write_all(bad_result.as_bytes())
                .await
                .expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        ready_rx.await.expect("listener bound");

        let client = NeuralApiClient::new(socket_path)
            .unwrap()
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let result = client.discover_capability("security").await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("Failed"),
            "Expected parse error: {}",
            err
        );
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_get_metrics_parse_error() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("metrics_parse.sock");
        let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();

        let socket_path_clone = socket_path.clone();
        tokio::spawn(async move {
            let listener = UnixListener::bind(&socket_path_clone).expect("bind");
            ready_tx.send(()).ok();
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let bad_result = r#"{"jsonrpc":"2.0","result":"not metrics object","id":1}"#;
            stream
                .write_all(bad_result.as_bytes())
                .await
                .expect("write");
            stream.write_all(b"\n").await.expect("write newline");
        });

        ready_rx.await.expect("listener bound");

        let client = NeuralApiClient::new(socket_path)
            .unwrap()
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let result = client.get_metrics().await;

        assert!(result.is_err());
    }
}

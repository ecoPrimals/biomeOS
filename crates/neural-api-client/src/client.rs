// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural API Client - Pure Rust capability-based routing client

use anyhow::Context;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::Duration;

use crate::connection::json_rpc_call;
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
}

impl NeuralApiClient {
    /// Create a new client with explicit socket path
    pub fn new(socket_path: impl Into<PathBuf>) -> anyhow::Result<Self> {
        Ok(Self {
            socket_path: socket_path.into(),
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(5),
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
    pub fn discover_socket(family_id: &str) -> PathBuf {
        if let Ok(paths) = biomeos_types::SystemPaths::new() {
            paths.primal_socket(&format!("neural-api-{}", family_id))
        } else {
            std::env::temp_dir().join(format!("neural-api-{}.sock", family_id))
        }
    }

    /// Set request timeout
    pub fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Set connection timeout
    pub fn with_connection_timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
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
        )
        .await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_new_with_pathbuf() {
        let path = PathBuf::from("/var/run/neural.sock");
        let client = NeuralApiClient::new(path).expect("new");
        assert_eq!(client.socket_path, PathBuf::from("/var/run/neural.sock"));
    }

    #[test]
    fn test_with_request_timeout() {
        let client = NeuralApiClient::new("/tmp/x.sock")
            .expect("new")
            .with_request_timeout(Duration::from_secs(10));
        assert_eq!(client.request_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_with_connection_timeout() {
        let client = NeuralApiClient::new("/tmp/x.sock")
            .expect("new")
            .with_connection_timeout(Duration::from_secs(2));
        assert_eq!(client.connection_timeout, Duration::from_secs(2));
    }

    #[test]
    fn test_with_both_timeouts() {
        let client = NeuralApiClient::new("/tmp/x.sock")
            .expect("new")
            .with_request_timeout(Duration::from_secs(60))
            .with_connection_timeout(Duration::from_millis(500));
        assert_eq!(client.request_timeout, Duration::from_secs(60));
        assert_eq!(client.connection_timeout, Duration::from_millis(500));
    }

    #[test]
    fn test_discover_fails_when_socket_missing() {
        let result = NeuralApiClient::discover("nonexistent-family-xyz-123");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("does not exist"));
        assert!(err.to_string().contains("Neural API"));
    }

    #[test]
    fn test_discover_socket_fallback_temp_dir() {
        let path = NeuralApiClient::discover_socket("fallback-test");
        assert!(
            path.to_string_lossy().ends_with("neural-api-fallback-test.sock"),
            "got: {}",
            path.display()
        );
    }

    #[tokio::test]
    #[ignore = "requires Neural API socket"]
    async fn test_proxy_http_integration() {
        let client = NeuralApiClient::discover("test").expect("discover");
        let _ = client
            .proxy_http("GET", "https://example.com", None, None)
            .await;
    }

    #[tokio::test]
    #[ignore = "requires Neural API socket"]
    async fn test_route_to_primal_integration() {
        let client = NeuralApiClient::discover("test").expect("discover");
        let _ = client
            .route_to_primal("secure_http", "health.check", serde_json::json!({}))
            .await;
    }

    #[tokio::test]
    #[ignore = "requires Neural API socket"]
    async fn test_get_metrics_integration() {
        let client = NeuralApiClient::discover("test").expect("discover");
        let _ = client.get_metrics().await;
    }
}

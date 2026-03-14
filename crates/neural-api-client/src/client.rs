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
            paths.primal_socket(&format!("neural-api-{family_id}"))
        } else {
            std::env::temp_dir().join(format!("neural-api-{family_id}.sock"))
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
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

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
}

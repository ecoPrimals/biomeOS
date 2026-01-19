//! HTTP/HTTPS Transport (DEPRECATED - Use atomic_client!)
//!
//! **⚠️ WARNING: This transport is deprecated and has C dependencies**
//!
//! ## Issues
//!
//! - **C Dependencies**: reqwest -> openssl-sys (blocks ecoBin!)
//! - **Cleartext**: No encryption (HTTP)
//! - **Slow**: ~10ms localhost overhead (vs 0.1ms Unix socket)
//! - **Non-Isomorphic**: Request/response only (no streaming)
//! - **Port Exposure**: Requires TCP port binding
//!
//! ## Migration Path
//!
//! All primals are evolving to JSON-RPC over Unix sockets via `atomic_client`.
//! This module is feature-gated and only available with `http-transport` feature.
//!
//! **DO NOT USE - Use `atomic_client` instead!**

#![cfg(feature = "http-transport")]

use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tracing::warn;

/// HTTP client for legacy primal communication
///
/// **DEPRECATED**: Use `JsonRpcUnixClient` instead.
#[derive(Debug, Clone)]
pub struct HttpClient {
    base_url: String,
    client: Client,
}

impl HttpClient {
    /// Create a new HTTP client
    ///
    /// # Arguments
    ///
    /// * `base_url` - Base URL (e.g., "http://127.0.0.1:8900")
    pub fn new(base_url: &str) -> Result<Self> {
        warn!(
            base_url = %base_url,
            "⚠️ Creating HTTP client (DEPRECATED - insecure, slow)"
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            base_url: base_url.to_string(),
            client,
        })
    }

    /// Call a JSON-RPC method over HTTP
    ///
    /// # Arguments
    ///
    /// * `method` - Method name
    /// * `params` - Method parameters
    pub async fn call_method(&self, method: &str, params: Value) -> Result<Value> {
        let url = format!("{}/rpc", self.base_url);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("HTTP request failed")?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP error: {}", response.status());
        }

        let response_json: Value = response
            .json()
            .await
            .context("Failed to parse HTTP response")?;

        // Extract result from JSON-RPC response
        response_json
            .get("result")
            .cloned()
            .context("Missing 'result' in HTTP JSON-RPC response")
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_http_client() {
        let client = HttpClient::new("http://127.0.0.1:8900").unwrap();
        assert_eq!(client.base_url(), "http://127.0.0.1:8900");
    }

    // Note: Integration tests with actual HTTP endpoints would go in tests/
}

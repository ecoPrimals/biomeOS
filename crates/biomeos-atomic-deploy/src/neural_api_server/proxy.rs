// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! HTTP proxy functionality for Neural API Server
//!
//! Enables primals to make HTTP/HTTPS requests through Tower Atomic
//! without direct dependencies on HTTP libraries or C crypto libraries.

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::{debug, info};

use super::NeuralApiServer;
use crate::neural_router::RoutingMetrics;

impl NeuralApiServer {
    /// Proxy HTTP request through Tower Atomic (Songbird + BearDog)
    ///
    /// This enables primals to make HTTP/HTTPS requests without direct dependencies
    /// on HTTP libraries or C crypto libraries (like ring).
    ///
    /// # TRUE PRIMAL Pattern
    /// Squirrel doesn't know about Songbird or BearDog - it just asks Neural API
    /// for "secure_http" capability, and the router discovers + forwards.
    pub async fn proxy_http(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        let params = params.as_ref().context("Missing parameters")?;
        let method = params["method"].as_str().context("Missing HTTP method")?;
        let url = params["url"].as_str().context("Missing URL")?;

        // Create a longer-lived binding for default headers
        let default_headers = json!({});
        let headers = params.get("headers").unwrap_or(&default_headers);
        let body = params.get("body");

        info!("🌐 Proxy HTTP: {} {}", method, url);

        // Discover Tower Atomic
        let atomic = self
            .router
            .discover_capability("secure_http")
            .await
            .context("Failed to discover Tower Atomic")?;

        debug!("   Discovered: {:?} primals", atomic.primals.len());

        // Forward to Songbird (handles HTTP/TLS)
        let http_params = json!({
            "method": method,
            "url": url,
            "headers": headers,
            "body": body
        });

        let result = self
            .router
            .forward_request(&atomic.primary_socket, "http.request", &http_params)
            .await?;

        // Log metrics
        let latency = start.elapsed().as_millis() as u64;
        self.router
            .log_metric(RoutingMetrics {
                request_id: Arc::from(request_id.as_str()),
                capability: Arc::from("secure_http"),
                method: Arc::from(format!("http.{method}").as_str()),
                routed_through: atomic.primals.iter().map(|p| p.name.clone()).collect(),
                latency_ms: latency,
                success: true,
                timestamp: Utc::now(),
                error: None,
            })
            .await;

        info!("   ✓ Proxied in {}ms", latency);

        Ok(result)
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use crate::neural_api_server::NeuralApiServer;

    fn create_test_server() -> NeuralApiServer {
        let temp = tempfile::tempdir().expect("temp dir");
        NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"))
    }

    #[tokio::test]
    async fn test_proxy_http_missing_params() {
        let server = create_test_server();
        let result = server.proxy_http(&None).await;
        let err = result.expect_err("should fail with missing params");
        assert!(
            err.to_string().contains("Missing") || err.to_string().contains("parameter"),
            "expected missing params error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_proxy_http_missing_method() {
        let server = create_test_server();
        let params = Some(serde_json::json!({
            "url": "https://example.com",
            "headers": {}
        }));
        let result = server.proxy_http(&params).await;
        let err = result.expect_err("should fail with missing method");
        assert!(
            err.to_string().contains("method") || err.to_string().contains("Method"),
            "expected missing method error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_proxy_http_missing_url() {
        let server = create_test_server();
        let params = Some(serde_json::json!({
            "method": "GET",
            "headers": {}
        }));
        let result = server.proxy_http(&params).await;
        let err = result.expect_err("should fail with missing URL");
        assert!(
            err.to_string().contains("URL") || err.to_string().contains("url"),
            "expected missing URL error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_proxy_http_method_not_string() {
        let server = create_test_server();
        let params = Some(serde_json::json!({
            "method": 123,
            "url": "https://example.com"
        }));
        let result = server.proxy_http(&params).await;
        let err = result.expect_err("should fail when method is not string");
        assert!(!err.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_proxy_http_url_not_string() {
        let server = create_test_server();
        let params = Some(serde_json::json!({
            "method": "GET",
            "url": ["not", "a", "string"]
        }));
        let result = server.proxy_http(&params).await;
        let err = result.expect_err("should fail when url is not string");
        assert!(!err.to_string().is_empty());
    }
}

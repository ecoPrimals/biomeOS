//! HTTP proxy functionality for Neural API Server
//!
//! Enables primals to make HTTP/HTTPS requests through Tower Atomic
//! without direct dependencies on HTTP libraries or C crypto libraries.

use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
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
                request_id: request_id.clone(),
                capability: "secure_http".to_string(),
                method: format!("http.{}", method),
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

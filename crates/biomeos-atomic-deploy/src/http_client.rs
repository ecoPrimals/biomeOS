// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! HTTP client for Tower Atomic integration.
//!
//! Provides HTTP/HTTPS via the discovery provider (network delegate): fetching binaries, update checks,
//! niche deployment (git clone), and remote health checks.

#![expect(
    dead_code,
    reason = "HTTP client for remote operations — wired when Songbird network delegate is live"
)]

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

        let discovery_provider = std::env::var(biomeos_types::env_config::vars::DISCOVERY_PROVIDER)
            .or_else(|_| std::env::var(biomeos_types::env_config::vars::NETWORK_PROVIDER))
            .ok()
            .or_else(|| {
                biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal(
                    "discovery",
                )
                .map(String::from)
            })
            .unwrap_or_else(|| primal_names::SONGBIRD.to_string());
        let discovery_socket = std::env::var(biomeos_types::env_config::vars::DISCOVERY_SOCKET)
            .or_else(|_| {
                std::env::var(biomeos_types::defaults::env_vars::socket_env_key(
                    &discovery_provider,
                ))
            })
            .or_else(|_| std::env::var(biomeos_types::env_config::vars::DISCOVERY_SOCKET_LEGACY))
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
    /// ```ignore
    /// let client = BiomeOsHttpClient::new();
    /// let body = client.get("http://example.com").await?;
    /// println!("Response: {}", body);
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
    /// ```ignore
    /// let client = BiomeOsHttpClient::new();
    /// let binary = client.fetch_binary("http://releases.example.com/beardog-v0.9.0").await?;
    /// std::fs::write("/tmp/beardog", binary)?;
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
#[path = "http_client_tests.rs"]
mod tests;

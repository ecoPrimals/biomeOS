// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Base HTTP client for primal communication (DEPRECATED - Use atomic_client!)
//!
//! ⚠️ WARNING: This module uses HTTP transport with C dependencies (reqwest->openssl-sys).
//! ⚠️ For ecoBin-compliant Pure Rust communication, use `atomic_client` with Unix sockets.
//!
//! This module provides the foundational HTTP client used by all primal clients
//! to communicate with their respective services.

#![cfg(feature = "http-transport")]

use anyhow::{Context, Result};
use reqwest::{Client, Method};
use serde_json::Value;
use std::time::Duration;

/// HTTP client for primal communication
///
/// This client provides a consistent interface for making HTTP requests to primals.
/// It handles:
/// - Request/response serialization
/// - Timeout management
/// - Error handling
/// - Connection reuse
///
/// # Design Philosophy
/// - **Fail Fast**: Timeouts prevent hanging on unresponsive primals
/// - **Clear Errors**: Errors include context about what failed
/// - **Reusable Connections**: Single client instance per primal
#[derive(Debug, Clone)]
pub struct PrimalHttpClient {
    client: Client,
    base_url: String,
    timeout: Duration,
}

impl PrimalHttpClient {
    /// Create a new HTTP client for a primal
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the primal (e.g., `http://localhost:3000`)
    ///
    /// # Panics
    /// Panics if the HTTP client cannot be created (should never happen in practice)
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::with_timeout(base_url, Duration::from_secs(30))
    }

    /// Create a new HTTP client with custom timeout
    ///
    /// # Arguments
    /// * `base_url` - Base URL of the primal
    /// * `timeout` - Request timeout duration
    pub fn with_timeout(base_url: impl Into<String>, timeout: Duration) -> Self {
        Self {
            client: Client::builder()
                .timeout(timeout)
                .build()
                .expect("Failed to create HTTP client"),
            base_url: base_url.into(),
            timeout,
        }
    }

    /// Execute a GET request
    ///
    /// # Arguments
    /// * `path` - Request path (e.g., `/api/v1/services`)
    ///
    /// # Errors
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn get(&self, path: &str) -> Result<Value> {
        self.request(Method::GET, path, None).await
    }

    /// Execute a POST request
    ///
    /// # Arguments
    /// * `path` - Request path
    /// * `body` - Request body as JSON
    ///
    /// # Errors
    /// Returns an error if the request fails or the response cannot be parsed.
    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        self.request(Method::POST, path, Some(body)).await
    }

    /// Execute a generic HTTP request
    ///
    /// # Arguments
    /// * `method` - HTTP method
    /// * `path` - Request path
    /// * `body` - Optional request body as JSON
    ///
    /// # Errors
    /// Returns an error if:
    /// - The request fails to send
    /// - The response status is not successful (2xx)
    /// - The response body cannot be parsed as JSON
    pub async fn request(&self, method: Method, path: &str, body: Option<Value>) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);

        let mut request = self.client.request(method.clone(), &url);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request
            .send()
            .await
            .with_context(|| format!("Failed to send {} request to {}", method, url))?;

        let status = response.status();
        if !status.is_success() {
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "<unable to read error body>".to_string());
            anyhow::bail!("Primal request failed: {} {} - {}", status, url, error_body);
        }

        response
            .json()
            .await
            .context("Failed to parse primal response as JSON")
    }

    /// Get the base URL of this client
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get the configured timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PrimalHttpClient::new("http://localhost:3000");
        assert_eq!(client.base_url(), "http://localhost:3000");
        assert_eq!(client.timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_client_with_custom_timeout() {
        let timeout = Duration::from_secs(60);
        let client = PrimalHttpClient::with_timeout("http://localhost:3000", timeout);
        assert_eq!(client.timeout(), timeout);
    }
}

// biomeOS HTTP Client - Tower Atomic Integration
//
// Provides HTTP/HTTPS capabilities for biomeOS via Songbird delegation
// Used for:
// - Fetching binaries from HTTP servers
// - Checking for updates (GitHub releases, etc.)
// - Niche deployment (git clone over HTTP)
// - Health checks of remote services

use anyhow::{Context, Result};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, info};

/// HTTP request builder for biomeOS
#[derive(Debug, Clone)]
pub struct BiomeOsHttpClient {
    /// Songbird socket path (Tower Atomic networking)
    songbird_socket: String,
}

/// HTTP response from Songbird
#[derive(Debug, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Value,
}

impl BiomeOsHttpClient {
    /// Create new HTTP client
    ///
    /// This delegates all HTTP requests to Songbird (Tower Atomic)
    /// Uses SocketNucleation for deterministic paths (no hardcoding)
    pub fn new() -> Self {
        use crate::nucleation::SocketNucleation;

        let songbird_socket = std::env::var("SONGBIRD_SOCKET").unwrap_or_else(|_| {
            let family_id =
                std::env::var("BIOMEOS_FAMILY_ID").unwrap_or_else(|_| "nat0".to_string());
            let mut nucleation = SocketNucleation::default();
            nucleation
                .assign_socket("songbird", &family_id)
                .to_string_lossy()
                .into_owned()
        });

        info!("🌐 biomeOS HTTP client initialized (via Tower Atomic)");
        debug!("   Songbird socket: {}", songbird_socket);

        Self { songbird_socket }
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
    pub async fn fetch_binary(&self, url: &str) -> Result<Vec<u8>> {
        info!("📦 Fetching binary: {}", url);
        let body = self.get(url).await?;

        // For now, HTTP returns string body
        // Note: Songbird returns string body; binary would need base64 encoding
        Ok(body.into_bytes())
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

        // Connect to Songbird
        let mut stream = UnixStream::connect(&self.songbird_socket)
            .await
            .context(format!(
                "Failed to connect to Songbird at {}",
                self.songbird_socket
            ))?;

        // Send request
        let request_json = serde_json::to_string(&rpc_request)?;
        debug!("→ Songbird: {}", request_json);
        stream.write_all(request_json.as_bytes()).await?;
        stream.shutdown().await?;

        // Read response
        let mut response_buf = String::new();
        stream.read_to_string(&mut response_buf).await?;
        debug!("← Songbird: {}", response_buf);

        // Parse JSON-RPC response
        let rpc_response: Value =
            serde_json::from_str(&response_buf).context("Failed to parse JSON-RPC response")?;

        // Check for errors
        if let Some(error) = rpc_response.get("error") {
            anyhow::bail!("Songbird RPC error: {}", error);
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
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = BiomeOsHttpClient::new();
        assert!(client.songbird_socket.contains("songbird"));
    }

    #[test]
    fn test_default() {
        let _client = BiomeOsHttpClient::default();
    }
}

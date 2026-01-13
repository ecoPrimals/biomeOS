//! HTTP/HTTPS protocol adapter

use reqwest::{Client, Method, Response};
use std::time::Duration;
use tracing::debug;

use crate::primal_client::error::{ApiError, Result};

/// HTTP/HTTPS protocol adapter
#[derive(Debug, Clone)]
pub struct HttpProtocolAdapter {
    client: Client,
}

impl HttpProtocolAdapter {
    /// Create new HTTP adapter with default configuration
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ApiError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self { client })
    }

    /// Create with custom timeout
    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(Duration::from_secs(90))
            .timeout(timeout)
            .build()
            .map_err(|e| ApiError::ConfigError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        Ok(Self { client })
    }

    /// Make a request to endpoint with given method and body
    pub async fn request(
        &self,
        endpoint: &str,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<Response> {
        debug!("HTTP {} {}", method, endpoint);

        let mut request = self.client.request(method.clone(), endpoint);

        if let Some(body_data) = body {
            request = request
                .header("Content-Type", "application/json")
                .body(body_data);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                ApiError::Timeout {
                    operation: format!("{} {}", method, endpoint),
                }
            } else {
                ApiError::RequestFailed {
                    message: format!("{} {} failed: {}", method, endpoint, e),
                    source: Some(Box::new(e)),
                }
            }
        })?;

        Ok(response)
    }

    /// Get protocol identifier
    pub fn protocol(&self) -> &str {
        "http"
    }

    /// Check if endpoint is supported by this adapter
    pub fn supports(&self, endpoint: &str) -> bool {
        endpoint.starts_with("http://") || endpoint.starts_with("https://")
    }
}

impl Default for HttpProtocolAdapter {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP adapter")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supports() {
        let adapter = HttpProtocolAdapter::new().unwrap();
        assert!(adapter.supports("http://localhost:9000"));
        assert!(adapter.supports("https://api.example.com"));
        assert!(!adapter.supports("grpc://localhost:50051"));
    }
}

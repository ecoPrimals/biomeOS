//! Inter-Primal Communication Helpers
//!
//! Standard patterns for secure communication between primals using BearDog BTSP.
//!
//! # Design Principles
//!
//! - **Secure by Default**: All communication via BearDog BTSP
//! - **Discover, Don't Hardcode**: Find primals at runtime
//! - **Handle Absence**: Graceful degradation if primal unavailable
//! - **Type-Safe**: Serialize/deserialize with serde
//!
//! # Example
//!
//! ```rust,no_run
//! use biomeos_primal_sdk::communication::PrimalClient;
//! use biomeos_primal_sdk::PrimalCapability;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Connect to security provider
//! let client = PrimalClient::connect_to_capability(
//!     PrimalCapability::encryption()
//! ).await?;
//!
//! // Send request
//! let response = client.request("method_name", serde_json::json!({
//!     "param": "value"
//! })).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

use crate::discovery::{DiscoveredPrimal, PrimalDiscovery};
use crate::PrimalCapability;

/// JSON-RPC 2.0 request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Value,
    id: u64,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)] // Part of JSON-RPC 2.0 wire format; required for deserialization
    jsonrpc: String,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    error: Option<JsonRpcError>,
    #[allow(dead_code)] // Part of JSON-RPC 2.0 wire format; required for deserialization
    id: u64,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(default)]
    #[allow(dead_code)] // Part of JSON-RPC 2.0 wire format; required for deserialization
    data: Option<Value>,
}

/// Client for communicating with other primals
pub struct PrimalClient {
    /// Target primal info
    primal: DiscoveredPrimal,
    /// Request counter
    request_id: std::sync::atomic::AtomicU64,
    /// Timeout for requests
    timeout: Duration,
}

impl PrimalClient {
    /// Connect to a primal by capability
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::communication::PrimalClient;
    /// # use biomeos_primal_sdk::PrimalCapability;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = PrimalClient::connect_to_capability(
    ///     PrimalCapability::new("discovery", "mdns", "1.0")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_to_capability(capability: PrimalCapability) -> Result<Self> {
        let primal = PrimalDiscovery::find_by_capability(capability).await?;
        Ok(Self::new(primal))
    }

    /// Create client for a discovered primal
    pub fn new(primal: DiscoveredPrimal) -> Self {
        Self {
            primal,
            request_id: std::sync::atomic::AtomicU64::new(1),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a request to the primal
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::communication::PrimalClient;
    /// # async fn example(client: PrimalClient) -> anyhow::Result<()> {
    /// let response = client.request("get_status", serde_json::json!({})).await?;
    /// println!("Status: {:?}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn request(&self, method: impl Into<String>, params: Value) -> Result<Value> {
        let id = self
            .request_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id,
        };

        let response = timeout(self.timeout, self.send_request(request))
            .await
            .context("Request timeout")?
            .context("Request failed")?;

        if let Some(error) = response.error {
            return Err(anyhow!("RPC error {}: {}", error.code, error.message));
        }

        response
            .result
            .ok_or_else(|| anyhow!("No result in response"))
    }

    /// Send request over Unix socket
    async fn send_request(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Connect to primal's socket
        let mut stream = UnixStream::connect(&self.primal.socket_path)
            .await
            .context(format!("Failed to connect to {}", self.primal.name))?;

        // Serialize request
        let request_json = serde_json::to_vec(&request)?;

        // Send request
        stream.write_all(&request_json).await?;
        stream.write_all(b"\n").await?; // Line-delimited JSON
        stream.flush().await?;

        // Read response
        let mut response_buf = Vec::new();
        stream.read_to_end(&mut response_buf).await?;

        // Deserialize response
        let response: JsonRpcResponse =
            serde_json::from_slice(&response_buf).context("Failed to parse JSON-RPC response")?;

        Ok(response)
    }

    /// Get primal information
    pub fn primal(&self) -> &DiscoveredPrimal {
        &self.primal
    }
}

/// Helper for creating secure tunnels via BearDog BTSP
pub struct SecureTunnel;

impl SecureTunnel {
    /// Establish secure tunnel to another primal via BearDog
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use biomeos_primal_sdk::communication::SecureTunnel;
    /// # async fn example() -> anyhow::Result<()> {
    /// let tunnel = SecureTunnel::establish("target-primal-id").await?;
    /// // Use tunnel for encrypted communication
    /// # Ok(())
    /// # }
    /// ```
    pub async fn establish(target_primal_id: impl Into<String>) -> Result<PathBuf> {
        // Discover BearDog (security provider) by security capability
        let beardog = PrimalDiscovery::find_by_capability(PrimalCapability::new(
            "security",
            "encryption",
            "1.0",
        ))
        .await?;

        // Request tunnel establishment via BearDog
        let client = PrimalClient::new(beardog);
        let response = client
            .request(
                "establish_tunnel",
                serde_json::json!({
                    "target": target_primal_id.into(),
                }),
            )
            .await?;

        // Extract tunnel socket path from response
        let tunnel_path = response["tunnel_socket"]
            .as_str()
            .ok_or_else(|| anyhow!("No tunnel socket in response"))?;

        Ok(PathBuf::from(tunnel_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "test_method".to_string(),
            params: serde_json::json!({"key": "value"}),
            id: 1,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test_method\""));
    }
}

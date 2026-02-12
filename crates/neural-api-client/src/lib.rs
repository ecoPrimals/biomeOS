//! Neural API Client - Pure Rust capability-based routing client
//!
//! This library enables primals to communicate with external services
//! and other primals **without** direct HTTP dependencies or knowledge
//! of other primals' existence.
//!
#![warn(missing_docs)]
#![deny(unsafe_code)]
//!
//! # TRUE PRIMAL Pattern
//!
//! Primals using this client have **zero knowledge** of:
//! - Other primals (Songbird, BearDog, etc.)
//! - HTTP/TLS implementation details
//! - Crypto implementation
//! - Socket paths of other services
//!
//! They only know:
//! - "I need a capability" (e.g., "secure_http")
//! - "Neural API is at this socket"
//!
//! # Example
//!
//! ```no_run
//! use neural_api_client::NeuralApiClient;
//! use std::collections::HashMap;
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover Neural API socket at runtime
//! let client = NeuralApiClient::discover("1894e909e454")?;
//!
//! // Call external HTTP API (no reqwest needed!)
//! let response = client.proxy_http(
//!     "POST",
//!     "https://api.anthropic.com/v1/messages",
//!     Some(HashMap::from([
//!         ("x-api-key".to_string(), "sk-...".to_string()),
//!     ])),
//!     Some(json!({
//!         "model": "claude-3-opus-20240229",
//!         "messages": [{"role": "user", "content": "Hello!"}]
//!     }))
//! ).await?;
//!
//! println!("Response: {}", response.body);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

mod error;
pub use error::NeuralApiError;

/// HTTP response from proxied request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body (as JSON string)
    pub body: String,
}

/// Information about discovered capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// Capability name
    pub capability: String,

    /// Atomic type (if applicable): "Tower", "Nest", "Node"
    pub atomic_type: Option<String>,

    /// Primals providing this capability
    pub primals: Vec<PrimalInfo>,

    /// Primary socket to route to
    pub primary_socket: PathBuf,
}

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name
    pub name: String,

    /// Socket path
    pub socket: PathBuf,

    /// Health status
    pub healthy: bool,

    /// Capabilities this primal provides
    pub capabilities: Vec<String>,
}

/// Routing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Total number of requests routed
    pub total_requests: usize,

    /// Individual metrics
    pub metrics: Vec<RoutingMetric>,
}

/// Individual routing metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetric {
    /// Request ID
    pub request_id: String,

    /// Capability requested
    pub capability: String,

    /// Method called
    pub method: String,

    /// Primals involved in routing
    pub routed_through: Vec<String>,

    /// Latency in milliseconds
    pub latency_ms: u64,

    /// Success status
    pub success: bool,

    /// Timestamp
    pub timestamp: String,

    /// Error message (if failed)
    pub error: Option<String>,
}

/// Neural API Client - Pure Rust capability-based routing
///
/// # TRUE PRIMAL Pattern
///
/// This client enables primals to communicate without knowing about
/// other primals' existence. All discovery happens at runtime via
/// the Neural API.
///
/// # Zero Unsafe Code
///
/// This client is 100% safe Rust with async I/O.
pub struct NeuralApiClient {
    /// Path to Neural API Unix socket
    socket_path: PathBuf,

    /// Request timeout
    request_timeout: Duration,

    /// Connection timeout
    connection_timeout: Duration,
}

impl NeuralApiClient {
    /// Create a new client with explicit socket path
    ///
    /// # Example
    ///
    /// ```no_run
    /// use neural_api_client::NeuralApiClient;
    ///
    /// let client = NeuralApiClient::new("/tmp/neural-api.sock")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn new(socket_path: impl Into<PathBuf>) -> Result<Self> {
        Ok(Self {
            socket_path: socket_path.into(),
            request_timeout: Duration::from_secs(30),
            connection_timeout: Duration::from_secs(5),
        })
    }

    /// Discover Neural API socket by family ID
    ///
    /// # TRUE PRIMAL Pattern
    ///
    /// Socket path is constructed at runtime, not hardcoded.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use neural_api_client::NeuralApiClient;
    ///
    /// // Discovers /tmp/neural-api-family.sock
    /// let client = NeuralApiClient::discover("1894e909e454")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn discover(family_id: &str) -> Result<Self> {
        let socket_path = Self::discover_socket(family_id);

        if !socket_path.exists() {
            anyhow::bail!(
                "Neural API not found: {} does not exist. Is Neural API running?",
                socket_path.display()
            );
        }

        Self::new(socket_path)
    }

    /// Discover socket path from family ID via XDG-compliant nucleation pattern
    ///
    /// Uses `SystemPaths` to find the runtime directory:
    /// 1. `$XDG_RUNTIME_DIR/biomeos/neural-api-{family_id}.sock`
    /// 2. `/tmp/biomeos-$USER/neural-api-{family_id}.sock`
    fn discover_socket(family_id: &str) -> PathBuf {
        // Use SystemPaths for XDG-compliant socket discovery
        if let Ok(paths) = biomeos_types::SystemPaths::new() {
            paths.primal_socket(&format!("neural-api-{}", family_id))
        } else {
            // Fallback if SystemPaths fails (shouldn't happen in practice)
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

    /// Proxy HTTP request through Tower Atomic (Songbird + BearDog)
    ///
    /// This enables primals to make HTTP/HTTPS requests **without**:
    /// - `reqwest` or `hyper` (no HTTP library deps)
    /// - `ring` or `openssl-sys` (no C crypto deps)
    /// - Knowledge of Songbird or BearDog
    ///
    /// # Example
    ///
    /// ```no_run
    /// use neural_api_client::NeuralApiClient;
    /// use std::collections::HashMap;
    /// use serde_json::json;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = NeuralApiClient::discover("1894e909e454")?;
    ///
    /// let response = client.proxy_http(
    ///     "POST",
    ///     "https://api.anthropic.com/v1/messages",
    ///     Some(HashMap::from([
    ///         ("x-api-key".to_string(), "sk-...".to_string()),
    ///         ("anthropic-version".to_string(), "2023-06-01".to_string()),
    ///     ])),
    ///     Some(json!({
    ///         "model": "claude-3-opus-20240229",
    ///         "max_tokens": 1024,
    ///         "messages": [{"role": "user", "content": "Hello!"}]
    ///     }))
    /// ).await?;
    ///
    /// println!("Status: {}", response.status);
    /// println!("Body: {}", response.body);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn proxy_http(
        &self,
        method: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<Value>,
    ) -> Result<HttpResponse> {
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
    ///
    /// # Example
    ///
    /// ```no_run
    /// use neural_api_client::NeuralApiClient;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = NeuralApiClient::discover("1894e909e454")?;
    ///
    /// let info = client.discover_capability("secure_http").await?;
    /// println!("Capability: {}", info.capability);
    /// println!("Atomic type: {:?}", info.atomic_type);
    /// for primal in info.primals {
    ///     println!("  - {} @ {:?}", primal.name, primal.socket);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_capability(&self, capability: &str) -> Result<CapabilityInfo> {
        let params = serde_json::json!({
            "capability": capability
        });

        let result = self.call("neural_api.discover_capability", &params).await?;

        serde_json::from_value(result).context("Failed to parse capability info")
    }

    /// Route generic JSON-RPC request to primal by capability
    ///
    /// # Example
    ///
    /// ```no_run
    /// use neural_api_client::NeuralApiClient;
    /// use serde_json::json;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = NeuralApiClient::discover("1894e909e454")?;
    ///
    /// // Call crypto function without knowing about BearDog
    /// let signature = client.route_to_primal(
    ///     "crypto_sign",
    ///     "ed25519.sign",
    ///     json!({"data": "...", "key_id": "..."})
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn route_to_primal(
        &self,
        capability: &str,
        method: &str,
        params: Value,
    ) -> Result<Value> {
        let request_params = serde_json::json!({
            "capability": capability,
            "method": method,
            "params": params
        });

        self.call("neural_api.route_to_primal", &request_params)
            .await
    }

    /// Get routing metrics (for observability/debugging)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use neural_api_client::NeuralApiClient;
    ///
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = NeuralApiClient::discover("1894e909e454")?;
    ///
    /// let metrics = client.get_metrics().await?;
    /// println!("Total requests: {}", metrics.total_requests);
    /// for metric in metrics.metrics {
    ///     println!("  {} - {}ms", metric.method, metric.latency_ms);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_metrics(&self) -> Result<RoutingMetrics> {
        let result = self
            .call("neural_api.get_routing_metrics", &Value::Null)
            .await?;

        serde_json::from_value(result).context("Failed to parse routing metrics")
    }

    /// Internal: Make JSON-RPC call to Neural API
    ///
    /// # Modern Idiomatic Rust
    ///
    /// - Async/await throughout
    /// - Result<T, E> for error handling
    /// - ? operator for propagation
    /// - No .unwrap() or .expect()
    async fn call(&self, method: &str, params: &Value) -> Result<Value> {
        // Connect to Neural API Unix socket
        let mut stream = timeout(
            self.connection_timeout,
            UnixStream::connect(&self.socket_path),
        )
        .await
        .context("Connection timeout")?
        .with_context(|| {
            format!(
                "Failed to connect to Neural API at {}",
                self.socket_path.display()
            )
        })?;

        // Build JSON-RPC 2.0 request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        // Send request
        let request_bytes = serde_json::to_vec(&request).context("Failed to serialize request")?;

        stream
            .write_all(&request_bytes)
            .await
            .context("Failed to write request")?;
        stream
            .write_all(b"\n")
            .await
            .context("Failed to write newline")?;
        stream.flush().await.context("Failed to flush stream")?;

        // Read response with timeout
        let mut response_bytes = Vec::new();
        timeout(
            self.request_timeout,
            stream.read_to_end(&mut response_bytes),
        )
        .await
        .context("Request timeout")?
        .context("Failed to read response")?;

        // Parse response
        let response: Value =
            serde_json::from_slice(&response_bytes).context("Failed to parse JSON-RPC response")?;

        // Check for JSON-RPC error
        if let Some(error) = response.get("error") {
            let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) as i32;
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            return Err(NeuralApiError::RpcError { code, message }.into());
        }

        // Extract result
        response
            .get("result")
            .ok_or_else(|| anyhow::anyhow!("Response missing 'result' field"))
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        // Test with any valid path (doesn't need to exist for construction)
        let client = NeuralApiClient::new("/tmp/test.sock");
        assert!(client.is_ok());
    }

    #[test]
    fn test_discover_socket_path() {
        let path = NeuralApiClient::discover_socket("1894e909e454");
        // Should end with the correct socket filename, regardless of XDG prefix
        assert!(
            path.to_string_lossy()
                .ends_with("neural-api-1894e909e454.sock"),
            "Socket path should end with neural-api-1894e909e454.sock, got: {}",
            path.display()
        );
    }

    #[test]
    fn test_discover_socket_path_custom() {
        let path = NeuralApiClient::discover_socket("production");
        // Should end with the correct socket filename
        assert!(
            path.to_string_lossy()
                .ends_with("neural-api-production.sock"),
            "Socket path should end with neural-api-production.sock, got: {}",
            path.display()
        );
    }

    #[test]
    fn test_timeout_configuration() {
        let client = NeuralApiClient::new("/tmp/test.sock")
            .unwrap()
            .with_request_timeout(Duration::from_secs(60))
            .with_connection_timeout(Duration::from_secs(10));

        assert_eq!(client.request_timeout, Duration::from_secs(60));
        assert_eq!(client.connection_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_json_rpc_request_building() {
        let params = serde_json::json!({
            "method": "POST",
            "url": "https://example.com",
            "headers": {},
            "body": null
        });

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "neural_api.proxy_http",
            "params": params,
            "id": 1
        });

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "neural_api.proxy_http");
        assert_eq!(request["id"], 1);
    }
}

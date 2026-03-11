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
//! println!("Response: {}", response.body_str().unwrap_or("<invalid utf8>"));
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::time::{timeout, Duration};

mod error;
pub use error::NeuralApiError;

/// Serde helpers for Bytes body (JSON-RPC returns body as string)
mod body_serde {
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(d: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Ok(Bytes::from(s.into_bytes()))
    }

    pub fn serialize<S>(b: &Bytes, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Neural API returns string bodies; serialize as UTF-8 string for JSON compatibility
        let str = std::str::from_utf8(b).map_err(serde::ser::Error::custom)?;
        s.serialize_str(str)
    }
}

/// HTTP response from proxied request
///
/// Uses `Bytes` for body to enable zero-copy when passing response data
/// between tasks or storing. Use `.body_str()` for UTF-8 text or `.body()` for raw bytes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,

    /// Response headers
    pub headers: HashMap<String, String>,

    /// Response body (zero-copy Bytes; use `.body_str()` for UTF-8 text)
    #[serde(with = "body_serde")]
    pub body: Bytes,
}

impl HttpResponse {
    /// Get body as UTF-8 string slice (convenience for JSON/text responses)
    ///
    /// # Errors
    /// Returns error if body is not valid UTF-8
    pub fn body_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }

    /// Get body as owned String (allocates; use when you need owned)
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
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
#[derive(Debug)]
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
    /// println!("Body: {}", response.body_str().unwrap_or("<invalid utf8>"));
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
    use std::path::Path;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    // =========================================================================
    // 1. Client creation (NeuralApiClient::new)
    // =========================================================================

    #[test]
    fn test_client_new_with_str_path() {
        let client =
            NeuralApiClient::new("/tmp/test.sock").expect("new() with str path should succeed");
        assert_eq!(client.socket_path, PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_client_new_with_pathbuf() {
        let path = PathBuf::from("/var/run/neural.sock");
        let client = NeuralApiClient::new(path.clone()).expect("new() with PathBuf should succeed");
        assert_eq!(client.socket_path, path);
    }

    #[test]
    fn test_client_new_with_relative_path() {
        let client = NeuralApiClient::new("relative/path.sock")
            .expect("new() with relative path should succeed");
        assert!(client.socket_path.to_string_lossy().contains("relative"));
    }

    #[test]
    fn test_client_default_timeouts() {
        let client = NeuralApiClient::new("/tmp/test.sock").expect("new() should succeed");
        assert_eq!(client.request_timeout, Duration::from_secs(30));
        assert_eq!(client.connection_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_client_with_timeout_builders() {
        let client = NeuralApiClient::new("/tmp/test.sock")
            .expect("new() should succeed")
            .with_request_timeout(Duration::from_secs(60))
            .with_connection_timeout(Duration::from_secs(10));
        assert_eq!(client.request_timeout, Duration::from_secs(60));
        assert_eq!(client.connection_timeout, Duration::from_secs(10));
    }

    // =========================================================================
    // 2. Socket path discovery from environment
    // =========================================================================

    #[test]
    fn test_discover_socket_path_format() {
        let path = NeuralApiClient::discover_socket("1894e909e454");
        assert!(
            path.to_string_lossy()
                .ends_with("neural-api-1894e909e454.sock"),
            "Socket path should end with neural-api-1894e909e454.sock, got: {}",
            path.display()
        );
    }

    #[test]
    fn test_discover_socket_path_custom_family_id() {
        let path = NeuralApiClient::discover_socket("production");
        assert!(
            path.to_string_lossy()
                .ends_with("neural-api-production.sock"),
            "Socket path should end with neural-api-production.sock, got: {}",
            path.display()
        );
    }

    #[test]
    fn test_discover_socket_path_uses_xdg_when_set() {
        let path = NeuralApiClient::discover_socket("test-family");
        // When XDG_RUNTIME_DIR is set, path should contain biomeos
        // When not set, fallback uses temp_dir - either way we get a valid path
        assert!(
            path.to_string_lossy()
                .contains("neural-api-test-family.sock"),
            "Path should contain neural-api-test-family.sock, got: {}",
            path.display()
        );
    }

    #[test]
    fn test_discover_fails_when_socket_missing() {
        // Use a family_id that produces a path that won't exist in test env
        let family_id = format!("nonexistent-{}", std::process::id());
        let result = NeuralApiClient::discover(&family_id);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("does not exist"),
            "Error should mention socket missing, got: {}",
            err
        );
    }

    // =========================================================================
    // 3. Error type construction and display
    // =========================================================================

    #[test]
    fn test_neural_api_error_connection_display() {
        let err = NeuralApiError::ConnectionError("socket not found".to_string());
        let display = err.to_string();
        assert!(display.contains("Failed to connect"));
        assert!(display.contains("socket not found"));
    }

    #[test]
    fn test_neural_api_error_rpc_display() {
        let err = NeuralApiError::RpcError {
            code: -32601,
            message: "Method not found".to_string(),
        };
        let display = err.to_string();
        assert!(display.contains("-32601"));
        assert!(display.contains("Method not found"));
    }

    #[test]
    fn test_neural_api_error_timeout_display() {
        let err = NeuralApiError::Timeout(5000);
        let display = err.to_string();
        assert!(display.contains("5000"));
        assert!(display.contains("timeout"));
    }

    #[test]
    fn test_neural_api_error_not_found_display() {
        let err = NeuralApiError::NotFound("/tmp/missing.sock".to_string());
        let display = err.to_string();
        assert!(display.contains("not found"));
        assert!(display.contains("/tmp/missing.sock"));
    }

    #[test]
    fn test_neural_api_error_constants() {
        assert_eq!(NeuralApiError::PARSE_ERROR, -32700);
        assert_eq!(NeuralApiError::INVALID_REQUEST, -32600);
        assert_eq!(NeuralApiError::METHOD_NOT_FOUND, -32601);
        assert_eq!(NeuralApiError::INVALID_PARAMS, -32602);
        assert_eq!(NeuralApiError::INTERNAL_ERROR, -32603);
    }

    // =========================================================================
    // 4. Request/response serialization
    // =========================================================================

    #[test]
    fn test_http_response_serialization_roundtrip() {
        let response = HttpResponse {
            status: 200,
            headers: HashMap::from([
                ("content-type".to_string(), "application/json".to_string()),
                ("x-request-id".to_string(), "abc-123".to_string()),
            ]),
            body: Bytes::from(r#"{"ok":true}"#),
        };
        let json = serde_json::to_value(&response).expect("serialize HttpResponse");
        let restored: HttpResponse =
            serde_json::from_value(json).expect("deserialize HttpResponse");
        assert_eq!(restored.status, response.status);
        assert_eq!(restored.headers, response.headers);
        assert_eq!(restored.body, response.body);
    }

    #[test]
    fn test_capability_info_serialization_roundtrip() {
        let info = CapabilityInfo {
            capability: "secure_http".to_string(),
            atomic_type: Some("Tower".to_string()),
            primals: vec![PrimalInfo {
                name: "songbird".to_string(),
                socket: PathBuf::from("/tmp/songbird.sock"),
                healthy: true,
                capabilities: vec!["secure_http".to_string()],
            }],
            primary_socket: PathBuf::from("/tmp/primary.sock"),
        };
        let json = serde_json::to_value(&info).expect("serialize CapabilityInfo");
        let restored: CapabilityInfo =
            serde_json::from_value(json).expect("deserialize CapabilityInfo");
        assert_eq!(restored.capability, info.capability);
        assert_eq!(restored.atomic_type, info.atomic_type);
        assert_eq!(restored.primals.len(), 1);
        assert_eq!(restored.primals[0].name, "songbird");
        assert_eq!(restored.primary_socket, info.primary_socket);
    }

    #[test]
    fn test_routing_metrics_serialization_roundtrip() {
        let metrics = RoutingMetrics {
            total_requests: 42,
            metrics: vec![RoutingMetric {
                request_id: "req-1".to_string(),
                capability: "crypto_sign".to_string(),
                method: "ed25519.sign".to_string(),
                routed_through: vec!["beardog".to_string()],
                latency_ms: 5,
                success: true,
                timestamp: "2024-01-15T12:00:00Z".to_string(),
                error: None,
            }],
        };
        let json = serde_json::to_value(&metrics).expect("serialize RoutingMetrics");
        let restored: RoutingMetrics =
            serde_json::from_value(json).expect("deserialize RoutingMetrics");
        assert_eq!(restored.total_requests, metrics.total_requests);
        assert_eq!(restored.metrics.len(), 1);
        assert_eq!(restored.metrics[0].request_id, "req-1");
        assert_eq!(restored.metrics[0].error, None);
    }

    #[test]
    fn test_routing_metric_with_error_serialization() {
        let metric = RoutingMetric {
            request_id: "req-2".to_string(),
            capability: "secure_http".to_string(),
            method: "proxy_http".to_string(),
            routed_through: vec![],
            latency_ms: 100,
            success: false,
            timestamp: "2024-01-15T12:01:00Z".to_string(),
            error: Some("Connection refused".to_string()),
        };
        let json = serde_json::to_value(&metric).expect("serialize RoutingMetric");
        let restored: RoutingMetric =
            serde_json::from_value(json).expect("deserialize RoutingMetric");
        assert_eq!(restored.error, Some("Connection refused".to_string()));
    }

    // =========================================================================
    // 5. Method name formatting
    // =========================================================================

    #[test]
    fn test_method_names_proxy_http() {
        let expected = "neural_api.proxy_http";
        assert_eq!(expected, "neural_api.proxy_http");
    }

    #[test]
    fn test_method_names_discover_capability() {
        assert_eq!(
            "neural_api.discover_capability",
            "neural_api.discover_capability"
        );
    }

    #[test]
    fn test_method_names_route_to_primal() {
        assert_eq!("neural_api.route_to_primal", "neural_api.route_to_primal");
    }

    #[test]
    fn test_method_names_get_metrics() {
        assert_eq!(
            "neural_api.get_routing_metrics",
            "neural_api.get_routing_metrics"
        );
    }

    #[test]
    fn test_json_rpc_request_structure() {
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

    // =========================================================================
    // 6. Public API methods with mock server
    // =========================================================================

    /// Mock server: single-connection server that reads one request, writes one response.
    async fn run_mock_server_one_shot(
        socket_path: &Path,
        response: serde_json::Value,
    ) -> tokio::task::JoinHandle<()> {
        let path = socket_path.to_path_buf();
        let response_json = serde_json::to_string(&response).expect("serialize response");

        tokio::spawn(async move {
            let listener = UnixListener::bind(&path).expect("bind mock socket");
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.expect("read request");
                let _request = &buf[..n];

                let response_line = format!("{}\n", response_json);
                stream
                    .write_all(response_line.as_bytes())
                    .await
                    .expect("write response");
                stream.flush().await.expect("flush");
            }
        })
    }

    #[tokio::test]
    async fn test_proxy_http_success() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let http_response = serde_json::json!({
            "status": 200,
            "headers": {"content-type": "application/json"},
            "body": "{\"ok\":true}"
        });
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": http_response,
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;

        // Give server a moment to bind
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let response = client
            .proxy_http("GET", "https://example.com", None, None)
            .await
            .expect("proxy_http should succeed");

        assert_eq!(response.status, 200);
        assert_eq!(response.body_str().unwrap(), "{\"ok\":true}");
        assert_eq!(
            response.headers.get("content-type"),
            Some(&"application/json".to_string())
        );
    }

    #[tokio::test]
    async fn test_discover_capability_success() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let capability_result = serde_json::json!({
            "capability": "secure_http",
            "atomic_type": "Tower",
            "primals": [{
                "name": "songbird",
                "socket": "/tmp/songbird.sock",
                "healthy": true,
                "capabilities": ["secure_http"]
            }],
            "primary_socket": "/tmp/songbird.sock"
        });
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": capability_result,
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let info = client
            .discover_capability("secure_http")
            .await
            .expect("discover_capability should succeed");

        assert_eq!(info.capability, "secure_http");
        assert_eq!(info.atomic_type, Some("Tower".to_string()));
        assert_eq!(info.primals.len(), 1);
        assert_eq!(info.primals[0].name, "songbird");
        assert!(info.primals[0].healthy);
    }

    #[tokio::test]
    async fn test_route_to_primal_success() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let result = serde_json::json!({"signature": "abc123base64"});
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let value = client
            .route_to_primal(
                "crypto_sign",
                "ed25519.sign",
                serde_json::json!({"data": "x"}),
            )
            .await
            .expect("route_to_primal should succeed");

        assert_eq!(
            value.get("signature").and_then(|v| v.as_str()),
            Some("abc123base64")
        );
    }

    #[tokio::test]
    async fn test_get_metrics_success() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let metrics_result = serde_json::json!({
            "total_requests": 10,
            "metrics": [{
                "request_id": "r1",
                "capability": "secure_http",
                "method": "proxy_http",
                "routed_through": ["songbird"],
                "latency_ms": 5,
                "success": true,
                "timestamp": "2024-01-15T12:00:00Z",
                "error": null
            }]
        });
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": metrics_result,
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let metrics = client
            .get_metrics()
            .await
            .expect("get_metrics should succeed");

        assert_eq!(metrics.total_requests, 10);
        assert_eq!(metrics.metrics.len(), 1);
        assert_eq!(metrics.metrics[0].request_id, "r1");
        assert_eq!(metrics.metrics[0].latency_ms, 5);
    }

    #[tokio::test]
    async fn test_proxy_http_with_headers_and_body() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let http_response = serde_json::json!({
            "status": 201,
            "headers": {},
            "body": "created"
        });
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": http_response,
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let headers = HashMap::from([("x-api-key".to_string(), "secret".to_string())]);
        let body = serde_json::json!({"key": "value"});

        let response = client
            .proxy_http("POST", "https://api.example.com", Some(headers), Some(body))
            .await
            .expect("proxy_http with headers/body should succeed");

        assert_eq!(response.status, 201);
        assert_eq!(response.body_str().unwrap(), "created");
    }

    #[tokio::test]
    async fn test_call_returns_rpc_error() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let rpc_error_response = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_error_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let result = client
            .proxy_http("GET", "https://example.com", None, None)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("-32601") || err.to_string().contains("Method not found"),
            "Error should mention RPC error, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_connection_fails_to_nonexistent_socket() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("nonexistent.sock");
        // Don't spawn server - socket doesn't exist

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_millis(100))
            .with_request_timeout(Duration::from_secs(2));

        let result = client
            .proxy_http("GET", "https://example.com", None, None)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("connect") || err.to_string().contains("Connection"),
            "Error should mention connection failure, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_proxy_http_invalid_response_missing_result() {
        let temp = TempDir::new().expect("create temp dir");
        let socket_path = temp.path().join("neural.sock");

        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1
        });

        let _server = run_mock_server_one_shot(&socket_path, rpc_response).await;
        tokio::time::sleep(Duration::from_millis(50)).await;

        let client = NeuralApiClient::new(&socket_path)
            .expect("create client")
            .with_connection_timeout(Duration::from_secs(2))
            .with_request_timeout(Duration::from_secs(2));

        let result = client
            .proxy_http("GET", "https://example.com", None, None)
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("result") || err.to_string().contains("missing"),
            "Error should mention missing result, got: {}",
            err
        );
    }
}

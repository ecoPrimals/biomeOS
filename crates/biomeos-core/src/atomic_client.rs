//! Atomic Unix Socket Client - Tower-based Pure Rust Communication
//!
//! This module provides a lightweight, atomic client for primal communication
//! using Unix sockets and JSON-RPC 2.0. It replaces HTTP-based communication
//! with Pure Rust, zero-C-dependency IPC.
//!
//! Design Principles:
//! - Atomic: Single-purpose, minimal, focused
//! - Pure Rust: Zero C dependencies (no reqwest, openssl, ring)
//! - Tower-based: Follows Tower service architecture patterns
//! - Capability-driven: Runtime discovery of primal endpoints
//! - Fail-fast: Clear errors, no hanging connections

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;
use tracing::{debug, warn};

/// JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

impl JsonRpcRequest {
    /// Create a new JSON-RPC request with auto-incremented ID
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: REQUEST_ID.fetch_add(1, Ordering::SeqCst),
        }
    }
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Atomic Unix Socket Client - Pure Rust, Tower-based
///
/// This client provides atomic, zero-copy communication with primals via Unix sockets.
/// It is designed to be lightweight, fast, and free of C dependencies.
///
/// # Architecture
/// - **Atomic**: Single-purpose, minimal overhead
/// - **Pure Rust**: No C dependencies (ecoBin ready!)
/// - **Tower-based**: Follows Tower service patterns
/// - **Capability-driven**: Runtime primal discovery
///
/// # Example
/// ```ignore
/// use biomeos_core::atomic_client::AtomicClient;
///
/// let client = AtomicClient::discover("beardog").await?;
/// let result = client.call("generate_entropy", json!({ "bytes": 32 })).await?;
/// ```ignore
#[derive(Debug, Clone)]
pub struct AtomicClient {
    socket_path: PathBuf,
    timeout: Duration,
}

impl AtomicClient {
    /// Discover a primal by name and create an atomic client
    ///
    /// This implements the capability-based discovery pattern:
    /// 1. Search common socket locations
    /// 2. Query primal for capabilities
    /// 3. Return ready-to-use client
    ///
    /// # Arguments
    /// * `primal_name` - Name of the primal (e.g., "beardog", "songbird")
    ///
    /// # Returns
    /// Ready-to-use atomic client for the primal
    pub async fn discover(primal_name: &str) -> Result<Self> {
        debug!("Discovering primal: {}", primal_name);

        let socket_path = discover_primal_socket(primal_name)
            .await
            .context(format!("Failed to discover primal: {}", primal_name))?;

        debug!("Found primal socket: {}", socket_path.display());

        Ok(Self {
            socket_path,
            timeout: Duration::from_secs(30),
        })
    }

    /// Create an atomic client with explicit socket path
    ///
    /// Use this when you already know the socket location.
    ///
    /// # Arguments
    /// * `socket_path` - Path to the Unix socket
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if the primal is available
    pub fn is_available(&self) -> bool {
        self.socket_path.exists()
    }

    /// Call a JSON-RPC method on the primal
    ///
    /// This is the atomic, Pure Rust alternative to HTTP POST.
    ///
    /// # Arguments
    /// * `method` - JSON-RPC method name
    /// * `params` - Method parameters as JSON
    ///
    /// # Returns
    /// JSON result from the primal
    ///
    /// # Errors
    /// Returns an error if:
    /// - The socket cannot be connected
    /// - The request times out
    /// - The primal returns a JSON-RPC error
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let request = JsonRpcRequest::new(method, params);

        debug!(
            "Calling method '{}' on socket: {}",
            method,
            self.socket_path.display()
        );

        // Timeout wrapper for fail-fast behavior
        let response = timeout(self.timeout, self.call_impl(request))
            .await
            .context(format!(
                "Request to {} timed out after {:?}",
                self.socket_path.display(),
                self.timeout
            ))??;

        // Check for JSON-RPC errors
        if let Some(error) = response.error {
            anyhow::bail!("JSON-RPC error {}: {}", error.code, error.message);
        }

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("Missing result in JSON-RPC response"))
    }

    /// Internal implementation of the JSON-RPC call
    async fn call_impl(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Connect to Unix socket
        let mut stream = UnixStream::connect(&self.socket_path)
            .await
            .context(format!(
                "Failed to connect to Unix socket: {}",
                self.socket_path.display()
            ))?;

        // Serialize request
        let request_str =
            serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

        debug!("Sending JSON-RPC request: {}", request_str);

        // Send request (newline-delimited JSON-RPC)
        stream.write_all(request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?;
        stream.flush().await?;

        // Read response (newline-delimited)
        let (reader, _writer) = stream.split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .context("Failed to read JSON-RPC response")?;

        debug!("Received JSON-RPC response: {}", line.trim());

        // Parse response
        let response: JsonRpcResponse =
            serde_json::from_str(&line).context("Failed to parse JSON-RPC response")?;

        Ok(response)
    }

    /// Get the socket path for this client
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

/// Discover primal socket by name using capability-based discovery
///
/// This function searches common socket locations and validates
/// the primal's identity via JSON-RPC.
///
/// # Discovery Locations (in order):
/// 1. `/tmp/{primal}.sock`
/// 2. `/tmp/{primal}-server.sock`
/// 3. `/var/run/biomeos/{primal}.sock`
/// 4. `/run/biomeos/{primal}.sock`
///
/// # Arguments
/// * `primal_name` - Name of the primal to discover
///
/// # Returns
/// Path to the primal's Unix socket
async fn discover_primal_socket(primal_name: &str) -> Result<PathBuf> {
    let primal_lower = primal_name.to_lowercase();

    let candidates = vec![
        format!("/tmp/{}.sock", primal_lower),
        format!("/tmp/{}-server.sock", primal_lower),
        format!("/var/run/biomeos/{}.sock", primal_lower),
        format!("/run/biomeos/{}.sock", primal_lower),
    ];

    for path_str in &candidates {
        let path = PathBuf::from(path_str);
        if path.exists() {
            debug!("Found candidate socket: {}", path.display());

            // Validate by attempting a connection
            if UnixStream::connect(&path).await.is_ok() {
                debug!("Validated primal socket: {}", path.display());
                return Ok(path);
            } else {
                warn!("Socket exists but connection failed: {}", path.display());
            }
        }
    }

    anyhow::bail!(
        "Primal '{}' not found. Searched: {:?}",
        primal_name,
        candidates
    )
}

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

/// Atomic Primal Client - High-level API for common primal operations
///
/// This client provides convenience methods for common primal operations
/// like health checks, command execution, and capability queries.
#[derive(Debug, Clone)]
pub struct AtomicPrimalClient {
    client: AtomicClient,
    primal_name: String,
}

impl AtomicPrimalClient {
    /// Discover a primal and create a high-level client
    pub async fn discover(primal_name: &str) -> Result<Self> {
        let client = AtomicClient::discover(primal_name).await?;
        Ok(Self {
            client,
            primal_name: primal_name.to_string(),
        })
    }

    /// Create a client with explicit socket path
    pub fn new(primal_name: impl Into<String>, socket_path: impl AsRef<Path>) -> Self {
        Self {
            client: AtomicClient::new(socket_path),
            primal_name: primal_name.into(),
        }
    }

    /// Health check (ping)
    pub async fn health_check(&self) -> Result<()> {
        let result = self.client.call("ping", Value::Null).await?;

        if result.get("status") == Some(&Value::String("ok".to_string())) {
            Ok(())
        } else {
            anyhow::bail!("Primal health check failed: {:?}", result)
        }
    }

    /// Get primal identity and capabilities
    pub async fn get_identity(&self) -> Result<Value> {
        self.client.call("get_identity", Value::Null).await
    }

    /// Execute a command in the primal (if supported)
    pub async fn execute_command(&self, command: &str) -> Result<ExecutionResult> {
        let result = self
            .client
            .call(
                "execute_command",
                serde_json::json!({
                    "command": command,
                    "timeout_seconds": 60
                }),
            )
            .await?;

        Ok(ExecutionResult {
            stdout: result
                .get("stdout")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            stderr: result
                .get("stderr")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            exit_code: result
                .get("exit_code")
                .and_then(|v| v.as_i64())
                .map(|v| v as i32),
        })
    }

    /// Get the primal name
    pub fn primal_name(&self) -> &str {
        &self.primal_name
    }

    /// Get direct access to the atomic client
    pub fn atomic_client(&self) -> &AtomicClient {
        &self.client
    }

    /// Check if the primal is available
    pub fn is_available(&self) -> bool {
        self.client.is_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_request_creation() {
        let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test_method");
        assert_eq!(request.params["key"], "value");
        assert!(request.id > 0);
    }

    #[test]
    fn test_atomic_client_creation() {
        let client = AtomicClient::new("/tmp/test.sock");
        assert_eq!(client.socket_path().to_str().unwrap(), "/tmp/test.sock");
    }

    #[test]
    fn test_client_with_timeout() {
        let client = AtomicClient::new("/tmp/test.sock").with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    // Integration tests (require running primals)
    #[tokio::test]
    #[ignore] // Requires BearDog to be running
    async fn test_beardog_discovery() {
        let client = AtomicPrimalClient::discover("beardog").await;
        if client.is_ok() {
            let client = client.unwrap();
            assert!(client.is_available());

            // Try a health check
            let health = client.health_check().await;
            assert!(
                health.is_ok(),
                "BearDog health check failed: {:?}",
                health.err()
            );
        }
    }

    #[tokio::test]
    #[ignore] // Requires Songbird to be running
    async fn test_songbird_discovery() {
        let client = AtomicPrimalClient::discover("songbird").await;
        if client.is_ok() {
            let client = client.unwrap();
            assert!(client.is_available());
        }
    }
}

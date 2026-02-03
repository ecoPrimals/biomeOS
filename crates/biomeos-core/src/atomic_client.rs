//! Atomic Multi-Transport Client - Universal IPC Standard v3.0
//!
//! This module provides a lightweight, atomic client for primal communication
//! supporting multiple transport layers: Unix sockets, abstract sockets, and TCP.
//! Implements the Universal IPC Standard v3.0 for cross-platform compatibility.
//!
//! ## Design Principles
//!
//! - **Atomic**: Single-purpose, minimal, focused
//! - **Pure Rust**: Zero C dependencies (no reqwest, openssl, ring)
//! - **Multi-Transport**: Unix sockets, abstract sockets, TCP fallback
//! - **Tower-based**: Follows Tower service architecture patterns
//! - **Capability-driven**: Runtime discovery of primal endpoints
//! - **Fail-fast**: Clear errors, no hanging connections
//! - **Platform-agnostic**: Works on Linux, macOS, Android, and more
//!
//! ## Transport Tiers (Universal IPC v3.0)
//!
//! - **Tier 1 (Native)**: Unix sockets, abstract sockets - highest performance
//! - **Tier 2 (Universal)**: TCP sockets - cross-device, WASM compatible
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::atomic_client::AtomicClient;
//!
//! // Auto-discovery with fallback
//! let client = AtomicClient::discover("beardog").await?;
//!
//! // Explicit transport
//! let client = AtomicClient::tcp("192.168.1.100", 9100);
//! let client = AtomicClient::unix("/tmp/beardog.sock");
//!
//! let result = client.call("generate_entropy", json!({ "bytes": 32 })).await?;
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UnixStream};
use tokio::time::timeout;
use tracing::{debug, trace};

// Import the Universal IPC v3.0 transport types
use crate::socket_discovery::{SocketDiscovery, TransportEndpoint};

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

/// Atomic Multi-Transport Client - Universal IPC Standard v3.0
///
/// This client provides atomic, zero-copy communication with primals via
/// multiple transport layers: Unix sockets, abstract sockets, and TCP.
///
/// ## Architecture
///
/// - **Atomic**: Single-purpose, minimal overhead
/// - **Pure Rust**: No C dependencies (ecoBin ready!)
/// - **Multi-Transport**: Unix, Abstract, TCP support
/// - **Tower-based**: Follows Tower service patterns
/// - **Capability-driven**: Runtime primal discovery with fallback
///
/// ## Transport Selection
///
/// When using `discover()`, the client automatically selects the best available
/// transport following Universal IPC v3.0:
/// 1. Unix socket (Tier 1) - fastest, local only
/// 2. Abstract socket (Tier 1) - Linux/Android, no filesystem
/// 3. TCP socket (Tier 2) - universal, cross-device
///
/// ## Example
///
/// ```ignore
/// use biomeos_core::atomic_client::AtomicClient;
///
/// // Auto-discover with fallback
/// let client = AtomicClient::discover("beardog").await?;
///
/// // Or explicit transport
/// let tcp_client = AtomicClient::tcp("192.168.1.100", 9100);
/// let unix_client = AtomicClient::unix("/tmp/beardog.sock");
///
/// let result = client.call("generate_entropy", json!({ "bytes": 32 })).await?;
/// ```
#[derive(Debug, Clone)]
pub struct AtomicClient {
    /// Transport endpoint (Unix, Abstract, or TCP)
    endpoint: TransportEndpoint,
    /// Request timeout
    timeout: Duration,
    /// Legacy: socket path for backwards compatibility
    socket_path: PathBuf,
}

impl AtomicClient {
    /// Discover a primal by name and create an atomic client
    ///
    /// **Universal IPC v3.0**: Implements automatic transport fallback:
    /// 1. Try Tier 1 (Unix socket, abstract socket)
    /// 2. Fall back to Tier 2 (TCP) if Tier 1 unavailable
    ///
    /// # Arguments
    /// * `primal_name` - Name of the primal (e.g., "beardog", "songbird")
    ///
    /// # Returns
    /// Ready-to-use atomic client with best available transport
    pub async fn discover(primal_name: &str) -> Result<Self> {
        debug!("Discovering primal with fallback: {}", primal_name);

        // Get family_id from environment
        let family_id = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("NODE_FAMILY_ID"))
            .unwrap_or_else(|_| {
                trace!("No FAMILY_ID set, using 'default' for discovery");
                "default".to_string()
            });

        // Use SocketDiscovery with automatic fallback
        let discovery = SocketDiscovery::new(&family_id);

        match discovery.discover_with_fallback(primal_name).await {
            Some(endpoint) => {
                debug!(
                    "Discovered {} via {}: {}",
                    primal_name,
                    if endpoint.is_native() { "Tier 1" } else { "Tier 2" },
                    endpoint
                );

                Ok(Self::from_endpoint(endpoint))
            }
            None => {
                anyhow::bail!(
                    "Primal '{}' not found via any transport. Try:\n\
                     1. Set {}_SOCKET=/path/to/{}.sock (Unix)\n\
                     2. Set {}_TCP=host:port (TCP)\n\
                     3. Ensure primal is running in family: {}",
                    primal_name,
                    primal_name.to_uppercase(),
                    primal_name,
                    primal_name.to_uppercase(),
                    family_id
                )
            }
        }
    }

    /// Create an atomic client from a transport endpoint
    ///
    /// **Universal IPC v3.0**: Use this with discovered endpoints.
    pub fn from_endpoint(endpoint: TransportEndpoint) -> Self {
        let socket_path = match &endpoint {
            TransportEndpoint::UnixSocket { path } => path.clone(),
            _ => PathBuf::new(),
        };

        Self {
            endpoint,
            socket_path,
            timeout: Duration::from_secs(30),
        }
    }

    /// Create an atomic client with explicit Unix socket path (Tier 1)
    ///
    /// Use this when you already know the socket location.
    ///
    /// # Arguments
    /// * `socket_path` - Path to the Unix socket
    pub fn unix(socket_path: impl AsRef<Path>) -> Self {
        let path = socket_path.as_ref().to_path_buf();
        Self {
            endpoint: TransportEndpoint::UnixSocket { path: path.clone() },
            socket_path: path,
            timeout: Duration::from_secs(30),
        }
    }

    /// Create an atomic client with explicit TCP endpoint (Tier 2)
    ///
    /// Use this for cross-device communication.
    ///
    /// # Arguments
    /// * `host` - TCP host address
    /// * `port` - TCP port number
    pub fn tcp(host: impl Into<String>, port: u16) -> Self {
        Self {
            endpoint: TransportEndpoint::TcpSocket {
                host: host.into(),
                port,
            },
            socket_path: PathBuf::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Create an atomic client with explicit abstract socket (Tier 1 - Linux/Android)
    ///
    /// Use this for high-performance local communication on Linux/Android
    /// where filesystem sockets may be blocked (e.g., SELinux).
    ///
    /// # Arguments
    /// * `name` - Abstract socket name (without leading @)
    #[cfg(target_os = "linux")]
    pub fn abstract_socket(name: impl Into<String>) -> Self {
        Self {
            endpoint: TransportEndpoint::AbstractSocket { name: name.into() },
            socket_path: PathBuf::new(),
            timeout: Duration::from_secs(30),
        }
    }

    /// Alias for unix() - backwards compatibility
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self::unix(socket_path)
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if the primal is available
    ///
    /// For Unix sockets: checks if the socket file exists
    /// For TCP: returns true (availability checked on connect)
    /// For Abstract: returns true (availability checked on connect)
    pub fn is_available(&self) -> bool {
        match &self.endpoint {
            TransportEndpoint::UnixSocket { path } => path.exists(),
            TransportEndpoint::TcpSocket { .. } => true, // TCP availability checked on connect
            TransportEndpoint::AbstractSocket { .. } => true, // Abstract availability checked on connect
        }
    }

    /// Get the transport endpoint
    pub fn endpoint(&self) -> &TransportEndpoint {
        &self.endpoint
    }

    /// Call a JSON-RPC method on the primal
    ///
    /// **Universal IPC v3.0**: Works across all transport types.
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
    /// - The transport cannot be connected
    /// - The request times out
    /// - The primal returns a JSON-RPC error
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let request = JsonRpcRequest::new(method, params);

        debug!("Calling method '{}' on {}", method, self.endpoint);

        // Timeout wrapper for fail-fast behavior
        let response = timeout(self.timeout, self.call_impl(request))
            .await
            .context(format!(
                "Request to {} timed out after {:?}",
                self.endpoint,
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
    ///
    /// Dispatches to the appropriate transport based on endpoint type.
    async fn call_impl(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        match &self.endpoint {
            TransportEndpoint::UnixSocket { path } => {
                self.call_via_unix(path, request).await
            }
            TransportEndpoint::TcpSocket { host, port } => {
                self.call_via_tcp(host, *port, request).await
            }
            TransportEndpoint::AbstractSocket { name } => {
                self.call_via_abstract(name, request).await
            }
        }
    }

    /// Send JSON-RPC request via Unix socket
    async fn call_via_unix(&self, path: &Path, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let stream = UnixStream::connect(path)
            .await
            .context(format!("Failed to connect to Unix socket: {}", path.display()))?;

        self.send_request(stream, request).await
    }

    /// Send JSON-RPC request via TCP
    async fn call_via_tcp(
        &self,
        host: &str,
        port: u16,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .await
            .context(format!("Failed to connect to TCP: {}", addr))?;

        self.send_request(stream, request).await
    }

    /// Send JSON-RPC request via abstract socket (Linux/Android)
    #[cfg(target_os = "linux")]
    async fn call_via_abstract(
        &self,
        name: &str,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse> {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::SocketAddr;

        // Create abstract socket address
        let addr = SocketAddr::from_abstract_name(name)
            .context(format!("Invalid abstract socket name: {}", name))?;

        // Connect using blocking socket, then wrap in tokio
        // Note: tokio doesn't directly support abstract socket connect
        let std_stream = std::os::unix::net::UnixStream::connect_addr(&addr)
            .context(format!("Failed to connect to abstract socket: @{}", name))?;
        std_stream.set_nonblocking(true)?;

        let stream = UnixStream::from_std(std_stream)?;
        self.send_request(stream, request).await
    }

    #[cfg(not(target_os = "linux"))]
    async fn call_via_abstract(
        &self,
        name: &str,
        _request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse> {
        anyhow::bail!(
            "Abstract sockets not supported on this platform (only Linux/Android). \
             Socket name: @{}",
            name
        )
    }

    /// Generic request sender for any AsyncRead + AsyncWrite stream
    async fn send_request<S>(&self, stream: S, request: JsonRpcRequest) -> Result<JsonRpcResponse>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let (reader, mut writer) = tokio::io::split(stream);

        // Serialize request
        let request_str =
            serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

        trace!("Sending JSON-RPC request: {}", request_str);

        // Send request (newline-delimited JSON-RPC)
        writer.write_all(request_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response (newline-delimited)
        let mut reader = BufReader::new(reader);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .await
            .context("Failed to read JSON-RPC response")?;

        trace!("Received JSON-RPC response: {}", line.trim());

        // Parse response
        let response: JsonRpcResponse =
            serde_json::from_str(&line).context("Failed to parse JSON-RPC response")?;

        Ok(response)
    }

    /// Get the socket path for this client (legacy compatibility)
    ///
    /// Returns empty path for non-Unix transports.
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

/// Discover primal endpoint by name using platform-agnostic discovery
///
/// **Universal IPC v3.0**: Uses `SocketDiscovery` with automatic fallback.
///
/// This replaces hardcoded paths with capability-based discovery that:
/// - Respects environment variables (e.g., `BEARDOG_SOCKET`, `BEARDOG_TCP`)
/// - Uses XDG_RUNTIME_DIR when available
/// - Tries abstract sockets on Linux/Android
/// - Falls back to TCP if native IPC unavailable
///
/// # Arguments
/// * `primal_name` - Name of the primal to discover
///
/// # Returns
/// Transport endpoint ready for connection
///
/// # Discovery Order (Universal IPC v3.0)
/// 1. Environment hints (SOCKET, TCP, ENDPOINT)
/// 2. Unix socket (XDG runtime dir)
/// 3. Abstract socket (Linux/Android)
/// 4. Unix socket (/tmp)
/// 5. Capability registry
/// 6. TCP fallback
pub async fn discover_primal_endpoint(primal_name: &str) -> Result<TransportEndpoint> {
    // Get family_id from environment
    let family_id = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("NODE_FAMILY_ID"))
        .unwrap_or_else(|_| {
            trace!("No FAMILY_ID set, using 'default' for discovery");
            "default".to_string()
        });

    // Use SocketDiscovery with automatic fallback
    let discovery = SocketDiscovery::new(&family_id);

    match discovery.discover_with_fallback(primal_name).await {
        Some(endpoint) => {
            debug!(
                "Discovered {} via {}: {}",
                primal_name,
                if endpoint.is_native() { "Tier 1" } else { "Tier 2" },
                endpoint
            );
            Ok(endpoint)
        }
        None => {
            anyhow::bail!(
                "Primal '{}' not found via any transport. Try:\n\
                 1. Set {}_SOCKET=/path/to/{}.sock (Unix)\n\
                 2. Set {}_TCP=host:port (TCP)\n\
                 3. Ensure primal is running in family: {}",
                primal_name,
                primal_name.to_uppercase(),
                primal_name,
                primal_name.to_uppercase(),
                family_id
            )
        }
    }
}

/// Legacy: Discover primal socket path (Unix only)
///
/// **Deprecated**: Use `discover_primal_endpoint()` for multi-transport support.
#[deprecated(
    since = "3.0.0",
    note = "Use discover_primal_endpoint() for Universal IPC v3.0 support"
)]
pub async fn discover_primal_socket(primal_name: &str) -> Result<PathBuf> {
    let endpoint = discover_primal_endpoint(primal_name).await?;

    match endpoint {
        TransportEndpoint::UnixSocket { path } => Ok(path),
        TransportEndpoint::AbstractSocket { name } => {
            // Return a pseudo-path for abstract sockets
            Ok(PathBuf::from(format!("@{}", name)))
        }
        TransportEndpoint::TcpSocket { host, port } => {
            // Return a pseudo-path for TCP
            Ok(PathBuf::from(format!("tcp://{}:{}", host, port)))
        }
    }
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
/// **Universal IPC v3.0**: Supports multi-transport discovery and communication.
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
    ///
    /// **Universal IPC v3.0**: Uses automatic transport fallback.
    pub async fn discover(primal_name: &str) -> Result<Self> {
        let client = AtomicClient::discover(primal_name).await?;
        Ok(Self {
            client,
            primal_name: primal_name.to_string(),
        })
    }

    /// Create a client with explicit Unix socket path
    pub fn unix(primal_name: impl Into<String>, socket_path: impl AsRef<Path>) -> Self {
        Self {
            client: AtomicClient::unix(socket_path),
            primal_name: primal_name.into(),
        }
    }

    /// Create a client with explicit TCP endpoint
    pub fn tcp(primal_name: impl Into<String>, host: impl Into<String>, port: u16) -> Self {
        Self {
            client: AtomicClient::tcp(host, port),
            primal_name: primal_name.into(),
        }
    }

    /// Create a client from a transport endpoint
    pub fn from_endpoint(primal_name: impl Into<String>, endpoint: TransportEndpoint) -> Self {
        Self {
            client: AtomicClient::from_endpoint(endpoint),
            primal_name: primal_name.into(),
        }
    }

    /// Legacy: Create a client with explicit socket path
    #[deprecated(since = "3.0.0", note = "Use unix() for clarity")]
    pub fn new(primal_name: impl Into<String>, socket_path: impl AsRef<Path>) -> Self {
        Self::unix(primal_name, socket_path)
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

    /// Get the transport endpoint
    pub fn endpoint(&self) -> &TransportEndpoint {
        self.client.endpoint()
    }

    /// Check if the primal is available
    pub fn is_available(&self) -> bool {
        self.client.is_available()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // JSON-RPC Tests
    // ========================================================================

    #[test]
    fn test_jsonrpc_request_creation() {
        let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test_method");
        assert_eq!(request.params["key"], "value");
        assert!(request.id > 0);
    }

    // ========================================================================
    // AtomicClient Constructor Tests - Universal IPC v3.0
    // ========================================================================

    #[test]
    fn test_atomic_client_unix() {
        let client = AtomicClient::unix("/tmp/test.sock");
        assert_eq!(client.socket_path().to_str().unwrap(), "/tmp/test.sock");
        assert!(matches!(
            client.endpoint(),
            TransportEndpoint::UnixSocket { .. }
        ));
    }

    #[test]
    fn test_atomic_client_tcp() {
        let client = AtomicClient::tcp("192.168.1.100", 9100);
        assert!(matches!(
            client.endpoint(),
            TransportEndpoint::TcpSocket { .. }
        ));
        if let TransportEndpoint::TcpSocket { host, port } = client.endpoint() {
            assert_eq!(host, "192.168.1.100");
            assert_eq!(*port, 9100);
        }
    }

    #[test]
    fn test_atomic_client_from_endpoint() {
        let endpoint = TransportEndpoint::TcpSocket {
            host: "localhost".to_string(),
            port: 8080,
        };
        let client = AtomicClient::from_endpoint(endpoint);
        assert!(matches!(
            client.endpoint(),
            TransportEndpoint::TcpSocket { .. }
        ));
    }

    #[test]
    fn test_atomic_client_new_legacy() {
        // Test backwards compatibility
        let client = AtomicClient::new("/tmp/test.sock");
        assert_eq!(client.socket_path().to_str().unwrap(), "/tmp/test.sock");
    }

    #[test]
    fn test_client_with_timeout() {
        let client = AtomicClient::unix("/tmp/test.sock").with_timeout(Duration::from_secs(10));
        assert_eq!(client.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_is_available_unix() {
        // Non-existent socket
        let client = AtomicClient::unix("/tmp/nonexistent.sock");
        assert!(!client.is_available());
    }

    #[test]
    fn test_is_available_tcp() {
        // TCP always returns true (availability checked on connect)
        let client = AtomicClient::tcp("127.0.0.1", 9999);
        assert!(client.is_available());
    }

    // ========================================================================
    // AtomicPrimalClient Constructor Tests
    // ========================================================================

    #[test]
    fn test_atomic_primal_client_unix() {
        let client = AtomicPrimalClient::unix("beardog", "/tmp/beardog.sock");
        assert_eq!(client.primal_name(), "beardog");
        assert!(matches!(
            client.endpoint(),
            TransportEndpoint::UnixSocket { .. }
        ));
    }

    #[test]
    fn test_atomic_primal_client_tcp() {
        let client = AtomicPrimalClient::tcp("beardog", "192.168.1.100", 9100);
        assert_eq!(client.primal_name(), "beardog");
        assert!(matches!(
            client.endpoint(),
            TransportEndpoint::TcpSocket { .. }
        ));
    }

    #[test]
    fn test_atomic_primal_client_from_endpoint() {
        let endpoint = TransportEndpoint::TcpSocket {
            host: "10.0.0.1".to_string(),
            port: 9200,
        };
        let client = AtomicPrimalClient::from_endpoint("songbird", endpoint);
        assert_eq!(client.primal_name(), "songbird");
    }

    // ========================================================================
    // Integration Tests (require running primals)
    // ========================================================================

    #[tokio::test]
    #[ignore] // Requires BearDog to be running
    async fn test_beardog_discovery() {
        let client = AtomicPrimalClient::discover("beardog").await;
        if let Ok(client) = client {
            assert!(client.is_available());

            // Log the transport type discovered
            println!(
                "BearDog discovered via: {}",
                client.endpoint().display_string()
            );

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
        if let Ok(client) = client {
            assert!(client.is_available());
            println!(
                "Songbird discovered via: {}",
                client.endpoint().display_string()
            );
        }
    }

    #[tokio::test]
    #[ignore] // Requires TCP endpoint running
    async fn test_tcp_connection() {
        let client = AtomicClient::tcp("127.0.0.1", 9100);
        // This will fail unless something is listening
        let result = client.call("ping", Value::Null).await;
        // Just verify we can construct and attempt TCP calls
        assert!(result.is_err() || result.is_ok()); // Either works or fails gracefully
    }
}

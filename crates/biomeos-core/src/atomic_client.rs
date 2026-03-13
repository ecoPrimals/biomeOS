// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    /// Protocol version (always "2.0")
    pub jsonrpc: String,
    /// Method name
    pub method: String,
    /// Method parameters
    pub params: Value,
    /// Request identifier
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
    /// Protocol version
    pub jsonrpc: String,
    /// Successful result payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error payload
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// Corresponding request identifier
    pub id: u64,
}

/// JSON-RPC 2.0 Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Numeric error code
    pub code: i32,
    /// Human-readable error message
    pub message: String,
    /// Optional structured data
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
    pub(crate) timeout: Duration,
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
                    if endpoint.is_native() {
                        "Tier 1"
                    } else {
                        "Tier 2"
                    },
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
    /// Use this for cross-device communication (raw newline-delimited JSON-RPC).
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

    /// Create an atomic client with HTTP JSON-RPC endpoint (Tier 2 - Inter-gate)
    ///
    /// Use this for inter-NUCLEUS communication via Songbird's HTTP `/jsonrpc`
    /// gateway. This is the preferred transport for covalent bond communication
    /// between NUCLEUS instances on LAN or internet.
    ///
    /// The port should be runtime-discovered via beacon exchange, not hardcoded.
    /// Songbird's default HTTP port is 8080 (configurable via `SONGBIRD_HTTP_PORT`).
    ///
    /// # Arguments
    /// * `host` - Remote host address (IP or hostname)
    /// * `port` - Remote Songbird HTTP port (discovered via beacon, default 8080)
    pub fn http(host: impl Into<String>, port: u16) -> Self {
        Self {
            endpoint: TransportEndpoint::HttpJsonRpc {
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
            TransportEndpoint::HttpJsonRpc { .. } => true, // HTTP availability checked on connect
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
                self.endpoint, self.timeout
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
            TransportEndpoint::UnixSocket { path } => self.call_via_unix(path, request).await,
            TransportEndpoint::TcpSocket { host, port } => {
                self.call_via_tcp(host, *port, request).await
            }
            TransportEndpoint::AbstractSocket { name } => {
                self.call_via_abstract(name, request).await
            }
            TransportEndpoint::HttpJsonRpc { host, port } => {
                self.call_via_http(host, *port, request).await
            }
        }
    }

    /// Send JSON-RPC request via Unix socket
    async fn call_via_unix(&self, path: &Path, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        let stream = UnixStream::connect(path).await.context(format!(
            "Failed to connect to Unix socket: {}",
            path.display()
        ))?;

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

    /// Send JSON-RPC request via HTTP POST to `/jsonrpc` endpoint
    ///
    /// This implements a minimal HTTP/1.1 POST client using raw `TcpStream`,
    /// keeping the zero-C-dependency Pure Rust guarantee. The remote Songbird
    /// instance serves JSON-RPC over HTTP at `POST /jsonrpc`, which forwards
    /// to the same IPC handler as the Unix socket (mesh.*, relay.*, health, etc.)
    ///
    /// EVOLUTION (Feb 2026): Replaces raw TCP JSON-RPC for inter-gate communication.
    /// Songbird's HTTP gateway is the covalent bond transport.
    async fn call_via_http(
        &self,
        host: &str,
        port: u16,
        request: JsonRpcRequest,
    ) -> Result<JsonRpcResponse> {
        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(&addr)
            .await
            .context(format!("Failed to connect to HTTP endpoint: {}", addr))?;

        // Serialize JSON-RPC request body
        let body =
            serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

        // Build minimal HTTP/1.1 POST request
        let http_request = format!(
            "POST /jsonrpc HTTP/1.1\r\n\
             Host: {}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\
             \r\n\
             {}",
            addr,
            body.len(),
            body
        );

        trace!("Sending HTTP JSON-RPC request to {}", addr);

        // Send HTTP request
        stream.write_all(http_request.as_bytes()).await?;
        stream.flush().await?;

        // Read full HTTP response
        let mut response_buf = Vec::new();
        let mut reader = BufReader::new(stream);
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => break, // EOF
                Ok(_) => response_buf.push(line),
                Err(e) => {
                    // Connection closed after body -- expected with Connection: close
                    if response_buf.is_empty() {
                        return Err(e).context("Failed to read HTTP response");
                    }
                    break;
                }
            }
        }

        // Find the JSON body after the blank line separator
        let full_response = response_buf.join("");
        let body_start = full_response
            .find("\r\n\r\n")
            .or_else(|| full_response.find("\n\n"))
            .map(|pos| {
                if full_response[pos..].starts_with("\r\n\r\n") {
                    pos + 4
                } else {
                    pos + 2
                }
            })
            .ok_or_else(|| anyhow::anyhow!("Malformed HTTP response: no body separator"))?;

        let json_body = full_response[body_start..].trim();

        trace!("Received HTTP JSON-RPC response: {}", json_body);

        // Parse JSON-RPC response from HTTP body
        let response: JsonRpcResponse = serde_json::from_str(json_body).context(format!(
            "Failed to parse JSON-RPC response from HTTP body: {}",
            &json_body[..json_body.len().min(200)]
        ))?;

        Ok(response)
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
                if endpoint.is_native() {
                    "Tier 1"
                } else {
                    "Tier 2"
                },
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

/// Result of a command execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Standard output from the command
    pub stdout: String,
    /// Standard error output
    pub stderr: String,
    /// Process exit code (if available)
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

// Tests are in atomic_client_tests.rs to keep this file under 1000 lines

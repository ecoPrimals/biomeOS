// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
//! use biomeos_types::constants::capability;
//!
//! // Prefer capability-based discovery (WateringHole standard)
//! let client = AtomicClient::discover_by_capability(capability::CRYPTO).await?;
//!
//! // Or by primal name (when known)
//! let client = AtomicClient::discover("beardog").await?;
//!
//! // Explicit transport
//! let client = AtomicClient::tcp("192.168.1.100", 9100);
//! let client = AtomicClient::unix("/tmp/beardog.sock");
//!
//! let result = client.call("generate_entropy", json!({ "bytes": 32 })).await?;
//! ```

use anyhow::{Context, Result};
use biomeos_types::IpcError;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::{TcpStream, UnixStream};
use tokio::time::timeout;
use tracing::{debug, trace};

// Import the Universal IPC v3.0 transport types
use crate::socket_discovery::{SocketDiscovery, TransportEndpoint};

// Re-export JSON-RPC types from biomeos-types for backwards compatibility
pub use biomeos_types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

// Re-export StreamItem for callers of call_stream
pub use biomeos_graph::StreamItem;

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
/// use biomeos_types::constants::capability;
///
/// // Capability-based discovery (preferred)
/// let client = AtomicClient::discover_by_capability(capability::CRYPTO).await?;
///
/// // Or by primal name when known
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
    /// * `primal_name` - Name of the primal (prefer `discover_by_capability` for capability-based discovery)
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

    /// Discover a primal by capability and create an atomic client
    ///
    /// **WateringHole standard**: No hardcoded primal names. Use capability constants
    /// from `biomeos_types::constants::capability` (e.g., `capability::CRYPTO`).
    ///
    /// # Arguments
    /// * `capability` - Capability to discover (e.g., `capability::CRYPTO`, `capability::STORAGE`)
    ///
    /// # Returns
    /// Ready-to-use atomic client for any primal providing the capability
    pub async fn discover_by_capability(capability: &str) -> Result<Self> {
        debug!("Discovering primal by capability: {}", capability);

        let family_id = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("NODE_FAMILY_ID"))
            .unwrap_or_else(|_| {
                trace!("No FAMILY_ID set, using 'default' for discovery");
                "default".to_string()
            });

        let discovery = SocketDiscovery::new(&family_id);

        // 1. Try capability registry first
        if let Some(socket) = discovery.discover_capability(capability).await {
            debug!(
                "Discovered capability {} via registry: {}",
                capability,
                socket.endpoint.display_string()
            );
            return Ok(Self::from_endpoint(socket.endpoint));
        }

        // 2. Taxonomy bootstrap: resolve capability → primal name, then discover
        if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_err()
            && let Some(primal_name) =
                biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability)
        {
            trace!(
                "Capability '{}' resolved to primal '{}' via taxonomy bootstrap",
                capability, primal_name
            );
            return Self::discover(primal_name).await;
        }

        anyhow::bail!(
            "No primal found for capability '{}'. Ensure a primal with this capability is running.",
            capability
        )
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
    pub fn tcp(host: impl AsRef<str>, port: u16) -> Self {
        Self {
            endpoint: TransportEndpoint::TcpSocket {
                host: Arc::from(host.as_ref()),
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
    /// Songbird's default HTTP port is [`biomeos_types::constants::ports::HTTP_BRIDGE`]
    /// (configurable via `SONGBIRD_HTTP_PORT`).
    ///
    /// # Arguments
    /// * `host` - Remote host address (IP or hostname)
    /// * `port` - Remote Songbird HTTP port (discovered via beacon, default
    ///   [`biomeos_types::constants::ports::HTTP_BRIDGE`])
    pub fn http(host: impl AsRef<str>, port: u16) -> Self {
        Self {
            endpoint: TransportEndpoint::HttpJsonRpc {
                host: Arc::from(host.as_ref()),
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
    pub fn abstract_socket(name: impl AsRef<str>) -> Self {
        Self {
            endpoint: TransportEndpoint::AbstractSocket {
                name: Arc::from(name.as_ref()),
            },
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
            TransportEndpoint::TcpSocket { .. }
            | TransportEndpoint::AbstractSocket { .. }
            | TransportEndpoint::HttpJsonRpc { .. } => true, // availability checked on connect
        }
    }

    /// Get the transport endpoint
    pub fn endpoint(&self) -> &TransportEndpoint {
        &self.endpoint
    }

    /// Call a JSON-RPC method on the primal (returns `anyhow::Result` for backward compatibility).
    ///
    /// **Universal IPC v3.0**: Works across all transport types.
    ///
    /// Use `try_call` for structured `IpcError` when you need to distinguish
    /// timeouts, method-not-found, or connection failures.
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
        self.try_call(method, params).await.map_err(Into::into)
    }

    /// Call a JSON-RPC method on the primal with structured error types.
    ///
    /// Returns `IpcError` for typed handling (e.g. `is_method_not_found()`, `is_timeout()`).
    ///
    /// # Arguments
    /// * `method` - JSON-RPC method name
    /// * `params` - Method parameters as JSON
    ///
    /// # Returns
    /// JSON result from the primal, or `IpcError` on failure
    pub async fn try_call(&self, method: &str, params: Value) -> Result<Value, IpcError> {
        let request = JsonRpcRequest::new(method, params);

        debug!("Calling method '{}' on {}", method, self.endpoint);

        let primal = self.endpoint.to_string();

        // Timeout wrapper for fail-fast behavior
        let response = match timeout(self.timeout, self.call_impl(request)).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => {
                return Err(match e.downcast::<serde_json::Error>() {
                    Ok(serde_err) => IpcError::Serialization(serde_err),
                    Err(e) => IpcError::ConnectionFailed {
                        primal: primal.clone(),
                        source: e,
                    },
                });
            }
            Err(_) => {
                return Err(IpcError::Timeout {
                    primal,
                    timeout_ms: self.timeout.as_millis() as u64,
                });
            }
        };

        // Check for JSON-RPC errors
        if let Some(error) = response.error {
            return Err(IpcError::JsonRpcError {
                primal,
                code: error.code as i32,
                message: error.message,
            });
        }

        response.result.ok_or(IpcError::MissingResult { primal })
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
        let addr = format!("{host}:{port}");
        let stream = TcpStream::connect(&addr)
            .await
            .context(format!("Failed to connect to TCP: {addr}"))?;

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
        let addr = format!("{host}:{port}");
        let mut stream = TcpStream::connect(&addr)
            .await
            .context(format!("Failed to connect to HTTP endpoint: {addr}"))?;

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
            .context(format!("Invalid abstract socket name: {name}"))?;

        // Connect using blocking socket, then wrap in tokio
        // Note: tokio doesn't directly support abstract socket connect
        let std_stream = std::os::unix::net::UnixStream::connect_addr(&addr)
            .context(format!("Failed to connect to abstract socket: @{name}"))?;
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

    /// Call a method that returns a stream of NDJSON items.
    ///
    /// Sends a single JSON-RPC request and reads multiple newline-delimited
    /// responses. Each line is parsed as a `StreamItem`. The stream ends when
    /// the server sends a `StreamItem::End` or closes the connection.
    ///
    /// This leverages the existing NDJSON framing — no new protocol needed.
    /// A primal that supports streaming simply writes multiple lines before
    /// closing the connection.
    ///
    /// Returns an `mpsc::Receiver` that yields `StreamItem`s as they arrive.
    pub async fn call_stream(
        &self,
        method: &str,
        params: Value,
    ) -> Result<tokio::sync::mpsc::Receiver<StreamItem>> {
        let request = JsonRpcRequest::new(method, params);
        let (tx, rx) = tokio::sync::mpsc::channel(64);

        let endpoint = self.endpoint.clone();
        let timeout_dur = self.timeout;
        let socket_path = self.socket_path.clone();

        tokio::spawn(async move {
            let result =
                Self::stream_impl(endpoint, timeout_dur, request, tx.clone(), &socket_path).await;

            if let Err(e) = result {
                let _ = tx
                    .send(StreamItem::Error {
                        node_id: String::new(),
                        message: format!("Stream transport error: {e}"),
                    })
                    .await;
                let _ = tx.send(StreamItem::End).await;
            }
        });

        Ok(rx)
    }

    /// Internal streaming implementation over any transport.
    async fn stream_impl(
        endpoint: TransportEndpoint,
        timeout_dur: Duration,
        request: JsonRpcRequest,
        tx: tokio::sync::mpsc::Sender<StreamItem>,
        socket_path: &Path,
    ) -> Result<()> {
        match &endpoint {
            TransportEndpoint::UnixSocket { path } => {
                let stream = timeout(timeout_dur, UnixStream::connect(path))
                    .await
                    .context("Unix connect timeout")?
                    .context(format!("Unix connect: {}", path.display()))?;
                Self::read_stream(stream, request, &tx).await
            }
            TransportEndpoint::TcpSocket { host, port } => {
                let addr = format!("{host}:{port}");
                let stream = timeout(timeout_dur, TcpStream::connect(&addr))
                    .await
                    .context("TCP connect timeout")?
                    .context(format!("TCP connect: {addr}"))?;
                Self::read_stream(stream, request, &tx).await
            }
            #[cfg(target_os = "linux")]
            TransportEndpoint::AbstractSocket { name } => {
                use std::os::linux::net::SocketAddrExt;
                use std::os::unix::net::SocketAddr;
                let addr = SocketAddr::from_abstract_name(name.as_bytes())?;
                let std_stream = std::os::unix::net::UnixStream::connect_addr(&addr)?;
                std_stream.set_nonblocking(true)?;
                let stream = UnixStream::from_std(std_stream)?;
                Self::read_stream(stream, request, &tx).await
            }
            _ => anyhow::bail!(
                "Streaming not supported for transport: {} (socket: {})",
                endpoint,
                socket_path.display()
            ),
        }
    }

    /// Send a request and read multiple NDJSON response lines as `StreamItem`s.
    async fn read_stream<S>(
        stream: S,
        request: JsonRpcRequest,
        tx: &tokio::sync::mpsc::Sender<StreamItem>,
    ) -> Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let (reader, mut writer) = tokio::io::split(stream);

        let request_str =
            serde_json::to_string(&request).context("Failed to serialize streaming request")?;

        writer.write_all(request_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            line.clear();
            let n = reader.read_line(&mut line).await?;
            if n == 0 {
                let _ = tx.send(StreamItem::End).await;
                break;
            }

            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Try parsing as StreamItem first, fall back to JsonRpcResponse
            if let Ok(item) = serde_json::from_str::<StreamItem>(trimmed) {
                let is_end = matches!(item, StreamItem::End);
                if tx.send(item).await.is_err() {
                    break;
                }
                if is_end {
                    break;
                }
            } else if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(trimmed) {
                // Standard JSON-RPC response — wrap as final StreamItem
                if let Some(result) = resp.result {
                    let _ = tx.send(StreamItem::Data(result)).await;
                }
                let _ = tx.send(StreamItem::End).await;
                break;
            } else {
                // Unrecognized line — forward as raw data
                let _ = tx
                    .send(StreamItem::Data(serde_json::Value::String(
                        trimmed.to_string(),
                    )))
                    .await;
            }
        }

        Ok(())
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

// Tests are in atomic_client_tests.rs to keep this file under 1000 lines

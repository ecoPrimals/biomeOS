// SPDX-License-Identifier: AGPL-3.0-or-later
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
//! let client = AtomicClient::tcp("192.0.2.100", 9100);
//! let client = AtomicClient::unix("/tmp/beardog.sock");
//!
//! let result = client.call("generate_entropy", json!({ "bytes": 32 })).await?;
//! ```

mod atomic_discovery;
mod atomic_rpc;
mod atomic_transport;

use anyhow::Result;
use biomeos_types::IpcError;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, trace};

use crate::socket_discovery::TransportEndpoint;

// Re-export JSON-RPC types from biomeos-types for backwards compatibility
pub use biomeos_types::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};

// Re-export StreamItem for callers of call_stream
pub use biomeos_graph::StreamItem;

/// Options for [`AtomicClient::discover_with_opts`] and [`discover_primal_endpoint_with_opts`].
#[derive(Debug, Default, Clone)]
pub struct DiscoverOpts<'a> {
    /// Override `FAMILY_ID` / `NODE_FAMILY_ID` resolution when set.
    pub family_id: Option<&'a str>,
    /// Per-call `{PRIMAL}_TCP` / socket overrides (same keys as real env vars).
    pub env_overrides: Option<&'a HashMap<String, String>>,
    /// Tier-2 TCP fallback string (replaces `{PRIMAL}_TCP`) when Tier 1 finds nothing.
    pub tcp_tier2_override: Option<&'a str>,
}

/// Options for [`AtomicClient::discover_by_capability_with_opts`].
#[derive(Debug, Default, Clone)]
pub struct DiscoverByCapabilityOpts<'a> {
    /// Override `FAMILY_ID` / `NODE_FAMILY_ID` resolution when set.
    pub family_id: Option<&'a str>,
    /// `Some(true)` = strict (no taxonomy bootstrap); `Some(false)` = allow taxonomy;
    /// `None` = read `BIOMEOS_STRICT_DISCOVERY` from the environment.
    pub strict_discovery: Option<bool>,
}

/// Atomic multi-transport JSON-RPC client (Universal IPC v3.0).
///
/// Design, transport tiers, and usage examples: [`atomic_client`](crate::atomic_client) module docs.
///
/// ```no_run
/// use biomeos_core::AtomicClient;
///
/// let _client = AtomicClient::unix("/tmp/biomeos-example.sock");
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
        Self::discover_with_opts(primal_name, DiscoverOpts::default()).await
    }

    /// Discover a primal with optional [`DiscoverOpts`] (for tests and explicit callers).
    pub async fn discover_with_opts(primal_name: &str, opts: DiscoverOpts<'_>) -> Result<Self> {
        let endpoint = atomic_discovery::discover_named_endpoint(primal_name, opts).await?;
        Ok(Self::from_endpoint(endpoint))
    }

    /// Discover a primal by capability and create an atomic client
    ///
    /// **`WateringHole` standard**: No hardcoded primal names. Use capability constants
    /// from `biomeos_types::constants::capability` (e.g., `capability::CRYPTO`).
    ///
    /// # Arguments
    /// * `capability` - Capability to discover (e.g., `capability::CRYPTO`, `capability::STORAGE`)
    ///
    /// # Returns
    /// Ready-to-use atomic client for any primal providing the capability
    pub async fn discover_by_capability(capability: &str) -> Result<Self> {
        Self::discover_by_capability_with_opts(capability, DiscoverByCapabilityOpts::default())
            .await
    }

    /// Discover by capability with optional overrides (see [`DiscoverByCapabilityOpts`]).
    pub async fn discover_by_capability_with_opts(
        capability: &str,
        opts: DiscoverByCapabilityOpts<'_>,
    ) -> Result<Self> {
        debug!("Discovering primal by capability: {}", capability);

        if let Some(endpoint) =
            atomic_discovery::discover_capability_registry_endpoint(capability, opts.family_id)
                .await
        {
            return Ok(Self::from_endpoint(endpoint));
        }

        if let Some(primal_name) =
            atomic_discovery::taxonomy_primal_for_capability(&opts, capability)
        {
            trace!(
                "Capability '{}' resolved to primal '{}' via taxonomy bootstrap",
                capability, primal_name
            );
            return Self::discover(primal_name.as_str()).await;
        }

        anyhow::bail!(
            "No primal found for capability '{capability}'. Ensure a primal with this capability is running."
        )
    }

    /// Create an atomic client from a transport endpoint
    ///
    /// **Universal IPC v3.0**: Use this with discovered endpoints.
    #[must_use]
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
    /// where filesystem sockets may be blocked (e.g., `SELinux`).
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

    /// Alias for `unix()` - backwards compatibility
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self::unix(socket_path)
    }

    /// Set the request timeout
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Check if the primal is available
    ///
    /// For Unix sockets: checks if the socket file exists
    /// For TCP: returns true (availability checked on connect)
    /// For Abstract: returns true (availability checked on connect)
    #[must_use]
    pub fn is_available(&self) -> bool {
        match &self.endpoint {
            TransportEndpoint::UnixSocket { path } => path.exists(),
            TransportEndpoint::TcpSocket { .. }
            | TransportEndpoint::AbstractSocket { .. }
            | TransportEndpoint::HttpJsonRpc { .. } => true, // availability checked on connect
        }
    }

    /// Get the transport endpoint
    #[must_use]
    pub const fn endpoint(&self) -> &TransportEndpoint {
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

        if let Some(error) = response.error {
            return Err(IpcError::JsonRpcError {
                primal,
                code: error.code as i32,
                message: error.message,
            });
        }

        response.result.ok_or(IpcError::MissingResult { primal })
    }

    /// Call a JSON-RPC method over a BTSP-authenticated channel.
    ///
    /// Only meaningful for `UnixSocket` endpoints pointing at family-scoped
    /// sockets. Falls back to raw JSON-RPC for non-Unix transports.
    pub async fn call_btsp(&self, method: &str, params: Value) -> Result<Value, IpcError> {
        let request = JsonRpcRequest::new(method, params);
        let primal = self.endpoint.to_string();

        let response = match timeout(self.timeout, self.call_btsp_impl(request)).await {
            Ok(Ok(resp)) => resp,
            Ok(Err(e)) => {
                return Err(IpcError::ConnectionFailed { primal, source: e });
            }
            Err(_) => {
                return Err(IpcError::Timeout {
                    primal,
                    timeout_ms: self.timeout.as_millis() as u64,
                });
            }
        };

        if let Some(error) = response.error {
            return Err(IpcError::JsonRpcError {
                primal,
                code: error.code as i32,
                message: error.message,
            });
        }

        response.result.ok_or(IpcError::MissingResult { primal })
    }

    async fn call_btsp_impl(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        match &self.endpoint {
            TransportEndpoint::UnixSocket { path } => {
                atomic_transport::jsonrpc_unix_btsp(path, request).await
            }
            _ => self.call_impl(request).await,
        }
    }

    /// Internal implementation of the JSON-RPC call
    ///
    /// Dispatches to the appropriate transport based on endpoint type.
    async fn call_impl(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        match &self.endpoint {
            TransportEndpoint::UnixSocket { path } => {
                atomic_transport::jsonrpc_unix(path, request).await
            }
            TransportEndpoint::TcpSocket { host, port } => {
                atomic_transport::jsonrpc_tcp(host, *port, request).await
            }
            TransportEndpoint::AbstractSocket { name } => {
                atomic_transport::jsonrpc_abstract(name, request).await
            }
            TransportEndpoint::HttpJsonRpc { host, port } => {
                atomic_transport::jsonrpc_http(host, *port, request).await
            }
        }
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
    pub fn call_stream(
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
                let stream = atomic_transport::connect_unix_timed(path, timeout_dur).await?;
                atomic_rpc::pump_ndjson_stream(stream, request, &tx).await
            }
            TransportEndpoint::TcpSocket { host, port } => {
                let stream = atomic_transport::connect_tcp_timed(host, *port, timeout_dur).await?;
                atomic_rpc::pump_ndjson_stream(stream, request, &tx).await
            }
            #[cfg(target_os = "linux")]
            TransportEndpoint::AbstractSocket { name } => {
                let stream = atomic_transport::connect_abstract(name)?;
                atomic_rpc::pump_ndjson_stream(stream, request, &tx).await
            }
            _ => anyhow::bail!(
                "Streaming not supported for transport: {} (socket: {})",
                endpoint,
                socket_path.display()
            ),
        }
    }

    /// Get the socket path for this client (legacy compatibility)
    ///
    /// Returns empty path for non-Unix transports.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

/// Discover primal endpoint by name using platform-agnostic discovery.
///
/// Delegates to [`AtomicClient::discover`], returning just the endpoint.
pub async fn discover_primal_endpoint(primal_name: &str) -> Result<TransportEndpoint> {
    discover_primal_endpoint_with_opts(primal_name, DiscoverOpts::default()).await
}

/// Like [`discover_primal_endpoint`] with optional [`DiscoverOpts`].
pub async fn discover_primal_endpoint_with_opts(
    primal_name: &str,
    opts: DiscoverOpts<'_>,
) -> Result<TransportEndpoint> {
    let client = AtomicClient::discover_with_opts(primal_name, opts).await?;
    Ok(client.endpoint().clone())
}

// Tests are in atomic_client_tests.rs to keep this file under 1000 lines

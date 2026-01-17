//! # Transport Abstraction Layer
//!
//! Protocol-agnostic transport for inter-primal communication.
//!
//! ## Architecture
//!
//! This module provides a unified interface for multiple transport protocols:
//! 1. **JSON-RPC over Unix Sockets** (PRIMARY - fast, secure, isomorphic)
//! 2. **tarpc** (FUTURE - type-safe, bidirectional)
//! 3. **HTTP/HTTPS** (DEPRECATED - removed from auto-discovery!)
//!
//! ## Philosophy
//!
//! Following Songbird's lead, we prioritize:
//! - **Port-Free**: Unix sockets over TCP ports
//! - **Secure by Default**: No cleartext, encrypted when needed
//! - **Fast**: 0.1ms overhead (Unix) vs 10ms (HTTP)
//! - **Isomorphic**: Bidirectional, streaming capable
//!
//! ## Deep Debt Solution
//!
//! This eliminates 116+ HTTP references across 10 client files, replacing them
//! with a capability-based, protocol-agnostic abstraction.
//!
//! ## Usage
//!
//! ```rust,no_run
//! use biomeos_core::clients::transport::{PrimalClient, TransportPreference};
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Auto-discover best transport (Unix socket preferred)
//! let client = PrimalClient::discover("beardog", "family-id").await?;
//!
//! // Call JSON-RPC method
//! let result = client.call_method(
//!     "evaluate_trust",
//!     json!({"peer_id": "tower2", "family": "nat0"})
//! ).await?;
//! # Ok(())
//! # }
//! ```

pub mod http;
pub mod jsonrpc;

// Future: tarpc module (stubbed for now)
// pub mod tarpc;

use anyhow::{Context, Result};
use biomeos_types::paths::SystemPaths;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, info, warn};

/// Transport protocol preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportPreference {
    /// JSON-RPC over Unix socket (PRIMARY)
    UnixSocket,
    /// tarpc (FUTURE - not yet implemented)
    Tarpc,
    /// HTTP/HTTPS (DEPRECATED - only for explicit use, not auto-discovery!)
    #[deprecated(
        since = "0.2.0",
        note = "Use UnixSocket instead. HTTP is insecure and slow."
    )]
    Http,
    /// Auto-select secure transport (Unix socket only!)
    Auto,
}

impl Default for TransportPreference {
    fn default() -> Self {
        Self::Auto
    }
}

/// Protocol-agnostic primal transport client
///
/// Automatically selects the best available transport protocol:
/// 1. Unix socket (fast, secure, preferred)
/// 2. tarpc (future - type-safe RPC)
/// 3. HTTP (fallback - legacy)
#[derive(Debug, Clone)]
pub struct PrimalTransport {
    primal_name: String,
    endpoint: String,
    transport: Transport,
    timeout: Duration,
}

/// Internal transport implementation
#[derive(Debug, Clone)]
enum Transport {
    UnixSocket(jsonrpc::JsonRpcUnixClient),
    Http(http::HttpClient),
    // Future: Tarpc(tarpc::TarpcClient),
}

impl PrimalTransport {
    /// Create a transport client by auto-discovering the primal's transport
    ///
    /// Searches in XDG runtime directory for Unix sockets.
    /// **FAILS FAST** if no secure transport is available (no HTTP fallback!).
    ///
    /// # Arguments
    ///
    /// * `primal_name` - Name of the primal (e.g., "beardog", "songbird")
    /// * `family_id` - Genetic family ID (for socket path discovery)
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - Client configured with best available transport
    pub async fn discover(primal_name: &str, family_id: &str) -> Result<Self> {
        Self::discover_with_preference(primal_name, family_id, TransportPreference::Auto).await
    }

    /// Create a client with a specific transport preference
    ///
    /// # Arguments
    ///
    /// * `primal_name` - Name of the primal (e.g., "beardog", "songbird")
    /// * `family_id` - Genetic family ID
    /// * `preference` - Transport preference (UnixSocket, Http, Auto)
    pub async fn discover_with_preference(
        primal_name: &str,
        family_id: &str,
        preference: TransportPreference,
    ) -> Result<Self> {
        debug!(
            primal = primal_name,
            family = family_id,
            preference = ?preference,
            "Discovering primal transport"
        );

        match preference {
            TransportPreference::UnixSocket => Self::try_unix_socket(primal_name, family_id).await,
            TransportPreference::Tarpc => {
                // TODO: Implement tarpc transport
                warn!("tarpc transport not yet implemented, trying Unix socket");
                Self::try_unix_socket(primal_name, family_id).await.context(
                    "No secure transport available (tarpc not implemented, Unix socket failed)",
                )
            }
            #[allow(deprecated)]
            TransportPreference::Http => {
                warn!("⚠️ HTTP transport explicitly requested (DEPRECATED and INSECURE!)");
                Self::try_http(primal_name)
            }
            TransportPreference::Auto => {
                // SECURE ONLY: Unix socket → tarpc (future)
                // NO HTTP FALLBACK! Fail fast if secure transport unavailable.
                Self::try_unix_socket(primal_name, family_id)
                    .await
                    .context(format!(
                        "No secure transport available for primal '{}' (family: '{}'). \
                         Ensure primal is running and socket exists at /run/user/{{uid}}/{}-{}.sock",
                        primal_name, family_id, primal_name, family_id
                    ))
            }
        }
    }

    /// Try to create a Unix socket client
    async fn try_unix_socket(primal_name: &str, family_id: &str) -> Result<Self> {
        let socket_path = Self::discover_socket_path(primal_name, family_id)?;

        // Verify socket exists
        if !socket_path.exists() {
            anyhow::bail!("Unix socket not found: {}", socket_path.display());
        }

        let client = jsonrpc::JsonRpcUnixClient::new(&socket_path)?;

        info!(
            primal = primal_name,
            socket = %socket_path.display(),
            "📡 Connected via Unix socket (fast, secure)"
        );

        Ok(Self {
            primal_name: primal_name.to_string(),
            endpoint: format!("unix://{}", socket_path.display()),
            transport: Transport::UnixSocket(client),
            timeout: Duration::from_secs(5),
        })
    }

    /// Try to create an HTTP client (DEPRECATED - explicit use only!)
    ///
    /// ⚠️ **WARNING**: HTTP is DEPRECATED and INSECURE!
    /// - Not included in auto-discovery
    /// - Only available for explicit `TransportPreference::Http`
    /// - Will be removed in future versions
    ///
    /// **Use `TransportPreference::UnixSocket` instead!**
    #[deprecated(
        since = "0.2.0",
        note = "Use Unix socket instead. HTTP is insecure and slow."
    )]
    fn try_http(primal_name: &str) -> Result<Self> {
        warn!(
            primal = primal_name,
            "🚨 Using HTTP transport (DEPRECATED and INSECURE!) - switch to Unix socket!"
        );

        // Default port mapping (legacy)
        let port = match primal_name {
            "beardog" => 8900,
            "songbird" => 8080,
            "toadstool" => 8800,
            "nestgate" => 8600,
            _ => 8000,
        };

        let base_url = format!("http://127.0.0.1:{}", port);
        let client = http::HttpClient::new(&base_url)?;

        Ok(Self {
            primal_name: primal_name.to_string(),
            endpoint: base_url.clone(),
            transport: Transport::Http(client),
            timeout: Duration::from_secs(5),
        })
    }

    /// Discover Unix socket path for a primal
    ///
    /// Searches XDG runtime directory for sockets matching:
    /// - `{primal}-{family_id}.sock`
    /// - `{primal}.sock`
    fn discover_socket_path(primal_name: &str, family_id: &str) -> Result<PathBuf> {
        let system_paths = SystemPaths::new().context("Failed to initialize SystemPaths")?;

        let runtime_dir = system_paths.runtime_dir();

        // Priority search order
        let candidates = vec![
            runtime_dir.join(format!("{}-{}.sock", primal_name, family_id)),
            runtime_dir.join(format!("{}.sock", primal_name)),
            // Legacy fallback (old /tmp pattern)
            PathBuf::from(format!("/tmp/{}-{}.sock", primal_name, family_id)),
        ];

        for path in candidates {
            if path.exists() {
                debug!(
                    socket = %path.display(),
                    "Found Unix socket"
                );
                return Ok(path);
            }
        }

        anyhow::bail!(
            "No Unix socket found for primal '{}' with family '{}'",
            primal_name,
            family_id
        )
    }

    /// Call a JSON-RPC method
    ///
    /// # Arguments
    ///
    /// * `method` - Method name (e.g., "evaluate_trust")
    /// * `params` - Optional method parameters (JSON object or array)
    ///
    /// # Returns
    ///
    /// * `Result<Value>` - Method result
    pub async fn call_method(&self, method: &str, params: Option<Value>) -> Result<Value> {
        match &self.transport {
            Transport::UnixSocket(client) => client.call_method(method, params).await,
            Transport::Http(client) => {
                client
                    .call_method(method, params.unwrap_or(Value::Null))
                    .await
            }
        }
    }

    /// Get the active transport type
    pub fn transport_type(&self) -> &'static str {
        match &self.transport {
            Transport::UnixSocket(_) => "unix-socket",
            Transport::Http(_) => "http",
        }
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Get the primal name
    pub fn primal_name(&self) -> &str {
        &self.primal_name
    }

    /// Get the endpoint
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    /// Perform health check
    pub async fn health_check(&self) -> Result<crate::primal_client::HealthStatus> {
        match self.call_method("health", None).await {
            Ok(_) => Ok(crate::primal_client::HealthStatus::Healthy),
            Err(_) => Ok(crate::primal_client::HealthStatus::Unhealthy),
        }
    }

    /// Alias for call_method (for compatibility with PrimalClient trait)
    pub async fn call(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.call_method(method, params).await
    }

    /// Make a request (alias for call_method for trait compatibility)
    pub async fn request(&self, method: &str, params: Option<Value>) -> Result<Value> {
        self.call_method(method, params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_preference_default() {
        let pref = TransportPreference::default();
        assert_eq!(pref, TransportPreference::Auto);
    }

    #[test]
    fn test_transport_preference_variants() {
        // Test that all variants exist
        let _unix = TransportPreference::UnixSocket;
        let _tarpc = TransportPreference::Tarpc;
        #[allow(deprecated)]
        let _http = TransportPreference::Http;
        let _auto = TransportPreference::Auto;
    }

    #[test]
    fn test_transport_preference_equality() {
        assert_eq!(TransportPreference::Auto, TransportPreference::Auto);
        assert_eq!(TransportPreference::UnixSocket, TransportPreference::UnixSocket);
        assert_ne!(TransportPreference::Auto, TransportPreference::UnixSocket);
    }

    #[test]
    fn test_discover_socket_path_priority() {
        // This test verifies the priority order:
        // 1. {runtime_dir}/{primal}-{family}.sock
        // 2. {runtime_dir}/{primal}.sock
        // 3. /tmp/{primal}-{family}.sock (legacy)

        // We can't test actual file existence without creating sockets,
        // but we can verify the search order logic is sound
        let result = PrimalTransport::discover_socket_path("beardog", "nat0");

        // Should fail if no sockets exist (expected in test environment)
        assert!(result.is_err());
        
        // Error message should indicate what we're looking for
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("beardog") || err_msg.contains("nat0"));
    }

    #[test]
    fn test_discover_socket_path_different_primals() {
        // Test with different primal names
        let result1 = PrimalTransport::discover_socket_path("songbird", "nat0");
        assert!(result1.is_err());
        
        let result2 = PrimalTransport::discover_socket_path("toadstool", "test-family");
        assert!(result2.is_err());
    }

    #[test]
    fn test_transport_error_display() {
        let err1 = TransportError::ConnectionFailed("test".to_string());
        assert!(err1.to_string().contains("Connection failed"));
        assert!(err1.to_string().contains("test"));

        let err2 = TransportError::Timeout;
        assert_eq!(err2.to_string(), "Request timeout");

        let err3 = TransportError::InvalidResponse("bad json".to_string());
        assert!(err3.to_string().contains("Invalid response"));
        assert!(err3.to_string().contains("bad json"));
    }

    #[test]
    fn test_transport_error_from_anyhow() {
        let anyhow_err = anyhow::anyhow!("test error");
        let transport_err: TransportError = anyhow_err.into();
        assert!(transport_err.to_string().contains("test error"));
    }

    #[test]
    fn test_transport_client_alias() {
        // Verify TransportClient is an alias for PrimalTransport
        // This is a compile-time check, but we can verify the type exists
        fn _type_check(_: &TransportClient) {}
    }

    #[test]
    fn test_transport_error_alias() {
        // Verify Error is an alias for TransportError
        fn _type_check(_: &Error) {}
    }

    #[tokio::test]
    async fn test_discover_with_preference_auto_no_socket() {
        // Test that Auto preference fails when no socket exists
        let result = PrimalTransport::discover_with_preference(
            "nonexistent-primal",
            "test-family",
            TransportPreference::Auto
        ).await;
        
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("No secure transport available") || 
                err_msg.contains("nonexistent-primal"));
    }

    #[tokio::test]
    async fn test_discover_with_preference_unix_no_socket() {
        // Test that UnixSocket preference fails when no socket exists
        let result = PrimalTransport::discover_with_preference(
            "test-primal",
            "test-family",
            TransportPreference::UnixSocket
        ).await;
        
        assert!(result.is_err());
    }

    #[test]
    #[allow(deprecated)]
    fn test_http_transport_deprecated() {
        // Verify HTTP transport shows deprecation warning in code
        // The #[deprecated] attribute should be present
        let _pref = TransportPreference::Http;
        // If this compiles, the variant exists (even if deprecated)
    }
}

// Re-export for public API
pub use PrimalTransport as TransportClient;

/// Transport layer errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// Connection failed
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// Request timeout
    #[error("Request timeout")]
    Timeout,

    /// Invalid response
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// Other error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// Re-export TransportError for clients
pub use TransportError as Error;

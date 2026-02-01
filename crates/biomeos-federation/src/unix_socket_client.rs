//! Isomorphic JSON-RPC client for primal communication
//!
//! **TRUE ecoBin v2.0:** Platform-agnostic IPC with automatic endpoint discovery.
//!
//! Implements JSON-RPC 2.0 client with automatic transport selection:
//! - Linux/macOS: Unix sockets (optimal)
//! - Android: TCP with XDG discovery file lookup
//! - Windows: Named pipes (future)
//!
//! This implements the Try→Detect→Adapt→Succeed pattern from songbird.

use anyhow::{Context, Result};
use biomeos_core::ipc::{detect_best_transport, Transport, TransportType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{debug, error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Value,
    pub id: u64,
}

impl JsonRpcRequest {
    pub fn new(method: impl Into<String>, params: Value) -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
            id: REQUEST_ID.fetch_add(1, Ordering::SeqCst),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Isomorphic JSON-RPC client with automatic transport detection
///
/// **TRUE ecoBin v2.0:** Platform-agnostic client that adapts to any transport.
///
/// # Isomorphism
///
/// The client automatically discovers the best available transport:
/// 1. **Try**: Attempt optimal transport (Unix socket at default path)
/// 2. **Detect**: Check for TCP discovery file if Unix socket unavailable
/// 3. **Adapt**: Connect via discovered TCP endpoint
/// 4. **Succeed**: Client connects regardless of platform
pub struct IsomorphicClient {
    socket_path: PathBuf,
}

impl IsomorphicClient {
    /// Create a new isomorphic client
    ///
    /// The client will automatically discover the best transport for this primal.
    pub fn new(socket_path: impl AsRef<Path>) -> Self {
        Self {
            socket_path: socket_path.as_ref().to_path_buf(),
        }
    }

    /// Check if any transport is available (Unix socket or TCP via discovery)
    pub fn is_available(&self) -> bool {
        // Try Unix socket first
        if self.socket_path.exists() {
            return true;
        }

        // Extract service name from socket path
        if let Some(filename) = self.socket_path.file_stem() {
            if let Some(service_name) = filename.to_str() {
                // Try detecting via discovery file
                if detect_best_transport(service_name).is_ok() {
                    return true;
                }
            }
        }

        false
    }

    /// Discover and connect to the primal's transport
    ///
    /// This implements automatic endpoint discovery:
    /// - First tries Unix socket at specified path
    /// - Falls back to reading TCP discovery file from XDG runtime dir
    async fn connect(&self) -> Result<Box<dyn biomeos_core::ipc::AsyncReadWrite>> {
        debug!("Discovering transport for: {}", self.socket_path.display());

        // Try optimal transport (Unix socket) first
        let transport = Transport::new(TransportType::UnixSocket {
            path: self.socket_path.clone(),
        });

        match transport.connect().await {
            Ok(stream) => {
                info!("✅ Connected via Unix socket: {}", self.socket_path.display());
                Ok(stream)
            }
            Err(_) => {
                // Try discovering via service name
                debug!("Unix socket unavailable, checking for TCP discovery file");
                
                let service_name = self
                    .socket_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid socket path: no filename"))?;
                
                match detect_best_transport(service_name) {
                    Ok(discovered_transport) => {
                        info!("📡 Discovered transport via detection");
                        discovered_transport.connect().await
                            .context("Failed to connect via discovered transport")
                    }
                    Err(e) => {
                        Err(anyhow::anyhow!(
                            "No transport available for {}: {}",
                            self.socket_path.display(),
                            e
                        ))
                    }
                }
            }
        }
    }

    /// Send a JSON-RPC request and receive response
    pub async fn call(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        // Connect via automatic discovery
        let stream = self.connect().await?;

        // Split stream for reading/writing
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        // Serialize request
        let request_str =
            serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

        debug!("Sending JSON-RPC request: {}", request_str);

        // Send request (newline-delimited)
        use tokio::io::AsyncWriteExt;
        writer.write_all(request_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        // Read response (newline-delimited)
        let mut line = String::new();
        use tokio::io::AsyncReadExt;
        reader
            .read_line(&mut line)
            .await
            .context("Failed to read JSON-RPC response")?;

        debug!("Received JSON-RPC response: {}", line);

        // Parse response
        let response: JsonRpcResponse =
            serde_json::from_str(&line).context("Failed to parse JSON-RPC response")?;

        // Check for errors
        if let Some(error) = &response.error {
            error!(
                "JSON-RPC error: code={}, message={}",
                error.code, error.message
            );
            return Err(anyhow::anyhow!(
                "JSON-RPC error: {} (code {})",
                error.message,
                error.code
            ));
        }

        Ok(response)
    }

    /// Helper to call a method and extract result
    pub async fn call_method(&self, method: impl Into<String>, params: Value) -> Result<Value> {
        let request = JsonRpcRequest::new(method, params);
        let response = self.call(request).await?;

        response
            .result
            .ok_or_else(|| anyhow::anyhow!("JSON-RPC response missing result field"))
    }
}

/// Legacy alias for backward compatibility
///
/// **DEPRECATED**: Use `IsomorphicClient` directly for TRUE ecoBin compliance.
pub type UnixSocketClient = IsomorphicClient;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_json_rpc_request_creation() {
        let request = JsonRpcRequest::new("test.method", json!({"key": "value"}));
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.method, "test.method");
        assert_eq!(request.params["key"], "value");
        assert!(request.id > 0);
    }

    #[test]
    fn test_isomorphic_client_creation() {
        let client = IsomorphicClient::new("/tmp/test.sock");
        assert_eq!(client.socket_path, PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_legacy_unix_socket_client_alias() {
        // Verify backward compatibility alias works
        let client = UnixSocketClient::new("/tmp/test.sock");
        assert_eq!(client.socket_path, PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_socket_availability() {
        let client = IsomorphicClient::new("/tmp/nonexistent.sock");
        // Will be false if neither Unix socket nor TCP discovery file exists
        assert!(!client.is_available());
    }
}

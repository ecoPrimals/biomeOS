//! Runtime primal discovery
//!
//! This module provides runtime discovery of primals without hardcoding.
//! Primals self-report their capabilities and are discovered dynamically.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

use crate::capability::{Capability, CapabilitySet};
use crate::FederationResult;

/// Primal information returned from query
#[derive(Debug, Clone)]
struct PrimalInfo {
    name: String,
    primal_type: String,
    capabilities: CapabilitySet,
}

/// A discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal name (e.g., "songbird", "beardog")
    pub name: String,

    /// Primal type/family (e.g., "federation", "security", "storage")
    pub primal_type: String,

    /// Capabilities this primal provides
    pub capabilities: CapabilitySet,

    /// Connection endpoints (Unix socket, UDP, HTTP fallback)
    pub endpoints: Vec<PrimalEndpoint>,

    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Endpoint types for primal communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PrimalEndpoint {
    UnixSocket { path: PathBuf },
    Udp { addr: SocketAddr },
    Http { url: String },
}

/// Primal discovery system
pub struct PrimalDiscovery {
    discovered_primals: HashMap<String, DiscoveredPrimal>,
}

impl PrimalDiscovery {
    /// Create a new primal discovery system
    pub fn new() -> Self {
        info!("Initializing primal discovery system");
        Self {
            discovered_primals: HashMap::new(),
        }
    }

    /// Discover primals at runtime
    ///
    /// This uses multiple discovery methods:
    /// 1. Unix socket scanning (/tmp/*.sock)
    /// 2. UDP multicast (Songbird discovery)
    /// 3. Environment variables (PRIMAL_* endpoints)
    pub async fn discover(&mut self) -> FederationResult<Vec<DiscoveredPrimal>> {
        info!("Starting primal discovery");

        // 1. Discover via Unix sockets
        self.discover_unix_sockets().await?;

        // 2. Discover via environment variables
        self.discover_from_env()?;

        // 3. TODO: Discover via Songbird UDP multicast (requires Songbird integration)
        // This would use Songbird's discovery API to find other nodes

        debug!("Discovered {} primals", self.discovered_primals.len());

        Ok(self.discovered_primals.values().cloned().collect())
    }

    /// Discover primals via Unix sockets
    async fn discover_unix_sockets(&mut self) -> FederationResult<()> {
        let socket_dir = PathBuf::from("/tmp");

        if !socket_dir.exists() {
            warn!("Socket directory does not exist: {}", socket_dir.display());
            return Ok(());
        }

        let mut entries = tokio::fs::read_dir(&socket_dir).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to read socket dir: {}", e))
        })?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to read entry: {}", e))
        })? {
            let path = entry.path();

            // Look for biomeOS primal sockets (e.g., songbird-*.sock, beardog-*.sock)
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(".sock") {
                    self.register_unix_socket_primal(&path).await;
                }
            }
        }

        Ok(())
    }

    /// Query a primal for its info via JSON-RPC
    ///
    /// This implements the TRUE PRIMAL principle: primals announce their own identity
    async fn query_primal_info(&self, socket_path: &PathBuf) -> FederationResult<PrimalInfo> {
        // Try to connect to the primal's socket
        let stream = UnixStream::connect(socket_path).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!(
                "Failed to connect to {}: {}",
                socket_path.display(),
                e
            ))
        })?;

        // Query for primal info via JSON-RPC
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "get_primal_info",
            "params": {},
            "id": 1
        });

        let request_bytes = serde_json::to_vec(&request).map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to serialize request: {}", e))
        })?;

        // Send request (simplified - production would use proper framing)
        let (mut read_half, mut write_half) = stream.into_split();
        write_half.write_all(&request_bytes).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to write request: {}", e))
        })?;
        write_half.write_all(b"\n").await.ok(); // Newline delimiter

        // Read response (simplified)
        let mut response_bytes = vec![0u8; 4096];
        let n = read_half.read(&mut response_bytes).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to read response: {}", e))
        })?;

        response_bytes.truncate(n);
        let response: serde_json::Value = serde_json::from_slice(&response_bytes).map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to parse response: {}", e))
        })?;

        // Extract primal info from response
        let result = response.get("result").ok_or_else(|| {
            crate::FederationError::DiscoveryError("No result in response".to_string())
        })?;

        let name = result
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let primal_type = result
            .get("primal_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse capabilities
        let caps = result
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| Capability::Custom(s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(PrimalInfo {
            name,
            primal_type,
            capabilities: CapabilitySet::from_vec(caps),
        })
    }

    /// Register a primal from a Unix socket
    ///
    /// EVOLUTION: Now query-based, not name-based!
    /// Primals announce their own identity and capabilities.
    async fn register_unix_socket_primal(&mut self, socket_path: &PathBuf) {
        let filename = socket_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        // Extract initial name from filename for logging/fallback only
        let socket_name = filename
            .split('-')
            .next()
            .unwrap_or(filename)
            .trim_end_matches(".sock")
            .to_string();

        // EVOLUTION: Query primal for its identity and capabilities
        // Instead of inferring from name, ask the primal directly
        let (primal_name, primal_type, capabilities) =
            match self.query_primal_info(socket_path).await {
                Ok(info) => (info.name, info.primal_type, info.capabilities),
                Err(e) => {
                    debug!(
                        "Could not query primal info from {}: {}. Using fallback.",
                        socket_path.display(),
                        e
                    );
                    // Fallback: use socket name, unknown type, no capabilities
                    (socket_name, "unknown".to_string(), CapabilitySet::new())
                }
            };

        let discovered = DiscoveredPrimal {
            name: primal_name.clone(),
            primal_type: primal_type.to_string(),
            capabilities,
            endpoints: vec![PrimalEndpoint::UnixSocket {
                path: socket_path.clone(),
            }],
            metadata: HashMap::from([
                ("discovered_via".to_string(), "unix_socket".to_string()),
                ("socket_path".to_string(), socket_path.display().to_string()),
            ]),
        };

        debug!(
            "Discovered primal: {} at {}",
            primal_name,
            socket_path.display()
        );
        self.discovered_primals.insert(primal_name, discovered);
    }

    /// Discover primals from environment variables
    fn discover_from_env(&mut self) -> FederationResult<()> {
        // Look for PRIMAL_*_ENDPOINT environment variables
        for (key, value) in std::env::vars() {
            if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
                let primal_name = key
                    .strip_prefix("PRIMAL_")
                    .and_then(|s| s.strip_suffix("_ENDPOINT"))
                    .unwrap_or("unknown")
                    .to_lowercase();

                // Parse endpoint
                if let Some(endpoint) = Self::parse_endpoint(&value) {
                    let discovered = DiscoveredPrimal {
                        name: primal_name.clone(),
                        primal_type: "unknown".to_string(),
                        capabilities: CapabilitySet::new(),
                        endpoints: vec![endpoint],
                        metadata: HashMap::from([
                            ("discovered_via".to_string(), "environment".to_string()),
                            ("env_var".to_string(), key.clone()),
                        ]),
                    };

                    debug!("Discovered primal from env: {} = {}", key, value);
                    self.discovered_primals.insert(primal_name, discovered);
                }
            }
        }

        Ok(())
    }

    /// Parse an endpoint string
    fn parse_endpoint(s: &str) -> Option<PrimalEndpoint> {
        if s.starts_with("unix://") {
            let path = s.strip_prefix("unix://").unwrap_or(s);
            Some(PrimalEndpoint::UnixSocket {
                path: PathBuf::from(path),
            })
        } else if s.starts_with("udp://") {
            let addr_str = s.strip_prefix("udp://").unwrap_or(s);
            addr_str
                .parse::<SocketAddr>()
                .ok()
                .map(|addr| PrimalEndpoint::Udp { addr })
        } else if s.starts_with("http://") || s.starts_with("https://") {
            Some(PrimalEndpoint::Http { url: s.to_string() })
        } else {
            None
        }
    }

    /// Get a discovered primal by name
    pub fn get(&self, name: &str) -> Option<&DiscoveredPrimal> {
        self.discovered_primals.get(name)
    }

    /// Get all discovered primals
    pub fn all(&self) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals.values().collect()
    }

    /// Get primals with a specific capability
    pub fn with_capability(&self, cap: &Capability) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals
            .values()
            .filter(|p| p.capabilities.has(cap))
            .collect()
    }
}

impl Default for PrimalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_parsing() {
        let unix_ep = PrimalDiscovery::parse_endpoint("unix:///tmp/test.sock");
        assert!(matches!(unix_ep, Some(PrimalEndpoint::UnixSocket { .. })));

        let udp_ep = PrimalDiscovery::parse_endpoint("udp://127.0.0.1:8080");
        assert!(matches!(udp_ep, Some(PrimalEndpoint::Udp { .. })));

        let http_ep = PrimalDiscovery::parse_endpoint("http://localhost:3000");
        assert!(matches!(http_ep, Some(PrimalEndpoint::Http { .. })));
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Runtime primal discovery
//!
//! This module provides runtime discovery of primals without hardcoding.
//! Primals self-report their capabilities and are discovered dynamically.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info, warn};

use crate::FederationResult;
use crate::capability::{Capability, CapabilitySet};

/// Primal information returned from query
#[derive(Debug, Clone)]
pub(crate) struct PrimalInfo {
    pub name: String,
    pub primal_type: String,
    pub capabilities: CapabilitySet,
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
    /// Unix domain socket endpoint
    UnixSocket {
        /// Path to the socket file
        path: PathBuf,
    },
    /// UDP endpoint
    Udp {
        /// Socket address (ip:port)
        addr: SocketAddr,
    },
    /// HTTP endpoint (fallback transport per IPC Protocol v3.0 Tier 2)
    Http {
        /// Full URL including scheme
        url: String,
    },
}

/// Primal discovery system
pub struct PrimalDiscovery {
    pub(crate) discovered_primals: HashMap<String, DiscoveredPrimal>,
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
    pub async fn discover(&mut self) -> FederationResult<Vec<DiscoveredPrimal>> {
        info!("Starting primal discovery");

        self.discover_unix_sockets().await?;
        self.discover_from_env()?;

        if let Err(e) = self.discover_via_discovery_provider().await {
            debug!("Discovery provider path unavailable: {} (non-fatal)", e);
        }

        debug!("Discovered {} primals", self.discovered_primals.len());

        Ok(self.discovered_primals.values().cloned().collect())
    }

    async fn discover_unix_sockets(&mut self) -> FederationResult<()> {
        let socket_dir = biomeos_types::paths::SystemPaths::new_lazy()
            .runtime_dir()
            .to_path_buf();
        self.discover_unix_sockets_in(&socket_dir).await
    }

    pub(crate) async fn discover_unix_sockets_in(
        &mut self,
        socket_dir: &std::path::Path,
    ) -> FederationResult<()> {
        if !socket_dir.exists() {
            warn!("Socket directory does not exist: {}", socket_dir.display());
            return Ok(());
        }

        // Non-fatal: directory may vanish between exists() check and read_dir() (TOCTOU),
        // or have restrictive permissions under instrumented builds.
        let mut entries = match tokio::fs::read_dir(socket_dir).await {
            Ok(entries) => entries,
            Err(e) => {
                warn!(
                    "Cannot read socket dir {}: {e} (non-fatal)",
                    socket_dir.display()
                );
                return Ok(());
            }
        };

        while let Some(entry) = entries.next_entry().await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to read entry: {e}"))
        })? {
            let path = entry.path();

            if let Some(filename) = path.file_name().and_then(|n| n.to_str())
                && std::path::Path::new(filename)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
            {
                self.register_unix_socket_primal(&path).await;
            }
        }

        Ok(())
    }

    async fn query_primal_info(&self, socket_path: &PathBuf) -> FederationResult<PrimalInfo> {
        let stream = UnixStream::connect(socket_path).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!(
                "Failed to connect to {}: {}",
                socket_path.display(),
                e
            ))
        })?;

        let request = biomeos_types::JsonRpcRequest::new("identity.info", serde_json::json!({}));

        let request_bytes = serde_json::to_vec(&request).map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to serialize request: {e}"))
        })?;

        let (read_half, mut write_half) = stream.into_split();
        write_half.write_all(&request_bytes).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to write request: {e}"))
        })?;
        write_half.write_all(b"\n").await.ok();

        write_half.flush().await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to flush request: {e}"))
        })?;
        write_half.shutdown().await.ok();

        let mut reader = BufReader::new(read_half);
        let mut response_line = String::new();
        let n = reader.read_line(&mut response_line).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to read response: {e}"))
        })?;
        if n == 0 {
            return Err(crate::FederationError::DiscoveryError(
                "Empty response from primal".to_string(),
            ));
        }
        let response: serde_json::Value =
            serde_json::from_str(response_line.trim()).map_err(|e| {
                crate::FederationError::DiscoveryError(format!("Failed to parse response: {e}"))
            })?;

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

    async fn register_unix_socket_primal(&mut self, socket_path: &PathBuf) {
        let filename = socket_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();

        let base = filename.split('-').next().unwrap_or_default();
        let socket_name = base.trim_end_matches(".sock").to_string();

        let (primal_name, primal_type, capabilities) =
            match self.query_primal_info(socket_path).await {
                Ok(info) => (info.name, info.primal_type, info.capabilities),
                Err(e) => {
                    debug!(
                        "Could not query primal info from {}: {}. Using fallback.",
                        socket_path.display(),
                        e
                    );
                    (socket_name, "unknown".to_string(), CapabilitySet::new())
                }
            };

        let discovered = DiscoveredPrimal {
            name: primal_name.clone(),
            primal_type,
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

    fn discover_from_env(&mut self) -> FederationResult<()> {
        for (key, value) in std::env::vars() {
            if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
                self.insert_primal_endpoint_from_env_key(&key, &value)?;
            }
        }

        Ok(())
    }

    /// Register `PRIMAL_*_ENDPOINT` pairs without reading the process environment (tests).
    #[cfg(test)]
    pub(crate) fn discover_from_primal_endpoint_pairs(
        &mut self,
        pairs: &[(&str, &str)],
    ) -> FederationResult<()> {
        for (key, value) in pairs {
            if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
                self.insert_primal_endpoint_from_env_key(key, value)?;
            }
        }
        Ok(())
    }

    fn insert_primal_endpoint_from_env_key(
        &mut self,
        key: &str,
        value: &str,
    ) -> FederationResult<()> {
        let primal_name = key
            .strip_prefix("PRIMAL_")
            .and_then(|s| s.strip_suffix("_ENDPOINT"))
            .unwrap_or_default()
            .to_lowercase();

        if let Some(endpoint) = Self::parse_endpoint(value) {
            let discovered = DiscoveredPrimal {
                name: primal_name.clone(),
                primal_type: "unknown".to_string(),
                capabilities: CapabilitySet::new(),
                endpoints: vec![endpoint],
                metadata: HashMap::from([
                    ("discovered_via".to_string(), "environment".to_string()),
                    ("env_var".to_string(), key.to_string()),
                ]),
            };

            debug!("Discovered primal from env: {} = {}", key, value);
            self.discovered_primals.insert(primal_name, discovered);
        }

        Ok(())
    }

    /// Parse an endpoint string. pub(crate) for tests.
    pub(crate) fn parse_endpoint(s: &str) -> Option<PrimalEndpoint> {
        if let Some(path) = s.strip_prefix("unix://") {
            Some(PrimalEndpoint::UnixSocket {
                path: PathBuf::from(path),
            })
        } else if let Some(addr_str) = s.strip_prefix("udp://") {
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
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&DiscoveredPrimal> {
        self.discovered_primals.get(name)
    }

    /// Get all discovered primals
    #[must_use]
    pub fn all(&self) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals.values().collect()
    }

    /// Get primals with a specific capability
    #[must_use]
    pub fn with_capability(&self, cap: &Capability) -> Vec<&DiscoveredPrimal> {
        self.discovered_primals
            .values()
            .filter(|p| p.capabilities.has(cap))
            .collect()
    }

    pub(crate) async fn discover_via_discovery_provider(&mut self) -> FederationResult<()> {
        let discovery_socket = Self::discover_discovery_provider()?;
        self.discover_via_discovery_socket_path(&discovery_socket)
            .await
    }

    /// Like [`Self::discover_via_discovery_provider`], using an explicit discovery provider Unix socket path (tests).
    pub(crate) async fn discover_via_discovery_socket_path(
        &mut self,
        discovery_socket: &str,
    ) -> FederationResult<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        debug!(
            "Querying discovery provider for UDP-discovered peers: {}",
            discovery_socket
        );

        let stream = UnixStream::connect(discovery_socket).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!(
                "Discovery provider connection failed: {e}"
            ))
        })?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let request = biomeos_types::JsonRpcRequest::new(
            "discovery.list_peers",
            serde_json::json!({
                "include_capabilities": true,
                "include_endpoints": true
            }),
        );

        let request_str = serde_json::to_string(&request)
            .map_err(|e| crate::FederationError::DiscoveryError(format!("JSON error: {e}")))?
            + "\n";

        writer
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| crate::FederationError::DiscoveryError(format!("Write error: {e}")))?;
        writer
            .flush()
            .await
            .map_err(|e| crate::FederationError::DiscoveryError(format!("Flush error: {e}")))?;

        let mut response_line = String::new();
        reader
            .read_line(&mut response_line)
            .await
            .map_err(|e| crate::FederationError::DiscoveryError(format!("Read error: {e}")))?;

        let response: serde_json::Value =
            serde_json::from_str(response_line.trim()).map_err(|e| {
                crate::FederationError::DiscoveryError(format!("JSON parse error: {e}"))
            })?;

        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or_default();
            return Err(crate::FederationError::DiscoveryError(format!(
                "Discovery provider list_peers failed: {msg}"
            )));
        }

        if let Some(result) = response.get("result")
            && let Some(peers) = result.get("peers").and_then(|p| p.as_array())
        {
            for peer in peers {
                self.register_discovery_peer(peer);
            }
            info!(
                "✅ Discovery provider UDP discovery found {} peers",
                peers.len()
            );
        }

        Ok(())
    }

    /// Find discovery provider socket via the 5-tier capability protocol.
    ///
    /// Delegates to [`biomeos_types::capability_discovery::discover_capability_socket`].
    pub(crate) fn discover_discovery_provider() -> FederationResult<String> {
        Self::discover_discovery_provider_with_env(&biomeos_types::capability_discovery::std_env)
    }

    /// Like [`Self::discover_discovery_provider`], with an injectable env lookup (tests).
    pub(crate) fn discover_discovery_provider_with_env(
        env: &dyn Fn(&str) -> Option<String>,
    ) -> FederationResult<String> {
        use biomeos_types::capability_discovery;

        capability_discovery::discover_capability_socket("discovery", env).ok_or_else(|| {
            crate::FederationError::DiscoveryError(
                "Discovery provider socket not found. \
                     Set DISCOVERY_PROVIDER_SOCKET or ensure the discovery provider is running."
                    .to_string(),
            )
        })
    }

    /// Register a peer discovered via the discovery provider's UDP multicast. pub(crate) for tests.
    pub(crate) fn register_discovery_peer(&mut self, peer: &serde_json::Value) {
        let node_id = match peer.get("node_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return,
        };

        let family_id = peer
            .get("family_id")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let capabilities = peer
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| Capability::Custom(s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let mut endpoints = Vec::new();

        if let Some(eps) = peer.get("endpoints") {
            if let Some(unix_socket) = eps.get("unix_socket").and_then(|v| v.as_str()) {
                endpoints.push(PrimalEndpoint::UnixSocket {
                    path: PathBuf::from(unix_socket),
                });
            }

            if let Some(udp_addr) = eps.get("udp").and_then(|v| v.as_str())
                && let Ok(addr) = udp_addr.parse::<SocketAddr>()
            {
                endpoints.push(PrimalEndpoint::Udp { addr });
            }
        }

        let primal_name = node_id.split(':').next().unwrap_or_default().to_string();

        let mut metadata = HashMap::new();
        metadata.insert("family_id".to_string(), family_id);
        metadata.insert("node_id".to_string(), node_id.clone());
        metadata.insert("discovered_via".to_string(), "discovery_udp".to_string());
        metadata.insert("discovered_at".to_string(), chrono::Utc::now().to_rfc3339());

        let discovered = DiscoveredPrimal {
            name: primal_name.clone(),
            primal_type: "remote".to_string(),
            capabilities: CapabilitySet::from_vec(capabilities),
            endpoints,
            metadata,
        };

        debug!("Registered discovery peer: {} (via UDP multicast)", node_id);
        self.discovered_primals.insert(primal_name, discovered);
    }
}

impl Default for PrimalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

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
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
    /// HTTP endpoint (temporary bridge)
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

        if let Err(e) = self.discover_via_songbird().await {
            debug!("Songbird discovery unavailable: {} (non-fatal)", e);
        }

        debug!("Discovered {} primals", self.discovered_primals.len());

        Ok(self.discovered_primals.values().cloned().collect())
    }

    async fn discover_unix_sockets(&mut self) -> FederationResult<()> {
        let socket_dir = biomeos_types::paths::SystemPaths::new_lazy()
            .runtime_dir()
            .to_path_buf();

        if !socket_dir.exists() {
            warn!("Socket directory does not exist: {}", socket_dir.display());
            return Ok(());
        }

        // Non-fatal: directory may vanish between exists() check and read_dir() (TOCTOU),
        // or have restrictive permissions under instrumented builds.
        let mut entries = match tokio::fs::read_dir(&socket_dir).await {
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

        let request = biomeos_types::JsonRpcRequest::new("get_primal_info", serde_json::json!({}));

        let request_bytes = serde_json::to_vec(&request).map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to serialize request: {e}"))
        })?;

        let (mut read_half, mut write_half) = stream.into_split();
        write_half.write_all(&request_bytes).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to write request: {e}"))
        })?;
        write_half.write_all(b"\n").await.ok();

        let mut response_bytes = vec![0u8; 4096];
        let n = read_half.read(&mut response_bytes).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Failed to read response: {e}"))
        })?;

        response_bytes.truncate(n);
        let response: serde_json::Value = serde_json::from_slice(&response_bytes).map_err(|e| {
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
            .unwrap_or("unknown");

        let socket_name = filename
            .split('-')
            .next()
            .unwrap_or(filename)
            .trim_end_matches(".sock")
            .to_string();

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
                let primal_name = key
                    .strip_prefix("PRIMAL_")
                    .and_then(|s| s.strip_suffix("_ENDPOINT"))
                    .unwrap_or("unknown")
                    .to_lowercase();

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

    /// Parse an endpoint string. pub(crate) for tests.
    pub(crate) fn parse_endpoint(s: &str) -> Option<PrimalEndpoint> {
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

    pub(crate) async fn discover_via_songbird(&mut self) -> FederationResult<()> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let songbird_socket = Self::discover_songbird_socket()?;

        debug!(
            "Querying Songbird for UDP-discovered peers: {}",
            songbird_socket
        );

        let stream = UnixStream::connect(&songbird_socket).await.map_err(|e| {
            crate::FederationError::DiscoveryError(format!("Songbird connection failed: {e}"))
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
                .unwrap_or("Unknown");
            return Err(crate::FederationError::DiscoveryError(format!(
                "Songbird discovery failed: {msg}"
            )));
        }

        if let Some(result) = response.get("result")
            && let Some(peers) = result.get("peers").and_then(|p| p.as_array())
        {
            for peer in peers {
                self.register_songbird_peer(peer);
            }
            info!("✅ Songbird UDP discovery found {} peers", peers.len());
        }

        Ok(())
    }

    /// Find discovery provider socket. pub(crate) for tests.
    pub(crate) fn discover_songbird_socket() -> FederationResult<String> {
        use biomeos_types::paths::SystemPaths;

        let provider = std::env::var("DISCOVERY_PROVIDER")
            .unwrap_or_else(|_| biomeos_types::primal_names::SONGBIRD.to_string());

        if let Ok(socket) =
            std::env::var("SONGBIRD_SOCKET").or_else(|_| std::env::var("DISCOVERY_SOCKET"))
        {
            return Ok(socket);
        }

        let paths = SystemPaths::new_lazy();

        if let Ok(family_id) = std::env::var("BIOMEOS_FAMILY_ID") {
            let family_socket = paths.primal_socket(&format!("{provider}-{family_id}"));
            if family_socket.exists() {
                return Ok(family_socket.display().to_string());
            }
        }

        let generic_socket = paths.primal_socket(&provider);
        if generic_socket.exists() {
            return Ok(generic_socket.display().to_string());
        }

        Err(crate::FederationError::DiscoveryError(format!(
            "Discovery provider '{provider}' socket not found. \
             Set SONGBIRD_SOCKET or ensure the discovery provider is running. \
             Checked: XDG runtime dir: {}",
            paths.runtime_dir().display()
        )))
    }

    /// Register a peer discovered via Songbird UDP multicast. pub(crate) for tests.
    pub(crate) fn register_songbird_peer(&mut self, peer: &serde_json::Value) {
        let node_id = match peer.get("node_id").and_then(|v| v.as_str()) {
            Some(id) => id.to_string(),
            None => return,
        };

        let family_id = peer
            .get("family_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
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

        let primal_name = node_id.split(':').next().unwrap_or(&node_id).to_string();

        let mut metadata = HashMap::new();
        metadata.insert("family_id".to_string(), family_id);
        metadata.insert("node_id".to_string(), node_id.clone());
        metadata.insert("discovered_via".to_string(), "songbird_udp".to_string());
        metadata.insert("discovered_at".to_string(), chrono::Utc::now().to_rfc3339());

        let discovered = DiscoveredPrimal {
            name: primal_name.clone(),
            primal_type: "remote".to_string(),
            capabilities: CapabilitySet::from_vec(capabilities),
            endpoints,
            metadata,
        };

        debug!("Registered Songbird peer: {} (via UDP multicast)", node_id);
        self.discovered_primals.insert(primal_name, discovered);
    }
}

impl Default for PrimalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

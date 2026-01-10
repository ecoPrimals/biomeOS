//! Layer 1: Physical Discovery
//!
//! **Delegates to Songbird** - No reimplementation!
//!
//! Songbird handles:
//! - UDP multicast discovery
//! - Primal announcements
//! - Capability indexing
//! - Network topology
//!
//! This layer just coordinates Songbird's existing APIs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info, warn};

use crate::{Endpoint, EndpointType, Error, Result};

/// Discovery request
#[derive(Debug, Clone)]
pub struct DiscoveryRequest {
    /// Capability to search for (e.g., "encryption", "compute")
    pub capability: String,
    /// Optional family ID filter
    pub family: Option<String>,
    /// Optional timeout
    pub timeout: Option<Duration>,
}

/// Discovered primal (from Songbird)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal name (discovered at runtime)
    pub primal: String,
    /// Node ID
    pub node_id: String,
    /// Family ID
    pub family_id: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Endpoints
    pub endpoints: Vec<Endpoint>,
    /// Signature (signed by BearDog)
    pub signature: String,
    /// Timestamp
    pub timestamp: String,
}

/// Physical discovery layer (delegates to Songbird)
#[async_trait]
pub trait PhysicalDiscovery: Send + Sync {
    /// Discover primals by capability
    ///
    /// This delegates to Songbird's `discover_by_capability` API
    async fn discover_by_capability(&self, request: &DiscoveryRequest) -> Result<Vec<DiscoveredPrimal>>;

    /// Discover primals by family
    ///
    /// This delegates to Songbird's `discover_by_family` API
    async fn discover_by_family(&self, family_id: &str) -> Result<Vec<DiscoveredPrimal>>;

    /// Announce this primal's capabilities
    ///
    /// This delegates to Songbird's `announce_capabilities` API
    async fn announce(&self, primal_info: &DiscoveredPrimal) -> Result<()>;
}

/// Discovery layer implementation (talks to Songbird via Unix socket)
pub struct DiscoveryLayer {
    /// Songbird Unix socket path (discovered at runtime, not hardcoded!)
    songbird_socket: Option<String>,
}

impl DiscoveryLayer {
    /// Create a new discovery layer
    ///
    /// **Deep Debt Principle**: Discovers Songbird at runtime, no hardcoding!
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Discovery Layer (delegating to Songbird)");
        
        // Discover Songbird socket (no hardcoded paths!)
        let songbird_socket = Self::discover_songbird_socket().await?;
        
        Ok(Self {
            songbird_socket: Some(songbird_socket),
        })
    }

    /// Discover Songbird's Unix socket
    ///
    /// **Deep Debt Principle**: Runtime discovery, not hardcoded!
    ///
    /// Checks in order:
    /// 1. Environment variable SONGBIRD_SOCKET
    /// 2. Standard runtime directory (/run/user/{uid}/songbird/)
    /// 3. Tmp directory (/tmp/songbird-*.sock)
    async fn discover_songbird_socket() -> Result<String> {
        debug!("Discovering Songbird socket (no hardcoded paths)");

        // 1. Check environment variable
        if let Ok(socket) = std::env::var("SONGBIRD_SOCKET") {
            debug!("Found Songbird socket via SONGBIRD_SOCKET env var: {}", socket);
            return Ok(socket);
        }

        // 2. Check runtime directory
        if let Ok(uid) = std::env::var("UID") {
            let runtime_path = format!("/run/user/{}/songbird/songbird.sock", uid);
            if tokio::fs::metadata(&runtime_path).await.is_ok() {
                debug!("Found Songbird socket in runtime directory: {}", runtime_path);
                return Ok(runtime_path);
            }
        }

        // 3. Check tmp directory (with glob pattern)
        let tmp_pattern = "/tmp/songbird-*.sock";
        debug!("Searching for Songbird socket: {}", tmp_pattern);
        
        // Use tokio::fs to read directory
        let mut read_dir = tokio::fs::read_dir("/tmp").await
            .map_err(|e| Error::discovery_failed(format!("Failed to read /tmp: {}", e), None))?;

        while let Some(entry) = read_dir.next_entry().await
            .map_err(|e| Error::discovery_failed(format!("Failed to read directory entry: {}", e), None))? 
        {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("songbird-") && filename.ends_with(".sock") {
                    debug!("Found Songbird socket: {}", path.display());
                    return Ok(path.to_string_lossy().to_string());
                }
            }
        }

        Err(Error::discovery_failed(
            "Could not discover Songbird socket. Is Songbird running?",
            Some("discovery".to_string()),
        ))
    }

    /// Get Songbird socket path
    fn songbird_socket(&self) -> Result<&str> {
        self.songbird_socket
            .as_deref()
            .ok_or_else(|| Error::discovery_failed("Songbird socket not initialized", None))
    }

    /// Call Songbird JSON-RPC API
    async fn call_songbird_rpc<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let socket_path = self.songbird_socket()?;
        
        debug!("Calling Songbird RPC: {} at {}", method, socket_path);

        // Use crate::client::unix_socket_client for actual implementation
        // (This avoids duplication and delegates to specialized code)
        crate::client::call_unix_socket_rpc(socket_path, method, params).await
    }
}

#[async_trait]
impl PhysicalDiscovery for DiscoveryLayer {
    async fn discover_by_capability(&self, request: &DiscoveryRequest) -> Result<Vec<DiscoveredPrimal>> {
        info!(
            capability = %request.capability,
            family = ?request.family,
            "Discovering primals by capability (via Songbird)"
        );

        let params = serde_json::json!({
            "capability": request.capability,
            "family_id": request.family,
        });

        let response: serde_json::Value = self
            .call_songbird_rpc("discover_by_capability", params)
            .await?;

        // Parse response
        let primals: Vec<DiscoveredPrimal> = serde_json::from_value(
            response.get("primals")
                .ok_or_else(|| Error::invalid_response("songbird", "Missing 'primals' field"))?
                .clone()
        )?;

        info!(count = primals.len(), "Discovered {} primals", primals.len());
        Ok(primals)
    }

    async fn discover_by_family(&self, family_id: &str) -> Result<Vec<DiscoveredPrimal>> {
        info!(family = %family_id, "Discovering primals by family (via Songbird)");

        let params = serde_json::json!({
            "family_id": family_id,
        });

        let response: serde_json::Value = self
            .call_songbird_rpc("discover_by_family", params)
            .await?;

        // Parse response
        let primals: Vec<DiscoveredPrimal> = serde_json::from_value(
            response.get("primals")
                .ok_or_else(|| Error::invalid_response("songbird", "Missing 'primals' field"))?
                .clone()
        )?;

        info!(count = primals.len(), "Discovered {} primals in family", primals.len());
        Ok(primals)
    }

    async fn announce(&self, primal_info: &DiscoveredPrimal) -> Result<()> {
        info!(
            primal = %primal_info.primal,
            node = %primal_info.node_id,
            "Announcing primal capabilities (via Songbird)"
        );

        let params = serde_json::json!({
            "primal_info": primal_info,
        });

        let _response: serde_json::Value = self
            .call_songbird_rpc("announce_capabilities", params)
            .await?;

        info!("Successfully announced capabilities");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test discovery request creation
    #[test]
    fn test_discovery_request() {
        let request = DiscoveryRequest {
            capability: "encryption".to_string(),
            family: Some("nat0".to_string()),
            timeout: Some(Duration::from_secs(5)),
        };

        assert_eq!(request.capability, "encryption");
        assert_eq!(request.family, Some("nat0".to_string()));
    }

    /// Test discovered primal parsing
    #[test]
    fn test_discovered_primal_parsing() {
        let json = r#"{
            "primal": "beardog",
            "node_id": "node-alpha",
            "family_id": "nat0",
            "capabilities": ["encryption", "identity"],
            "endpoints": [
                {"endpoint_type": "unix_socket", "address": "/tmp/beardog.sock"}
            ],
            "signature": "sig123",
            "timestamp": "2026-01-09T00:00:00Z"
        }"#;

        let primal: DiscoveredPrimal = serde_json::from_str(json).unwrap();
        assert_eq!(primal.primal, "beardog");
        assert_eq!(primal.capabilities.len(), 2);
    }
}


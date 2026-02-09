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
//!
//! **Deep Debt Evolution**:
//! - Uses `CapabilityTaxonomy` (enum) instead of strings
//! - Uses `SystemPaths` for XDG compliance
//! - Runtime discovery, no hardcoded paths

use async_trait::async_trait;
use biomeos_types::{CapabilityTaxonomy, SystemPaths};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, info};

use crate::{Endpoint, Error, Result};

/// Discovery request
#[derive(Debug, Clone)]
pub struct DiscoveryRequest {
    /// Capability to search for (using taxonomy!)
    pub capability: CapabilityTaxonomy,
    /// Optional family ID filter
    pub family: Option<String>,
    /// Optional timeout
    pub timeout: Option<Duration>,
}

impl DiscoveryRequest {
    /// Create a new discovery request
    #[must_use]
    pub fn new(capability: CapabilityTaxonomy) -> Self {
        Self {
            capability,
            family: None,
            timeout: None,
        }
    }

    /// Set family filter
    #[must_use]
    pub fn with_family(mut self, family: impl Into<String>) -> Self {
        self.family = Some(family.into());
        self
    }

    /// Set timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
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
    /// Signature (signed by `BearDog`)
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
    async fn discover_by_capability(
        &self,
        request: &DiscoveryRequest,
    ) -> Result<Vec<DiscoveredPrimal>>;

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
    /// System paths (XDG-compliant)
    _paths: SystemPaths,
}

impl DiscoveryLayer {
    /// Create a new discovery layer
    ///
    /// **Deep Debt Principle**: Discovers Songbird at runtime, no hardcoding!
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - System paths cannot be initialized (XDG directories unavailable)
    /// - Songbird socket cannot be discovered (Songbird not running or socket not found)
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Discovery Layer (delegating to Songbird)");

        // Get XDG-compliant paths
        let paths = SystemPaths::new().map_err(|e| {
            Error::discovery_failed(format!("Failed to initialize SystemPaths: {e}"), None)
        })?;

        // Discover Songbird socket (no hardcoded paths!)
        let songbird_socket = Self::discover_songbird_socket(&paths).await?;

        Ok(Self {
            songbird_socket: Some(songbird_socket),
            _paths: paths,
        })
    }

    /// Discover Songbird's Unix socket
    ///
    /// **Deep Debt Evolution**: Uses `SystemPaths`, not hardcoded paths!
    ///
    /// Checks in order:
    /// 1. Environment variable `SONGBIRD_SOCKET`
    /// 2. XDG runtime directory (`SystemPaths`)
    /// 3. Scan runtime directory for songbird-*.sock
    async fn discover_songbird_socket(paths: &SystemPaths) -> Result<String> {
        debug!("Discovering Songbird socket (XDG-compliant, no hardcoded paths)");

        // 1. Check environment variable
        if let Ok(socket) = std::env::var("SONGBIRD_SOCKET") {
            debug!(
                "Found Songbird socket via SONGBIRD_SOCKET env var: {}",
                socket
            );
            return Ok(socket);
        }

        // 2. Try standard songbird socket in runtime directory
        let standard_socket = paths.primal_socket("songbird");
        if tokio::fs::metadata(&standard_socket).await.is_ok() {
            debug!(
                "Found Songbird socket at XDG location: {}",
                standard_socket.display()
            );
            return Ok(standard_socket.to_string_lossy().to_string());
        }

        // 3. Scan runtime directory for any songbird-*.sock
        let runtime_dir = paths.runtime_dir();
        debug!(
            "Scanning runtime directory for Songbird socket: {}",
            runtime_dir.display()
        );

        let mut read_dir = tokio::fs::read_dir(runtime_dir).await.map_err(|e| {
            Error::discovery_failed(format!("Failed to read runtime dir: {e}"), None)
        })?;

        while let Some(entry) = read_dir.next_entry().await.map_err(|e| {
            Error::discovery_failed(format!("Failed to read directory entry: {e}"), None)
        })? {
            let path = entry.path();
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.starts_with("songbird-")
                    && std::path::Path::new(filename)
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                {
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
    async fn discover_by_capability(
        &self,
        request: &DiscoveryRequest,
    ) -> Result<Vec<DiscoveredPrimal>> {
        info!(
            capability = %request.capability,
            family = ?request.family,
            "Discovering primals by capability (via Songbird)"
        );

        // Convert taxonomy to string for Songbird API
        let capability_str = request.capability.to_string();

        let params = serde_json::json!({
            "capability": capability_str,
            "family_id": request.family,
        });

        let response: serde_json::Value = self
            .call_songbird_rpc("discover_by_capability", params)
            .await?;

        // Parse response
        let primals: Vec<DiscoveredPrimal> = serde_json::from_value(
            response
                .get("primals")
                .ok_or_else(|| Error::invalid_response("songbird", "Missing 'primals' field"))?
                .clone(),
        )?;

        info!(
            count = primals.len(),
            capability = %request.capability,
            "Discovered {} primals with capability",
            primals.len()
        );
        Ok(primals)
    }

    async fn discover_by_family(&self, family_id: &str) -> Result<Vec<DiscoveredPrimal>> {
        info!(family = %family_id, "Discovering primals by family (via Songbird)");

        let params = serde_json::json!({
            "family_id": family_id,
        });

        let response: serde_json::Value =
            self.call_songbird_rpc("discover_by_family", params).await?;

        // Parse response
        let primals: Vec<DiscoveredPrimal> = serde_json::from_value(
            response
                .get("primals")
                .ok_or_else(|| Error::invalid_response("songbird", "Missing 'primals' field"))?
                .clone(),
        )?;

        info!(
            count = primals.len(),
            "Discovered {} primals in family",
            primals.len()
        );
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

    /// Test discovery request creation with taxonomy
    #[test]
    fn test_discovery_request_with_taxonomy() {
        let request = DiscoveryRequest::new(CapabilityTaxonomy::Encryption)
            .with_family("1894e909e454")
            .with_timeout(Duration::from_secs(5));

        assert!(matches!(request.capability, CapabilityTaxonomy::Encryption));
        assert_eq!(request.family, Some("1894e909e454".to_string()));
        assert_eq!(request.timeout, Some(Duration::from_secs(5)));
    }

    /// Test discovery request builder pattern
    #[test]
    fn test_discovery_request_builder() {
        let request = DiscoveryRequest::new(CapabilityTaxonomy::Discovery);
        assert!(matches!(request.capability, CapabilityTaxonomy::Discovery));
        assert_eq!(request.family, None);
    }

    /// Test capability taxonomy to string conversion
    #[test]
    fn test_capability_taxonomy_conversion() {
        let cap = CapabilityTaxonomy::Encryption;
        let s = format!("{cap:?}");
        assert_eq!(s, "Encryption");

        let cap2 = CapabilityTaxonomy::P2PFederation;
        let s2 = format!("{cap2:?}");
        assert_eq!(s2, "P2PFederation");
    }

    /// Test discovered primal parsing
    #[test]
    fn test_discovered_primal_parsing() {
        let json = r#"{
            "primal": "beardog",
            "node_id": "node-alpha",
            "family_id": "1894e909e454",
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
        assert!(primal.capabilities.contains(&"encryption".to_string()));
        assert!(primal.capabilities.contains(&"identity".to_string()));
    }
}

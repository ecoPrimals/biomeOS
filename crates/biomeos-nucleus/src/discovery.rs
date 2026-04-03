// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Layer 1: Physical Discovery
//!
//! **Delegates to the mesh discovery provider** (`Songbird` at runtime) — no reimplementation.
//!
//! The discovery service handles:
//! - UDP multicast discovery
//! - Primal announcements
//! - Capability indexing
//! - Network topology
//!
//! This layer coordinates those APIs over the `"discovery"` capability socket.
//!
//! **Deep Debt Evolution**:
//! - Uses `CapabilityTaxonomy` (enum) instead of strings
//! - Uses `SystemPaths` for XDG compliance
//! - Runtime discovery, no hardcoded paths

use async_trait::async_trait;
use biomeos_types::{CapabilityTaxonomy, SystemPaths, primal_names};
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
    pub const fn new(capability: CapabilityTaxonomy) -> Self {
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
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Discovered primal (from the discovery provider)
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

/// Physical discovery layer (delegates to the discovery provider)
#[async_trait]
pub trait PhysicalDiscovery: Send + Sync {
    /// Discover primals by capability
    ///
    /// Delegates to `discover_by_capability` on the discovery provider
    async fn discover_by_capability(
        &self,
        request: &DiscoveryRequest,
    ) -> Result<Vec<DiscoveredPrimal>>;

    /// Discover primals by family
    ///
    /// Delegates to `discover_by_family` on the discovery provider
    async fn discover_by_family(&self, family_id: &str) -> Result<Vec<DiscoveredPrimal>>;

    /// Announce this primal's capabilities
    ///
    /// Delegates to `announce_capabilities` on the discovery provider
    async fn announce(&self, primal_info: &DiscoveredPrimal) -> Result<()>;
}

/// Discovery layer implementation (mesh discovery provider via Unix socket)
pub struct DiscoveryLayer {
    /// Discovery provider Unix socket path (discovered at runtime, not hardcoded!)
    discovery_socket: Option<String>,
    /// System paths (XDG-compliant)
    _paths: SystemPaths,
}

#[cfg(test)]
impl DiscoveryLayer {
    /// Construct with a known discovery socket (skips filesystem discovery; tests only).
    #[must_use]
    pub fn from_discovery_socket_for_test(discovery_socket: String, paths: SystemPaths) -> Self {
        Self {
            discovery_socket: Some(discovery_socket),
            _paths: paths,
        }
    }

    /// Construct with no socket to exercise `discovery_socket()` error paths (tests only).
    #[must_use]
    pub fn with_no_socket_for_test(paths: SystemPaths) -> Self {
        Self {
            discovery_socket: None,
            _paths: paths,
        }
    }
}

impl DiscoveryLayer {
    /// Create a new discovery layer
    ///
    /// **Deep Debt Principle**: Discovers the discovery provider at runtime, no hardcoding!
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - System paths cannot be initialized (XDG directories unavailable)
    /// - Discovery provider socket cannot be discovered (service not running or socket not found)
    #[expect(
        clippy::unused_async,
        reason = "public API contract — callers already .await"
    )]
    pub async fn new() -> Result<Self> {
        info!("Initializing NUCLEUS Discovery Layer (discovery provider)");

        // Get XDG-compliant paths
        let paths = SystemPaths::new().map_err(|e| {
            Error::discovery_failed(format!("Failed to initialize SystemPaths: {e}"), None)
        })?;

        let discovery_socket = Self::discover_discovery_provider_socket(&paths)?;

        Ok(Self {
            discovery_socket: Some(discovery_socket),
            _paths: paths,
        })
    }

    /// Discover the mesh discovery provider's Unix socket via the 5-tier capability discovery protocol.
    ///
    /// Wraps [`biomeos_types::capability_discovery::discover_capability_socket`] with the
    /// `"discovery"` capability and consistent [`Error`] mapping.
    fn discover_discovery_provider_socket(_paths: &SystemPaths) -> Result<String> {
        use biomeos_types::capability_discovery;

        debug!("Discovering discovery provider socket (5-tier capability discovery)");

        capability_discovery::discover_capability_socket(
            "discovery",
            &capability_discovery::std_env,
        )
        .ok_or_else(|| {
            Error::discovery_failed(
                "Could not discover discovery provider socket. Is the discovery service running?",
                Some("discovery".to_string()),
            )
        })
    }

    fn discovery_socket(&self) -> Result<&str> {
        self.discovery_socket.as_deref().ok_or_else(|| {
            Error::discovery_failed("Discovery provider socket not initialized", None)
        })
    }

    /// Call the discovery provider JSON-RPC API
    async fn call_discovery_rpc<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<T> {
        let socket_path = self.discovery_socket()?;

        debug!(
            "Calling discovery provider RPC: {} at {}",
            method, socket_path
        );

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
            "Discovering primals by capability (via discovery provider)"
        );

        // Convert taxonomy to string for Songbird API
        let capability_str = request.capability.to_string();

        let params = serde_json::json!({
            "capability": capability_str,
            "family_id": request.family,
        });

        let mut response: serde_json::Value = self
            .call_discovery_rpc("discover_by_capability", params)
            .await?;

        if response.get("primals").is_none() {
            return Err(Error::invalid_response(
                primal_names::SONGBIRD,
                "Missing 'primals' field",
            ));
        }
        let primals: Vec<DiscoveredPrimal> = serde_json::from_value(response["primals"].take())?;

        info!(
            count = primals.len(),
            capability = %request.capability,
            "Discovered {} primals with capability",
            primals.len()
        );
        Ok(primals)
    }

    async fn discover_by_family(&self, family_id: &str) -> Result<Vec<DiscoveredPrimal>> {
        info!(family = %family_id, "Discovering primals by family (via discovery provider)");

        let params = serde_json::json!({
            "family_id": family_id,
        });

        let mut response: serde_json::Value = self
            .call_discovery_rpc("discover_by_family", params)
            .await?;

        if response.get("primals").is_none() {
            return Err(Error::invalid_response(
                primal_names::SONGBIRD,
                "Missing 'primals' field",
            ));
        }
        let primals: Vec<DiscoveredPrimal> = serde_json::from_value(response["primals"].take())?;

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
            "Announcing primal capabilities (via discovery provider)"
        );

        let params = serde_json::json!({
            "primal_info": primal_info,
        });

        let _response: serde_json::Value = self
            .call_discovery_rpc("announce_capabilities", params)
            .await?;

        info!("Successfully announced capabilities");
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use biomeos_types::JsonRpcResponse;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    fn spawn_one_shot_jsonrpc_server(
        socket_path: impl AsRef<std::path::Path>,
        result_payload: serde_json::Value,
    ) {
        let socket_path = socket_path.as_ref().to_path_buf();
        let listener = UnixListener::bind(&socket_path).expect("bind mock songbird socket");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader
                .read_line(&mut line)
                .await
                .expect("read jsonrpc line");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse request");
            let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let response = JsonRpcResponse::success(id, result_payload);
            let mut stream = reader.into_inner();
            let body = serde_json::to_string(&response).expect("serialize response");
            stream
                .write_all(format!("{body}\n").as_bytes())
                .await
                .expect("write response");
        });
    }

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

    #[test]
    fn test_discovery_request_default_timeout_none() {
        let req = DiscoveryRequest::new(CapabilityTaxonomy::Discovery);
        assert!(req.timeout.is_none());
    }

    #[test]
    fn test_discovery_request_with_family() {
        let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption).with_family("fam-123");
        assert_eq!(req.family.as_deref(), Some("fam-123"));
    }

    #[test]
    fn test_discovered_primal_serialization_roundtrip() {
        let primal = DiscoveredPrimal {
            primal: "songbird".to_string(),
            node_id: "n1".to_string(),
            family_id: "f1".to_string(),
            capabilities: vec!["discovery".to_string()],
            endpoints: vec![],
            signature: "sig".to_string(),
            timestamp: "2026-01-01T00:00:00Z".to_string(),
        };
        let json = serde_json::to_string(&primal).unwrap();
        let parsed: DiscoveredPrimal = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.primal, primal.primal);
        assert_eq!(parsed.node_id, primal.node_id);
    }

    #[test]
    fn test_capability_taxonomy_to_string_for_songbird() {
        let cap = CapabilityTaxonomy::Discovery;
        let s = cap.to_string();
        assert!(!s.is_empty());
    }

    #[tokio::test]
    async fn test_discover_by_capability_success_via_mock_songbird() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird-mock.sock");
        let primal_json = serde_json::json!([{
            "primal": "beardog",
            "node_id": "n1",
            "family_id": "fam",
            "capabilities": ["encryption"],
            "endpoints": [],
            "signature": "sig",
            "timestamp": "2026-01-01T00:00:00Z"
        }]);
        spawn_one_shot_jsonrpc_server(sock.clone(), serde_json::json!({ "primals": primal_json }));

        let paths =
            SystemPaths::new_with_xdg_overrides(Some(dir.path()), Some(dir.path())).expect("paths");
        let layer =
            DiscoveryLayer::from_discovery_socket_for_test(sock.to_string_lossy().into(), paths);

        let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
        let out = layer
            .discover_by_capability(&req)
            .await
            .expect("discover_by_capability");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].primal, "beardog");
    }

    #[tokio::test]
    async fn test_discover_by_capability_missing_primals_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird-mock.sock");
        spawn_one_shot_jsonrpc_server(sock.clone(), serde_json::json!({ "wrong": [] }));

        let paths =
            SystemPaths::new_with_xdg_overrides(Some(dir.path()), Some(dir.path())).expect("paths");
        let layer =
            DiscoveryLayer::from_discovery_socket_for_test(sock.to_string_lossy().into(), paths);

        let req = DiscoveryRequest::new(CapabilityTaxonomy::Discovery);
        let err = layer
            .discover_by_capability(&req)
            .await
            .expect_err("expected invalid response");
        let msg = err.to_string();
        assert!(
            msg.contains("primals") || msg.contains("Invalid") || msg.contains("response"),
            "{msg}"
        );
    }

    #[tokio::test]
    async fn test_discover_by_family_missing_primals_field() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird-mock.sock");
        spawn_one_shot_jsonrpc_server(sock.clone(), serde_json::json!({}));

        let paths =
            SystemPaths::new_with_xdg_overrides(Some(dir.path()), Some(dir.path())).expect("paths");
        let layer =
            DiscoveryLayer::from_discovery_socket_for_test(sock.to_string_lossy().into(), paths);

        let err = layer
            .discover_by_family("fam-1")
            .await
            .expect_err("expected missing primals");
        assert!(err.to_string().contains("primals") || err.to_string().contains("response"));
    }

    #[tokio::test]
    async fn test_discovery_layer_no_socket_returns_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let paths =
            SystemPaths::new_with_xdg_overrides(Some(dir.path()), Some(dir.path())).expect("paths");
        let layer = DiscoveryLayer::with_no_socket_for_test(paths);

        let req = DiscoveryRequest::new(CapabilityTaxonomy::Discovery);
        let err = layer
            .discover_by_capability(&req)
            .await
            .expect_err("expected socket error");
        assert!(
            err.to_string().contains("Discovery")
                || err.to_string().contains("discovery")
                || err.to_string().contains("socket")
        );
    }

    #[tokio::test]
    async fn test_discover_jsonrpc_error_from_discovery_provider() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird-mock.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let req: serde_json::Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let err = biomeos_types::JsonRpcError::method_not_found();
            let response = JsonRpcResponse::error(id, err);
            let mut stream = reader.into_inner();
            let body = serde_json::to_string(&response).expect("serialize");
            stream
                .write_all(format!("{body}\n").as_bytes())
                .await
                .expect("write");
        });

        let paths =
            SystemPaths::new_with_xdg_overrides(Some(dir.path()), Some(dir.path())).expect("paths");
        let layer =
            DiscoveryLayer::from_discovery_socket_for_test(sock.to_string_lossy().into(), paths);

        let req = DiscoveryRequest::new(CapabilityTaxonomy::P2PFederation);
        let err = layer
            .discover_by_capability(&req)
            .await
            .expect_err("jsonrpc error");
        assert!(
            err.to_string().contains("JSON-RPC") || err.to_string().contains("jsonrpc"),
            "{}",
            err
        );
    }

    #[tokio::test]
    async fn test_announce_capabilities_via_mock() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("songbird-mock.sock");
        spawn_one_shot_jsonrpc_server(sock.clone(), serde_json::json!({ "ok": true }));

        let paths =
            SystemPaths::new_with_xdg_overrides(Some(dir.path()), Some(dir.path())).expect("paths");
        let layer =
            DiscoveryLayer::from_discovery_socket_for_test(sock.to_string_lossy().into(), paths);

        let info = DiscoveredPrimal {
            primal: "test".to_string(),
            node_id: "n".to_string(),
            family_id: "f".to_string(),
            capabilities: vec![],
            endpoints: vec![],
            signature: "s".to_string(),
            timestamp: "t".to_string(),
        };
        layer.announce(&info).await.expect("announce");
    }
}

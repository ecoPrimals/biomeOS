// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Universal Primal Adapter (UPA) Client
//!
//! The UPA client provides a protocol-agnostic interface for communicating with
//! primal services. It abstracts away the underlying protocol (HTTP, mDNS, gRPC, etc.)
//! and provides a consistent API for service registration, peer discovery, and communication.
//!
//! # Protocol Support
//!
//! - **HTTP/REST**: Standard HTTP endpoints
//! - **mDNS/UDP**: Local network discovery (Songbird's native protocol)
//! - **gRPC**: High-performance RPC (future)
//! - **Custom**: Extensible for new protocols
//!
//! # Architecture
//!
//! Based on the proven implementation from:
//! `phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/`
//!
//! # Example
//!
//! ```no_run
//! use biomeos_core::clients::upa::{UpaClient, UpaConfig, RegisterNodeRequest, Capability};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create UPA client (auto-detects protocol)
//!     let upa = UpaClient::new(UpaConfig {
//!         service_endpoint: "http://localhost:8080".to_string(),
//!         timeout_secs: 30,
//!     })?;
//!
//!     // Register this node with capabilities
//!     let node_id = upa.register_node(RegisterNodeRequest {
//!         node_id: "my-node".to_string(),
//!         node_type: "beardog".to_string(),
//!         capabilities: vec![
//!             Capability::new("btsp"),
//!             Capability::new("encryption"),
//!         ],
//!         endpoint: "http://127.0.0.1:8081".to_string(),
//!     }).await?;
//!
//!     println!("Registered as: {}", node_id);
//!
//!     // Discover a peer by capability
//!     let peer = upa.find_peer("p2p").await?;
//!     println!("Found peer at: {}", peer.endpoint);
//!
//!     Ok(())
//! }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::clients::base::PrimalHttpClient;

/// UPA client configuration
#[derive(Debug, Clone)]
pub struct UpaConfig {
    /// Service endpoint URL
    /// Examples:
    /// - "http://localhost:9090" (HTTP)
    /// - "mdns://songbird-tower.local" (mDNS/UDP)
    /// - "grpc://localhost:50051" (gRPC - future)
    pub service_endpoint: String,

    /// Request timeout in seconds
    pub timeout_secs: u64,
}

impl Default for UpaConfig {
    fn default() -> Self {
        Self {
            service_endpoint: String::new(),
            timeout_secs: 30,
        }
    }
}

/// Universal Primal Adapter client
///
/// Provides protocol-agnostic communication with primal services.
/// Based on proven phase1 implementation.
#[derive(Debug, Clone)]
pub struct UpaClient {
    config: UpaConfig,
    http: Option<PrimalHttpClient>,
    protocol: Protocol,
}

/// Protocol type detected from endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Protocol {
    Http,
    Mdns,
    Grpc,
}

impl UpaClient {
    /// Create a new UPA client
    ///
    /// Automatically detects the protocol from the endpoint URL:
    /// - `http://` or `https://` → HTTP
    /// - `mdns://` → mDNS/UDP
    /// - `grpc://` → gRPC
    ///
    /// # Errors
    /// Returns an error if the endpoint protocol is not supported.
    pub fn new(config: UpaConfig) -> Result<Self> {
        let protocol = Self::detect_protocol(&config.service_endpoint)?;

        let http = if protocol == Protocol::Http {
            Some(PrimalHttpClient::with_timeout(
                &config.service_endpoint,
                Duration::from_secs(config.timeout_secs),
            ))
        } else {
            None
        };

        Ok(Self {
            config,
            http,
            protocol,
        })
    }

    /// Detect protocol from endpoint URL
    fn detect_protocol(endpoint: &str) -> Result<Protocol> {
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            Ok(Protocol::Http)
        } else if endpoint.starts_with("mdns://") {
            Ok(Protocol::Mdns)
        } else if endpoint.starts_with("grpc://") {
            Ok(Protocol::Grpc)
        } else {
            anyhow::bail!(
                "Unsupported protocol in endpoint: {}. Expected http://, https://, mdns://, or grpc://",
                endpoint
            )
        }
    }

    /// Register a node with the orchestration service
    ///
    /// This is the first step in primal coordination. After registration,
    /// the node can discover peers and establish connections.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::upa::{UpaClient, UpaConfig, RegisterNodeRequest, Capability};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let upa = UpaClient::new(UpaConfig::default())?;
    /// let node_id = upa.register_node(RegisterNodeRequest {
    ///     node_id: "alice".to_string(),
    ///     node_type: "beardog".to_string(),
    ///     capabilities: vec![Capability::new("btsp")],
    ///     endpoint: "http://127.0.0.1:8081".to_string(),
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_node(&self, request: RegisterNodeRequest) -> Result<String> {
        match self.protocol {
            Protocol::Http => self.register_node_http(request).await,
            Protocol::Mdns => self.register_node_mdns(request).await,
            Protocol::Grpc => anyhow::bail!("gRPC protocol not yet implemented"),
        }
    }

    /// Register node via HTTP
    async fn register_node_http(&self, request: RegisterNodeRequest) -> Result<String> {
        let http = self
            .http
            .as_ref()
            .context("HTTP client not initialized")?;

        let body = serde_json::to_value(&request)?;
        let response = http
            .post("/api/v1/registry/register", body) // Updated path for Songbird
            .await
            .context("Failed to register node via HTTP")?;

        let response: RegisterNodeResponse = serde_json::from_value(response)
            .context("Failed to parse registration response")?;

        Ok(response.node_id)
    }

    /// Register node via mDNS
    async fn register_node_mdns(&self, _request: RegisterNodeRequest) -> Result<String> {
        // mDNS registration is typically automatic via service announcement
        // For now, return a placeholder. Full implementation would use:
        // - mdns-sd crate for service discovery
        // - UDP communication for registration
        anyhow::bail!(
            "mDNS registration not yet fully implemented. \
             For mDNS services like Songbird, registration happens via service announcement. \
             See phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/ for reference."
        )
    }

    /// Find a peer by name
    ///
    /// Queries the orchestration service to find a peer's endpoint.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::upa::{UpaClient, UpaConfig};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let upa = UpaClient::new(UpaConfig::default())?;
    /// let peer = upa.find_peer("bob").await?;
    /// println!("Bob is at: {}", peer.endpoint);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_peer(&self, peer_name: &str) -> Result<PeerInfo> {
        match self.protocol {
            Protocol::Http => self.find_peer_http(peer_name).await,
            Protocol::Mdns => self.find_peer_mdns(peer_name).await,
            Protocol::Grpc => anyhow::bail!("gRPC protocol not yet implemented"),
        }
    }

    /// Find peer via HTTP
    async fn find_peer_http(&self, peer_name: &str) -> Result<PeerInfo> {
        let http = self
            .http
            .as_ref()
            .context("HTTP client not initialized")?;

        // Use Songbird's capability-based discovery endpoint
        let body = serde_json::json!({
            "capability": peer_name
        });
        let response = http
            .post("/api/v1/registry/find_peer", body) // Updated path for Songbird
            .await
            .with_context(|| format!("Failed to find peer '{}' via HTTP", peer_name))?;

        serde_json::from_value(response).context("Failed to parse peer info")
    }

    /// Find peer via mDNS
    async fn find_peer_mdns(&self, _peer_name: &str) -> Result<PeerInfo> {
        // mDNS peer discovery would use service browsing
        // Full implementation would use mdns-sd crate
        anyhow::bail!(
            "mDNS peer discovery not yet fully implemented. \
             For mDNS services, use service browsing with mdns-sd crate. \
             See phase1 implementation for reference."
        )
    }

    /// List all registered nodes
    ///
    /// Returns all nodes registered with the orchestration service.
    pub async fn list_nodes(&self) -> Result<Vec<PeerInfo>> {
        match self.protocol {
            Protocol::Http => self.list_nodes_http().await,
            Protocol::Mdns => self.list_nodes_mdns().await,
            Protocol::Grpc => anyhow::bail!("gRPC protocol not yet implemented"),
        }
    }

    /// List nodes via HTTP
    async fn list_nodes_http(&self) -> Result<Vec<PeerInfo>> {
        let http = self
            .http
            .as_ref()
            .context("HTTP client not initialized")?;

        let response = http
            .get("/api/v1/registry/nodes") // Updated path for Songbird
            .await
            .context("Failed to list nodes via HTTP")?;

        let response: ListNodesResponse = serde_json::from_value(response)
            .context("Failed to parse nodes list")?;

        Ok(response.nodes)
    }

    /// List nodes via mDNS
    async fn list_nodes_mdns(&self) -> Result<Vec<PeerInfo>> {
        anyhow::bail!(
            "mDNS node listing not yet fully implemented. \
             For mDNS services, use service browsing. \
             See phase1 implementation for reference."
        )
    }

    /// Get the service endpoint
    pub fn endpoint(&self) -> &str {
        &self.config.service_endpoint
    }

    /// Get the detected protocol
    pub fn protocol(&self) -> &str {
        match self.protocol {
            Protocol::Http => "http",
            Protocol::Mdns => "mdns",
            Protocol::Grpc => "grpc",
        }
    }
}

/// Node registration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterNodeRequest {
    /// Unique node identifier
    pub node_id: String,

    /// Node type (e.g., "beardog", "songbird")
    pub node_type: String,

    /// Node capabilities (Songbird format: objects with capability_type and metadata)
    pub capabilities: Vec<Capability>,

    /// Node endpoint (IP:port or URL)
    pub endpoint: String,
}

/// Capability in Songbird's expected format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    /// Capability type (e.g., "compute", "storage", "p2p")
    pub capability_type: String,

    /// Capability metadata (optional attributes)
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

impl Capability {
    /// Create a new capability with the given type
    pub fn new(capability_type: impl Into<String>) -> Self {
        Self {
            capability_type: capability_type.into(),
            metadata: Default::default(),
        }
    }

    /// Create a capability with metadata
    pub fn with_metadata(
        capability_type: impl Into<String>,
        metadata: std::collections::HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            capability_type: capability_type.into(),
            metadata,
        }
    }
}

/// Node registration response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RegisterNodeResponse {
    node_id: String,
    #[serde(default)]
    message: String,
}

/// Peer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer node ID
    pub node_id: String,

    /// Peer endpoint (IP:port)
    pub endpoint: String,

    /// Peer capabilities
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Peer metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

/// List nodes response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ListNodesResponse {
    nodes: Vec<PeerInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_detection() {
        assert_eq!(
            UpaClient::detect_protocol("http://localhost:9090").unwrap(),
            Protocol::Http
        );
        assert_eq!(
            UpaClient::detect_protocol("https://example.com").unwrap(),
            Protocol::Http
        );
        assert_eq!(
            UpaClient::detect_protocol("mdns://songbird.local").unwrap(),
            Protocol::Mdns
        );
        assert_eq!(
            UpaClient::detect_protocol("grpc://localhost:50051").unwrap(),
            Protocol::Grpc
        );
        assert!(UpaClient::detect_protocol("invalid://test").is_err());
    }

    #[test]
    fn test_config_default() {
        let config = UpaConfig::default();
        assert_eq!(config.timeout_secs, 30);
        assert!(config.service_endpoint.is_empty());
    }

    #[test]
    fn test_client_creation_http() {
        let config = UpaConfig {
            service_endpoint: "http://localhost:9090".to_string(),
            timeout_secs: 30,
        };
        let client = UpaClient::new(config).unwrap();
        assert_eq!(client.protocol(), "http");
        assert!(client.http.is_some());
    }

    #[test]
    fn test_client_creation_mdns() {
        let config = UpaConfig {
            service_endpoint: "mdns://songbird.local".to_string(),
            timeout_secs: 30,
        };
        let client = UpaClient::new(config).unwrap();
        assert_eq!(client.protocol(), "mdns");
        assert!(client.http.is_none());
    }

    #[test]
    fn test_register_request_serialization() {
        let request = RegisterNodeRequest {
            node_id: "alice".to_string(),
            node_type: "beardog".to_string(),
            capabilities: vec![Capability::new("btsp")],
            endpoint: "http://127.0.0.1:8081".to_string(),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("alice"));
        assert!(json.contains("beardog"));
        assert!(json.contains("btsp"));
    }

    #[test]
    fn test_capability_creation() {
        let cap = Capability::new("p2p");
        assert_eq!(cap.capability_type, "p2p");
        assert!(cap.metadata.is_empty());

        let mut metadata = std::collections::HashMap::new();
        metadata.insert("btsp_enabled".to_string(), serde_json::json!(true));
        let cap_with_meta = Capability::with_metadata("p2p", metadata);
        assert_eq!(cap_with_meta.capability_type, "p2p");
        assert_eq!(cap_with_meta.metadata.len(), 1);
    }

    #[test]
    fn test_peer_info_deserialization() {
        let json = r#"{
            "node_id": "bob",
            "endpoint": "127.0.0.1:8082",
            "capabilities": ["btsp", "encryption"],
            "metadata": {"version": "1.0.0"}
        }"#;

        let peer: PeerInfo = serde_json::from_str(json).unwrap();
        assert_eq!(peer.node_id, "bob");
        assert_eq!(peer.endpoint, "127.0.0.1:8082");
        assert_eq!(peer.capabilities.len(), 2);
        assert_eq!(peer.metadata.get("version").unwrap(), "1.0.0");
    }
}


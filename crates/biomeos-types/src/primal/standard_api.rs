// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` Standard Primal API
//!
//! All primals MUST implement these standard methods to enable
//! infant bootstrapping and capability-based discovery.
//!
//! ## Philosophy
//!
//! "Each primal is born knowing only itself. Through the standard API,
//! it can answer fundamental questions about its identity and capabilities,
//! enabling others to discover and compose with it."
//!
//! ## Standard Methods
//!
//! All primals expose these via JSON-RPC:
//!
//! - `biomeos.identity` - Who am I?
//! - `biomeos.capabilities` - What can I do?
//! - `biomeos.health` - How am I?
//! - `biomeos.peers` - Who do I know?
//!
//! ## Example
//!
//! ```ignore
//! use biomeos_types::primal::standard_api::{BiomeOSStandardAPI, PrimalIdentity};
//! use biomeos_types::capability_taxonomy::PrimalCapability;
//!
//! struct MyPrimal;
//!
//! #[async_trait::async_trait]
//! impl BiomeOSStandardAPI for MyPrimal {
//!     async fn biomeos_identity(&self) -> Result<PrimalIdentity, Box<dyn std::error::Error + Send + Sync>> {
//!         Ok(PrimalIdentity {
//!             name: "my-primal".to_string(),
//!             version: env!("CARGO_PKG_VERSION").to_string(),
//!             capabilities: vec![],
//!             description: Some("My example primal".to_string()),
//!         })
//!     }
//!     
//!     // ... implement other methods
//! }
//! ```

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

// EVOLVED (Jan 27, 2026): Import from unified capabilities module
pub use crate::capability_taxonomy::CapabilityTaxonomy as PrimalCapability;

/// Standard `BiomeOS` primal API
///
/// All primals MUST implement these methods to participate in
/// capability-based discovery and composition.
#[async_trait]
pub trait BiomeOSStandardAPI: Send + Sync {
    /// Get primal identity (who am I?)
    ///
    /// Returns the primal's self-reported identity, including
    /// its name, version, and capabilities.
    ///
    /// **JSON-RPC**: `biomeos.identity`
    async fn biomeos_identity(
        &self,
    ) -> Result<PrimalIdentity, Box<dyn std::error::Error + Send + Sync>>;

    /// Get capabilities (what can I do?)
    ///
    /// Returns the list of capabilities this primal provides.
    ///
    /// **JSON-RPC**: `biomeos.capabilities`
    async fn biomeos_capabilities(
        &self,
    ) -> Result<Vec<PrimalCapability>, Box<dyn std::error::Error + Send + Sync>>;

    /// Health check (how am I?)
    ///
    /// Returns the primal's current health status.
    ///
    /// **JSON-RPC**: `biomeos.health`
    async fn biomeos_health(
        &self,
    ) -> Result<HealthStatus, Box<dyn std::error::Error + Send + Sync>>;

    /// Get known peers (who do I know?)
    ///
    /// Returns the list of other primals this primal has discovered.
    ///
    /// **JSON-RPC**: `biomeos.peers`
    async fn biomeos_peers(
        &self,
    ) -> Result<Vec<PeerInfo>, Box<dyn std::error::Error + Send + Sync>>;
}

/// Primal identity information
///
/// Self-reported identity of a primal, used for discovery and composition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PrimalIdentity {
    /// Primal's self-reported name
    pub name: String,

    /// Primal's version (semantic versioning recommended)
    pub version: String,

    /// Capabilities this primal provides
    pub capabilities: Vec<PrimalCapability>,

    /// Optional human-readable description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Health status
///
/// Standard health status for all primals
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum HealthStatus {
    /// Primal is healthy and operational
    Healthy,

    /// Primal is degraded but functional
    Degraded,

    /// Primal is unhealthy
    Unhealthy,

    /// Health status unknown
    #[default]
    Unknown,
}

/// Peer information
///
/// Information about a discovered peer primal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerInfo {
    /// Peer's name
    pub name: String,

    /// Peer's capabilities
    pub capabilities: Vec<PrimalCapability>,

    /// How to connect to this peer (Unix socket path or URL)
    pub endpoint: String,

    /// Last time we successfully communicated with this peer
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen: Option<String>,
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;

    #[test]
    fn test_primal_identity_serialization() {
        let identity = PrimalIdentity {
            name: "test-primal".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![],
            description: Some("Test primal".to_string()),
        };

        let json = serde_json::to_string(&identity).unwrap();
        let deserialized: PrimalIdentity = serde_json::from_str(&json).unwrap();

        assert_eq!(identity, deserialized);
    }

    #[test]
    fn test_primal_identity_default_roundtrip() {
        let identity = PrimalIdentity {
            name: "minimal".to_string(),
            version: "0.1.0".to_string(),
            capabilities: vec![],
            description: None,
        };
        let json = serde_json::to_string(&identity).unwrap();
        let back: PrimalIdentity = serde_json::from_str(&json).unwrap();
        assert_eq!(identity, back);
    }

    #[test]
    fn test_health_status_default() {
        assert_eq!(HealthStatus::default(), HealthStatus::Unknown);
    }

    #[test]
    fn test_health_status_serde() {
        for status in [
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
            HealthStatus::Unknown,
        ] {
            let json = serde_json::to_string(&status).unwrap();
            let back: HealthStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(status, back);
        }
    }

    #[test]
    fn test_peer_info_serde_roundtrip() {
        let peer = PeerInfo {
            name: "peer-primal".to_string(),
            capabilities: vec![],
            endpoint: "unix:///run/biomeos/peer.sock".to_string(),
            last_seen: Some("2025-01-01T00:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&peer).unwrap();
        let back: PeerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(peer, back);
    }

    #[test]
    fn test_peer_info_without_last_seen() {
        let peer = PeerInfo {
            name: "new-peer".to_string(),
            capabilities: vec![],
            endpoint: "http://localhost:8080".to_string(),
            last_seen: None,
        };
        let json = serde_json::to_string(&peer).unwrap();
        assert!(!json.contains("last_seen"));
        let back: PeerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(peer.name, back.name);
        assert_eq!(back.last_seen, None);
    }
}

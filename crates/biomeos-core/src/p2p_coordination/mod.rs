//! P2P Coordination Module
//!
//! BiomeOS coordinates peer-to-peer capabilities across primals in pure Rust.
//!
//! # Architecture
//!
//! This module provides **agnostic, capability-based P2P coordination**:
//! - Discovers primals by capability (not by name)
//! - Coordinates BTSP tunnels (any security primal + any discovery primal)
//! - Coordinates BirdSong encryption (any security primal + any discovery primal)
//! - Coordinates lineage-gated relay (any security primal + any routing primal)
//!
//! # Philosophy
//!
//! **Agnostic**: Works with any primal that provides the capability
//! **Capability-Based**: Discovers what primals can do, not what they're called
//! **Pure Rust**: All coordination logic in Rust, not shell scripts
//! **Sovereignty-Respecting**: Primals choose to cooperate
//!
//! # Example
//!
//! ```rust,no_run
//! use biomeos_core::p2p_coordination::P2PCoordinator;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // BiomeOS discovers primals by capability (agnostic!)
//! let coordinator = P2PCoordinator::new_from_discovery().await?;
//!
//! // Coordinate BTSP tunnel (works with any security + discovery primal)
//! let tunnel = coordinator.create_secure_tunnel(
//!     "node-a",
//!     "node-b",
//!     lineage_proof,
//! ).await?;
//!
//! // Enable encrypted discovery (works with any security + discovery primal)
//! coordinator.enable_encrypted_discovery("family-id").await?;
//! # Ok(())
//! # }
//! ```

pub mod adapters;
pub mod birdsong;
pub mod btsp;
pub mod types;

pub use adapters::{BeardogSecurityAdapter, SongbirdDiscoveryAdapter};
pub use birdsong::BirdSongCoordinator;
pub use btsp::BtspCoordinator;
pub use types::*;

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::sync::Arc;

/// Capability required for security operations (encryption, key exchange, etc.)
pub const CAPABILITY_SECURITY: &str = "security";

/// Capability required for discovery operations (service discovery, mesh coordination)
pub const CAPABILITY_DISCOVERY: &str = "discovery";

/// Capability required for routing operations (NAT traversal, relay)
pub const CAPABILITY_ROUTING: &str = "routing";

/// Trait for any primal that can provide security capabilities
///
/// This trait is **agnostic** - it works with BearDog, but also with any other
/// security primal that implements these operations.
#[async_trait]
pub trait SecurityProvider: Send + Sync {
    /// Request a secure tunnel between two nodes
    async fn request_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        proof: &LineageProof,
    ) -> Result<TunnelRequest>;

    /// Check tunnel health
    async fn check_tunnel_health(&self, tunnel_id: &str) -> Result<TunnelHealth>;

    /// Generate encryption keys for broadcast discovery
    async fn generate_broadcast_keys(&self, family_id: &str) -> Result<BroadcastKeys>;

    /// Verify lineage relationship between nodes
    async fn verify_lineage(&self, requester: &str, target: &str) -> Result<LineageInfo>;
}

/// Trait for any primal that can provide discovery capabilities
///
/// This trait is **agnostic** - it works with Songbird, but also with any other
/// discovery primal that implements these operations.
#[async_trait]
pub trait DiscoveryProvider: Send + Sync {
    /// Register a secure transport endpoint
    async fn register_transport(&self, endpoint: &TransportEndpoint) -> Result<()>;

    /// Enable encrypted discovery mode
    async fn enable_encrypted_mode(&self, config: EncryptedDiscoveryConfig) -> Result<()>;

    /// Check transport health
    async fn check_transport_health(&self, transport_id: &str) -> Result<TransportHealth>;

    /// Test encrypted broadcast
    async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest>;
}

/// Trait for any primal that can provide routing capabilities
///
/// This trait is **agnostic** - works with any routing primal
#[async_trait]
pub trait RoutingProvider: Send + Sync {
    /// Request a lineage-gated relay
    async fn request_relay(
        &self,
        requester: &str,
        target: &str,
        lineage: LineageInfo,
    ) -> Result<RelayOffer>;

    /// Accept a relay offer
    async fn accept_relay(&self, offer: &RelayOffer) -> Result<RelayConnection>;
}

/// Main P2P coordinator that discovers and coordinates primals
///
/// This coordinator is **capability-based**: it discovers what primals can do,
/// not what they're called. It works with any combination of primals that provide
/// the required capabilities.
pub struct P2PCoordinator {
    /// Security provider (discovered by capability)
    security: Arc<dyn SecurityProvider>,

    /// Discovery provider (discovered by capability)
    discovery: Arc<dyn DiscoveryProvider>,

    /// Optional routing provider (discovered by capability)
    routing: Option<Arc<dyn RoutingProvider>>,
}

impl P2PCoordinator {
    /// Create coordinator by discovering primals with required capabilities
    ///
    /// This is **agnostic** - it finds any primal with the required capability,
    /// regardless of what it's called.
    pub async fn new_from_discovery() -> Result<Self> {
        // TODO: Implement capability-based discovery
        // This will use the universal API adapter to find primals by capability
        unimplemented!("Capability-based discovery coming in next implementation")
    }

    /// Create coordinator with explicit providers (for testing/advanced usage)
    pub fn new(
        security: Arc<dyn SecurityProvider>,
        discovery: Arc<dyn DiscoveryProvider>,
        routing: Option<Arc<dyn RoutingProvider>>,
    ) -> Self {
        Self {
            security,
            discovery,
            routing,
        }
    }

    /// Create a secure tunnel between two nodes
    ///
    /// This coordinates:
    /// 1. Security provider creates the tunnel
    /// 2. Discovery provider registers the endpoints
    /// 3. Returns tunnel info for monitoring
    pub async fn create_secure_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        proof: LineageProof,
    ) -> Result<TunnelInfo> {
        let coordinator = BtspCoordinator::new(
            self.security.clone(),
            self.discovery.clone(),
        );

        coordinator.create_tunnel(node_a, node_b, proof).await
    }

    /// Enable encrypted discovery (BirdSong mode)
    ///
    /// This coordinates:
    /// 1. Security provider generates broadcast keys
    /// 2. Discovery provider switches to encrypted mode
    /// 3. Tests encryption is working
    pub async fn enable_encrypted_discovery(&self, family_id: &str) -> Result<DiscoveryMode> {
        let coordinator = BirdSongCoordinator::new(
            self.security.clone(),
            self.discovery.clone(),
        );

        coordinator.enable_encrypted_discovery(family_id).await
    }

    /// Coordinate lineage-gated relay for NAT traversal
    ///
    /// This requires a routing provider (optional capability)
    pub async fn coordinate_relay(
        &self,
        requester: &str,
        target: &str,
    ) -> Result<RelayInfo> {
        let routing = self.routing.as_ref()
            .context("No routing provider available for relay coordination")?;

        let coordinator = BirdSongCoordinator::new(
            self.security.clone(),
            self.discovery.clone(),
        );

        coordinator.coordinate_relay(requester, target, routing.clone()).await
    }

    /// Monitor tunnel health
    pub async fn monitor_tunnel(&self, tunnel_id: &str) -> Result<OverallHealth> {
        let security_health = self.security
            .check_tunnel_health(tunnel_id)
            .await
            .context("Failed to check tunnel health from security provider")?;

        let transport_health = self.discovery
            .check_transport_health(tunnel_id)
            .await
            .context("Failed to check transport health from discovery provider")?;

        let status = Self::compute_status(&security_health, &transport_health);
        
        Ok(OverallHealth {
            tunnel_id: tunnel_id.to_string(),
            security_health,
            transport_health,
            status,
        })
    }

    fn compute_status(security: &TunnelHealth, transport: &TransportHealth) -> HealthStatus {
        if security.status == HealthStatus::Healthy && transport.status == HealthStatus::Healthy {
            HealthStatus::Healthy
        } else if security.status == HealthStatus::Degraded || transport.status == HealthStatus::Degraded {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_constants() {
        assert_eq!(CAPABILITY_SECURITY, "security");
        assert_eq!(CAPABILITY_DISCOVERY, "discovery");
        assert_eq!(CAPABILITY_ROUTING, "routing");
    }
}


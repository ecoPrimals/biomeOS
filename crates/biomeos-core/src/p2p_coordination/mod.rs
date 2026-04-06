// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! P2P Coordination Module
//!
//! `BiomeOS` coordinates peer-to-peer capabilities across primals in pure Rust.
//!
//! # Architecture
//!
//! This module provides **agnostic, capability-based P2P coordination**:
//! - Discovers primals by capability (not by name)
//! - Coordinates BTSP tunnels (any security primal + any discovery primal)
//! - Coordinates `BirdSong` encryption (any security primal + any discovery primal)
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
//! ```ignore
//! use biomeos_core::p2p_coordination::P2PCoordinator;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Note: new_from_discovery() requires live primal integration
//! // Documentation of the capability-based discovery pattern
//!
//! // When primals are integrated, this will work:
//! // let coordinator = P2PCoordinator::new_from_discovery().await?;
//! // let tunnel = coordinator.create_secure_tunnel("node-a", "node-b", lineage).await?;
//! # Ok(())
//! # }
//! ```

pub mod birdsong;
pub mod btsp;
pub(crate) mod socket_providers;
pub mod types;

pub use birdsong::BirdSongCoordinator;
pub use btsp::BtspCoordinator;
pub use types::*;

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

/// Capability required for security operations (encryption, key exchange, etc.)
pub const CAPABILITY_SECURITY: &str = "security";

/// Capability required for discovery operations (service discovery, mesh coordination)
pub const CAPABILITY_DISCOVERY: &str = "discovery";

/// Capability required for routing operations (NAT traversal, relay)
pub const CAPABILITY_ROUTING: &str = "routing";

/// Trait for any primal that can provide security capabilities
///
/// This trait is **agnostic** - it works with any primal providing crypto/security
/// capability (discovered at runtime).
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
/// This trait is **agnostic** - it works with any primal providing discovery
/// capability (discovered at runtime).
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

/// Configuration for [`P2PCoordinator::new_from_discovery_with_config`]: explicit strict mode and
/// socket discovery roots without mutating process environment (tests, embedding).
#[derive(Debug, Clone, Default)]
pub struct P2pDiscoveryConfig {
    /// `Some(true)` = strict (no taxonomy bootstrap); `Some(false)` = allow taxonomy;
    /// `None` = read `BIOMEOS_STRICT_DISCOVERY` from the environment.
    pub strict_discovery: Option<bool>,
    /// Override XDG runtime dir for [`crate::socket_discovery::SocketDiscovery`] (e.g. isolated empty dirs in tests).
    pub xdg_runtime_dir: Option<PathBuf>,
}

fn strict_discovery_resolved(config: &P2pDiscoveryConfig) -> bool {
    config
        .strict_discovery
        .unwrap_or_else(|| std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok())
}

impl P2PCoordinator {
    /// Create coordinator by discovering primals with required capabilities
    ///
    /// This is **agnostic** - it finds any primal with the required capability,
    /// regardless of what it's called.
    pub async fn new_from_discovery() -> Result<Self> {
        Self::new_from_discovery_with_config(&P2pDiscoveryConfig::default()).await
    }

    /// Like [`Self::new_from_discovery`], with optional [`P2pDiscoveryConfig`] overrides.
    pub async fn new_from_discovery_with_config(config: &P2pDiscoveryConfig) -> Result<Self> {
        tracing::info!("🔍 Discovering P2P coordination capabilities...");

        // Discover security provider (capability: crypto/security)
        let security = Self::discover_security_provider(config).await?;
        tracing::info!("✅ Security provider discovered");

        // Discover discovery provider (capability: discovery)
        let discovery = Self::discover_discovery_provider(config).await?;
        tracing::info!("✅ Discovery provider discovered");

        // Routing is optional
        let routing = Self::discover_routing_provider(config).await.ok();
        if routing.is_some() {
            tracing::info!("✅ Routing provider discovered");
        } else {
            tracing::info!("⚠️  No routing provider - using direct connections");
        }

        Ok(Self::new(security, discovery, routing))
    }

    /// Discover a primal that provides security capabilities
    ///
    /// Uses capability-based discovery to find any primal providing security/encryption.
    /// Works with `BearDog` or any compatible security primal.
    async fn discover_security_provider(
        config: &P2pDiscoveryConfig,
    ) -> Result<Arc<dyn SecurityProvider>> {
        use crate::socket_discovery::SocketDiscovery;

        tracing::info!("🔐 Discovering security provider (capability: security)");

        let family_id = crate::family_discovery::get_family_id();
        let mut discovery = SocketDiscovery::new(&family_id);
        if let Some(ref p) = config.xdg_runtime_dir {
            discovery = discovery.with_xdg_override(p);
        }

        // Try capability strings from taxonomy (security, encryption, crypto)
        use biomeos_types::constants::capability;
        for cap in [
            biomeos_types::constants::capabilities::SECURITY,
            "encryption",
            capability::CRYPTO,
        ] {
            if let Some(primal) = discovery.discover_capability(cap).await {
                tracing::info!(
                    "✅ Found security provider: {:?} at {}",
                    primal.primal_name,
                    primal.path.display()
                );
                return Ok(Arc::new(SocketSecurityProvider::new(primal.path)));
            }
        }

        // Taxonomy bootstrap: resolve capability → primal name, then discover by path
        if strict_discovery_resolved(config) {
            anyhow::bail!(
                "BIOMEOS_STRICT_DISCOVERY=1: No security provider found via capability registry. \
                 Ensure a primal with security capability is running and registered."
            );
        }
        if let Some(primal_name) = biomeos_types::CapabilityTaxonomy::resolve_to_primal("security")
            .or_else(|| biomeos_types::CapabilityTaxonomy::resolve_to_primal("encryption"))
        {
            tracing::warn!(
                "⚠️  Capability registry unavailable; using taxonomy bootstrap for security. Set BIOMEOS_STRICT_DISCOVERY=1 to require registry-based discovery."
            );
            if let Some(primal) = discovery.discover_primal(primal_name).await {
                return Ok(Arc::new(SocketSecurityProvider::new(primal.path)));
            }
        }

        anyhow::bail!(
            "No security provider found. Ensure a primal with security capability is running."
        )
    }

    /// Discover a primal that provides discovery/registry capabilities
    ///
    /// Uses capability-based discovery to find any primal providing discovery/registry.
    /// Works with Songbird or any compatible discovery primal.
    async fn discover_discovery_provider(
        config: &P2pDiscoveryConfig,
    ) -> Result<Arc<dyn DiscoveryProvider>> {
        use crate::socket_discovery::SocketDiscovery;

        tracing::info!("🔍 Discovering discovery provider (capability: discovery)");

        let family_id = crate::family_discovery::get_family_id();
        let mut discovery = SocketDiscovery::new(&family_id);
        if let Some(ref p) = config.xdg_runtime_dir {
            discovery = discovery.with_xdg_override(p);
        }

        // Try capability strings from taxonomy (discovery, mesh, registry)
        use biomeos_types::constants::capability;
        for cap in [
            biomeos_types::constants::capabilities::DISCOVERY,
            capability::MESH_NETWORKING,
            "registry",
            "http",
        ] {
            if let Some(primal) = discovery.discover_capability(cap).await {
                tracing::info!(
                    "✅ Found discovery provider: {:?} at {}",
                    primal.primal_name,
                    primal.path.display()
                );
                return Ok(Arc::new(SocketDiscoveryProvider::new(primal.path)));
            }
        }

        // Taxonomy bootstrap: resolve capability → primal name, then discover by path
        if strict_discovery_resolved(config) {
            anyhow::bail!(
                "BIOMEOS_STRICT_DISCOVERY=1: No discovery provider found via capability registry. \
                 Ensure a primal with discovery capability is running and registered."
            );
        }
        if let Some(primal_name) = biomeos_types::CapabilityTaxonomy::resolve_to_primal("discovery")
            .or_else(|| biomeos_types::CapabilityTaxonomy::resolve_to_primal("registry"))
        {
            tracing::warn!(
                "⚠️  Capability registry unavailable; using taxonomy bootstrap for discovery. Set BIOMEOS_STRICT_DISCOVERY=1 to require registry-based discovery."
            );
            if let Some(primal) = discovery.discover_primal(primal_name).await {
                return Ok(Arc::new(SocketDiscoveryProvider::new(primal.path)));
            }
        }

        anyhow::bail!(
            "No discovery provider found. Ensure a primal with discovery capability is running."
        )
    }

    /// Discover a primal that provides routing capabilities (optional)
    async fn discover_routing_provider(
        config: &P2pDiscoveryConfig,
    ) -> Result<Arc<dyn RoutingProvider>> {
        use crate::socket_discovery::SocketDiscovery;

        tracing::info!("🔀 Discovering routing provider (capability: routing)");

        let family_id = crate::family_discovery::get_family_id();

        let mut discovery = SocketDiscovery::new(&family_id);
        if let Some(ref p) = config.xdg_runtime_dir {
            discovery = discovery.with_xdg_override(p);
        }

        use biomeos_types::constants::capability;
        if let Some(primal) = discovery
            .discover_capability(capability::GATEWAY)
            .await
            .or(discovery.discover_capability(CAPABILITY_ROUTING).await)
        {
            tracing::info!(
                "✅ Found routing provider: {:?} at {}",
                primal.primal_name,
                primal.path.display()
            );
            return Ok(Arc::new(SocketRoutingProvider::new(primal.path)));
        }

        anyhow::bail!("No routing provider found (optional)")
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
        let coordinator = BtspCoordinator::new(self.security.clone(), self.discovery.clone());

        coordinator.create_tunnel(node_a, node_b, proof).await
    }

    /// Enable encrypted discovery (`BirdSong` mode)
    ///
    /// This coordinates:
    /// 1. Security provider generates broadcast keys
    /// 2. Discovery provider switches to encrypted mode
    /// 3. Tests encryption is working
    pub async fn enable_encrypted_discovery(&self, family_id: &str) -> Result<DiscoveryMode> {
        let coordinator = BirdSongCoordinator::new(self.security.clone(), self.discovery.clone());

        coordinator.enable_encrypted_discovery(family_id).await
    }

    /// Coordinate lineage-gated relay for NAT traversal
    ///
    /// This requires a routing provider (optional capability)
    pub async fn coordinate_relay(&self, requester: &str, target: &str) -> Result<RelayInfo> {
        let routing = self
            .routing
            .as_ref()
            .context("No routing provider available for relay coordination")?;

        let coordinator = BirdSongCoordinator::new(self.security.clone(), self.discovery.clone());

        coordinator
            .coordinate_relay(requester, target, routing.clone())
            .await
    }

    /// Monitor tunnel health
    pub async fn monitor_tunnel(&self, tunnel_id: &str) -> Result<OverallHealth> {
        let security_health = self
            .security
            .check_tunnel_health(tunnel_id)
            .await
            .context("Failed to check tunnel health from security provider")?;

        let transport_health = self
            .discovery
            .check_transport_health(tunnel_id)
            .await
            .context("Failed to check transport health from discovery provider")?;

        let status = compute_status_impl(&security_health, &transport_health);

        Ok(OverallHealth {
            tunnel_id: tunnel_id.to_string(),
            security_health,
            transport_health,
            status,
        })
    }
}

/// Compute combined health status from security and transport (testable pure function)
pub(crate) fn compute_status_impl(
    security: &TunnelHealth,
    transport: &TransportHealth,
) -> HealthStatus {
    if security.status == HealthStatus::Healthy && transport.status == HealthStatus::Healthy {
        HealthStatus::Healthy
    } else if security.status == HealthStatus::Degraded
        || transport.status == HealthStatus::Degraded
    {
        HealthStatus::Degraded
    } else {
        HealthStatus::Unhealthy
    }
}

// Socket-based provider implementations extracted to socket_providers.rs
// - Unified 3x duplicated send_rpc into single SocketRpcClient
// - Eliminated hardcoded "127.0.0.1" in tunnel endpoints
// - Reduced mod.rs from 870+ to ~400 lines (traits + coordinator only)
use socket_providers::{SocketDiscoveryProvider, SocketRoutingProvider, SocketSecurityProvider};

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "p2p coordination unit tests"
)]
mod tests;

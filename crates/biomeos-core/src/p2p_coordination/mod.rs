// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

impl P2PCoordinator {
    /// Create coordinator by discovering primals with required capabilities
    ///
    /// This is **agnostic** - it finds any primal with the required capability,
    /// regardless of what it's called.
    pub async fn new_from_discovery() -> Result<Self> {
        tracing::info!("🔍 Discovering P2P coordination capabilities...");

        // Discover security provider (capability: crypto/security)
        let security = Self::discover_security_provider().await?;
        tracing::info!("✅ Security provider discovered");

        // Discover discovery provider (capability: discovery)
        let discovery = Self::discover_discovery_provider().await?;
        tracing::info!("✅ Discovery provider discovered");

        // Routing is optional
        let routing = Self::discover_routing_provider().await.ok();
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
    /// Works with BearDog or any compatible security primal.
    async fn discover_security_provider() -> Result<Arc<dyn SecurityProvider>> {
        use crate::socket_discovery::SocketDiscovery;

        tracing::info!("🔐 Discovering security provider (capability: security)");

        let family_id = crate::family_discovery::get_family_id();
        let discovery = SocketDiscovery::new(&family_id);

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
        if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_err() {
            if let Some(primal_name) =
                biomeos_types::CapabilityTaxonomy::resolve_to_primal("security")
                    .or_else(|| biomeos_types::CapabilityTaxonomy::resolve_to_primal("encryption"))
            {
                tracing::warn!(
                    "⚠️  Capability registry unavailable; using taxonomy bootstrap for security. Set BIOMEOS_STRICT_DISCOVERY=1 to require registry-based discovery."
                );
                if let Some(primal) = discovery.discover_primal(primal_name).await {
                    return Ok(Arc::new(SocketSecurityProvider::new(primal.path)));
                }
            }
        } else {
            anyhow::bail!(
                "BIOMEOS_STRICT_DISCOVERY=1: No security provider found via capability registry. \
                 Ensure a primal with security capability is running and registered."
            );
        }

        anyhow::bail!(
            "No security provider found. Ensure a primal with security capability is running."
        )
    }

    /// Discover a primal that provides discovery/registry capabilities
    ///
    /// Uses capability-based discovery to find any primal providing discovery/registry.
    /// Works with Songbird or any compatible discovery primal.
    async fn discover_discovery_provider() -> Result<Arc<dyn DiscoveryProvider>> {
        use crate::socket_discovery::SocketDiscovery;

        tracing::info!("🔍 Discovering discovery provider (capability: discovery)");

        let family_id = crate::family_discovery::get_family_id();
        let discovery = SocketDiscovery::new(&family_id);

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
        if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_err() {
            if let Some(primal_name) =
                biomeos_types::CapabilityTaxonomy::resolve_to_primal("discovery")
                    .or_else(|| biomeos_types::CapabilityTaxonomy::resolve_to_primal("registry"))
            {
                tracing::warn!(
                    "⚠️  Capability registry unavailable; using taxonomy bootstrap for discovery. Set BIOMEOS_STRICT_DISCOVERY=1 to require registry-based discovery."
                );
                if let Some(primal) = discovery.discover_primal(primal_name).await {
                    return Ok(Arc::new(SocketDiscoveryProvider::new(primal.path)));
                }
            }
        } else {
            anyhow::bail!(
                "BIOMEOS_STRICT_DISCOVERY=1: No discovery provider found via capability registry. \
                 Ensure a primal with discovery capability is running and registered."
            );
        }

        anyhow::bail!(
            "No discovery provider found. Ensure a primal with discovery capability is running."
        )
    }

    /// Discover a primal that provides routing capabilities (optional)
    async fn discover_routing_provider() -> Result<Arc<dyn RoutingProvider>> {
        use crate::socket_discovery::SocketDiscovery;

        tracing::info!("🔀 Discovering routing provider (capability: routing)");

        let family_id = crate::family_discovery::get_family_id();

        let discovery = SocketDiscovery::new(&family_id);

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

    /// Enable encrypted discovery (BirdSong mode)
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
// DEEP DEBT REFACTORING (Feb 7, 2026):
// - Unified 3x duplicated send_rpc into single SocketRpcClient
// - Eliminated hardcoded "127.0.0.1" in tunnel endpoints
// - Reduced mod.rs from 870+ to ~400 lines (traits + coordinator only)
use socket_providers::{SocketDiscoveryProvider, SocketRoutingProvider, SocketSecurityProvider};

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
    use biomeos_test_utils::TestEnvGuard;
    use std::time::SystemTime;

    /// Mock security provider for testing
    struct MockSecurityProvider;
    #[async_trait::async_trait]
    impl SecurityProvider for MockSecurityProvider {
        async fn request_tunnel(
            &self,
            node_a: &str,
            node_b: &str,
            _proof: &LineageProof,
        ) -> Result<TunnelRequest> {
            Ok(TunnelRequest {
                id: format!("tunnel-{node_a}-{node_b}"),
                endpoint_a: TransportEndpoint {
                    node_id: node_a.to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 9000,
                    protocol: "tcp".to_string(),
                    secure: true,
                },
                endpoint_b: TransportEndpoint {
                    node_id: node_b.to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 9001,
                    protocol: "tcp".to_string(),
                    secure: true,
                },
                encryption_key: bytes::Bytes::new(),
                created_at: SystemTime::now(),
            })
        }
        async fn check_tunnel_health(&self, tunnel_id: &str) -> Result<TunnelHealth> {
            Ok(TunnelHealth {
                encryption_status: HealthStatus::Healthy,
                forward_secrecy: true,
                last_key_rotation: None,
                status: if tunnel_id.contains("bad") {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Healthy
                },
            })
        }
        async fn generate_broadcast_keys(&self, family_id: &str) -> Result<BroadcastKeys> {
            Ok(BroadcastKeys {
                broadcast_key: bytes::Bytes::from(format!("key-{family_id}")),
                lineage_proof: LineageProof {
                    lineage_id: family_id.to_string(),
                    depth: 0,
                    proof: bytes::Bytes::new(),
                    timestamp: SystemTime::now(),
                },
                generated_at: SystemTime::now(),
            })
        }
        async fn verify_lineage(&self, requester: &str, target: &str) -> Result<LineageInfo> {
            Ok(LineageInfo {
                is_ancestor: requester != target,
                depth: 1,
                proof: LineageProof {
                    lineage_id: requester.to_string(),
                    depth: 0,
                    proof: bytes::Bytes::new(),
                    timestamp: SystemTime::now(),
                },
            })
        }
    }

    /// Mock discovery provider for testing
    struct MockDiscoveryProvider;
    #[async_trait::async_trait]
    impl DiscoveryProvider for MockDiscoveryProvider {
        async fn register_transport(&self, _endpoint: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _config: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }
        async fn check_transport_health(&self, transport_id: &str) -> Result<TransportHealth> {
            Ok(TransportHealth {
                connection_status: if transport_id.contains("bad") {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Healthy
                },
                latency_ms: Some(5),
                packet_loss: None,
                status: if transport_id.contains("bad") {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Healthy
                },
            })
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            Ok(BroadcastTest {
                encrypted: true,
                timestamp: SystemTime::now(),
                success: true,
            })
        }
    }

    /// Mock routing provider for testing
    struct MockRoutingProvider;
    #[async_trait::async_trait]
    impl RoutingProvider for MockRoutingProvider {
        async fn request_relay(
            &self,
            _requester: &str,
            _target: &str,
            lineage: LineageInfo,
        ) -> Result<RelayOffer> {
            Ok(RelayOffer {
                relay_node: "relay-node".to_string(),
                relay_endpoint: TransportEndpoint {
                    node_id: "relay".to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 9002,
                    protocol: "tcp".to_string(),
                    secure: true,
                },
                expires_at: SystemTime::now() + std::time::Duration::from_secs(300),
                lineage_verified: lineage.is_ancestor,
            })
        }
        async fn accept_relay(&self, offer: &RelayOffer) -> Result<RelayConnection> {
            Ok(RelayConnection {
                connection_id: format!("conn-{}", offer.relay_node),
                relay_node: offer.relay_node.clone(),
                established_at: SystemTime::now(),
                status: RelayStatus::Active,
            })
        }
    }

    #[test]
    fn test_p2p_coordinator_new_with_explicit_providers() {
        let security: Arc<dyn SecurityProvider> = Arc::new(MockSecurityProvider);
        let discovery: Arc<dyn DiscoveryProvider> = Arc::new(MockDiscoveryProvider);
        let routing: Option<Arc<dyn RoutingProvider>> = Some(Arc::new(MockRoutingProvider));
        let coordinator = P2PCoordinator::new(security, discovery, routing);
        // Construction succeeds
        drop(coordinator);
    }

    #[test]
    fn test_p2p_coordinator_new_without_routing() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider);
        let coordinator = P2PCoordinator::new(security, discovery, None);
        drop(coordinator);
    }

    #[tokio::test]
    async fn test_create_secure_tunnel() {
        let coordinator = P2PCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(MockDiscoveryProvider),
            None,
        );
        let proof = LineageProof {
            lineage_id: "family-1".to_string(),
            depth: 0,
            proof: bytes::Bytes::new(),
            timestamp: SystemTime::now(),
        };
        let tunnel = coordinator
            .create_secure_tunnel("node-a", "node-b", proof)
            .await
            .expect("create tunnel");
        assert_eq!(tunnel.tunnel_id, "tunnel-node-a-node-b");
        assert_eq!(tunnel.endpoints.len(), 2);
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery() {
        let coordinator = P2PCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(MockDiscoveryProvider),
            None,
        );
        let mode = coordinator
            .enable_encrypted_discovery("family-123")
            .await
            .expect("enable encrypted discovery");
        assert_eq!(mode, DiscoveryMode::Encrypted);
    }

    #[tokio::test]
    async fn test_coordinate_relay_with_routing() {
        let coordinator = P2PCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(MockDiscoveryProvider),
            Some(Arc::new(MockRoutingProvider)),
        );
        let relay = coordinator
            .coordinate_relay("requester", "target")
            .await
            .expect("coordinate relay");
        assert_eq!(relay.relay_node, "relay-node");
        assert_eq!(relay.requester, "requester");
        assert_eq!(relay.target, "target");
        assert_eq!(relay.status, RelayStatus::Active);
    }

    #[tokio::test]
    async fn test_coordinate_relay_without_routing_fails() {
        let coordinator = P2PCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(MockDiscoveryProvider),
            None,
        );
        let result = coordinator.coordinate_relay("requester", "target").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No routing provider")
        );
    }

    #[tokio::test]
    async fn test_monitor_tunnel_healthy() {
        let coordinator = P2PCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(MockDiscoveryProvider),
            None,
        );
        let health = coordinator
            .monitor_tunnel("tunnel-1")
            .await
            .expect("monitor tunnel");
        assert_eq!(health.tunnel_id, "tunnel-1");
        assert_eq!(health.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_monitor_tunnel_unhealthy() {
        let coordinator = P2PCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(MockDiscoveryProvider),
            None,
        );
        let health = coordinator
            .monitor_tunnel("bad-tunnel")
            .await
            .expect("monitor tunnel");
        assert_eq!(health.status, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_compute_status_impl() {
        let healthy = TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Healthy,
        };
        let transport_healthy = TransportHealth {
            connection_status: HealthStatus::Healthy,
            latency_ms: Some(10),
            packet_loss: None,
            status: HealthStatus::Healthy,
        };
        assert_eq!(
            compute_status_impl(&healthy, &transport_healthy),
            HealthStatus::Healthy
        );

        let degraded = TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Degraded,
        };
        assert_eq!(
            compute_status_impl(&degraded, &transport_healthy),
            HealthStatus::Degraded
        );

        let unhealthy = TransportHealth {
            connection_status: HealthStatus::Healthy,
            latency_ms: None,
            packet_loss: None,
            status: HealthStatus::Unhealthy,
        };
        assert_eq!(
            compute_status_impl(&healthy, &unhealthy),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_capability_constants() {
        assert_eq!(CAPABILITY_SECURITY, "security");
        assert_eq!(CAPABILITY_DISCOVERY, "discovery");
        assert_eq!(CAPABILITY_ROUTING, "routing");
    }

    #[test]
    fn test_compute_status_both_degraded() {
        let security = TunnelHealth {
            encryption_status: HealthStatus::Degraded,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Degraded,
        };
        let transport = TransportHealth {
            connection_status: HealthStatus::Healthy,
            latency_ms: Some(100),
            packet_loss: None,
            status: HealthStatus::Healthy,
        };
        assert_eq!(
            compute_status_impl(&security, &transport),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn test_compute_status_both_unhealthy() {
        let security = TunnelHealth {
            encryption_status: HealthStatus::Unhealthy,
            forward_secrecy: false,
            last_key_rotation: None,
            status: HealthStatus::Unhealthy,
        };
        let transport = TransportHealth {
            connection_status: HealthStatus::Unhealthy,
            latency_ms: None,
            packet_loss: Some(50.0),
            status: HealthStatus::Unhealthy,
        };
        assert_eq!(
            compute_status_impl(&security, &transport),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_compute_status_security_healthy_transport_degraded() {
        let security = TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Healthy,
        };
        let transport = TransportHealth {
            connection_status: HealthStatus::Degraded,
            latency_ms: Some(500),
            packet_loss: None,
            status: HealthStatus::Degraded,
        };
        assert_eq!(
            compute_status_impl(&security, &transport),
            HealthStatus::Degraded
        );
    }

    #[tokio::test]
    async fn test_monitor_tunnel_security_provider_error_message() {
        struct BadSec;
        #[async_trait::async_trait]
        impl SecurityProvider for BadSec {
            async fn request_tunnel(
                &self,
                _: &str,
                _: &str,
                _: &LineageProof,
            ) -> Result<TunnelRequest> {
                anyhow::bail!("skip")
            }
            async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
                anyhow::bail!("security-down")
            }
            async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
                anyhow::bail!("skip")
            }
            async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
                anyhow::bail!("skip")
            }
        }
        let coordinator =
            P2PCoordinator::new(Arc::new(BadSec), Arc::new(MockDiscoveryProvider), None);
        let err = coordinator
            .monitor_tunnel("tid")
            .await
            .expect_err("security should fail");
        let chain = format!("{err:#}");
        assert!(chain.contains("security-down"), "got {chain}");
    }

    #[tokio::test]
    async fn test_monitor_tunnel_discovery_provider_error_message() {
        struct BadDisc;
        #[async_trait::async_trait]
        impl DiscoveryProvider for BadDisc {
            async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
                Ok(())
            }
            async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
                Ok(())
            }
            async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
                anyhow::bail!("transport-down")
            }
            async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
                anyhow::bail!("skip")
            }
        }
        let coordinator =
            P2PCoordinator::new(Arc::new(MockSecurityProvider), Arc::new(BadDisc), None);
        let err = coordinator
            .monitor_tunnel("tid")
            .await
            .expect_err("transport should fail");
        let chain = format!("{err:#}");
        assert!(chain.contains("transport-down"), "got {chain}");
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_new_from_discovery_strict_without_sockets_fails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _sock = TestEnvGuard::set("BIOMEOS_SOCKET_DIR", temp.path().to_str().expect("utf8"));
        let _strict = TestEnvGuard::set("BIOMEOS_STRICT_DISCOVERY", "1");
        let result = P2PCoordinator::new_from_discovery().await;
        let err = result.err().expect("expected empty socket dir to fail");
        let msg = err.to_string();
        assert!(
            msg.contains("security") || msg.contains("registry"),
            "{msg}"
        );
    }

    #[test]
    fn test_compute_status_security_degraded_transport_unhealthy() {
        let security = TunnelHealth {
            encryption_status: HealthStatus::Degraded,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Degraded,
        };
        let transport = TransportHealth {
            connection_status: HealthStatus::Unhealthy,
            latency_ms: None,
            packet_loss: None,
            status: HealthStatus::Unhealthy,
        };
        assert_eq!(
            compute_status_impl(&security, &transport),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn test_compute_status_security_unhealthy_transport_degraded() {
        let security = TunnelHealth {
            encryption_status: HealthStatus::Unhealthy,
            forward_secrecy: false,
            last_key_rotation: None,
            status: HealthStatus::Unhealthy,
        };
        let transport = TransportHealth {
            connection_status: HealthStatus::Degraded,
            latency_ms: Some(1),
            packet_loss: None,
            status: HealthStatus::Degraded,
        };
        assert_eq!(
            compute_status_impl(&security, &transport),
            HealthStatus::Degraded
        );
    }

    #[tokio::test]
    async fn test_create_secure_tunnel_propagates_btsp_error() {
        struct FailSec;
        #[async_trait::async_trait]
        impl SecurityProvider for FailSec {
            async fn request_tunnel(
                &self,
                _: &str,
                _: &str,
                _: &LineageProof,
            ) -> Result<TunnelRequest> {
                anyhow::bail!("tunnel-request-fail")
            }
            async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
                anyhow::bail!("skip")
            }
            async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
                anyhow::bail!("skip")
            }
            async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
                anyhow::bail!("skip")
            }
        }
        let coordinator =
            P2PCoordinator::new(Arc::new(FailSec), Arc::new(MockDiscoveryProvider), None);
        let proof = LineageProof {
            lineage_id: "x".to_string(),
            depth: 0,
            proof: bytes::Bytes::new(),
            timestamp: std::time::SystemTime::now(),
        };
        let err = coordinator
            .create_secure_tunnel("a", "b", proof)
            .await
            .expect_err("tunnel");
        let chain = format!("{err:#}");
        assert!(chain.contains("tunnel-request-fail"), "{chain}");
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery_propagates_error() {
        struct FailKeys;
        #[async_trait::async_trait]
        impl SecurityProvider for FailKeys {
            async fn request_tunnel(
                &self,
                _: &str,
                _: &str,
                _: &LineageProof,
            ) -> Result<TunnelRequest> {
                anyhow::bail!("skip")
            }
            async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
                anyhow::bail!("skip")
            }
            async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
                anyhow::bail!("keys-fail")
            }
            async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
                anyhow::bail!("skip")
            }
        }
        let coordinator =
            P2PCoordinator::new(Arc::new(FailKeys), Arc::new(MockDiscoveryProvider), None);
        let err = coordinator
            .enable_encrypted_discovery("fam")
            .await
            .expect_err("enc");
        assert!(format!("{err:#}").contains("keys-fail"));
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_new_from_discovery_non_strict_empty_dir_errors() {
        let temp = tempfile::tempdir().expect("tempdir");
        let _sock = TestEnvGuard::set("BIOMEOS_SOCKET_DIR", temp.path().to_str().expect("utf8"));
        let _strict = TestEnvGuard::remove("BIOMEOS_STRICT_DISCOVERY");
        let result = P2PCoordinator::new_from_discovery().await;
        let err = result.err().expect("expected empty socket dir");
        assert!(err.to_string().contains("security") || err.to_string().contains("No security"));
    }
}

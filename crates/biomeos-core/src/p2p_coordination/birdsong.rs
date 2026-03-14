// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BirdSong Encrypted Discovery Coordination
//!
//! BiomeOS coordinates BirdSong (encrypted, lineage-based discovery) between
//! any security primal and any discovery primal in pure Rust.
//!
//! # What is BirdSong?
//!
//! "A broadcast that is obvious to family and noise otherwise"
//! - Family members (verified lineage) can decrypt and discover services
//! - Others see only encrypted noise
//! - No IP address exposure in cleartext
//!
//! # Agnostic Design
//!
//! This coordinator works with **any** primals that implement:
//! - `SecurityProvider` - Provides lineage verification and encryption keys
//! - `DiscoveryProvider` - Provides broadcast discovery
//! - `RoutingProvider` (optional) - Provides lineage-gated relay

use super::{
    DiscoveryMode, DiscoveryProvider, EncryptedDiscoveryConfig, RelayInfo, RelayStatus,
    RoutingProvider, SecurityProvider,
};
use anyhow::{Context, Result};
use bytes::Bytes;
use std::sync::Arc;

/// BirdSong discovery coordinator
///
/// Coordinates encrypted discovery between:
/// - Any security primal (provides lineage verification and encryption)
/// - Any discovery primal (provides broadcast discovery)
/// - Any routing primal (optional, for lineage-gated relay)
pub struct BirdSongCoordinator {
    /// Security provider (agnostic - works with any security primal)
    security: Arc<dyn SecurityProvider>,

    /// Discovery provider (agnostic - works with any discovery primal)
    discovery: Arc<dyn DiscoveryProvider>,
}

impl BirdSongCoordinator {
    /// Create a new BirdSong coordinator
    ///
    /// # Arguments
    ///
    /// * `security` - Any primal providing security capabilities (e.g., BearDog)
    /// * `discovery` - Any primal providing discovery capabilities (e.g., Songbird)
    ///
    /// # Philosophy
    ///
    /// This constructor is **agnostic** - it accepts any primal that implements
    /// the required traits, regardless of what it's called.
    pub fn new(security: Arc<dyn SecurityProvider>, discovery: Arc<dyn DiscoveryProvider>) -> Self {
        Self {
            security,
            discovery,
        }
    }

    /// Enable encrypted discovery (BirdSong mode)
    ///
    /// # Coordination Flow
    ///
    /// 1. Request broadcast keys from security provider
    /// 2. Configure discovery provider for encrypted mode
    /// 3. Verify encryption is working
    /// 4. Return active discovery mode
    ///
    /// This flow is **primal-agnostic** - it works with any combination of
    /// security and discovery primals.
    pub async fn enable_encrypted_discovery(&self, family_id: &str) -> Result<DiscoveryMode> {
        // Step 1: Request BirdSong keys from security provider
        let broadcast_keys = self
            .security
            .generate_broadcast_keys(family_id)
            .await
            .context("Security provider failed to generate broadcast keys")?;

        // Step 2: Configure discovery provider for encrypted mode
        let config = EncryptedDiscoveryConfig {
            encryption_key: broadcast_keys.broadcast_key,
            lineage_filter: broadcast_keys.lineage_proof,
            mode: DiscoveryMode::Encrypted,
        };

        self.discovery
            .enable_encrypted_mode(config)
            .await
            .context("Discovery provider failed to enable encrypted mode")?;

        // Step 3: Verify encryption is working
        let test = self
            .discovery
            .test_encrypted_broadcast()
            .await
            .context("Failed to test encrypted broadcast")?;

        if !test.encrypted {
            anyhow::bail!("BirdSong encryption verification failed - broadcasts not encrypted");
        }

        if !test.success {
            anyhow::bail!("BirdSong encryption verification failed - test unsuccessful");
        }

        // Step 4: Return active mode
        Ok(DiscoveryMode::Encrypted)
    }

    /// Coordinate lineage-gated relay for NAT traversal
    ///
    /// This is BirdSong's killer feature: NAT traversal without TURN servers!
    ///
    /// # How It Works
    ///
    /// 1. Verify lineage relationship (requester → target)
    /// 2. Find ancestor node willing to relay
    /// 3. Establish relay connection
    /// 4. No central TURN server needed!
    ///
    /// # Coordination Flow
    ///
    /// 1. Security provider verifies lineage
    /// 2. Routing provider requests relay from ancestor
    /// 3. Routing provider accepts relay offer
    /// 4. Return relay information
    pub async fn coordinate_relay(
        &self,
        requester: &str,
        target: &str,
        routing: Arc<dyn RoutingProvider>,
    ) -> Result<RelayInfo> {
        // Step 1: Verify lineage relationship
        let lineage = self
            .security
            .verify_lineage(requester, target)
            .await
            .context("Security provider failed to verify lineage")?;

        if !lineage.is_ancestor {
            anyhow::bail!(
                "Lineage verification failed: {} is not an ancestor of {}",
                target,
                requester
            );
        }

        // Step 2: Request relay from ancestor node
        let relay_offer = routing
            .request_relay(requester, target, lineage.clone())
            .await
            .context("Routing provider failed to request relay")?;

        if !relay_offer.lineage_verified {
            anyhow::bail!("Relay offer received but lineage not verified");
        }

        // Step 3: Accept relay offer
        let relay_connection = routing
            .accept_relay(&relay_offer)
            .await
            .context("Routing provider failed to accept relay")?;

        // Step 4: Return relay information
        Ok(RelayInfo {
            relay_node: relay_connection.relay_node,
            requester: requester.to_string(),
            target: target.to_string(),
            status: RelayStatus::Active,
        })
    }

    /// Disable encrypted discovery (return to plaintext mode)
    ///
    /// This is useful for:
    /// - Trusted LAN environments
    /// - Debugging
    /// - Performance optimization
    pub async fn disable_encrypted_discovery(&self) -> Result<DiscoveryMode> {
        let config = EncryptedDiscoveryConfig {
            encryption_key: Bytes::new(),
            lineage_filter: super::LineageProof {
                lineage_id: String::new(),
                depth: 0,
                proof: Bytes::new(),
                timestamp: std::time::SystemTime::now(),
            },
            mode: DiscoveryMode::Plaintext,
        };

        self.discovery
            .enable_encrypted_mode(config)
            .await
            .context("Discovery provider failed to disable encrypted mode")?;

        Ok(DiscoveryMode::Plaintext)
    }

    /// Check current discovery mode
    pub async fn get_discovery_mode(&self) -> Result<DiscoveryMode> {
        // Test if encryption is active
        let test = self.discovery.test_encrypted_broadcast().await?;

        if test.encrypted {
            Ok(DiscoveryMode::Encrypted)
        } else {
            Ok(DiscoveryMode::Plaintext)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::p2p_coordination::{
        BroadcastKeys, BroadcastTest, LineageInfo, LineageProof, RelayConnection, RelayOffer,
        RelayStatus, TransportEndpoint, TransportHealth, TunnelHealth, TunnelRequest,
    };
    use async_trait::async_trait;
    use std::time::SystemTime;

    struct MockSecurityProvider;

    #[async_trait]
    impl SecurityProvider for MockSecurityProvider {
        async fn request_tunnel(
            &self,
            _node_a: &str,
            _node_b: &str,
            _proof: &LineageProof,
        ) -> Result<TunnelRequest> {
            Err(anyhow::anyhow!("not used in birdsong tests"))
        }

        async fn check_tunnel_health(&self, _tunnel_id: &str) -> Result<TunnelHealth> {
            Err(anyhow::anyhow!("not used in birdsong tests"))
        }

        async fn generate_broadcast_keys(&self, _family_id: &str) -> Result<BroadcastKeys> {
            Ok(BroadcastKeys {
                broadcast_key: Bytes::from_static(&[1, 2, 3]),
                lineage_proof: LineageProof {
                    lineage_id: "test".to_string(),
                    depth: 0,
                    proof: Bytes::new(),
                    timestamp: SystemTime::now(),
                },
                generated_at: SystemTime::now(),
            })
        }

        async fn verify_lineage(&self, _requester: &str, _target: &str) -> Result<LineageInfo> {
            Ok(LineageInfo {
                is_ancestor: true,
                depth: 1,
                proof: LineageProof {
                    lineage_id: "test".to_string(),
                    depth: 1,
                    proof: Bytes::new(),
                    timestamp: SystemTime::now(),
                },
            })
        }
    }

    struct MockDiscoveryProvider {
        encrypted: bool,
        success: bool,
    }

    #[async_trait]
    impl DiscoveryProvider for MockDiscoveryProvider {
        async fn register_transport(&self, _endpoint: &TransportEndpoint) -> Result<()> {
            Ok(())
        }

        async fn enable_encrypted_mode(&self, _config: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }

        async fn check_transport_health(&self, _transport_id: &str) -> Result<TransportHealth> {
            Err(anyhow::anyhow!("not used in birdsong tests"))
        }

        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            Ok(BroadcastTest {
                encrypted: self.encrypted,
                timestamp: SystemTime::now(),
                success: self.success,
            })
        }
    }

    #[test]
    fn test_discovery_mode() {
        assert_eq!(DiscoveryMode::Plaintext, DiscoveryMode::Plaintext);
        assert_ne!(DiscoveryMode::Plaintext, DiscoveryMode::Encrypted);
    }

    #[test]
    fn test_birdsong_coordinator_new() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let _coordinator = BirdSongCoordinator::new(security, discovery);
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: true,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .enable_encrypted_discovery("family-1")
            .await
            .expect("enable_encrypted_discovery should succeed");
        assert_eq!(mode, DiscoveryMode::Encrypted);
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery_fails_when_not_encrypted() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let err = coordinator
            .enable_encrypted_discovery("family-1")
            .await
            .expect_err("should fail when broadcast not encrypted");
        assert!(err.to_string().contains("encrypted"));
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery_fails_when_test_unsuccessful() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: true,
            success: false,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let err = coordinator
            .enable_encrypted_discovery("family-1")
            .await
            .expect_err("should fail when test unsuccessful");
        assert!(
            err.to_string().contains("unsuccessful") || err.to_string().contains("verification")
        );
    }

    #[tokio::test]
    async fn test_disable_encrypted_discovery() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .disable_encrypted_discovery()
            .await
            .expect("disable_encrypted_discovery should succeed");
        assert_eq!(mode, DiscoveryMode::Plaintext);
    }

    #[tokio::test]
    async fn test_get_discovery_mode_encrypted() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: true,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .get_discovery_mode()
            .await
            .expect("get_discovery_mode should succeed");
        assert_eq!(mode, DiscoveryMode::Encrypted);
    }

    #[tokio::test]
    async fn test_get_discovery_mode_plaintext() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .get_discovery_mode()
            .await
            .expect("get_discovery_mode should succeed");
        assert_eq!(mode, DiscoveryMode::Plaintext);
    }

    struct MockRoutingProvider;

    #[async_trait]
    impl super::RoutingProvider for MockRoutingProvider {
        async fn request_relay(
            &self,
            _requester: &str,
            _target: &str,
            _lineage: LineageInfo,
        ) -> Result<RelayOffer> {
            Ok(RelayOffer {
                relay_node: "relay-1".to_string(),
                relay_endpoint: TransportEndpoint {
                    node_id: "relay-1".to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 9000,
                    protocol: "tcp".to_string(),
                    secure: true,
                },
                expires_at: SystemTime::now(),
                lineage_verified: true,
            })
        }

        async fn accept_relay(&self, _offer: &RelayOffer) -> Result<RelayConnection> {
            Ok(RelayConnection {
                connection_id: "conn-1".to_string(),
                relay_node: "relay-1".to_string(),
                established_at: SystemTime::now(),
                status: RelayStatus::Active,
            })
        }
    }

    #[tokio::test]
    async fn test_coordinate_relay() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let routing = Arc::new(MockRoutingProvider);
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let relay_info = coordinator
            .coordinate_relay("requester-1", "target-1", routing)
            .await
            .expect("coordinate_relay should succeed");
        assert_eq!(relay_info.relay_node, "relay-1");
        assert_eq!(relay_info.requester, "requester-1");
        assert_eq!(relay_info.target, "target-1");
        assert_eq!(relay_info.status, RelayStatus::Active);
    }

    struct MockSecurityProviderNonAncestor;

    #[async_trait]
    impl SecurityProvider for MockSecurityProviderNonAncestor {
        async fn request_tunnel(
            &self,
            _node_a: &str,
            _node_b: &str,
            _proof: &LineageProof,
        ) -> Result<TunnelRequest> {
            Err(anyhow::anyhow!("not used"))
        }

        async fn check_tunnel_health(&self, _tunnel_id: &str) -> Result<TunnelHealth> {
            Err(anyhow::anyhow!("not used"))
        }

        async fn generate_broadcast_keys(&self, _family_id: &str) -> Result<BroadcastKeys> {
            Err(anyhow::anyhow!("not used"))
        }

        async fn verify_lineage(&self, _requester: &str, _target: &str) -> Result<LineageInfo> {
            Ok(LineageInfo {
                is_ancestor: false,
                depth: 0,
                proof: LineageProof {
                    lineage_id: "test".to_string(),
                    depth: 0,
                    proof: Bytes::new(),
                    timestamp: SystemTime::now(),
                },
            })
        }
    }

    #[tokio::test]
    async fn test_coordinate_relay_fails_when_not_ancestor() {
        let security = Arc::new(MockSecurityProviderNonAncestor);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let routing = Arc::new(MockRoutingProvider);
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let err = coordinator
            .coordinate_relay("requester", "target", routing)
            .await
            .expect_err("should fail when target is not ancestor");
        assert!(err.to_string().contains("ancestor") || err.to_string().contains("Lineage"));
    }
}

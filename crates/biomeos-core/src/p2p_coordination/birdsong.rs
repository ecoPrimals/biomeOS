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
            encryption_key: vec![],
            lineage_filter: super::LineageProof {
                lineage_id: String::new(),
                depth: 0,
                proof: vec![],
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

    #[test]
    fn test_discovery_mode() {
        assert_eq!(DiscoveryMode::Plaintext, DiscoveryMode::Plaintext);
        assert_ne!(DiscoveryMode::Plaintext, DiscoveryMode::Encrypted);
    }
}

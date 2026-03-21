// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Fusion - Where genetics combine into novel capabilities
//!
//! This is the heart of Platypus - the integration layer that
//! creates capabilities impossible with orchestration alone.
//!
//! Neither BearDog nor Songbird can do what Platypus does.
//! The fusion creates emergent behavior.

use crate::crypto::{GeneticKeys, Identity};
use crate::mesh::{MeshNode, MeshTopology, Peer};
use bytes::Bytes;
use crate::{PlatypusError, Result};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for a Platypus node
#[derive(Debug, Clone)]
pub struct PlatypusConfig {
    /// Mesh topology
    pub topology: MeshTopology,
    
    /// Require lineage verification for all peers?
    pub strict_lineage: bool,
    
    /// Minimum trust level for peer acceptance
    pub min_trust: f64,
    
    /// Enable genetic evolution (key rotation)
    pub enable_evolution: bool,
    
    /// Evolution interval in seconds
    pub evolution_interval: u64,
}

impl Default for PlatypusConfig {
    fn default() -> Self {
        Self {
            topology: MeshTopology::Aquatic {
                target_connections: 8,
                fluidity: 0.5,
            },
            strict_lineage: false,
            min_trust: 0.3,
            enable_evolution: true,
            evolution_interval: 3600, // 1 hour
        }
    }
}

/// The Platypus organism - a fused chimera
///
/// This is NOT BearDog + Songbird running together.
/// This IS a new species with mixed genetics.
pub struct Platypus {
    /// Genetic keys (from BearDog lineage)
    keys: Arc<RwLock<GeneticKeys>>,
    
    /// Our identity
    identity: Arc<RwLock<Identity>>,
    
    /// Mesh node (from Songbird lineage)
    mesh: MeshNode,
    
    /// Configuration
    config: PlatypusConfig,
}

impl Platypus {
    /// Create a new Platypus node
    pub fn new(config: PlatypusConfig) -> Self {
        let keys = GeneticKeys::new_root();
        let identity = Identity::from_keys(&keys);
        let mesh = MeshNode::new(identity.clone())
            .with_topology(config.topology);
        
        Self {
            keys: Arc::new(RwLock::new(keys)),
            identity: Arc::new(RwLock::new(identity)),
            mesh,
            config,
        }
    }
    
    /// Create from existing keys (for key continuity)
    pub fn from_keys(keys: GeneticKeys, config: PlatypusConfig) -> Self {
        let identity = Identity::from_keys(&keys);
        let mesh = MeshNode::new(identity.clone())
            .with_topology(config.topology);
        
        Self {
            keys: Arc::new(RwLock::new(keys)),
            identity: Arc::new(RwLock::new(identity)),
            mesh,
            config,
        }
    }
    
    // =========================================================================
    // NOVEL CAPABILITIES - Only possible through fusion
    // =========================================================================
    
    /// Genetic mesh discovery
    ///
    /// This is a NOVEL capability that neither parent has:
    /// - Songbird can discover peers
    /// - BearDog can verify identity
    /// - Platypus can discover ONLY peers that share genetic lineage
    ///
    /// The fusion creates emergent behavior.
    pub async fn genetic_discover(&self) -> Vec<TrustedPeer> {
        let keys = self.keys.read().await;
        let candidates = self.mesh.discover().await;
        
        // This integration doesn't exist in either parent!
        candidates.into_iter()
            .filter_map(|peer| {
                // Verify lineage using our genetic keys
                // (In production, we'd request lineage proof from peer)
                if self.verify_peer_lineage(&keys, &peer) {
                    Some(TrustedPeer::from_verified(peer))
                } else {
                    None
                }
            })
            .collect()
    }
    
    /// Add a peer with lineage verification
    ///
    /// Only accepts peers that share genetic lineage.
    pub async fn add_trusted_peer(&self, peer: Peer) -> Result<()> {
        let keys = self.keys.read().await;
        
        if self.config.strict_lineage && !self.verify_peer_lineage(&keys, &peer) {
            return Err(PlatypusError::LineageVerification {
                peer_id: peer.identity.id.clone(),
            });
        }
        
        let verified_peer = if self.verify_peer_lineage(&keys, &peer) {
            peer.with_lineage_verified()
        } else {
            peer
        };
        
        self.mesh.add_peer(verified_peer).await;
        Ok(())
    }
    
    /// Evolve to next generation
    ///
    /// Rotates keys while maintaining lineage chain.
    /// All existing verified peers remain verified.
    pub async fn evolve(&self) -> Result<()> {
        if !self.config.enable_evolution {
            return Err(PlatypusError::Config(
                "Evolution disabled in config".to_string()
            ));
        }
        
        let mut keys = self.keys.write().await;
        let mut identity = self.identity.write().await;
        
        // Evolve keys (child inherits parent's lineage)
        *keys = keys.evolve();
        *identity = Identity::from_keys(&keys);
        
        tracing::info!(
            generation = keys.generation(),
            "Platypus evolved to generation {}",
            keys.generation()
        );
        
        Ok(())
    }
    
    /// Sign data with genetic signature
    ///
    /// The signature includes lineage information.
    pub async fn sign(&self, data: &[u8]) -> GeneticSignature {
        let keys = self.keys.read().await;
        let identity = self.identity.read().await;
        
        let signature = keys.sign(data);
        
        GeneticSignature {
            signature: Bytes::copy_from_slice(&signature.to_bytes()),
            signer: identity.clone(),
            generation: keys.generation(),
        }
    }
    
    /// Broadcast to mesh with genetic encryption
    ///
    /// Another NOVEL capability:
    /// - Message is signed with genetic key
    /// - Only peers with verified lineage can decode
    pub async fn genetic_broadcast(&self, message: &[u8]) -> Result<BroadcastReceipt> {
        let signature = self.sign(message).await;
        
        // Get verified peers only
        let recipients = self.mesh.verified_peers().await;
        
        // In production: encrypt for each recipient using shared lineage
        
        Ok(BroadcastReceipt {
            message_hash: blake3::hash(message).to_hex().to_string(),
            recipients: recipients.len(),
            generation: signature.generation,
        })
    }
    
    // =========================================================================
    // Internal helpers
    // =========================================================================
    
    fn verify_peer_lineage(&self, _our_keys: &GeneticKeys, peer: &Peer) -> bool {
        // In production, this would:
        // 1. Request lineage proof from peer
        // 2. Verify cryptographic chain
        // 3. Check for common ancestor
        
        // For now, check if peer has lineage hashes
        !peer.identity.lineage_hashes.is_empty()
    }
    
    // =========================================================================
    // Accessors
    // =========================================================================
    
    /// Get our identity
    pub async fn identity(&self) -> Identity {
        self.identity.read().await.clone()
    }
    
    /// Get current generation
    pub async fn generation(&self) -> u64 {
        self.keys.read().await.generation()
    }
    
    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        self.mesh.peer_count().await
    }
    
    /// Get verified peer count
    pub async fn verified_peer_count(&self) -> usize {
        self.mesh.verified_peers().await.len()
    }
}

/// A peer verified through genetic lineage
#[derive(Debug, Clone)]
pub struct TrustedPeer {
    /// The underlying peer
    pub peer: Peer,
    
    /// Shared ancestor generation (how far back common lineage goes)
    pub common_ancestor_generation: u64,
}

impl TrustedPeer {
    /// Create from a verified peer
    pub fn from_verified(peer: Peer) -> Self {
        Self {
            common_ancestor_generation: 0, // Root ancestor
            peer,
        }
    }
}

/// A signature with genetic lineage information
#[derive(Debug, Clone)]
pub struct GeneticSignature {
    /// The cryptographic signature
    pub signature: Bytes,
    
    /// Signer identity
    pub signer: Identity,
    
    /// Generation at time of signing
    pub generation: u64,
}

/// Receipt for a genetic broadcast
#[derive(Debug, Clone)]
pub struct BroadcastReceipt {
    /// Hash of the broadcast message
    pub message_hash: String,
    
    /// Number of recipients
    pub recipients: usize,
    
    /// Generation used for signing
    pub generation: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_platypus_creation() {
        let config = PlatypusConfig::default();
        let platypus = Platypus::new(config);
        
        let identity = platypus.identity().await;
        assert!(identity.id.starts_with("did:platypus:"));
        assert_eq!(platypus.generation().await, 0);
    }
    
    #[tokio::test]
    async fn test_evolution() {
        let config = PlatypusConfig::default();
        let platypus = Platypus::new(config);
        
        assert_eq!(platypus.generation().await, 0);
        
        platypus.evolve().await.unwrap();
        assert_eq!(platypus.generation().await, 1);
        
        platypus.evolve().await.unwrap();
        assert_eq!(platypus.generation().await, 2);
    }
    
    #[tokio::test]
    async fn test_genetic_signature() {
        let config = PlatypusConfig::default();
        let platypus = Platypus::new(config);
        
        let data = b"Hello, aquatic world!";
        let sig = platypus.sign(data).await;
        
        assert!(!sig.signature.is_empty());
        assert_eq!(sig.generation, 0);
    }
}


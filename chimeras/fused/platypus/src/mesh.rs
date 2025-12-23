//! Mesh networking - inherited from Songbird lineage
//!
//! This module contains genetic material from Songbird, adapted
//! for the Platypus organism's aquatic niche.

use crate::crypto::Identity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A peer in the mesh network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    /// Peer identity
    pub identity: Identity,
    
    /// Network address
    pub address: Option<SocketAddr>,
    
    /// Capabilities advertised by this peer
    pub capabilities: Vec<String>,
    
    /// Trust level (0.0 - 1.0)
    pub trust: f64,
    
    /// Last seen timestamp
    pub last_seen: u64,
    
    /// Is this peer verified through lineage?
    pub lineage_verified: bool,
}

impl Peer {
    /// Create a new peer from identity
    pub fn new(identity: Identity) -> Self {
        Self {
            identity,
            address: None,
            capabilities: Vec::new(),
            trust: 0.0,
            last_seen: 0,
            lineage_verified: false,
        }
    }
    
    /// Mark as lineage-verified with trust boost
    pub fn with_lineage_verified(mut self) -> Self {
        self.lineage_verified = true;
        self.trust = (self.trust + 0.5).min(1.0);
        self
    }
    
    /// Set address
    pub fn with_address(mut self, addr: SocketAddr) -> Self {
        self.address = Some(addr);
        self
    }
    
    /// Add capability
    pub fn with_capability(mut self, cap: impl Into<String>) -> Self {
        self.capabilities.push(cap.into());
        self
    }
}

/// A node in the mesh network
pub struct MeshNode {
    /// Our identity
    identity: Identity,
    
    /// Known peers
    peers: Arc<RwLock<HashMap<String, Peer>>>,
    
    /// Mesh topology type
    topology: MeshTopology,
}

/// Mesh topology configuration
#[derive(Debug, Clone, Copy)]
pub enum MeshTopology {
    /// Full mesh - everyone connects to everyone
    Full,
    
    /// Ring - each node connects to neighbors
    Ring,
    
    /// Star - all nodes connect to central hub
    Star,
    
    /// Aquatic - fluid, adaptive topology (Platypus specialty)
    Aquatic {
        /// Target number of connections
        target_connections: usize,
        /// How quickly to adapt
        fluidity: f64,
    },
}

impl Default for MeshTopology {
    fn default() -> Self {
        Self::Aquatic {
            target_connections: 8,
            fluidity: 0.5,
        }
    }
}

impl MeshNode {
    /// Create a new mesh node
    pub fn new(identity: Identity) -> Self {
        Self {
            identity,
            peers: Arc::new(RwLock::new(HashMap::new())),
            topology: MeshTopology::default(),
        }
    }
    
    /// Set topology
    pub fn with_topology(mut self, topology: MeshTopology) -> Self {
        self.topology = topology;
        self
    }
    
    /// Add a peer
    pub async fn add_peer(&self, peer: Peer) {
        let mut peers = self.peers.write().await;
        peers.insert(peer.identity.id.clone(), peer);
    }
    
    /// Get all peers
    pub async fn peers(&self) -> Vec<Peer> {
        let peers = self.peers.read().await;
        peers.values().cloned().collect()
    }
    
    /// Get peers with lineage verification
    pub async fn verified_peers(&self) -> Vec<Peer> {
        let peers = self.peers.read().await;
        peers.values()
            .filter(|p| p.lineage_verified)
            .cloned()
            .collect()
    }
    
    /// Discover peers (stub - would use actual discovery protocol)
    pub async fn discover(&self) -> Vec<Peer> {
        // In real implementation, this would:
        // 1. Broadcast discovery request
        // 2. Collect responses
        // 3. Return candidate peers
        
        // For now, return empty
        vec![]
    }
    
    /// Get our identity
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
    
    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }
}

/// Message types for mesh communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeshMessage {
    /// Discovery request
    Discover {
        /// Requesting peer identity
        from: Identity,
        /// Capabilities we're looking for
        seeking: Vec<String>,
    },
    
    /// Discovery response
    DiscoverResponse {
        /// Responding peer
        peer: Peer,
    },
    
    /// Data message
    Data {
        /// Sender
        from: String,
        /// Recipient (or broadcast)
        to: Option<String>,
        /// Payload
        payload: Vec<u8>,
        /// Signature
        signature: Vec<u8>,
    },
    
    /// Lineage verification request
    LineageVerify {
        /// Requesting peer
        from: Identity,
    },
    
    /// Lineage verification response
    LineageVerifyResponse {
        /// Full lineage chain for verification
        lineage: Vec<String>,
        /// Generation
        generation: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::GeneticKeys;
    
    #[tokio::test]
    async fn test_mesh_node() {
        let keys = GeneticKeys::new_root();
        let identity = Identity::from_keys(&keys);
        let node = MeshNode::new(identity);
        
        assert_eq!(node.peer_count().await, 0);
    }
    
    #[tokio::test]
    async fn test_add_peer() {
        let keys = GeneticKeys::new_root();
        let identity = Identity::from_keys(&keys);
        let node = MeshNode::new(identity.clone());
        
        let peer_keys = GeneticKeys::new_root();
        let peer_identity = Identity::from_keys(&peer_keys);
        let peer = Peer::new(peer_identity);
        
        node.add_peer(peer).await;
        assert_eq!(node.peer_count().await, 1);
    }
}


# 🐦 Songbird Integration Plan for Primal Discovery

**Date:** January 8, 2026  
**Status:** ⏳ **Ready for Integration - Awaiting Songbird UDP Multicast API**

---

## 🎯 Overview

This document outlines how biomeOS will integrate with Songbird for runtime primal discovery via UDP multicast. This replaces hardcoded primal names and enables zero-config mesh networking.

---

## 🔍 Integration Points

### 1. **UDP Multicast Discovery**

**Location**: `crates/biomeos-federation/src/discovery.rs`  
**Function**: `PrimalDiscovery::discover()`  
**Line**: ~91

**Current Code**:
```rust
pub async fn discover(&mut self) -> FederationResult<Vec<DiscoveredPrimal>> {
    info!("Starting primal discovery");
    
    // 1. Discover via Unix sockets
    self.discover_unix_sockets().await?;
    
    // 2. Discover via environment variables
    self.discover_from_env()?;
    
    // 3. TODO: Discover via Songbird UDP multicast (requires Songbird integration)
    // This would use Songbird's discovery API to find other nodes
    
    debug!("Discovered {} primals", self.discovered_primals.len());
    
    Ok(self.discovered_primals.values().cloned().collect())
}
```

**Integration**:
```rust
pub async fn discover(&mut self) -> FederationResult<Vec<DiscoveredPrimal>> {
    info!("Starting primal discovery");
    
    // 1. Discover via Unix sockets
    self.discover_unix_sockets().await?;
    
    // 2. Discover via environment variables
    self.discover_from_env()?;
    
    // 3. Discover via Songbird UDP multicast
    self.discover_via_songbird().await?;
    
    debug!("Discovered {} primals", self.discovered_primals.len());
    
    Ok(self.discovered_primals.values().cloned().collect())
}

/// Discover primals via Songbird UDP multicast
async fn discover_via_songbird(&mut self) -> FederationResult<()> {
    info!("Discovering primals via Songbird UDP multicast");
    
    // Create Songbird client
    let songbird_client = SongbirdClient::from_discovery()?;
    
    // Query Songbird for all nodes in the family
    let discovered_nodes = songbird_client
        .discover_nodes()
        .await
        .map_err(|e| FederationError::DiscoveryError(format!("Songbird discovery failed: {}", e)))?;
    
    // Convert Songbird nodes to DiscoveredPrimals
    for node in discovered_nodes {
        for primal in node.primals {
            let discovered = DiscoveredPrimal {
                name: primal.name.clone(),
                primal_type: primal.primal_type.clone(),
                capabilities: Self::parse_capabilities(&primal.capabilities),
                endpoints: vec![Self::parse_endpoint(&primal.endpoint)],
                metadata: std::collections::HashMap::from([
                    ("discovered_via".to_string(), "songbird_udp".to_string()),
                    ("node_id".to_string(), node.node_id.clone()),
                    ("family_id".to_string(), node.family_id.clone()),
                ]),
            };
            
            debug!("Discovered primal via Songbird: {} on node {}", primal.name, node.node_id);
            self.discovered_primals.insert(primal.name.clone(), discovered);
        }
    }
    
    info!("Songbird discovery complete: {} primals found", self.discovered_primals.len());
    Ok(())
}
```

---

### 2. **Songbird Client Implementation**

**Location**: `crates/biomeos-federation/src/songbird_client.rs` (to be created)

```rust
//! Songbird client for primal discovery via UDP multicast

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

use crate::discovery::PrimalDiscovery;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdNode {
    pub node_id: String,
    pub family_id: String,
    pub primals: Vec<SongbirdPrimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdPrimal {
    pub name: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryRequest {
    pub requester_node_id: String,
    pub family_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub nodes: Vec<SongbirdNode>,
}

pub struct SongbirdClient {
    multicast_addr: SocketAddr,
    local_node_id: String,
}

impl SongbirdClient {
    /// Create a Songbird client from runtime discovery
    pub fn from_discovery() -> Result<Self> {
        // Get local node ID
        let local_node_id = std::env::var("NODE_ID")
            .unwrap_or_else(|_| "unknown-node".to_string());
        
        // Songbird multicast address (standard)
        let multicast_addr = "224.0.0.251:5353".parse()?;
        
        Ok(Self {
            multicast_addr,
            local_node_id,
        })
    }
    
    /// Discover all nodes in the network via UDP multicast
    pub async fn discover_nodes(&self) -> Result<Vec<SongbirdNode>> {
        info!("Sending Songbird discovery request to {}", self.multicast_addr);
        
        // Create UDP socket
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.set_broadcast(true)?;
        
        // Prepare discovery request
        let request = DiscoveryRequest {
            requester_node_id: self.local_node_id.clone(),
            family_id: None, // Discover all families
        };
        
        let request_json = serde_json::to_string(&request)?;
        
        // Send multicast discovery request
        socket.send_to(request_json.as_bytes(), self.multicast_addr).await?;
        
        // Listen for responses (with timeout)
        let mut discovered_nodes = Vec::new();
        let timeout = tokio::time::Duration::from_secs(5);
        
        tokio::select! {
            _ = tokio::time::sleep(timeout) => {
                debug!("Songbird discovery timeout reached");
            }
            result = self.receive_responses(&socket) => {
                discovered_nodes = result?;
            }
        }
        
        info!("Songbird discovery complete: {} nodes found", discovered_nodes.len());
        Ok(discovered_nodes)
    }
    
    /// Receive discovery responses from Songbird
    async fn receive_responses(&self, socket: &UdpSocket) -> Result<Vec<SongbirdNode>> {
        let mut discovered_nodes = Vec::new();
        let mut buf = vec![0u8; 65536];
        
        loop {
            let (len, _addr) = socket.recv_from(&mut buf).await?;
            let response_json = String::from_utf8_lossy(&buf[..len]);
            
            match serde_json::from_str::<DiscoveryResponse>(&response_json) {
                Ok(response) => {
                    for node in response.nodes {
                        debug!("Discovered node via Songbird: {}", node.node_id);
                        discovered_nodes.push(node);
                    }
                }
                Err(e) => {
                    warn!("Failed to parse Songbird response: {}", e);
                }
            }
        }
    }
    
    /// Discover nodes in a specific family
    pub async fn discover_family(&self, family_id: &str) -> Result<Vec<SongbirdNode>> {
        // Similar to discover_nodes but filters by family_id
        unimplemented!("Family-specific discovery not yet implemented")
    }
}
```

---

### 3. **Songbird API Requirements**

#### **Discovery Request Format**

**Multicast Group**: `224.0.0.251:5353` (mDNS standard)  
**Protocol**: UDP  
**Format**: JSON

**Request**:
```json
{
  "requester_node_id": "node-alpha-laptop",
  "family_id": null
}
```

**Response** (from each node):
```json
{
  "nodes": [
    {
      "node_id": "node-beta-desktop",
      "family_id": "nat0",
      "primals": [
        {
          "name": "songbird",
          "primal_type": "federation",
          "capabilities": ["discovery", "voice", "video"],
          "endpoint": "unix:///tmp/songbird-node-beta.sock"
        },
        {
          "name": "beardog",
          "primal_type": "security",
          "capabilities": ["encryption", "authentication"],
          "endpoint": "unix:///tmp/beardog-node-beta.sock"
        }
      ]
    }
  ]
}
```

---

## 🎯 Benefits of Songbird Integration

### 1. **Zero-Config Mesh Networking**
- Nodes automatically discover each other on the LAN
- No manual configuration required
- Works across different network topologies

### 2. **Dynamic Topology**
- Nodes can join/leave at runtime
- Automatic failover
- Load balancing across available nodes

### 3. **Family-Based Discovery**
- Filter by genetic lineage (family_id)
- Only discover trusted nodes
- Hierarchical discovery for sub-federations

### 4. **Primal Capability Discovery**
- Discover what primals are running on each node
- Query by capability (e.g., "find all nodes with storage")
- Enable intelligent workload distribution

---

## 📋 Integration Checklist

### Prerequisites
- [ ] Songbird has stable UDP multicast API
- [ ] Songbird implements discovery request/response format
- [ ] Songbird supports family-based filtering
- [ ] Songbird API documentation is complete

### Phase 1: Basic Discovery
- [ ] Add `songbird_client.rs` to `biomeos-federation`
- [ ] Implement `SongbirdClient::from_discovery()`
- [ ] Implement `discover_nodes()` with UDP multicast
- [ ] Test discovery in local environment

### Phase 2: Family Filtering
- [ ] Implement `discover_family()` for family-specific discovery
- [ ] Add family_id to discovery requests
- [ ] Filter responses by family_id

### Phase 3: Capability Queries
- [ ] Add capability-based discovery API
- [ ] Implement `discover_by_capability()` function
- [ ] Test workload distribution

### Phase 4: Integration with PrimalDiscovery
- [ ] Update `PrimalDiscovery::discover()` to call Songbird
- [ ] Merge Unix socket, env var, and Songbird results
- [ ] Handle conflicts (same primal from multiple sources)

---

## 🎯 Design Principles

### 1. **Composability**
- Songbird handles discovery
- biomeOS consumes discovery results
- Clear API boundaries

### 2. **Fallback Mechanisms**
- If Songbird unavailable, use Unix sockets + env vars
- Graceful degradation
- No single point of failure

### 3. **Security**
- Only discover nodes with matching family_id
- Verify genetic lineage via BearDog
- Encrypted communication via BTSP

### 4. **Performance**
- Cache discovery results
- Periodic refresh (e.g., every 30 seconds)
- Lazy discovery on-demand

---

## 🚀 Usage Example

```rust
use biomeos_federation::discovery::PrimalDiscovery;
use biomeos_federation::capability::Capability;

#[tokio::main]
async fn main() -> Result<()> {
    // Create discovery system
    let mut discovery = PrimalDiscovery::new();
    
    // Discover all primals (includes Songbird UDP multicast)
    let primals = discovery.discover().await?;
    
    println!("Discovered {} primals:", primals.len());
    for primal in &primals {
        println!("  - {} ({})", primal.name, primal.primal_type);
        println!("    Endpoints: {:?}", primal.endpoints);
        println!("    Capabilities: {:?}", primal.capabilities.all());
    }
    
    // Find all primals with storage capability
    let storage_primals = discovery.with_capability(&Capability::Storage);
    println!("\nPrimals with storage: {}", storage_primals.len());
    
    // Find specific primal
    if let Some(beardog) = discovery.get("beardog") {
        println!("\nFound BearDog at: {:?}", beardog.endpoints);
    }
    
    Ok(())
}
```

---

## 📚 Related Documentation

- `SPORE_INCUBATION_HIERARCHICAL_FEDERATION_JAN8.md` - System design
- `BEARDOG_INTEGRATION_PLAN_JAN8.md` - BearDog integration
- Songbird API Documentation (external)

---

## ✅ Status

**Current**: ⏳ Awaiting Songbird UDP Multicast API  
**Blocked By**: Songbird discovery request/response format  
**Estimated Effort**: 2-3 hours once Songbird API is ready  

**🌟 biomeOS is ready to integrate - waiting on Songbird!**


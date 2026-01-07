# 🔐 BTSP Responsibility Analysis - January 7, 2026

## 🎯 The Question

> "Tower stack uses RPC but comms between towers is still HTTP? BTSP is for P2P - we just have it as secure, encrypted P2P. Is this a biomeOS, BearDog, or Songbird issue?"

## ✅ Answer: **Songbird's Responsibility**

**BTSP tunnel establishment between towers is SONGBIRD's job.**

## 🏗️ Architecture Breakdown

### Current State: Hybrid
```
┌─────────────────────────────────────────────────────────┐
│  Tower-to-Tower Communication (LAN)                     │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  1. Discovery: UDP Multicast ✅                         │
│     └─ Songbird broadcasts identity                    │
│                                                          │
│  2. Trust Evaluation: Unix Socket + JSON-RPC ✅         │
│     └─ Songbird → BearDog: "Is this peer trusted?"     │
│                                                          │
│  3. P2P Communication: HTTPS ⚠️  (LEGACY)               │
│     └─ Songbird → Remote Songbird: HTTPS API           │
│                                                          │
│  TARGET:                                                 │
│  3. P2P Communication: BTSP Tunnel ⏭️  (NOT IMPLEMENTED)│
│     └─ Songbird → BearDog: "Create tunnel to peer"     │
│     └─ BearDog: Creates encrypted BTSP tunnel          │
│     └─ Songbird ←[BTSP]→ Remote Songbird               │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

## 🔧 Component Responsibilities

### BearDog ✅ (Security Primal)
**Role**: Provide BTSP tunnel infrastructure

**Capabilities**:
- ✅ Create BTSP tunnels (implemented)
- ✅ Manage tunnel encryption keys (implemented)
- ✅ Monitor tunnel health (implemented)
- ✅ Close tunnels (implemented)

**API**:
```rust
// BearDog exposes:
beardog.establish_tunnel(peer_id, peer_endpoint) -> TunnelInfo
beardog.get_tunnel_status(tunnel_id) -> TunnelStatus
beardog.close_tunnel(tunnel_id) -> Result<()>
```

**Code Evidence**:
```rust
// From: crates/biomeos-core/src/clients/beardog.rs
/// BTSP (BirdSong Tunnel Protocol) tunnel management
pub async fn establish_tunnel(&self, peer_id: &str, peer_endpoint: &str) -> Result<TunnelInfo>
pub async fn get_tunnel_status(&self, tunnel_id: &str) -> Result<TunnelStatus>
pub async fn close_tunnel(&self, tunnel_id: &str) -> Result<()>
```

**Status**: ✅ **COMPLETE** - BearDog has all BTSP capabilities ready

---

### Songbird ⚠️ (Discovery & Orchestration Primal)
**Role**: Orchestrate peer-to-peer connections

**Current Behavior**:
```rust
// After successful trust evaluation:
1. Discovery: Find peer via UDP multicast ✅
2. Trust Check: Ask BearDog "Is peer trusted?" ✅
3. Connect: Use HTTPS to peer ⚠️  WRONG!
```

**Should Be**:
```rust
// After successful trust evaluation:
1. Discovery: Find peer via UDP multicast ✅
2. Trust Check: Ask BearDog "Is peer trusted?" ✅
3. Request Tunnel: Ask BearDog "Create BTSP tunnel to peer" ⏭️
4. Use Tunnel: Communicate via BTSP (encrypted, port-free) ⏭️
```

**What's Missing in Songbird**:
```rust
// After trust evaluation returns "same_genetic_family":
async fn handle_trusted_peer(&self, peer: &Peer) -> Result<()> {
    // ❌ Currently does:
    let https_url = format!("https://{}:{}", peer.address, peer.port);
    self.http_client.connect(&https_url).await?;
    
    // ✅ Should do:
    let tunnel = self.beardog_client
        .establish_tunnel(&peer.node_id, &peer.endpoint)
        .await?;
    self.use_btsp_tunnel(tunnel).await?;
}
```

**Status**: ⏭️ **NOT IMPLEMENTED** - Songbird needs to request BTSP tunnels

---

### biomeOS 🔄 (Orchestrator)
**Role**: Coordinate between primals (local only)

**What biomeOS Does**:
- ✅ Start primals (BearDog, Songbird)
- ✅ Configure environment variables
- ✅ Connect Songbird to BearDog via Unix socket
- ✅ Monitor primal health

**What biomeOS Does NOT Do**:
- ❌ Coordinate tower-to-tower communication
- ❌ Manage BTSP tunnels between towers
- ❌ Handle peer-to-peer federation

**Why**: biomeOS is **local orchestration only**. Inter-tower communication is Songbird's domain.

**Status**: ✅ **COMPLETE** - biomeOS has done its job

---

## 🎯 Responsibility Matrix

| Task | BearDog | Songbird | biomeOS |
|------|---------|----------|---------|
| Provide BTSP API | ✅ Done | - | - |
| Create tunnels | ✅ Done | - | - |
| Discover peers | - | ✅ Done | - |
| Evaluate trust | ✅ Done | 🔄 Requests | - |
| **Request tunnels** | - | ⏭️ **TODO** | - |
| Use tunnels | - | ⏭️ **TODO** | - |
| Start primals | - | - | ✅ Done |
| Local IPC | - | - | ✅ Done |

## 🔍 The Gap: Songbird Federation Logic

### Current Flow (v3.14.1):
```
Songbird discovers peer
  ↓
Songbird → BearDog: "Evaluate trust for peer X"
  ↓
BearDog: "same_genetic_family" ✅
  ↓
Songbird: Connect via HTTPS ⚠️
```

### Target Flow (BTSP):
```
Songbird discovers peer
  ↓
Songbird → BearDog: "Evaluate trust for peer X"
  ↓
BearDog: "same_genetic_family" ✅
  ↓
Songbird → BearDog: "Create BTSP tunnel to peer X" ⏭️
  ↓
BearDog: Returns tunnel_id, encryption keys ⏭️
  ↓
Songbird: Use BTSP tunnel for P2P ⏭️
```

## 📋 What Songbird Needs to Implement

### 1. Tunnel Request Logic
```rust
// After successful trust evaluation in discovery_bridge.rs
if trust_decision.decision == TrustDecision::AutoAccept {
    // NEW: Request BTSP tunnel
    let tunnel_req = self.request_btsp_tunnel(&peer).await?;
    info!("✅ BTSP tunnel established: {}", tunnel_req.tunnel_id);
}
```

### 2. BTSP Client
```rust
// Add to Songbird's dependencies
use songbird_universal::beardog_client::BearDogClient;

impl DiscoveryBridge {
    async fn request_btsp_tunnel(&self, peer: &Peer) -> Result<TunnelInfo> {
        // Connect to local BearDog
        let beardog = BearDogClient::new(
            &self.security_endpoint // unix:///tmp/beardog-nat0-tower1.sock
        );
        
        // Request tunnel
        let tunnel = beardog.establish_tunnel(
            &peer.node_id,
            &peer.endpoint
        ).await?;
        
        Ok(tunnel)
    }
}
```

### 3. BTSP Transport Layer
```rust
// Replace HTTPS client with BTSP client
impl PeerCommunication {
    async fn send_to_peer(&self, peer: &Peer, msg: &Message) -> Result<()> {
        // OLD:
        // let url = format!("https://{}:{}", peer.address, peer.port);
        // self.https_client.post(&url).json(msg).send().await?;
        
        // NEW:
        let tunnel = self.get_or_create_tunnel(peer).await?;
        self.btsp_client.send_over_tunnel(tunnel.id, msg).await?;
        Ok(())
    }
}
```

## 🚀 Implementation Plan

### Phase 1: Songbird → BearDog BTSP Integration
**Owner**: Songbird team  
**Files to Modify**:
- `songbird/src/app/discovery_bridge.rs`
- `songbird/src/app/connection_manager.rs`
- `songbird-universal/src/beardog_client.rs` (new)

**Changes**:
1. After trust evaluation succeeds, request BTSP tunnel from BearDog
2. Store tunnel_id in peer registry
3. Use tunnel for subsequent peer communication

**ETA**: 4-6 hours

### Phase 2: Replace HTTPS with BTSP
**Owner**: Songbird team  
**Changes**:
1. Create BTSP transport layer
2. Replace HTTP client calls with BTSP tunnel sends
3. Deprecate HTTPS ports (make optional for debugging)

**ETA**: 8-12 hours

### Phase 3: Full Port-Free Federation
**Owner**: Songbird team  
**Result**:
- No HTTPS ports needed for federation
- All P2P via BTSP tunnels
- HTTPS optional (disabled by default)

**ETA**: Full day

## 🎯 Summary

### Question: Who's responsible for BTSP between towers?
**Answer**: **SONGBIRD**

### Current Status:
- BearDog: ✅ Has BTSP tunnel creation API ready
- Songbird: ⚠️ Still using HTTPS (legacy)
- biomeOS: ✅ Local orchestration complete

### The Issue:
**Songbird needs to call BearDog's BTSP API** after trust evaluation succeeds, instead of using HTTPS.

### Why This Happened:
1. BearDog implemented BTSP capabilities (done months ago)
2. Songbird hasn't been updated to USE those capabilities yet
3. Federation works via HTTPS (legacy path)
4. BTSP integration was deferred (technical debt)

### Priority:
🟡 **MEDIUM** - Current HTTPS federation works and is secure (TLS), but:
- Not port-free (architectural goal)
- Not using genetic lineage encryption (BTSP feature)
- Technical debt blocking "true" port-free architecture

---

**Date**: January 7, 2026, 21:20 UTC  
**Conclusion**: This is a **Songbird implementation gap**, not a biomeOS or BearDog issue.  
**Next Step**: Songbird team needs to integrate BearDog's BTSP API into federation logic.


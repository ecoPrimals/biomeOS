# Sovereign Beacon Mesh - Evolution Handoff

**Created**: February 6, 2026  
**Status**: ARCHITECTURE COMPLETE - Implementation In Progress  
**Priority**: HIGH - Enables true sovereign NAT traversal  
**Crate**: `songbird-onion-relay`  
**Location**: `/home/eastgate/Development/ecoPrimals/phase1/songbird/crates/songbird-onion-relay/`

---

## Executive Summary

We've designed and partially implemented a **distributed beacon mesh** that enables family devices to discover and communicate across the internet **without port forwarding or external infrastructure**.

### The Problem

Two devices behind symmetric NAT cannot connect without an intermediary. Traditional solutions require:
- Port forwarding (manual network config)
- External TURN servers (not sovereign)
- VPS hosting (ongoing cost)

### The Solution

**Tor onion services for bootstrap, then organic mesh growth:**

1. First device creates Tor .onion address (reachable without port forward)
2. Other devices connect via Tor (outbound works through any NAT)
3. Once connected, every device becomes a potential relay for others
4. Mesh grows organically - Tor becomes fallback only

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│              SOVEREIGN BEACON MESH                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  BOOTSTRAP (Tor - used once per new device)                     │
│  ┌──────────────────────────────────────────────────────────────┐
│  │ Tower creates .onion address via Arti (Pure Rust Tor)        │
│  │ .onion embedded in family beacon seed (encrypted)            │
│  │ New devices connect outbound via Tor → join mesh             │
│  └──────────────────────────────────────────────────────────────┘
│                                                                 │
│  HOLE PUNCH (attempt direct P2P)                                │
│  ┌──────────────────────────────────────────────────────────────┐
│  │ Exchange STUN-discovered addresses via signaling             │
│  │ Coordinated simultaneous UDP open                            │
│  │ ~30% success with symmetric NAT, ~90% with cone NAT          │
│  └──────────────────────────────────────────────────────────────┘
│                                                                 │
│  MESH RELAY (distributed fallback)                              │
│  ┌──────────────────────────────────────────────────────────────┐
│  │ Every connected device becomes a relay                       │
│  │ Auto-path-finding: Local > Direct > Family > Tor             │
│  │ Multi-hop: A→B→C if A can't reach C directly                 │
│  │ No single point of failure                                   │
│  └──────────────────────────────────────────────────────────────┘
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## What Was Built

### Crate: `songbird-onion-relay`

Location: `phase1/songbird/crates/songbird-onion-relay/`

| File | Lines | Status | Purpose |
|------|-------|--------|---------|
| `lib.rs` | ~40 | ✅ Complete | Crate entry, re-exports |
| `error.rs` | ~40 | ✅ Complete | Error types |
| `signaling.rs` | ~180 | ✅ Complete | Signaling protocol (transport-agnostic) |
| `coordinator.rs` | ~360 | ✅ Complete | Hole punch coordination |
| `mesh.rs` | ~350 | ✅ Complete | Distributed relay mesh |
| `tor_transport.rs` | 0 | ❌ Not started | Arti integration |

### Rendezvous Server Evolution

Location: `phase1/songbird/rendezvous/`

The existing rendezvous WebSocket server was enhanced to become a **beacon relay**:

| File | Status | Changes |
|------|--------|---------|
| `src/websocket.rs` | ✅ Modified | Added `RelayMessage` protocol, peer tracking, message forwarding |
| `Cargo.toml` | ✅ Modified | Added `lazy_static` for peer connections |

The server now supports structured JSON messages (`Forward`, `Beacon`, `ListPeers`, `Ping/Pong`) and can route encrypted beacons between connected peers.

### Key Types

```rust
/// Relay endpoint types (priority order)
pub enum EndpointType {
    Local { addr: SocketAddr },      // Priority 0: Same LAN
    Direct { addr: SocketAddr },     // Priority 1: Hole punch succeeded
    FamilyRelay { relay_node_id },   // Priority 2: Via family member
    TorOnion { onion_addr },         // Priority 3: Bootstrap/fallback
}

/// Signaling messages (transport-agnostic)
pub enum SignalingMessage {
    Register { peer_info, encrypted_beacon },
    Query { target_node_id },
    PunchRequest { from, to, nonce },
    PunchAck { from, nonce, start_at_ms },
    PunchResult { nonce, success, connected_addr },
    RelayData { from, to, data },
    // ...
}

/// Beacon mesh state
pub struct BeaconMesh {
    my_node_id: String,
    endpoints: HashMap<String, Vec<RelayEndpoint>>,
    my_onion: Option<String>,
    bootstrap_onions: Vec<String>,
    best_paths: HashMap<String, RelayEndpoint>,
}

/// Hole punch coordinator
pub struct HolePunchCoordinator {
    socket: Arc<UdpSocket>,
    stun_client: StunClient,
    config: HolePunchConfig,
}
```

---

## What Needs to Be Done

### ⚠️ ARCHITECTURE EVOLUTION (Feb 6, 2026)

**Songbird team is building a lighter "Sovereign Onion Service"** instead of using the heavy Arti library (~5MB). This follows the **TRUE PRIMAL** pattern - all crypto delegated to BearDog.

See: `BEARDOG_ONION_CRYPTO_HANDOFF_FEB06_2026.md`

### Phase 1: BearDog SHA3-256 (1 hour) - PARALLEL WORK

**Owner**: BearDog Team

BearDog needs **one new method** for .onion address derivation:

```rust
// beardog.crypto.sha3_256
// Used to compute .onion address checksum per Tor v3 spec
```

**Existing BearDog crypto** (already available):
- ✅ Ed25519 signing/verification (for .onion identity)
- ✅ X25519 key exchange (for session keys)
- ✅ ChaCha20-Poly1305 (for data encryption)
- ✅ HMAC-SHA256 (for HKDF key derivation)
- ⚠️ SHA3-256 (NEED TO ADD)

**Effort**: ~1 hour

### Phase 1b: Sovereign Onion Service (Songbird - 4 hours) - PARALLEL WORK

**Owner**: Songbird Team

Songbird is building `songbird-sovereign-onion` crate that:
- Has ZERO crypto dependencies (all delegated to BearDog)
- Generates valid Tor v3 .onion addresses
- Handles onion routing without full Arti

**Key difference from Arti approach**:
- Arti: ~5MB, full Tor client, 10-30s bootstrap
- Sovereign: ~200KB, purpose-built, instant start

### Phase 1c: Update `songbird-onion-relay` (biomeOS - 2 hours) - PARALLEL WORK

**Owner**: biomeOS Team

Update our `songbird-onion-relay` crate to use the new Songbird Sovereign Onion:

```rust
// Instead of:
// use arti_client::{TorClient, TorClientConfig};

// Use:
use songbird_sovereign_onion::{OnionIdentity, OnionService};

pub struct OnionTransport {
    identity: OnionIdentity,
    service: Option<OnionService>,
}

impl OnionTransport {
    /// Create new onion transport (delegating crypto to BearDog)
    pub async fn new(beardog: &BeardogCryptoClient) -> Result<Self> {
        let identity = OnionIdentity::generate_via_beardog(beardog).await?;
        
        Ok(Self {
            identity,
            service: None,
        })
    }
    
    /// Start accepting connections on our .onion
    pub async fn serve(&mut self, port: u16) -> Result<String> {
        let service = OnionService::bind(
            &self.identity,
            port,
        ).await?;
        
        let address = self.identity.onion_address();
        self.service = Some(service);
        
        Ok(address)
    }
    
    /// Connect to another .onion address
    pub async fn connect(&self, onion_addr: &str, port: u16) -> Result<OnionStream> {
        OnionService::connect(onion_addr, port).await
    }
}
```

**Benefits over Arti**:
- Much smaller binary (~200KB vs ~5MB)
- Faster startup (no Tor bootstrap)
- TRUE PRIMAL compliant (crypto in BearDog)
- Purpose-built for our use case

### Phase 2: Wire into Songbird IPC (1-2 days)

Add JSON-RPC methods to `songbird-universal-ipc`:

```rust
// In service.rs, add to handle() match:

"mesh.status" => {
    let mesh = self.beacon_mesh.read().await;
    let reachable = mesh.get_reachable_nodes().await;
    let my_onion = mesh.my_onion.read().await.clone();
    
    json!({
        "node_id": mesh.my_node_id,
        "reachable_nodes": reachable.len(),
        "my_onion": my_onion,
        "paths": mesh.best_paths.read().await.len(),
    })
}

"mesh.find_path" => {
    let target = params.get("target_node_id").as_str()?;
    let path = self.beacon_mesh.find_relay_for(target).await;
    // ...
}

"mesh.announce" => {
    let msg = self.beacon_mesh.announce_as_relay().await;
    // Broadcast via BirdSong
}
```

### Phase 3: Integration with Existing Relay (1 day)

Wire `BeaconMesh` into `songbird-lineage-relay`:

```rust
// In relay.rs, use mesh for path selection:

impl RelayDiscovery {
    pub async fn request_relay_via_mesh(
        &self,
        target: NodeId,
        mesh: &BeaconMesh,
    ) -> Result<Arc<RelaySession>> {
        // 1. Check mesh for best path
        if let Some(endpoint) = mesh.find_relay_for(&target.0).await {
            match endpoint.endpoint_type {
                EndpointType::Direct { addr } => {
                    // Direct connection available
                    return self.create_direct_session(target, addr).await;
                }
                EndpointType::FamilyRelay { relay_node_id } => {
                    // Route through family member
                    return self.create_relayed_session(target, relay_node_id).await;
                }
                EndpointType::TorOnion { onion_addr } => {
                    // Fall back to Tor
                    return self.create_tor_session(target, onion_addr).await;
                }
                _ => {}
            }
        }
        
        // 2. Fall back to existing BirdSong relay discovery
        self.request_relay(target, None).await
    }
}
```

### Phase 4: BirdSong Layered Encryption (2-3 days)

Evolve BirdSong to support layered encryption:

```rust
/// Layered BirdSong message
pub struct LayeredBirdSong {
    /// Unencrypted routing header
    pub header: RoutingHeader,
    
    /// Layer 1: Family-encrypted (beacon seed)
    /// Everyone in family can read
    pub family_layer: Vec<u8>,
    
    /// Layer 2: Lineage-encrypted (optional)
    /// Only direct ancestors/descendants
    pub lineage_layer: Option<Vec<u8>>,
    
    /// Layer 3: Device-specific (optional)
    /// Only target device can read
    pub device_layer: Option<Vec<u8>>,
}

pub struct RoutingHeader {
    /// Sender's .onion for replies
    pub reply_onion: Option<String>,
    /// Message type
    pub msg_type: u8,
    /// TTL for gossip
    pub ttl: u8,
    /// Deduplication nonce
    pub nonce: [u8; 16],
}
```

---

## Onion Service Considerations

### Architecture Evolution: Sovereign Onion vs Arti

| Approach | Binary Size | Startup | Crypto | Status |
|----------|-------------|---------|--------|--------|
| **Arti (original plan)** | ~5MB | 10-30s | Built-in | Superseded |
| **Sovereign Onion (new)** | ~200KB | Instant | BearDog delegation | In progress |

**Why the change?**

Songbird team identified that Arti is overkill for our use case:
- We don't need full Tor client functionality
- We only need .onion address generation + basic routing
- TRUE PRIMAL requires crypto in BearDog anyway

### Why Onion Addresses for Bootstrap?

The user explicitly requested a **pure Rust, self-hosted solution** that:
- Works across bi-symmetric NAT
- Requires zero external dependencies
- Requires no port forwarding
- Enables any connection to become a relay

Onion addresses uniquely satisfy all requirements:
- **No port forward needed**: Outbound connections work through any NAT
- **Globally reachable**: Any device can connect to a `.onion`
- **Cryptographic identity**: Address IS the public key
- **Family-encrypted**: Embedded in beacon seeds

### Risks (Discussed with Team)

| Risk | Mitigation |
|------|------------|
| Legal/perception | Tor is legal; used only for bootstrap, not data |
| Latency (300-800ms) | Used only for initial signaling, then direct P2P |
| Network dependency | Mesh works without onion after initial connection |
| Tor network reliance | Only for .onion routing; crypto is ours |

### Alternatives Considered

| Alternative | Pros | Cons |
|-------------|------|------|
| Full Arti | Feature-complete | Heavy (~5MB), slow bootstrap |
| I2P | Similar anonymity | Smaller network, less mature |
| WireGuard | Lower latency | Needs one reachable endpoint |
| Public TURN | Works now | Not sovereign |
| Port forward | Simple | Manual config per-site |

**Decision**: Sovereign Onion Service because it's:
- Pure Rust (no C dependencies)
- Lightweight (~200KB vs ~5MB)
- TRUE PRIMAL compliant (crypto in BearDog)
- Creates reachable endpoint without port forward
- Used minimally (just bootstrap signaling)

---

## Testing Plan

### Unit Tests (in crate)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird/crates/songbird-onion-relay
cargo test
```

### Integration Test

```bash
# Terminal 1: Tower with Tor
RUST_LOG=info cargo run -- server --socket tower-mesh --features tor

# Terminal 2: Check mesh
echo '{"jsonrpc":"2.0","method":"mesh.status","id":1}' | nc -U tower-mesh

# Terminal 3: Pixel connects
# (Deploy to Pixel, connect via ADB)
```

### Cross-NAT Validation

1. Tower on home ISP (symmetric NAT)
2. Pixel on iPhone hotspot (symmetric NAT)
3. Tower creates .onion
4. Pixel connects via Tor
5. Exchange STUN addresses
6. Attempt hole punch
7. Verify: direct connection OR relay working

---

## Files Reference

### New Crate Structure

```
phase1/songbird/crates/songbird-onion-relay/
├── Cargo.toml              # Dependencies (Arti optional)
├── src/
│   ├── lib.rs              # Entry point
│   ├── error.rs            # Error types
│   ├── signaling.rs        # Signaling protocol
│   ├── coordinator.rs      # Hole punch coordinator
│   ├── mesh.rs             # Distributed relay mesh
│   └── tor_transport.rs    # TODO: Arti integration
```

### Related Files to Update

| File | Change Needed |
|------|---------------|
| `songbird-universal-ipc/src/service.rs` | Add mesh.* methods |
| `songbird-lineage-relay/src/coordinator.rs` | Use mesh for path selection |
| `songbird-discovery/src/birdsong/` | Layered encryption |
| `songbird/Cargo.toml` | Add `onion-relay` feature |
| `rendezvous/src/websocket.rs` | Already enhanced ✅ |

---

## Specs Created

| Spec | Location |
|------|----------|
| Sovereign BirdSong Mesh | `biomeOS/specs/SOVEREIGN_BIRDSONG_MESH_SPEC.md` |
| NAT Traversal Evolution | `biomeOS/specs/SOVEREIGN_NAT_TRAVERSAL_EVOLUTION.md` |

---

## Key Design Decisions

1. **Tor as optional feature** - `--features tor` to keep binary small by default
2. **Transport-agnostic signaling** - Same protocol works over Tor, WebSocket, TCP
3. **Auto-path-finding** - Mesh automatically selects best route (Local > Direct > Family > Tor)
4. **No single point of failure** - Any node can relay for any other
5. **Layered encryption** - Family can see routing, only target sees content
6. **Distributed mesh** - After initial connection, any connection becomes a potential relay

---

## Timeline Estimate

### Parallel Work Streams (3 Teams)

```
Day 1-2:
┌─────────────────────────────────────────────────────────────┐
│ BearDog Team          │ Songbird Team        │ biomeOS Team │
├───────────────────────┼──────────────────────┼──────────────┤
│ SHA3-256 method       │ Sovereign Onion      │ Update       │
│ (~1 hour)             │ Service (~4 hours)   │ onion-relay  │
│                       │                      │ (~2 hours)   │
└───────────────────────┴──────────────────────┴──────────────┘

Day 2-3:
┌─────────────────────────────────────────────────────────────┐
│            Integration Testing (All Teams)                   │
│            - BearDog ↔ Songbird crypto delegation           │
│            - biomeOS lifecycle coordination                  │
└─────────────────────────────────────────────────────────────┘

Day 3-4:
┌─────────────────────────────────────────────────────────────┐
│            IPC Wiring + Mesh Integration                     │
│            - mesh.status, mesh.find_path methods            │
│            - Wire BeaconMesh into relay                      │
└─────────────────────────────────────────────────────────────┘

Day 4-5:
┌─────────────────────────────────────────────────────────────┐
│            Layered BirdSong + E2E Testing                    │
│            - Family/lineage/device encryption layers         │
│            - Cross-NAT validation (Tower ↔ Pixel)            │
└─────────────────────────────────────────────────────────────┘
```

| Phase | Team | Effort | Parallel? |
|-------|------|--------|-----------|
| BearDog SHA3-256 | BearDog | 1 hour | ✅ |
| Sovereign Onion | Songbird | 4 hours | ✅ |
| Update onion-relay | biomeOS | 2 hours | ✅ |
| Integration test | All | 2 hours | After parallel |
| IPC Wiring | biomeOS | 4 hours | Sequential |
| Relay integration | biomeOS | 2 hours | Sequential |
| Layered BirdSong | Songbird | 4 hours | Parallel w/above |
| E2E Testing | All | 4 hours | Final |
| **Total (serial)** | | **~23 hours** | |
| **Total (parallel)** | | **~5 days** | |

---

## Questions for Teams

### For Songbird Team

1. **Sovereign Onion API**: What will the `songbird-sovereign-onion` crate exports look like? Need to plan biomeOS integration.

2. **BearDog discovery**: Using same pattern as TLS 1.3? (`CRYPTO_PROVIDER_SOCKET` env var)

3. **Fallback priority**: Current order is Local > Direct > Family > TorOnion. Adjust?

4. **Key rotation**: How often to rotate .onion identities? Per-session? Daily? Family-wide?

### For BearDog Team

1. **SHA3-256 urgency**: Can this be prioritized? Blocks onion address derivation.

2. **Key storage**: Will .onion identity keys be stored in BearDog's secure keystore?

3. **Audit logging**: What format for onion service crypto operations?

### For biomeOS Team (Us) - ✅ PREPARATION COMPLETE

1. **Deployment graph**: ✅ `graphs/sovereign_onion_genome.toml` - Created with 4 phases

2. **`songbird-onion-relay` update**: ✅ In progress (Songbird repo)
   - Arti deps removed
   - Onion feature prepared for `songbird-sovereign-onion`

3. **Mesh IPC methods**: ✅ `specs/MESH_IPC_METHODS_SPEC.md` - Full JSON-RPC spec

4. **Capability translations**: ✅ `capability_translation.rs` updated with:
   - `mesh.status`, `mesh.find_path`, `mesh.announce`, `mesh.peers`, `mesh.health_check`
   - `punch.request`, `punch.status`
   - `stun.discover`, `stun.detect_nat_type`
   - `relay.serve`, `relay.status`, `relay.allocate`
   - `onion.create_service`, `onion.get_address`, `onion.connect`, `onion.status`
   - `crypto.sha3_256`, `onion.hash_checksum` (for BearDog)

5. **Neural API routing**: ✅ Direct method syntax sugar added
   - `mesh.*`, `punch.*`, `stun.*`, `relay.*`, `onion.*` methods route via capability.call

6. **Capability taxonomy**: ✅ `CapabilityTaxonomy` enum updated with:
   - `MeshRelay`, `HolePunch`, `StunClient`, `OnionService`, `RelayServer`

7. **NetworkConfig**: ✅ Environment-driven STUN server resolution
   - `BIOMEOS_STUN_SERVER`, `BIOMEOS_STUN_SERVERS`, `BIOMEOS_NO_PUBLIC_STUN`

8. **Path refactoring**: ✅ Using `biomeos_types::socket_path()` and `SystemPaths`

---

## Context Recovery

Full conversation transcript with detailed technical discussion available at:
`/home/eastgate/.cursor/projects/home-eastgate-Development-ecoPrimals-phase2-biomeOS/agent-transcripts/e912c725-1292-435c-a683-b562688bc91d.txt`

Related handoffs:
- **`BEARDOG_ONION_CRYPTO_HANDOFF_FEB06_2026.md`** - BearDog SHA3-256 + crypto delegation (NEW)
- `SOVEREIGN_NAT_TRAVERSAL_HANDOFF_FEB05_2026.md` - NAT traversal status
- `SONGBIRD_EVOLUTION_HANDOFF_FEB_05_2026.md` - Prior Songbird evolution

Related specs:
- `specs/SOVEREIGN_BIRDSONG_MESH_SPEC.md` - Full architecture spec
- `specs/SOVEREIGN_NAT_TRAVERSAL_EVOLUTION.md` - NAT evolution design

---

---

## Summary: Three-Team Parallel Execution

| Team | Deliverable | Effort | Status |
|------|-------------|--------|--------|
| **BearDog** | `beardog.crypto.sha3_256` method | 1 hour | 🔄 Pending |
| **Songbird** | `songbird-sovereign-onion` crate | 4 hours | 🔄 In Progress |
| **biomeOS** | Capability wiring, deployment graph | 4 hours | ✅ **COMPLETE** |

### biomeOS Preparation Complete (Feb 6, 2026)

All biomeOS infrastructure is ready for integration:

```
✅ graphs/sovereign_onion_genome.toml      - Deployment orchestration
✅ specs/MESH_IPC_METHODS_SPEC.md          - JSON-RPC method spec
✅ capability_translation.rs              - Mesh/punch/onion translations
✅ neural_api_server/routing.rs           - Direct method routing
✅ capability_taxonomy.rs                 - New capability enums
✅ stun_extension.rs                      - Environment-driven STUN config
✅ live_discovery.rs, trust.rs            - Path refactoring
```

**Next Steps**: Integration testing once BearDog SHA3-256 and Songbird Sovereign Onion are ready.

**Key Files**:
- `BEARDOG_ONION_CRYPTO_HANDOFF_FEB06_2026.md` - BearDog team's tasks
- `SOVEREIGN_BEACON_MESH_HANDOFF_FEB06_2026.md` - This document (biomeOS team)
- Songbird's `songbird-sovereign-onion` crate - Songbird team's work

---

*This evolution enables true sovereign connectivity - family devices can find and reach each other anywhere in the world without depending on external infrastructure.*

# Sovereign NAT Traversal Evolution

**Purpose**: Pure Rust, zero-external-dependency NAT traversal for biomeOS ecosystem  
**Status**: Architecture Specification  
**Created**: February 5, 2026  
**Author**: biomeOS Integration Team

---

## Executive Summary

Deploy and connect **anywhere** without external infrastructure dependencies. This specification defines how Songbird and biomeOS evolve to provide complete NAT traversal using only family-owned resources.

### Core Principle

> **The family IS the infrastructure.**  
> Tower becomes the relay server. Any family device with connectivity helps others connect.

---

## Current State Analysis

### What Exists (✅ Complete)

| Component | Location | Status |
|-----------|----------|--------|
| **STUN Server** | `songbird-stun/src/server.rs` | ✅ Pure Rust RFC 5389 |
| **STUN Client** | `songbird-stun/src/client.rs` | ✅ Multi-server racing |
| **UDP Hole Punch** | `songbird-lineage-relay/src/udp_hole_punch.rs` | ✅ Configurable attempts |
| **Lineage Relay Discovery** | `songbird-lineage-relay/src/relay.rs` | ✅ BirdSong coordination |
| **Relay Session** | `songbird-lineage-relay/src/relay.rs` | ✅ Session management |
| **Multi-tier Config** | `songbird-types/src/config/stun_relay.rs` | ✅ Strategy selection |

### What's Missing (❌ Gaps)

| Component | Gap | Impact |
|-----------|-----|--------|
| **Relay Service** | `RelaySession.send()` is stub | No actual packet forwarding |
| **STUN Server Exposure** | No `stun.serve` JSON-RPC | Can't start self-hosted STUN |
| **Relay Server Exposure** | No `relay.serve` JSON-RPC | Can't start relay service |
| **Full ICE** | No candidate gathering | Limited NAT compatibility |

---

## Architecture: Sovereign NAT Traversal

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        SOVEREIGN NAT TRAVERSAL                              │
│                                                                             │
│  "The family IS the infrastructure"                                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  TIER 1: Direct Connection (80-95% success for non-symmetric NAT)          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  1. Discover public address (STUN)                                   │   │
│  │  2. Exchange addresses via Dark Forest beacon                        │   │
│  │  3. Simultaneous UDP open (hole punch)                               │   │
│  │  4. Direct P2P established                                           │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼ (fails for symmetric NAT)              │
│  TIER 2: Family STUN Server                                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Tower runs Songbird STUN server (pure Rust)                         │   │
│  │  - Address discovery without public STUN                             │   │
│  │  - NAT type detection                                                │   │
│  │  - Zero external dependency                                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼ (symmetric NAT both sides)             │
│  TIER 3: Family Relay (Lineage-Gated)                                       │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Ancestor with connectivity relays for descendants                   │   │
│  │                                                                      │   │
│  │  ┌────────┐         ┌────────────┐         ┌────────┐              │   │
│  │  │ Pixel  │◄───────►│   Tower    │◄───────►│  USB   │              │   │
│  │  │(hotspot)│  relay  │(home ISP)  │  relay  │ (LAN)  │              │   │
│  │  └────────┘         └────────────┘         └────────┘              │   │
│  │                           │                                         │   │
│  │  - BearDog lineage verification                                     │   │
│  │  - BirdSong encrypted coordination                                  │   │
│  │  - Privacy masking options                                          │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                    │                                        │
│                                    ▼ (no family relay available)            │
│  TIER 4: Public STUN Fallback (address discovery only)                      │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Google, Cloudflare, Nextcloud STUN                                  │   │
│  │  - Only exposes IP:port (metadata)                                   │   │
│  │  - No beacon content exposed                                         │   │
│  │  - Last resort for address discovery                                 │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Evolution Phases

### Phase 1: STUN Server Exposure (Ready Now)

**Goal**: Enable Tower to serve as family STUN server

**Songbird Changes**:
```rust
// New JSON-RPC method: stun.serve
pub async fn handle_stun_serve(
    params: StunServeParams,
) -> Result<StunServeResponse> {
    let bind_addr: SocketAddr = params.bind_address.parse()?;
    let server = StunServer::new(bind_addr);
    
    // Spawn server task
    let handle = tokio::spawn(async move {
        server.run().await
    });
    
    Ok(StunServeResponse {
        status: "running",
        bind_address: bind_addr.to_string(),
    })
}
```

**biomeOS Integration**:
```toml
# Tower atomic bootstrap includes STUN server
[stun_server]
enabled = true
bind_address = "0.0.0.0:3478"
external_address = "auto"  # Discover via public STUN
```

### Phase 2: Relay Service Implementation

**Goal**: Complete the `RelaySession.send()` stub with actual packet forwarding

**Current Stub** (`relay.rs:93`):
```rust
pub async fn send(&self, data: &[u8]) -> Result<()> {
    // In real implementation, this would send through UDP socket to relay
    debug!("Sending {} bytes through relay...", data.len());
    Ok(())
}
```

**Evolution**:
```rust
/// Relay service that forwards packets between peers
pub struct RelayService {
    /// UDP socket for relay traffic
    socket: Arc<UdpSocket>,
    
    /// Active relay sessions: session_id -> (peer_a_addr, peer_b_addr)
    sessions: Arc<RwLock<HashMap<Uuid, RelaySessionState>>>,
    
    /// Lineage authority for verification
    authority: Arc<dyn RelayAuthority>,
}

impl RelayService {
    /// Start relay service
    pub async fn serve(&self, bind_addr: SocketAddr) -> Result<()> {
        let socket = UdpSocket::bind(bind_addr).await?;
        info!("🔄 Relay service listening on {}", bind_addr);
        
        loop {
            let mut buf = vec![0u8; 65535];
            let (len, src_addr) = socket.recv_from(&mut buf).await?;
            
            // Check if this is a relay packet
            if let Some(session) = self.find_session_for_source(src_addr).await {
                // Forward to the other peer
                let target = session.other_peer(src_addr);
                socket.send_to(&buf[..len], target).await?;
                
                // Update stats
                session.record_relayed(len).await;
            }
        }
    }
    
    /// Allocate relay for authorized peer
    pub async fn allocate(
        &self,
        requester: NodeId,
        target: NodeId,
    ) -> Result<RelayAllocation> {
        // Verify lineage authorization via BearDog
        let auth = self.authority.authorize_relay(&self.my_id, &requester).await?;
        
        if !auth.authorized {
            return Err(LineageRelayError::RelayDenied("Not family".into()));
        }
        
        // Create session
        let session_id = Uuid::new_v4();
        let allocation = RelayAllocation {
            session_id,
            relay_address: self.socket.local_addr()?,
            ttl: Duration::from_secs(auth.ttl_seconds),
        };
        
        // Store session
        self.sessions.write().await.insert(session_id, RelaySessionState::new(
            requester, target, auth.masking_level,
        ));
        
        Ok(allocation)
    }
}
```

### Phase 3: JSON-RPC Integration

**New Methods**:

```json
// Start relay service
{"jsonrpc":"2.0","method":"relay.serve","params":{"bind_address":"0.0.0.0:3479"},"id":1}

// Request relay allocation
{"jsonrpc":"2.0","method":"relay.allocate","params":{"target":"pixel8a"},"id":2}

// Get relay status
{"jsonrpc":"2.0","method":"relay.status","params":{},"id":3}
```

### Phase 4: Multi-Tier Orchestration

**Goal**: Seamless fallback from direct → relay

```rust
impl MultiTierCoordinator {
    pub async fn connect(&self, target: NodeId) -> Result<Connection> {
        // Tier 1: Try direct connection
        if let Ok(conn) = self.try_direct_connection(&target).await {
            info!("✅ Direct connection established");
            return Ok(conn);
        }
        
        // Tier 2: Try hole punch
        if let Ok(conn) = self.try_hole_punch(&target).await {
            info!("✅ Hole punch successful");
            return Ok(conn);
        }
        
        // Tier 3: Fall back to family relay
        if let Ok(conn) = self.try_family_relay(&target).await {
            info!("✅ Family relay established");
            return Ok(conn);
        }
        
        Err(LineageRelayError::AllTiersFailed)
    }
    
    async fn try_family_relay(&self, target: &NodeId) -> Result<Connection> {
        // Find ancestor with connectivity
        let relay_node = self.find_relay_ancestor().await?;
        
        // Request relay allocation
        let allocation = self.request_relay(&relay_node, target).await?;
        
        // Create relayed connection
        Ok(Connection::Relayed(RelayedConnection {
            relay_address: allocation.relay_address,
            session_id: allocation.session_id,
        }))
    }
}
```

---

## Deployment Scenarios

### Scenario 1: Tower at Home (Port Forwarded)

```
Tower (192.0.2.10) ──► Router ──► Internet
        │
        ├── STUN Server on :3478
        ├── Relay Service on :3479
        └── Songbird Orchestrator

Router Port Forwards:
  UDP 3478 → 192.0.2.10:3478  (STUN)
  UDP 3479 → 192.0.2.10:3479  (Relay)
  UDP 49152-65535 → 192.0.2.10 (Relay data)
```

**Family devices use Tower for**:
- Public address discovery (STUN)
- Relay when hole punch fails
- No external STUN/TURN dependencies

### Scenario 2: Tower Behind Symmetric NAT (No Port Forward)

```
Tower (symmetric NAT) ◄──► Public STUN ──► Pixel (symmetric NAT)
        │                                        │
        └────────── Family Relay ────────────────┘
                         │
                         ▼
              Cloud VPS (micro, $5/mo)
              Running Songbird relay
```

**For extreme NAT scenarios**:
- Deploy tiny Songbird instance on cloud VPS
- Same lineage-gated relay, just public IP
- Still family-owned infrastructure

### Scenario 3: LAN Only (No Internet)

```
Tower (LAN) ◄──WiFi──► Pixel (LAN) ◄──WiFi──► USB Primal

All devices discover each other via:
  - Dark Forest LAN broadcast
  - Direct connection (no NAT traversal needed)
```

---

## Privacy Model

| Tier | Who Sees What | Trust Level |
|------|--------------|-------------|
| Direct | Nobody (P2P) | ✅ Maximum |
| Family STUN | Tower sees address queries | ✅ High (family) |
| Family Relay | Tower sees encrypted traffic | ✅ High (family) |
| Public STUN | Google sees IP:port | ⚠️ Medium (metadata only) |

**Key**: All beacon content is BirdSong encrypted. External observers only see encrypted blobs.

---

## Implementation Priority

### Immediate (biomeOS can do now)

1. **Enable Songbird STUN server** - Already implemented, needs JSON-RPC exposure
2. **Configure Tower as STUN server** - Multi-tier config update
3. **Port forwarding documentation** - Router setup guide

### Short-term (Songbird evolution)

1. **Relay service implementation** - Complete the stub
2. **`relay.serve` JSON-RPC** - Expose relay service
3. **Multi-tier connection fallback** - Seamless tier switching

### Medium-term

1. **Cloud relay option** - Deploy to VPS for extreme NAT
2. **Full NAT type detection** - RFC 5780 support
3. **Relay metrics/monitoring** - Bandwidth, latency, usage

---

## Success Criteria

1. **Pixel on iPhone hotspot** can connect to **Tower at home** without:
   - External STUN servers
   - coturn or any C dependencies
   - Manual IP address exchange

2. **Connection established** within 5 seconds for any NAT combination

3. **Zero external dependencies** - Family owns all infrastructure

---

## Files to Modify

### Songbird (phase1)

| File | Change |
|------|--------|
| `songbird-orchestrator/src/bin_interface/server.rs` | Add `stun.serve` method |
| `songbird-lineage-relay/src/relay.rs` | Implement `RelayService` |
| `songbird-universal-ipc/src/handlers/` | Add relay handler |
| `songbird-types/src/config/stun_relay.rs` | Add relay serve config |

### biomeOS (phase2)

| File | Change |
|------|--------|
| `graphs/tower_atomic_bootstrap.toml` | Enable STUN server |
| `config/stun/multi_tier.toml` | Configure family STUN |
| `crates/biomeos-core/src/stun_extension.rs` | Add relay extension |

---

## Conclusion

The path to "deploy and connect anywhere" is clear:

1. **Songbird already has 80%** of what's needed (STUN server, hole punch, relay discovery)
2. **Complete the relay service** (actual packet forwarding)
3. **Expose via JSON-RPC** (stun.serve, relay.serve)
4. **Tower becomes the family's TURN server**

No external dependencies. No C code. Pure Rust. Family-owned infrastructure.

---

**Next Action**: Create Songbird PR to expose `stun.serve` JSON-RPC method

# Relay-Assisted Coordinated Punch — Ownership Handoff

**Date**: February 11, 2026  
**From**: biomeOS Integration Session  
**To**: Songbird, BearDog, biomeOS/NeuralAPI, NestGate teams  
**Priority**: P0 — This is the evolution that turns 5% symmetric→symmetric punch into 60-80%  
**Status**: Architecture analysis complete, ready for implementation

---

## Executive Summary

Today's testing confirmed: Tower + Pixel + LiveSpore are all behind **symmetric NAT**.
Blind hole punching between symmetric NATs has a ~5% success rate. But the existing
infrastructure (rendezvous, relay, STUN, hole-punch coordinator) already provides 90%
of the building blocks for a **relay-assisted coordinated punch** protocol that dramatically
improves this rate — without any new external dependencies.

The insight: **the relay is not just a fallback — it's the coordination channel that makes
the punch deterministic.** Devices meet at the rendezvous, start relaying, observe port
allocation patterns, then attempt a precisely coordinated simultaneous punch. If it works:
drop relay, go direct. If it fails: keep relay. You never go backwards.

### What Exists vs What's Needed

| Component | Location | Status |
|-----------|----------|--------|
| `HolePunchCoordinator` | `songbird-onion-relay/src/coordinator.rs` | ✅ Full signaling + timing |
| `detect_nat_type()` | Same file | ✅ 2-probe basic detection |
| `RelaySession.send()` | `songbird-lineage-relay/src/relay.rs` | ✅ Real UDP forwarding |
| `RelayProtocol` (wire format) | `songbird-lineage-relay/src/relay_protocol.rs` | ✅ Full encode/decode |
| `RelayDiscovery` (BirdSong broadcast) | `songbird-lineage-relay/src/relay.rs` | ✅ Ancestor relay |
| Self-hosted STUN server | Tower `:3478` | ✅ Running live |
| HTTPS rendezvous | `biomeos-api` `/rendezvous/beacon` | ✅ Working, Dark Forest gated |
| Neural API capability routing | `biomeos-atomic-deploy` | ✅ `punch.*`, `relay.*`, `stun.*` |
| **Relay server listen loop** | `songbird-lineage-relay` | ❌ Needs: accept + dispatch |
| **Port pattern probing** | `songbird-stun` or coordinator | ❌ Needs: N-probe STUN |
| **Punch-from-relay promotion** | Coordinator + relay integration | ❌ Needs: wiring |
| **Coordinator wired to orchestrator** | `punch.request` RPC | ❌ Returns "not_initialized" |

---

## Protocol: Relay-Assisted Coordinated Punch

```
Phase 1: MEET (existing ✅)
  Pixel ──[Dark Forest beacon]──> Tower Rendezvous <──[beacon]── USB
  Tower verifies lineage via capability.call("birdsong","decrypt")

Phase 2: RELAY (client exists ✅, server needs wiring)
  Pixel ←──[RelaySession UDP]──> Tower Relay <──[RelaySession UDP]──> USB
  Data flowing immediately. Fallback is active.

Phase 3: OBSERVE (new — port prediction)
  Pixel sends 5 STUN probes to Tower:3478 → ports [41200, 41201, 41202, 41203, 41204]
  USB   sends 5 STUN probes to Tower:3478 → ports [52100, 52105, 52110, 52115, 52120]
  
  Tower detects:
    Pixel: sequential (step=1), predict next = 41205
    USB:   skip-5 (step=5), predict next = 52125

Phase 4: COORDINATED PUNCH (coordinator exists ✅, needs prediction input)
  Tower signals both peers via relay channel:
    "Punch at T=1739318400.100, Pixel→52125, USB→41205"
  Both punch at SAME INSTANT using predicted ports.

Phase 5: PROMOTE or KEEP
  SUCCESS → swap data path from relay to direct P2P
  FAIL    → keep relay (zero disruption, no user impact)
```

---

## Ownership Boundaries

### 🧠 biomeOS / Neural API — The Coordinator Brain

**Role**: Strategy selection, multi-primal orchestration, HTTPS rendezvous, Dark Forest gating.  
**Principle**: biomeOS decides WHAT to do. Primals decide HOW.

| What biomeOS Owns | Where | Current State |
|--------------------|-------|---------------|
| Capability routing for all NAT ops | `capability_translation.rs` | ✅ `punch.*`, `relay.*`, `stun.*` → Songbird |
| HTTPS rendezvous endpoint | `biomeos-api/handlers/rendezvous.rs` | ✅ Working, Dark Forest gated |
| Dark Forest gate (token verification) | `biomeos-api/dark_forest_gate.rs` | ✅ Uses `capability.call("birdsong","decrypt")` |
| Connection strategy decision | **NEW**: `biomeos-core/connection_strategy.rs` | ❌ Needs building |
| NAT info aggregation | **NEW**: part of connection strategy | ❌ Needs building |

#### What biomeOS Does NOT Own

- ❌ UDP socket operations (Songbird)
- ❌ STUN protocol (Songbird)
- ❌ Punch packet send/recv (Songbird)
- ❌ Relay packet forwarding (Songbird)
- ❌ BirdSong encryption (BearDog)
- ❌ Lineage key derivation (BearDog)

#### New biomeOS Work

**1. Connection Strategy Orchestrator** (~100 lines)

Sits in `biomeos-core`. Given a target peer, selects the best connection tier:

```rust
/// Orchestrates the relay-assisted punch flow via Neural API
pub async fn connect_to_peer(peer_id: &str, neural_api: &NeuralApiClient) -> ConnectionResult {
    // 1. Check if peer is on local LAN (mesh.auto_discover)
    if let Ok(direct) = neural_api.call("mesh.auto_discover", json!({})).await {
        if has_peer(&direct, peer_id) { return ConnectionResult::Direct; }
    }
    
    // 2. Get our NAT type (stun.detect_nat_type)
    let our_nat = neural_api.call("stun.detect_nat_type", json!({})).await?;
    
    // 3. If both non-symmetric: direct punch (punch.request)
    // 4. If either symmetric: relay-assisted punch (relay.allocate → punch.coordinate)
    // 5. Fallback: pure relay (relay.allocate)
}
```

**2. Rendezvous Endpoint Evolution** (~30 lines)

Extend `POST /rendezvous/beacon` to include STUN results and connection preferences
in the beacon payload, so peers exchange NAT info during the rendezvous handshake:

```json
{
  "encrypted_beacon": "...",
  "dark_forest_token": "...",
  "connection_info": {
    "stun_results": { "public_addr": "1.2.3.4:41200", "nat_type": "symmetric" },
    "relay_endpoint": "192.168.1.144:3479",
    "stun_server": "192.168.1.144:3478"
  }
}
```

---

### 🐦 Songbird — The Network Engine

**Role**: All UDP/TCP transport, STUN, relay, punch, mesh, IGD.  
**Principle**: Songbird owns all sockets and wire protocols. It executes what biomeOS requests.

| What Songbird Owns | Where | Current State |
|---------------------|-------|---------------|
| STUN client (public addr discovery) | `songbird-stun/src/client.rs` | ✅ Multi-server racing |
| STUN server (self-hosted) | `songbird-stun/src/server.rs` | ✅ RFC 5389, live on :3478 |
| NAT type detection (basic) | `songbird-onion-relay/src/coordinator.rs` | ✅ 2-probe comparison |
| UDP hole punch | `songbird-lineage-relay/src/udp_hole_punch.rs` | ✅ 10 attempts, configurable |
| Hole punch coordinator | `songbird-onion-relay/src/coordinator.rs` | ✅ Full signaling flow |
| Relay client (session) | `songbird-lineage-relay/src/relay.rs` | ✅ `send()` is real |
| Relay wire protocol | `songbird-lineage-relay/src/relay_protocol.rs` | ✅ Full binary protocol |
| Relay discovery (BirdSong) | `songbird-lineage-relay/src/relay.rs` | ✅ Ancestor broadcast |
| Mesh overlay | `songbird-orchestrator` mesh handlers | ✅ init/status/peers/announce |
| IGD / UPnP | `songbird-igd` (spec) | ⚠️ Spec'd, not built |
| **Relay server listen loop** | `songbird-lineage-relay` | ❌ **Needs building** |
| **Port pattern probing** | `songbird-stun` or coordinator | ❌ **Needs building** |
| **punch.coordinate (relay-assisted)** | `songbird-onion-relay/src/coordinator.rs` | ❌ **Needs building** |
| **Wire coordinator to orchestrator** | `punch.request` RPC handler | ❌ **Needs building** |

#### What Songbird Does NOT Own

- ❌ Which connection strategy to use (biomeOS)
- ❌ Cryptographic identity / lineage verification (BearDog)
- ❌ Relay authorization decisions (BearDog)
- ❌ HTTPS rendezvous serving (biomeos-api)
- ❌ Beacon persistence (NestGate)

#### New Songbird Work

**1. Port Pattern Probing** (~50 lines in `songbird-stun`)

New RPC method: `stun.probe_port_pattern`

```rust
/// Probe STUN server N times to detect NAT port allocation pattern
pub async fn probe_port_pattern(stun_server: &str, probes: usize) -> PortPattern {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let mut ports = Vec::new();
    
    for _ in 0..probes {
        if let Ok(addr) = stun_bind(&socket, stun_server).await {
            ports.push(addr.port());
        }
    }
    
    let deltas: Vec<i32> = ports.windows(2)
        .map(|w| w[1] as i32 - w[0] as i32)
        .collect();
    
    if deltas.is_empty() {
        return PortPattern::Unknown;
    }
    
    let all_same = deltas.iter().all(|d| *d == deltas[0]);
    if all_same && deltas[0].abs() <= 10 {
        PortPattern::Sequential {
            step: deltas[0],
            last_port: *ports.last().unwrap(),
            predicted_next: (*ports.last().unwrap() as i32 + deltas[0]) as u16,
            confidence: 0.85,
        }
    } else {
        PortPattern::Random { observed: ports }
    }
}
```

**2. Relay Server Listen Loop** (~80 lines in `songbird-lineage-relay`)

The existing `RelayProtocol::parse()` and `RelaySession` need a server-side counterpart:

```rust
/// Server-side relay: accept allocations, dispatch DataPackets between sessions
pub async fn serve_relay(bind_addr: SocketAddr, authority: Arc<dyn RelayAuthority>) {
    let socket = UdpSocket::bind(bind_addr).await?;
    let sessions: HashMap<Uuid, (SocketAddr, SocketAddr)> = HashMap::new(); // session → (peer_a, peer_b)
    
    loop {
        let (len, from_addr) = socket.recv_from(&mut buf).await?;
        match RelayProtocol::parse(&buf[..len])? {
            RelayProtocol::AllocateRequest(req) => {
                // Verify lineage via BearDog
                let auth = authority.authorize_relay(&req.relay_node, &req.requester).await?;
                if auth.authorized {
                    let session_id = Uuid::new_v4();
                    sessions.insert(session_id, (from_addr, req.target_addr));
                    // Send AllocationResponse
                }
            }
            RelayProtocol::DataPacket { session_id, data } => {
                // Look up session, forward to the OTHER peer
                if let Some((peer_a, peer_b)) = sessions.get(&session_id) {
                    let target = if from_addr == *peer_a { peer_b } else { peer_a };
                    socket.send_to(&data, target).await?;
                }
            }
            // ... Refresh, Deallocate
        }
    }
}
```

**3. Relay-Assisted Punch Coordination** (~40 lines in coordinator)

New RPC method: `punch.coordinate` — extends existing `punch_to_peer()`:

```rust
/// Relay-assisted coordinated punch
/// Uses active relay session as signaling channel + port predictions
pub async fn coordinate_relay_punch(
    &self,
    peer_node_id: &str,
    relay_session: &RelaySession,
    our_port_pattern: &PortPattern,
    peer_port_pattern: &PortPattern,
) -> Result<PunchResult> {
    // 1. Predict next ports for both sides
    let our_predicted = our_port_pattern.predict_next();
    let peer_predicted = peer_port_pattern.predict_next();
    
    // 2. Coordinate timing via relay channel
    let start_at = SystemTime::now() + Duration::from_millis(200);
    relay_session.send(&SignalingMessage::PunchCoordinate {
        our_predicted_port: our_predicted,
        peer_predicted_port: peer_predicted,
        start_at_ms: start_at.as_millis(),
    }.encode()).await?;
    
    // 3. Wait for start time, then punch
    sleep_until(start_at).await;
    
    // 4. Spray predicted ports (± window for prediction error)
    for offset in -3..=3 {
        let target_port = (peer_predicted as i32 + offset) as u16;
        socket.send_to(punch_msg, (peer_addr.ip(), target_port)).await?;
    }
    
    // 5. Listen for response
    // ... existing punch receive logic
}
```

**4. Wire Coordinator to Orchestrator RPC**

The `punch.request` method currently returns `"hole_punch_coordinator_not_initialized"`.
Wire it to instantiate `HolePunchCoordinator` on first use and delegate:

```
punch.request → HolePunchCoordinator::punch_to_peer()
punch.coordinate → HolePunchCoordinator::coordinate_relay_punch()  (NEW)
punch.status → query coordinator state
stun.probe_port_pattern → probe_port_pattern()  (NEW)
stun.detect_nat_type → detect_nat_type()  (already exists in coordinator)
```

---

### 🐻 BearDog — The Cryptographic Authority

**Role**: Identity verification, encryption, relay authorization.  
**Principle**: BearDog says WHO you are. It never touches a socket.

| What BearDog Owns | Where | Current State |
|--------------------|-------|---------------|
| Lineage verification | `birdsong.verify_lineage` | ✅ Challenge-response |
| Beacon encryption | `birdsong.generate_encrypted_beacon` | ✅ ChaCha20-Poly1305 |
| Beacon decryption | `birdsong.decrypt` / `birdsong.decrypt_beacon` | ✅ Family gate |
| HMAC key derivation | BearDog crypto module | ✅ Beacon keys |
| Blake3 hashing | `crypto.blake3_hash` | ✅ |
| x25519 key exchange | BearDog crypto module | ✅ |
| **Relay authorization** | `RelayAuthority` trait | ⚠️ Trait defined in Songbird, needs BearDog impl |

#### New BearDog Work

**1. Implement `RelayAuthority` trait** (~30 lines)

Songbird's `songbird-lineage-relay` defines `RelayAuthority`. BearDog needs to implement it
so that relay sessions are lineage-gated:

```rust
/// BearDog implements relay authorization via lineage verification
impl RelayAuthority for BearDogRelayAuth {
    async fn authorize_relay(&self, relay_node: &NodeId, requester: &NodeId) -> Result<RelayAuthorization> {
        // Verify requester shares our lineage (same .family.seed)
        let is_family = self.verify_lineage(requester).await?;
        Ok(RelayAuthorization {
            authorized: is_family,
            masking_level: if is_family { MaskingLevel::Transparent } else { MaskingLevel::Blocked },
            ttl_seconds: 300,
            // ...
        })
    }
}
```

**2. Expose `relay.authorize` RPC** (~10 lines)

BearDog should expose this via JSON-RPC so Songbird's relay server can call it:

```
relay.authorize(requester_node_id, requester_lineage_proof) → { authorized: bool, masking: "transparent" }
```

This gets registered in the Neural API as: `capability.call("relay", "authorize")` → BearDog

---

### 🏠 NestGate — Persistent State

**Role**: Store NAT info, known peers, relay preferences. Survives restarts.  
**Principle**: NestGate remembers. Other primals are ephemeral.

| What NestGate Owns | Where | Current State |
|---------------------|-------|---------------|
| `.known_beacons.json` | NestGate data dir | ⚠️ Schema defined, persistence partial |
| Meeting records | Beacon genetics | ⚠️ In-memory via `biomeos-spore` |
| NAT type cache | `.known_beacons.json` → `nat_traversal` | ❌ Not persisted |
| Relay preferences | `.known_beacons.json` → `family_relay` | ❌ Not persisted |

#### New NestGate Work

**1. Persist NAT info in `.known_beacons.json`** (~20 lines)

After STUN probes, store results so future connections skip the probe phase:

```json
{
  "nat_traversal": {
    "our_nat_type": "symmetric",
    "port_pattern": { "type": "sequential", "step": 1, "confidence": 0.85 },
    "last_probed": "2026-02-11T20:00:00Z",
    "family_relay": {
      "tower": { "relay_addr": "192.168.1.144:3479", "stun_addr": "192.168.1.144:3478" }
    }
  }
}
```

---

## nat0 Remnant in Songbird

During this analysis, a `nat0` default was found in Songbird source:

```
songbird-orchestrator/src/ipc/unix/handlers/standard_methods.rs:37
    let family_id = std::env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "nat0".to_string());
```

**This must be fixed** — Songbird should use the same family discovery as biomeOS:
1. Check `FAMILY_ID` env var (biomeOS standard)
2. Fall back to `SONGBIRD_FAMILY_ID` (Songbird-specific override)
3. Never default to `"nat0"` — use `"default"` or derive from seed

Additional `nat0` references exist across ~70 files in the Songbird tree (tests, docs,
sessions, source). A cleanup sweep should replace these with seed-derived IDs or test
fixtures using `"test_cf7e8729"` patterns.

---

## Capability Translation Updates

Register new methods in `capability_translation.rs`:

```rust
// In the "network" domain (Songbird provider):
("stun.probe_port_pattern", "stun.probe_port_pattern"),   // NEW
("punch.coordinate", "punch.coordinate"),                   // NEW

// In the "security" domain (BearDog provider):
("relay.authorize", "relay.authorize"),                     // NEW
```

Register new Neural API routing sugar in `routing.rs`:

```rust
| "stun.probe_port_pattern"
| "punch.coordinate"
| "relay.authorize"
```

---

## Build Order

```
  Step 1: Songbird — Port pattern probing (stun.probe_port_pattern)
     │    No dependencies. Pure STUN client addition.
     │
  Step 2: Songbird — Relay server listen loop (relay.serve wiring)
     │    Depends on: existing RelayProtocol, existing RelayAuthority trait
     │    Parallel with step 1
     │
  Step 3: BearDog — Implement RelayAuthority + relay.authorize RPC
     │    Depends on: Step 2 (trait definition already exists)
     │    Parallel with steps 1-2
     │
  Step 4: Songbird — Wire coordinator to orchestrator (punch.request + punch.coordinate)
     │    Depends on: Steps 1, 2 (coordinator uses STUN probes + relay as signaling)
     │
  Step 5: biomeOS — Connection strategy orchestrator
     │    Depends on: Steps 1-4 (calls capability.call for each step)
     │
  Step 6: NestGate — Persist NAT info
     │    Can happen anytime, improves cold-start performance
     │
  Step 7: Songbird — nat0 cleanup sweep
          Can happen anytime, independent of protocol work
```

Steps 1, 2, 3 are fully parallelizable.
Steps 1-3 combined: ~160 lines of new Rust code.
Step 4: ~40 lines of wiring.
Step 5: ~100 lines of orchestration logic.

**Total new code: ~300 lines across 4 codebases.**

---

## Pixel Blocker: aarch64 Full Build

**CRITICAL**: None of the above matters until the Pixel has a full Songbird build.
The current Pixel Songbird (13.7MB, Feb 9) is a minimal build missing the orchestrator
module that contains ALL mesh/STUN/relay/discovery/birdsong subsystems.

**Before any relay-punch work**: Rebuild aarch64 Songbird with full orchestrator features
and redeploy to Pixel. This is prerequisite for testing ANY of the above.

---

## Success Criteria

| Test | Expected Result |
|------|-----------------|
| `stun.probe_port_pattern` on Tower (5 probes) | Detects sequential or skip-N pattern |
| `stun.probe_port_pattern` on Pixel (5 probes) | Detects pattern + predicts next port |
| `relay.serve` on Tower + `relay.allocate` from Pixel | Active relay session, data flowing |
| `punch.coordinate` with relay channel | Both peers punch at coordinated time |
| Direct connectivity after successful punch | Data path swaps from relay to direct |
| Failed punch → relay still active | Zero disruption, data continues flowing |
| `capability.call("relay","authorize")` | BearDog verifies lineage before relay |

---

*The family IS the infrastructure. The relay IS the coordination channel.*


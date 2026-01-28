# Songbird LAN Discovery Evolution Handoff

## Issue Summary

**Problem**: UDP multicast (224.0.0.251:2300) doesn't reliably cross network boundaries between wifi and ethernet interfaces on consumer routers due to IGMP snooping and multicast filtering.

**Impact**: LiveSpore USB deployments on different towers can't discover each other even when on the same LAN subnet.

**Requirement**: Discovery must work **without firewall exceptions, sudo, or special permissions** - true portable pure Rust.

## Current Implementation

```
┌─────────────────────────────────────────────────────────────────┐
│                   Current: UDP Multicast Only                   │
├─────────────────────────────────────────────────────────────────┤
│  Broadcaster → 224.0.0.251:2300 (multicast)                    │
│  Listener   ← joins multicast group                            │
│                                                                 │
│  ✅ Works: Same interface (eth ↔ eth, wifi ↔ wifi)             │
│  ❌ Fails: Cross interface (eth ↔ wifi) on many routers        │
│  ❌ Fails: Some corporate networks block multicast              │
└─────────────────────────────────────────────────────────────────┘
```

## Recommended Evolution: Multi-Transport Discovery

### Priority 1: Subnet Broadcast Fallback
```rust
// When multicast fails, fall back to subnet broadcast
// This works on same subnet regardless of interface
broadcast_addresses: vec![
    "224.0.0.251:2300".parse().unwrap(),  // Primary: multicast
    "192.168.1.255:2300".parse().unwrap(), // Fallback: subnet broadcast
    "255.255.255.255:2300".parse().unwrap(), // Last resort: global broadcast
]
```

### Priority 2: mDNS Integration (Port 5353)
mDNS is **pre-allowed** on most systems (used by Bonjour, Avahi):
```rust
// Use existing mDNS infrastructure
// Port 5353 typically whitelisted
let mdns = MdnsService::new("_songbird._tcp.local")?;
mdns.register_with_txt(lineage_txt_records)?;
```

### Priority 3: TCP Rendezvous (Pure Outbound)
For NAT traversal without relay servers:
```rust
// Both peers make outbound TCP connections
// to each other's last-known addresses
// Outbound TCP doesn't need firewall rules
async fn tcp_hole_punch(peer_addr: SocketAddr) -> Result<TcpStream> {
    // Simultaneous open technique
}
```

### Priority 4: HTTP/WebSocket Beacon Tunnel
Use HTTP as transport layer (always allowed):
```rust
// Beacons sent as HTTP POST to known discovery endpoint
// Works through corporate proxies
POST /discovery/beacon HTTP/1.1
Content-Type: application/octet-stream

[encrypted_beacon_bytes]
```

## Immediate Workaround

For current deployment, can use environment variable to add broadcast fallback:

```bash
export SONGBIRD_BROADCAST_ADDRESSES="224.0.0.251:2300,192.168.1.255:2300"
```

## Test Matrix

| Transport | Same Interface | Cross Interface | Through NAT | Corporate |
|-----------|---------------|-----------------|-------------|-----------|
| Multicast | ✅ | ❌ | ❌ | ❌ |
| Subnet Broadcast | ✅ | ✅ | ❌ | ⚠️ |
| mDNS (5353) | ✅ | ✅ | ❌ | ✅ |
| TCP Rendezvous | ✅ | ✅ | ✅ | ✅ |
| HTTP Tunnel | ✅ | ✅ | ✅ | ✅ |

## Implementation Priority

1. **Add subnet broadcast to default addresses** (quick win)
2. **HTTP Bootstrap → UDP Escalation** (architecturally correct)
3. **Implement mDNS backend** (high compatibility)
4. **TCP hole punching** (NAT traversal)

## HTTP Bootstrap → UDP Escalation Pattern

**Key Insight**: HTTP/S is for external APIs, but can bootstrap LAN discovery:

```
┌─────────────────────────────────────────────────────────────────┐
│           HTTP Bootstrap → UDP Escalation                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Phase 1: HTTP Bootstrap (always works)                         │
│  ─────────────────────────────────────────                      │
│  Tower A → HTTP broadcast/mDNS: "I'm here, UDP endpoint: X"    │
│  Tower B → HTTP broadcast/mDNS: "I'm here, UDP endpoint: Y"    │
│                                                                 │
│  Phase 2: Direct UDP (efficient)                                │
│  ────────────────────────────────                               │
│  Tower A ←──── UDP JSON-RPC ────→ Tower B                       │
│           (Dark Forest encrypted beacons)                       │
│                                                                 │
│  BearDog TLS 1.3 secures the channel                           │
│  HTTP is just for bootstrap, not ongoing traffic               │
└─────────────────────────────────────────────────────────────────┘
```

**Implementation**:
```rust
// Phase 1: HTTP Bootstrap
async fn http_bootstrap_announce(&self) -> Result<()> {
    // Announce via HTTP (mDNS-SD or subnet broadcast on port 80)
    let announcement = json!({
        "node_id": self.node_id,
        "family_hash": self.family_hash,  // Not actual ID
        "udp_endpoint": format!("{}:{}", self.local_ip, self.udp_port),
        "capabilities_hash": self.capabilities_hash,
    });
    
    // Use mDNS TXT records or HTTP multicast
    self.mdns.announce(announcement).await?;
    Ok(())
}

// Phase 2: Escalate to UDP after bootstrap
async fn escalate_to_udp(&self, peer: &PeerInfo) -> Result<()> {
    // Now we know peer's UDP endpoint from HTTP bootstrap
    // Switch to efficient UDP Dark Forest beacons
    self.udp_broadcaster.add_known_peer(peer.udp_endpoint).await?;
    Ok(())
}
```

**Why This Works**:
- HTTP/mDNS (port 80/5353) typically allowed through firewalls
- Bootstrap reveals minimal info (just endpoint, not identity)
- UDP used for efficient ongoing Dark Forest beacons
- BearDog TLS 1.3 secures actual communication
- No firewall rules needed for UDP since we know the endpoint

## Files to Modify

- `songbird-discovery/src/anonymous/broadcaster.rs` - Add fallback addresses
- `songbird-discovery/src/anonymous/listener.rs` - Listen on multiple transports
- `songbird-orchestrator/src/app/discovery_startup.rs` - Transport selection logic
- `songbird-types/src/config/discovery.rs` - Configuration for transports

## Verification

After evolution, this should work:
```bash
# Tower A (ethernet)
./deploy.sh  # Starts Songbird

# Tower B (wifi, same subnet)
./deploy.sh  # Starts Songbird

# Both should discover each other within 30 seconds
# Without any firewall changes
```

## ROOT CAUSE: Port 0 in Discovery Beacons

**Finding (Jan 28, 2026)**: Songbird beacons contain `"port": 0`, causing peer rejection.

**Source**: `songbird-orchestrator/src/app/core.rs:533-540`
```rust
async fn start_http_server(&self) -> Result<u16> {
    // Unix sockets ONLY - no TCP binding
    info!("🔒 Songbird uses Unix sockets ONLY (Concentrated Gap strategy)");
    Ok(0) // No port used  ← PROBLEM
}
```

**Design Conflict**:
| Component | Expectation |
|-----------|-------------|
| Songbird internals | Unix sockets only (no TCP exposure) |
| Discovery protocol | TCP port for peer connection |
| Result | `port: 0` → "Invalid port" rejection |

**Clarified Architecture: Dual-Mode Songbird**

```
┌─────────────────────────────────────────────────────────────────┐
│                 SONGBIRD DUAL-MODE OPERATION                    │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  EXTERNAL GATEWAY (TCP Port 8080)     INTERNAL IPC (Unix:0)    │
│  ────────────────────────────────     ─────────────────────    │
│  • LAN beacon broadcasts              • Inter-primal JSON-RPC  │
│  • Initial peer handshake             • BearDog ↔ Songbird     │
│  • Federation discovery               • Squirrel ↔ Neural API  │
│  • External API gateway               • Zero network exposure  │
│                                                                 │
│  ESCALATION: TCP discovery → Unix secure RPC                   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key Principle**: TCP port is **external only** for beacons. Once peers discover each other:
1. Exchange Unix socket paths over the TCP channel
2. Escalate to Unix socket JSON-RPC for all real communication
3. TCP channel can be closed or kept for heartbeats only

**Evolution Required**:

1. **Add external beacon port** (separate from internal):
```rust
struct SongbirdConfig {
    // Internal: Unix socket (always)
    internal_socket: PathBuf,  // /tmp/songbird-nat0.sock
    
    // External: TCP for LAN discovery (optional, off by default)
    external_beacon_port: Option<u16>,  // 8080 when federation enabled
}
```

2. **Beacon advertises external port, not internal**:
```rust
fn create_beacon(&self) -> DiscoveryMessage {
    DiscoveryMessage {
        // External gateway for initial contact
        port: self.config.external_beacon_port.unwrap_or(0),
        
        // Include Unix socket hint for escalation
        socket_hint: Some(self.config.internal_socket.to_string()),
        
        // ... rest of beacon
    }
}
```

3. **Peer connection escalation**:
```rust
async fn connect_to_peer(&self, peer: &DiscoveredPeer) -> Result<PeerConnection> {
    // Phase 1: TCP handshake (external)
    let tcp_stream = TcpStream::connect(peer.external_addr).await?;
    
    // Phase 2: Verify lineage via BearDog
    let lineage_ok = self.beardog.verify_peer_lineage(&peer).await?;
    if !lineage_ok {
        return Err("Lineage verification failed");
    }
    
    // Phase 3: Exchange Unix socket paths (secure)
    let peer_socket = exchange_socket_paths(&tcp_stream, &self.internal_socket).await?;
    
    // Phase 4: Escalate to Unix socket (if same machine) or keep TCP (if remote)
    if is_local_peer(&peer) {
        // Same machine: use Unix socket (fastest, most secure)
        Ok(PeerConnection::Unix(UnixStream::connect(peer_socket).await?))
    } else {
        // Remote machine: keep TCP but upgrade to TLS
        Ok(PeerConnection::Tls(upgrade_to_tls(tcp_stream).await?))
    }
}
```

**Benefits**:
- Internal primal communication stays on Unix sockets (secure, fast)
- External discovery works over TCP (LAN reachable)
- Lineage verification before any real communication
- Automatic escalation to best transport

---

## Neural API Evolution: Primal Lifecycle Management

**Current Gap**: If primals crash, nothing respawns them.

**Apoptosis Pattern** (programmed death):
- Primals can request graceful shutdown
- Resources released cleanly
- State persisted for resurrection

**Resurrection Pattern** (maintenance respawn):
- Neural API monitors primal health
- Detects crashes via socket timeout
- Respawns from deployment graph
- Restores state from last checkpoint

**Implementation**:
```toml
# graphs/primal_lifecycle.toml
[lifecycle]
health_check_interval_ms = 5000
max_restart_attempts = 3
restart_backoff_ms = [1000, 5000, 30000]

[[lifecycle.monitors]]
primal = "beardog"
socket = "/tmp/beardog-nat0.sock"
health_method = "health.check"
on_failure = "restart"

[[lifecycle.monitors]]
primal = "songbird"
socket = "/tmp/songbird-nat0.sock"
health_method = "health"
on_failure = "restart"
depends_on = ["beardog"]  # Restart order matters
```

---
*Generated: 2026-01-28*
*Family: nat0*
*Issues: port:0 beacon rejection, lifecycle management*

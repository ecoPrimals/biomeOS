# Songbird STUN/Rendezvous JSON-RPC Handoff

**Date**: January 29, 2026 (Updated)  
**From**: biomeOS Team  
**To**: Songbird Team  
**Priority**: High  
**Status**: **ROOT CAUSE IDENTIFIED** - Simple wiring fix needed

---

## Executive Summary

Songbird v8.18.0 has **fully implemented** STUN/Discovery handlers in `IpcServiceHandler`, but `bin_interface.rs` uses `HttpHandler` which only exposes HTTP methods. **The handlers exist - they just need to be wired in.**

## Root Cause

```rust
// bin_interface.rs line ~216
// CURRENT (only HTTP methods):
let handler_clone = HttpHandler::with_default_discovery();

// NEEDED (all methods including STUN/Discovery):
let handler_clone = IpcServiceHandler::new(...);
```

### Implemented Methods (in IpcServiceHandler)

```rust
// crates/songbird-universal-ipc/src/service.rs lines 466-482
"stun.get_public_address" => self.handle_stun_get_public_address(params).await,
"stun.bind" => self.handle_stun_bind(params).await,
"discovery.peers" => self.handle_discovery_peers(params).await,
"rendezvous.register" => self.handle_rendezvous_register(params).await,
"rendezvous.lookup" => self.handle_rendezvous_lookup(params).await,
"peer.connect" => self.handle_peer_connect(params).await,
```

### The Fix (One-Line Change)

In `crates/songbird-orchestrator/src/bin_interface.rs`, change the handler construction to use `IpcServiceHandler` instead of `HttpHandler`, or add the STUN/Discovery dispatch to `HttpHandler`.

---

## Previous Analysis (Still Valid)

## Current State

### ✅ What's Working

| Feature | Status | Evidence |
|---------|--------|----------|
| UDP Discovery Beacons | ✅ | Both spores broadcasting on port 2300 |
| TCP Gateway Ports | ✅ | Listening on 8081, 8082 |
| LAN Discovery | ✅ | Other tower (192.168.1.134) actively connecting |
| HTTPS to External | ✅ | api.github.com working (350-400ms) |
| `songbird-stun` Crate | ✅ | Exists with STUN binding logic |

### ❌ What's Missing

```bash
# All these return "Unknown method"
echo '{"jsonrpc":"2.0","method":"stun.get_public_address","params":{},"id":1}' | nc -U songbird.sock
echo '{"jsonrpc":"2.0","method":"stun.bind","params":{},"id":2}' | nc -U songbird.sock
echo '{"jsonrpc":"2.0","method":"discovery.peers","params":{},"id":3}' | nc -U songbird.sock
echo '{"jsonrpc":"2.0","method":"rendezvous.register","params":{},"id":4}' | nc -U songbird.sock
```

---

## Required JSON-RPC Methods

### 1. STUN Public Address Discovery

```rust
// Method: stun.get_public_address
// Purpose: Get reflexive address from STUN server for NAT traversal

#[derive(Deserialize)]
struct StunGetPublicAddressParams {
    /// STUN server (e.g., "stun.l.google.com:19302")
    server: Option<String>,
    /// Local port to bind (default: ephemeral)
    local_port: Option<u16>,
}

#[derive(Serialize)]
struct StunGetPublicAddressResult {
    /// Public IP:port as seen by STUN server
    public_address: String,
    /// Local bound address
    local_address: String,
    /// STUN server used
    server: String,
    /// NAT type detected (if determinable)
    nat_type: Option<String>,
}
```

**Example Usage:**
```json
// Request
{"jsonrpc":"2.0","method":"stun.get_public_address","params":{"server":"stun.l.google.com:19302"},"id":1}

// Response
{"jsonrpc":"2.0","result":{"public_address":"203.0.113.45:54321","local_address":"192.168.1.144:54321","server":"stun.l.google.com:19302","nat_type":"full_cone"},"id":1}
```

### 2. STUN Binding Request

```rust
// Method: stun.bind
// Purpose: Create and maintain a STUN binding for hole punching

#[derive(Deserialize)]
struct StunBindParams {
    /// STUN server
    server: String,
    /// Local port to bind
    local_port: u16,
    /// Keep-alive interval (seconds)
    keepalive_secs: Option<u64>,
}

#[derive(Serialize)]
struct StunBindResult {
    /// Binding ID for reference
    binding_id: String,
    /// Mapped address
    mapped_address: String,
    /// Binding lifetime (seconds)
    lifetime_secs: u64,
}
```

### 3. Discovery Peers List

```rust
// Method: discovery.peers
// Purpose: List discovered peers from UDP beacons

#[derive(Serialize)]
struct DiscoveryPeersResult {
    peers: Vec<DiscoveredPeer>,
    total_count: usize,
}

#[derive(Serialize)]
struct DiscoveredPeer {
    /// Peer's node ID
    node_id: String,
    /// Peer's family ID
    family_id: String,
    /// IP:port from beacon
    address: String,
    /// TCP gateway port (if advertised)
    tcp_port: Option<u16>,
    /// Capabilities advertised
    capabilities: Vec<String>,
    /// Last seen timestamp
    last_seen: String,
    /// Signal strength / latency
    quality: Option<f64>,
}
```

**Example Usage:**
```json
// Request
{"jsonrpc":"2.0","method":"discovery.peers","params":{},"id":1}

// Response
{"jsonrpc":"2.0","result":{"peers":[{"node_id":"node-gamma","family_id":"nat0","address":"192.168.1.144:2300","tcp_port":8082,"capabilities":["crypto","tls"],"last_seen":"2026-01-29T02:26:00Z","quality":0.95}],"total_count":1},"id":1}
```

### 4. Rendezvous Registration

```rust
// Method: rendezvous.register
// Purpose: Register with a rendezvous server for NAT traversal

#[derive(Deserialize)]
struct RendezvousRegisterParams {
    /// Rendezvous server URL
    server: String,
    /// Our node ID
    node_id: String,
    /// Our family ID (for family-scoped discovery)
    family_id: String,
    /// Public address (from STUN)
    public_address: String,
}

#[derive(Serialize)]
struct RendezvousRegisterResult {
    /// Registration ID
    registration_id: String,
    /// Expiry time
    expires_at: String,
    /// Rendezvous token for peers
    rendezvous_token: String,
}
```

### 5. Rendezvous Lookup

```rust
// Method: rendezvous.lookup
// Purpose: Find a peer via rendezvous server

#[derive(Deserialize)]
struct RendezvousLookupParams {
    /// Rendezvous server URL
    server: String,
    /// Target node ID or family ID
    target: String,
}

#[derive(Serialize)]
struct RendezvousLookupResult {
    /// Found peers
    peers: Vec<RendezvousPeer>,
}

#[derive(Serialize)]
struct RendezvousPeer {
    node_id: String,
    family_id: String,
    public_address: String,
    rendezvous_token: String,
}
```

### 6. Peer Connect (Hole Punch)

```rust
// Method: peer.connect
// Purpose: Initiate connection to peer using hole punching

#[derive(Deserialize)]
struct PeerConnectParams {
    /// Target peer address
    target_address: String,
    /// Our STUN binding (for symmetric NAT)
    our_binding: Option<String>,
    /// Rendezvous token (if using rendezvous)
    rendezvous_token: Option<String>,
}

#[derive(Serialize)]
struct PeerConnectResult {
    /// Connection ID
    connection_id: String,
    /// Connection state
    state: String, // "connecting", "connected", "failed"
    /// Established channel info
    channel: Option<PeerChannel>,
}

#[derive(Serialize)]
struct PeerChannel {
    /// Local endpoint
    local_address: String,
    /// Remote endpoint  
    remote_address: String,
    /// Protocol (udp/tcp)
    protocol: String,
    /// Latency (ms)
    latency_ms: Option<u64>,
}
```

---

## Integration with biomeOS Dark Forest Protocol

### Current Flow (Working)
```
1. UDP Beacon broadcast (port 2300) ✅
2. Peer receives beacon ✅
3. Family verification via BearDog ✅
4. Birdsong encrypted channel ✅
```

### Desired Flow (Needs STUN Methods)
```
1. UDP Beacon broadcast (port 2300) ✅
2. Peer receives beacon ✅
3. STUN: Get public address for NAT traversal ❌
4. Rendezvous: Register with relay server ❌
5. Rendezvous: Lookup peer ❌
6. Peer Connect: Hole punch to peer ❌
7. Family verification via BearDog ✅
8. Birdsong encrypted channel ✅
9. Protocol escalation to tarpc (future)
```

---

## Implementation Hints

### Existing Code Location

The STUN logic likely exists in:
```
crates/songbird-stun/src/
├── lib.rs           # STUN binding logic
├── client.rs        # STUN client
└── message.rs       # STUN message parsing
```

### Handler Registration

Add to `songbird-universal-ipc/src/handlers/`:
```rust
// stun_handler.rs
pub struct StunHandler {
    stun_client: Arc<StunClient>,
}

impl StunHandler {
    pub async fn get_public_address(&self, params: StunGetPublicAddressParams) -> Result<Value> {
        // Use existing songbird-stun crate
    }
    
    pub async fn bind(&self, params: StunBindParams) -> Result<Value> {
        // Create/maintain STUN binding
    }
}
```

Register in method router:
```rust
"stun.get_public_address" => stun_handler.get_public_address(params).await,
"stun.bind" => stun_handler.bind(params).await,
"discovery.peers" => discovery_handler.list_peers(params).await,
"rendezvous.register" => rendezvous_handler.register(params).await,
"rendezvous.lookup" => rendezvous_handler.lookup(params).await,
"peer.connect" => peer_handler.connect(params).await,
```

---

## Testing

### Unit Tests
```rust
#[tokio::test]
async fn test_stun_get_public_address() {
    let handler = StunHandler::new();
    let params = json!({"server": "stun.l.google.com:19302"});
    let result = handler.get_public_address(params).await.unwrap();
    assert!(result.get("public_address").is_some());
}

#[tokio::test]
async fn test_discovery_peers_empty() {
    let handler = DiscoveryHandler::new();
    let result = handler.list_peers(json!({})).await.unwrap();
    assert!(result.get("peers").unwrap().as_array().is_some());
}
```

### Integration Test (biomeOS)
```bash
# Start two spores
./start_spore.sh node-alpha 8081
./start_spore.sh node-gamma 8082

# Get STUN addresses
ALPHA_ADDR=$(echo '{"jsonrpc":"2.0","method":"stun.get_public_address","params":{},"id":1}' | nc -U /run/user/1000/biomeos/songbird-node-alpha.sock | jq -r '.result.public_address')
GAMMA_ADDR=$(echo '{"jsonrpc":"2.0","method":"stun.get_public_address","params":{},"id":1}' | nc -U /run/user/1000/biomeos/songbird-node-gamma.sock | jq -r '.result.public_address')

# List discovered peers
echo '{"jsonrpc":"2.0","method":"discovery.peers","params":{},"id":1}' | nc -U /run/user/1000/biomeos/songbird-node-alpha.sock

# Connect peer
echo '{"jsonrpc":"2.0","method":"peer.connect","params":{"target_address":"'$GAMMA_ADDR'"}}' | nc -U /run/user/1000/biomeos/songbird-node-alpha.sock
```

---

## Priority Order

1. **`stun.get_public_address`** - Highest priority, enables NAT detection
2. **`discovery.peers`** - See who's on the network
3. **`peer.connect`** - Direct peer connection
4. **`rendezvous.register`/`rendezvous.lookup`** - For relay fallback

---

## Additional Context

### Why This Matters

The Dark Forest protocol uses:
1. **Noisy broadcasts** - UDP beacons with encrypted tags ✅
2. **Encrypted lineage handshakes** - Birdsong encrypt/decrypt ✅
3. **NAT traversal** - STUN for hole punching ❌ **BLOCKED**
4. **Rendezvous relay** - For symmetric NAT fallback ❌ **BLOCKED**

Without STUN methods, spores behind NAT cannot establish direct connections, limiting biomeOS to LAN-only deployment.

### Current Evidence

From biomeOS validation (Jan 29, 2026):
```
=== Test 1: Query Public Address via Google STUN ===
{"jsonrpc":"2.0","error":{"code":-32603,"message":"Unknown method: stun.get_public_address"},"id":10}

=== Test 2: Check available discovery/STUN methods ===
  stun.bind: ✗ Unknown method: stun.bind
  discovery.peers: ✗ Unknown method: discovery.peers
  rendezvous.register: ✗ Unknown method: rendezvous.register
```

### Related Handoffs
- `SONGBIRD_EVOLUTION_HANDOFF.md` - HTTP headers fix (COMPLETE)
- `SONGBIRD_LAN_DISCOVERY_HANDOFF.md` - Port:0 beacon fix (COMPLETE)

---

## Contact

- **biomeOS Lead**: Available for integration testing
- **Slack**: #songbird-evolution
- **Timeline**: Needed for v8.15.0 release

---

**Generated**: 2026-01-29  
**Version**: Songbird v8.14.0 → v8.15.0 target  
**Status**: 🔴 BLOCKING cross-spore rendezvous


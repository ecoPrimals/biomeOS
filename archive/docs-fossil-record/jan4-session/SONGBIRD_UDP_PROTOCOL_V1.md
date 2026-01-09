# 🌊 Songbird UDP Protocol Specification v1.0

**Status**: Design Phase  
**Target**: Phase 1 Implementation  
**Goal**: Pure UDP multicast discovery with zero configuration

---

## 🎯 Protocol Overview

### Purpose

**Songbird UDP Protocol** enables:
- Zero-configuration peer discovery
- Automatic capability advertisement
- Family-based peer identification
- Secure foundation for BearDog encryption

### Key Characteristics

- **Transport**: UDP Multicast (IPv4/IPv6)
- **Format**: JSON (human-readable, debug-friendly)
- **Addressing**: 224.0.0.251:5353 (mDNS-compatible)
- **Interval**: 30 seconds (configurable)
- **TTL**: 90 seconds (3x interval)

---

## 📡 Packet Format

### Announcement Packet

**Type**: `peer_announcement`

```json
{
  "version": "1.0",
  "type": "peer_announcement",
  "timestamp": 1704326400,
  "ttl": 90,
  "peer": {
    "family_id": "nat0",
    "node_id": "beardog:family:nat0:hostname_d5635638",
    "hostname": "pop-os",
    "capabilities": [
      "Security",
      "Encryption",
      "Trust"
    ],
    "endpoints": {
      "unix_socket": "/tmp/beardog-nat0.sock",
      "debug_http": "127.0.0.1:9000"
    }
  },
  "signature": "base64_encoded_signature"
}
```

### Response Packet

**Type**: `peer_response`

```json
{
  "version": "1.0",
  "type": "peer_response",
  "timestamp": 1704326401,
  "in_response_to": "beardog:family:nat0:hostname_d5635638",
  "peer": {
    "family_id": "nat0",
    "node_id": "beardog:family:nat0:hostname_e7746749",
    "hostname": "strandgate",
    "capabilities": [
      "Security",
      "Encryption",
      "Trust",
      "Discovery"
    ],
    "endpoints": {
      "unix_socket": "/tmp/beardog-nat0.sock",
      "songbird_socket": "/tmp/songbird-nat0.sock"
    }
  },
  "signature": "base64_encoded_signature"
}
```

### Goodbye Packet

**Type**: `peer_goodbye`

```json
{
  "version": "1.0",
  "type": "peer_goodbye",
  "timestamp": 1704326500,
  "peer": {
    "node_id": "beardog:family:nat0:hostname_d5635638"
  },
  "reason": "graceful_shutdown",
  "signature": "base64_encoded_signature"
}
```

---

## 🌐 Network Configuration

### Multicast Address

**IPv4**: `224.0.0.251` (mDNS range)  
**Port**: `5353` (mDNS port)  
**Reason**: Compatible with mDNS infrastructure, well-known

**IPv6**: `ff02::fb` (link-local mDNS)  
**Port**: `5353`

### Socket Configuration

```rust
use socket2::{Socket, Domain, Type, Protocol};

let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
socket.set_reuse_address(true)?;
socket.set_reuse_port(true)?;  // Allow multiple instances
socket.set_multicast_loop_v4(true)?;  // Hear own broadcasts (useful for testing)
socket.set_multicast_ttl_v4(255)?;  // LAN-wide
socket.bind(&"0.0.0.0:5353".parse()?)?;

// Join multicast group
socket.join_multicast_v4(&"224.0.0.251".parse()?, &"0.0.0.0".parse()?)?;
```

---

## ⏱️ Timing Specification

### Announcement Interval

**Default**: 30 seconds  
**Min**: 10 seconds  
**Max**: 300 seconds (5 minutes)

**Jitter**: ±2 seconds (prevent synchronization)

### TTL (Time To Live)

**Default**: 90 seconds (3x interval)  
**Formula**: `ttl = interval * 3`

### Timeout Detection

**Peer timeout**: No announcement for 90 seconds  
**Action**: Remove peer from registry  
**Grace period**: 10 seconds extra for network delays

---

## 🔐 Security

### Signature

**Algorithm**: Ed25519  
**Input**: JSON string (without `signature` field)  
**Output**: Base64-encoded signature

**Purpose**:
- Prevent spoofing
- Verify peer authenticity
- Foundation for BearDog trust evaluation

**Implementation**:
```rust
use ed25519_dalek::{Keypair, Signature, Signer};

let keypair = Keypair::generate(&mut OsRng);
let message = serde_json::to_string(&packet_without_sig)?;
let signature = keypair.sign(message.as_bytes());
let sig_base64 = base64::encode(signature.to_bytes());
```

### Family Verification

**BearDog's Role**:
1. Songbird receives packet
2. Songbird verifies signature
3. Songbird notifies BearDog of peer
4. **BearDog evaluates family trust**
5. BearDog decides: accept/reject/watch

**Separation of Concerns**:
- Songbird: Discovery + Signature verification
- BearDog: Trust evaluation + Encryption

---

## 📊 Peer Registry

### Data Structure

```rust
pub struct PeerRegistry {
    peers: HashMap<NodeId, PeerInfo>,
    capabilities: HashMap<Capability, Vec<NodeId>>,
    last_seen: HashMap<NodeId, Instant>,
}

pub struct PeerInfo {
    node_id: String,
    family_id: String,
    hostname: String,
    capabilities: Vec<String>,
    endpoints: HashMap<String, String>,
    signature_key: PublicKey,
    first_seen: Instant,
    last_seen: Instant,
    trust_level: TrustLevel,  // Set by BearDog
}

pub enum TrustLevel {
    Unknown,       // Not yet evaluated
    High,          // Same family
    Medium,        // Different family, but verified
    Low,           // Suspicious
    Blocked,       // Explicitly rejected
}
```

### Operations

**Add Peer**:
```rust
fn add_peer(&mut self, packet: PeerAnnouncement) -> Result<()> {
    // Verify signature
    self.verify_signature(&packet)?;
    
    // Add to registry
    let peer_info = PeerInfo::from_packet(packet);
    self.peers.insert(peer_info.node_id.clone(), peer_info.clone());
    
    // Index by capability
    for cap in &peer_info.capabilities {
        self.capabilities.entry(cap.clone())
            .or_insert_with(Vec::new)
            .push(peer_info.node_id.clone());
    }
    
    // Notify BearDog for trust evaluation
    self.notify_beardog(peer_info)?;
    
    Ok(())
}
```

**Update Last Seen**:
```rust
fn update_last_seen(&mut self, node_id: &NodeId) {
    self.last_seen.insert(node_id.clone(), Instant::now());
}
```

**Remove Expired**:
```rust
fn remove_expired(&mut self) {
    let now = Instant::now();
    let timeout = Duration::from_secs(100);  // 90s TTL + 10s grace
    
    let expired: Vec<NodeId> = self.last_seen.iter()
        .filter(|(_, &last_seen)| now.duration_since(last_seen) > timeout)
        .map(|(node_id, _)| node_id.clone())
        .collect();
    
    for node_id in expired {
        self.remove_peer(&node_id);
    }
}
```

---

## 🔌 Unix Socket IPC

### Purpose

**Inter-Primal Communication**:
- BearDog ↔ Songbird
- Other Primals ↔ Songbird
- **NOT for peer-to-peer** (use Songbird as intermediary)

### Socket Path

**Pattern**: `/tmp/songbird-{family_id}.sock`  
**Example**: `/tmp/songbird-nat0.sock`

**Permissions**: `0600` (owner only)

### Protocol

**Transport**: JSON-RPC over Unix domain socket  
**Format**: Line-delimited JSON

**Example**:
```json
{"jsonrpc":"2.0","method":"register","params":{"capabilities":["Security"]},"id":1}
{"jsonrpc":"2.0","result":{"status":"registered","node_id":"..."},"id":1}
```

### API Methods

**`register`**: Primal registers capabilities
```json
{
  "method": "register",
  "params": {
    "primal_type": "BearDog",
    "capabilities": ["Security", "Encryption", "Trust"],
    "endpoints": {
      "unix_socket": "/tmp/beardog-nat0.sock"
    }
  }
}
```

**`discover`**: Query for peers with capability
```json
{
  "method": "discover",
  "params": {
    "capability": "Storage",
    "family_filter": "nat0"  // Optional
  }
}
```

**`subscribe`**: Subscribe to events
```json
{
  "method": "subscribe",
  "params": {
    "event_types": ["peer_discovered", "peer_lost", "trust_updated"]
  }
}
```

**`send`**: Send message to peer via Songbird
```json
{
  "method": "send",
  "params": {
    "to": "node_id_or_capability",
    "message": {"type": "request", "data": {...}}
  }
}
```

---

## 🧪 Testing

### Local Testing (Same Machine)

**Scenario**: Multiple instances on localhost

**Configuration**:
```toml
[songbird]
multicast_addr = "224.0.0.251:5353"
unix_socket = "/tmp/songbird-{family_id}.sock"
announce_interval_secs = 10  # Faster for testing
```

**Expected**: All instances discover each other

### LAN Testing

**Scenario**: Multiple machines on same network

**Verification**:
```bash
# Machine 1
sudo tcpdump -i eth0 -n 'udp port 5353'
# Should see multicast packets

# Machine 2
songbird discover --family nat0
# Should see Machine 1
```

### Performance Testing

**Metrics**:
- Discovery time: < 1 second
- Packet loss tolerance: 90%
- CPU usage: < 1%
- Memory: < 10MB

---

## 📋 Implementation Checklist

### Phase 1: Core UDP

- [ ] UDP multicast socket setup
- [ ] Announcement packet broadcast
- [ ] Announcement packet reception
- [ ] Peer registry (add/remove/query)
- [ ] TTL expiration handling

### Phase 2: Security

- [ ] Ed25519 signature generation
- [ ] Signature verification
- [ ] Replay attack prevention (timestamp checking)
- [ ] Rate limiting (prevent DoS)

### Phase 3: Unix Socket IPC

- [ ] Unix socket server
- [ ] JSON-RPC protocol
- [ ] register/discover/subscribe/send methods
- [ ] Event notification system

### Phase 4: Integration

- [ ] BearDog trust evaluation hook
- [ ] Connection establishment via discovered peers
- [ ] Encrypted channel setup
- [ ] Health monitoring

---

## 🎯 Success Criteria

### Minimum Viable

- ✅ 2+ instances discover each other locally
- ✅ Signature verification working
- ✅ Peer registry maintains state
- ✅ TTL expiration removes old peers
- ✅ Unix socket IPC functional

### Production Ready

- ✅ Cross-machine discovery working
- ✅ Family-based filtering
- ✅ BearDog integration complete
- ✅ Encrypted connections established
- ✅ Fractal scaling proven (10+ instances)

---

**Status**: Specification complete. Ready for implementation in Songbird codebase.

**Next**: Implement in `phase1/songbird` or create new `songbird-udp` module in biomeOS.


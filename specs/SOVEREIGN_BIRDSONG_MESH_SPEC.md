# Sovereign BirdSong Mesh Specification

**Version**: 1.0.0  
**Status**: PROPOSED  
**Created**: February 6, 2026  
**Priority**: HIGH - Enables true sovereign connectivity

---

## Executive Summary

Enable family devices to discover and communicate across the internet without:
- Port forwarding
- External relay services
- Centralized infrastructure
- Trusting any third party

**Key Insight**: Tor hidden services (via Arti - Pure Rust) provide reachability without port forwarding. Combined with BirdSong's layered encryption, we get fully sovereign mesh networking.

---

## Architecture

### The BirdSong Mesh

```
┌────────────────────────────────────────────────────────────────┐
│                    BIRDSONG MESH TOPOLOGY                       │
│                                                                 │
│    Tower ◄────────► .onion relay ◄─────────► Pixel             │
│      │                  ▲                      │                │
│      │                  │                      │                │
│      ▼                  │                      ▼                │
│   Laptop ◄──────────────┴──────────────► Phone                 │
│                                                                 │
│  All connections are OUTBOUND via Tor (works through any NAT)  │
│  Any node with .onion becomes a relay for others               │
│  Beacons gossip through the mesh (store-and-forward)           │
│                                                                 │
└────────────────────────────────────────────────────────────────┘
```

### Layered Encryption Model

```rust
/// BirdSong message with layered encryption
pub struct LayeredBirdSong {
    /// Layer 0: Unencrypted routing header
    pub header: RoutingHeader,
    
    /// Layer 1: Family-encrypted (beacon seed)
    /// - Peer announcements
    /// - Relay offers
    /// - Capability broadcasts
    pub family_layer: EncryptedPayload,
    
    /// Layer 2: Lineage-encrypted (for ancestors/descendants)
    /// - Trust escalation
    /// - Private coordination
    pub lineage_layer: Option<EncryptedPayload>,
    
    /// Layer 3: Device-specific (recipient's public key)
    /// - E2E encrypted data
    /// - Commands
    /// - Sensitive information
    pub device_layer: Option<EncryptedPayload>,
}

pub struct RoutingHeader {
    /// Sender's .onion address (for replies)
    pub reply_onion: Option<String>,  // e.g., "abc123xyz.onion:3490"
    
    /// Message type (for routing decisions)
    pub msg_type: MessageType,
    
    /// TTL for gossip (prevents infinite propagation)
    pub ttl: u8,
    
    /// Nonce for deduplication
    pub nonce: [u8; 16],
}
```

### Onion Service Integration

```rust
/// Pure Rust Tor integration via Arti
pub struct OnionRelay {
    /// Arti client for Tor connectivity
    client: arti_client::TorClient,
    
    /// Our onion service
    onion_service: tor_hsservice::HsService,
    
    /// Our .onion address
    onion_address: String,
    
    /// BirdSong broadcaster for mesh gossip
    birdsong: Arc<BirdSongBroadcaster>,
}

impl OnionRelay {
    /// Start onion relay service
    pub async fn start() -> Result<Self> {
        // 1. Initialize Arti Tor client
        let client = arti_client::TorClient::bootstrap().await?;
        
        // 2. Create onion service (no port forwarding needed!)
        let onion_service = client.launch_onion_service(
            OnionServiceConfig {
                port: 3490,  // Relay port
                ..Default::default()
            }
        ).await?;
        
        // 3. Get our .onion address
        let onion_address = onion_service.onion_address().to_string();
        
        // 4. Announce via BirdSong
        // (encrypted so only family can read)
        
        Ok(Self { client, onion_service, onion_address, birdsong })
    }
    
    /// Connect to another family member's onion relay
    pub async fn connect_to_peer(&self, peer_onion: &str) -> Result<PeerConnection> {
        // Connect via Tor (outbound - works through any NAT!)
        let stream = self.client.connect((peer_onion, 3490)).await?;
        Ok(PeerConnection::new(stream))
    }
}
```

---

## Bootstrap Flow

### First Device (Genesis)

```
1. Generate family beacon seed (shared secret)
2. Start Arti Tor client
3. Create onion service → get .onion address
4. Store .onion in beacon for family discovery
5. Ready to accept family connections
```

### New Device Joining

```
Option A: Physical Handoff (Maximum Security)
1. Receive beacon seed via QR code / NFC / USB
2. Beacon contains .onion address of existing relay
3. Connect to .onion via Tor (outbound, works through NAT)
4. Authenticate via lineage verification
5. Now part of the mesh

Option B: Mesh Discovery (If Already Know One Member)
1. Already have beacon seed from previous setup
2. Connect to known family member
3. Receive updated mesh topology
4. Connect to additional .onion relays for redundancy
```

---

## Message Propagation (Gossip)

```
When a beacon arrives:
1. Decrypt Layer 1 (family seed)
2. Check nonce for deduplication
3. If new message AND ttl > 0:
   a. Process locally (if relevant)
   b. Decrement TTL
   c. Forward to all connected peers (except source)
4. If has Layer 2/3, attempt decryption if we're the target
```

---

## Dependencies

### Pure Rust (No C Dependencies)

```toml
[dependencies]
# Tor client and onion services
arti-client = "0.38"
tor-hsservice = "0.31"

# Already in Songbird
songbird-discovery = { path = "../songbird-discovery" }  # BirdSong
songbird-lineage-relay = { path = "../songbird-lineage-relay" }  # Relay
```

---

## Security Properties

| Property | Mechanism |
|----------|-----------|
| No IP exposure | Tor hides real IPs |
| Family-only discovery | Beacon seed encryption |
| Lineage privacy | Layered encryption |
| Forward secrecy | Ephemeral keys per session |
| Censorship resistance | Tor network |
| No central point of failure | Mesh topology |

---

## Evolution Phases

### Phase 1: Arti Integration (2-3 days)
- Add arti-client dependency
- Create OnionRelay struct
- Integrate with Songbird server startup
- Test basic .onion creation

### Phase 2: Mesh Gossip (2-3 days)
- Implement LayeredBirdSong struct
- Add gossip propagation to existing BirdSong
- Implement deduplication (nonce tracking)
- Test mesh formation

### Phase 3: Bootstrap Flow (1-2 days)
- QR code generation for beacon seed + .onion
- Physical handoff protocol
- Mesh topology sharing

### Phase 4: Production Hardening (2-3 days)
- Vanguard relays (when Arti supports)
- Connection redundancy
- Offline message queuing

---

## Open Questions

1. **Tor latency**: Onion routing adds latency. Acceptable for coordination?
2. **Bootstrap diversity**: What if first device is offline?
3. **Key rotation**: How often to rotate .onion addresses?
4. **Bandwidth**: Gossip can amplify traffic. Rate limiting?

---

## Alternatives Considered

| Alternative | Pros | Cons |
|------------|------|------|
| Public STUN/TURN | Works now | Not sovereign |
| Port forwarding | Simple | Manual setup, ISP dependent |
| Cloudflare Tunnel | Easy | Trust Cloudflare |
| I2P | Alternative to Tor | Smaller network |
| Custom gossip over UDP | Lower latency | Needs bootstrap |

**Selected**: Tor (Arti) because:
- Pure Rust
- Large network
- Proven security
- No port forwarding
- Active development

---

## References

- [Arti Documentation](https://arti.torproject.org/)
- [tor-hsservice crate](https://docs.rs/tor-hsservice)
- BirdSong Protocol Spec (archived)
- Lineage Relay Spec (archived)

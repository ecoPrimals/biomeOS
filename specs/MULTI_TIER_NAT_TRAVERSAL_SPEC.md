# Multi-Tier NAT Traversal Specification

**Version**: 1.0.0  
**Status**: IMPLEMENTED  
**Date**: February 5, 2026

---

## Executive Summary

biomeOS implements a sovereignty-first multi-tier NAT traversal architecture that prioritizes self-hosted infrastructure while maintaining robust fallbacks to public services. The system uses genetic lineage for encrypted relay and public STUN only for address discovery (zero payload exposure).

---

## Architecture Overview

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                    MULTI-TIER NAT TRAVERSAL                                   ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                               ║
║  ┌─────────────────────────────────────────────────────────────────────────┐ ║
║  │ TIER 1: Genetic Lineage Relay (Highest Trust)                           │ ║
║  │ ├── Zero external dependencies                                          │ ║
║  │ ├── Encrypted with mito beacon seed (family-only)                       │ ║
║  │ ├── Tower offers relay to family members                                │ ║
║  │ └── Data path: Device → Family Node → Target Device                     │ ║
║  └─────────────────────────────────────────────────────────────────────────┘ ║
║                              ↓ (fallback)                                     ║
║  ┌─────────────────────────────────────────────────────────────────────────┐ ║
║  │ TIER 2: Self-Hosted STUN (High Trust)                                   │ ║
║  │ ├── coturn on Tower/HPC (future)                                        │ ║
║  │ ├── Your infrastructure, your keys                                      │ ║
║  │ ├── Second verification after public discovery                          │ ║
║  │ └── Eliminates external STUN dependency                                 │ ║
║  └─────────────────────────────────────────────────────────────────────────┘ ║
║                              ↓ (fallback)                                     ║
║  ┌─────────────────────────────────────────────────────────────────────────┐ ║
║  │ TIER 3: Public STUN (Medium Trust - Address Discovery Only)             │ ║
║  │ ├── "International neutral ground" (Google, Cloudflare, Nextcloud)      │ ║
║  │ ├── Metadata exposure: IP:port + timing ONLY                            │ ║
║  │ ├── NO beacon content, capabilities, or lineage exposed                 │ ║
║  │ └── Server rotation for privacy (prevent tracking)                      │ ║
║  └─────────────────────────────────────────────────────────────────────────┘ ║
║                              ↓ (future)                                       ║
║  ┌─────────────────────────────────────────────────────────────────────────┐ ║
║  │ TIER 4: Rendezvous (Low Trust - Friend Gaming)                          │ ║
║  │ ├── Steam, Discord integration (future)                                 │ ║
║  │ ├── Piggyback on existing gaming infrastructure                         │ ║
║  │ └── Convenience for friend scenarios                                    │ ║
║  └─────────────────────────────────────────────────────────────────────────┘ ║
║                                                                               ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

---

## Privacy Model

### What Public STUN Servers See

| Data | Exposed? | Notes |
|------|----------|-------|
| Public IP:port | ✅ Yes | Fundamental to STUN |
| Request timing | ✅ Yes | When you query |
| NAT type | ✅ Yes | If you test |
| Beacon content | ❌ No | Never sent to STUN |
| Capabilities | ❌ No | Only in encrypted beacon |
| Lineage data | ❌ No | Only in encrypted beacon |
| Node ID | ❌ No | Only in encrypted beacon |

### Privacy Mitigations

1. **Server Rotation**: Rotate public STUN servers hourly (prevent tracking)
2. **Minimal Metadata**: Strip unnecessary headers from STUN requests
3. **Randomized Timing**: Add jitter to prevent traffic analysis
4. **Self-Hosted Option**: Use Tier 2 to eliminate public STUN entirely

---

## Connection Flow

### Scenario: Pixel (iPhone Hotspot) → Tower (Home ISP)

```
Step 1: Address Discovery (Tier 3: Public STUN)
┌─────────────────────────────────────────────────────────────────────┐
│ Tower                                    Google STUN               │
│   │                                         │                       │
│   │──── STUN Binding Request ──────────────►│                       │
│   │◄─── 198.51.100.1:32822 ─────────────│                       │
│                                                                     │
│ Pixel                                    Google STUN               │
│   │                                         │                       │
│   │──── STUN Binding Request ──────────────►│                       │
│   │◄─── 107.122.244.113:62847 ─────────────│                       │
└─────────────────────────────────────────────────────────────────────┘
Metadata exposed: IP:port only (no beacon content)

Step 2: Beacon Exchange (Tier 1: Genetic Lineage)
┌─────────────────────────────────────────────────────────────────────┐
│ Tower generates encrypted beacon:                                   │
│   birdsong.generate_encrypted_beacon({                             │
│     node_id: "tower",                                              │
│     capabilities: ["ai-server", "gpu", "family-relay"],            │
│     public_endpoint: "198.51.100.1:32822"                       │
│   })                                                               │
│   → Encrypted with mito beacon seed                                │
│                                                                     │
│ Pixel decrypts beacon:                                             │
│   birdsong.decrypt_beacon(tower_beacon)                            │
│   → is_family: true                                                │
│   → Now has Tower's public endpoint + capabilities                 │
└─────────────────────────────────────────────────────────────────────┘
Data path: Encrypted, family-only

Step 3: Connection Attempt (UDP Hole Punch)
┌─────────────────────────────────────────────────────────────────────┐
│ Both sides have symmetric NAT → Direct hole punch FAILS            │
│                                                                     │
│ Tower: 198.51.100.1:* ─────X────► Pixel: 107.122.244.113:*     │
│ Pixel: 107.122.244.113:* ─────X────► Tower: 198.51.100.1:*     │
│                                                                     │
│ Symmetric NAT changes port per destination = unpredictable         │
└─────────────────────────────────────────────────────────────────────┘

Step 4: Family Relay Fallback (Tier 1)
┌─────────────────────────────────────────────────────────────────────┐
│ Tower offers relay service (can_offer_relay: true)                 │
│                                                                     │
│ Pixel ──► Tower LAN (192.0.2.10) ──► Target                    │
│   │                  │                                             │
│   └── Encrypted with mito seed ──────┘                             │
│                                                                     │
│ Data encrypted end-to-end with family beacon seed                  │
│ Tower relay sees encrypted blob only (no content)                  │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Configuration Files

### Multi-Tier STUN Config

Location: `config/stun/multi_tier.toml`

```toml
[general]
enabled = true
strategy = "sovereignty-first"  # sovereignty-first | fastest-first | lineage-only

[lineage]
enabled = true
prefer_lineage = true
relay_offer_mode = "automatic"

[public_stun]
enabled = true
use_as_fallback_only = true
rotate_servers = true

[[public_stun.servers]]
address = "stun.nextcloud.com:3478"
vetted = true
comment = "Open-source, privacy-focused"
```

### Known Beacons (v3.1.0)

Location: `.known_beacons.json`

```json
{
  "version": "3.1.0",
  "schema": "evolved-genetic-v2",
  "nat_traversal": {
    "strategy": "sovereignty-first",
    "family_relay": {
      "enabled": true,
      "relay_nodes": ["tower"]
    }
  },
  "this_node": {
    "nat_info": {
      "type": "symmetric",
      "can_offer_relay": true
    }
  }
}
```

---

## NAT Type Compatibility Matrix

| Initiator NAT | Target NAT | Direct Punch | Relay Needed |
|---------------|------------|--------------|--------------|
| Full Cone | Full Cone | ✅ 95% | No |
| Full Cone | Restricted | ✅ 90% | No |
| Restricted | Restricted | ✅ 85% | No |
| Port-Restricted | Port-Restricted | ⚠️ 80% | Sometimes |
| Symmetric | Any | ❌ 30% | **Yes** |
| Any | Symmetric | ❌ 30% | **Yes** |
| **Symmetric** | **Symmetric** | ❌ 5% | **Always** |

Current deployment (Tower + Pixel): Both **Symmetric NAT** → Relay required

---

## Self-Hosted STUN Server (Future)

### Option A: coturn (Recommended)

```bash
# Install coturn
sudo apt install coturn

# Configure /etc/turnserver.conf
listening-port=3478
fingerprint
lt-cred-mech
realm=biomeos.local
server-name=tower.biomeos.local

# For TURN (relay) functionality:
relay-ip=192.0.2.10
external-ip=198.51.100.1

# Start
sudo systemctl enable coturn
sudo systemctl start coturn
```

### Option B: Pure Rust STUN Server

Add to `songbird-stun`:

```rust
// Future: Add StunServer alongside StunClient
pub struct StunServer {
    bind_addr: SocketAddr,
    // ...
}

impl StunServer {
    pub async fn run(&self) -> Result<(), StunError> {
        // Handle STUN binding requests
        // Return mapped address
    }
}
```

---

## Sync Evolution

The `.known_beacons.json` syncs across family members, allowing:

1. **Relay Discovery**: Pixel learns Tower offers relay
2. **NAT Info Sharing**: All nodes know each other's NAT type
3. **Capability Evolution**: New nodes inherit family's NAT knowledge
4. **STUN Server Sharing**: When self-hosted STUN available, all family uses it

```
Sync Flow:
┌────────┐     encrypted beacon      ┌────────┐
│ Tower  │ ◄──────────────────────► │ Pixel  │
│        │   .known_beacons.json    │        │
│ relay: │   syncs NAT info +       │ relay: │
│  true  │   relay capabilities     │  false │
└────────┘                          └────────┘
```

---

## Security Considerations

1. **Public STUN Trust**: Only address discovery, no payload
2. **Beacon Encryption**: All family data encrypted with mito seed
3. **Relay Privacy**: Tower relay sees encrypted blob only
4. **Lineage Verification**: BearDog verifies family membership
5. **Server Rotation**: Prevents single-server tracking

---

## Implementation Status

| Component | Status | Notes |
|-----------|--------|-------|
| Multi-tier config | ✅ Implemented | `config/stun/multi_tier.toml` |
| Known beacons v3.1 | ✅ Implemented | NAT info + relay capabilities |
| Public STUN client | ✅ Working | `songbird-stun` crate |
| Beacon exchange | ✅ Validated | Cross-NAT encryption verified |
| Family relay | ⚠️ Infrastructure exists | `songbird-lineage-relay` crate |
| Self-hosted STUN | 🔧 Planned | coturn or pure Rust |
| Hole punch coordination | 🔧 Planned | Signaling server needed |

---

## Next Steps

1. **Test Family Relay**: Verify `songbird-lineage-relay` works with current setup
2. **Install coturn**: Self-hosted STUN on Tower
3. **Add Signaling**: Coordinate simultaneous hole punch attempts
4. **Sync to Pixel**: Push updated `.known_beacons.json`

---

**Sovereignty Gradient**: Lineage > Self-Hosted > Public STUN > Rendezvous

The system always prefers higher-trust tiers but gracefully falls back for connectivity.

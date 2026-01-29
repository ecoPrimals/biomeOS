# Mobile & Hardware Anchor Architecture

**Date**: January 29, 2026  
**Status**: 🟡 Design Complete, Awaiting Implementation  
**Devices**: Pixel 8a (Android HSM), SoloKey (FIDO2/U2F Root)

---

## 🎯 Executive Summary

biomeOS can use **mobile phones** and **hardware security keys** as trust anchors:

1. **Pixel 8a**: Android Keystore / Titan M2 HSM for hardware-backed lineage root
2. **SoloKey**: FIDO2/U2F physical root for proving "human in the loop"
3. **Tower Integration**: Both can serve as HSM providers for local/remote NUCLEUS

This enables:
- 📱 **Your lineage travels with you** (phone in pocket)
- 🔑 **Physical attestation** (SoloKey proves possession)
- 🌐 **Remote STUN gateway** (phone on mobile network bridges NAT)
- 🤝 **LiveSpore bootstrapping** (validate new hardware from phone)

---

## 🏗️ Architecture Overview

```
                    ┌────────────────────────────────────────┐
                    │           Trust Hierarchy              │
                    └────────────────────────────────────────┘
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
            ┌───────────┐     ┌───────────┐     ┌───────────┐
            │  SoloKey  │     │ Pixel 8a  │     │  LiveSpore│
            │  (Root)   │     │  (Mobile) │     │  (USB)    │
            └─────┬─────┘     └─────┬─────┘     └─────┬─────┘
                  │                 │                 │
                  │  FIDO2/U2F      │  JSON-RPC       │  JSON-RPC
                  │  Challenge      │  over WiFi/     │  over Unix
                  │                 │  Mobile         │  Socket
                  ▼                 ▼                 ▼
            ┌────────────────────────────────────────────────┐
            │              Tower Atomic (BearDog)            │
            │         • Validates lineage from all roots     │
            │         • Derives family keys hierarchically   │
            │         • Provides TLS/crypto to Songbird      │
            └────────────────────────────────────────────────┘
```

---

## 📱 Pixel 8a: Mobile HSM Anchor

### Hardware Capabilities

| Component | Capability |
|-----------|------------|
| **Tensor G3** | ARM64 crypto acceleration |
| **Titan M2** | Hardware Security Module, StrongBox |
| **Android Keystore** | Hardware-backed key storage |
| **Biometrics** | Fingerprint/Face unlock gates |

### Deployment Modes

#### Mode 1: HSM-Only (Tower Lite) ⭐ Recommended

```toml
[beardog_hsm]
mode = "hsm-only"
provider = "android-keystore"
strongbox_preferred = true

[tower.config]
beardog_mode = "hsm-only"   # No full BearDog, just HSM operations
songbird_role = "observer"   # Participates but doesn't route
biomeos_role = "participant" # No orchestration
```

**Use Cases**:
- Store master `.family.seed` in hardware
- Sign deployment manifests
- 2FA for sensitive operations
- Bridge for NAT traversal (mobile network → local LAN)

#### Mode 2: Full NUCLEUS (Mobile Node)

```toml
[deployment]
deployment_type = "tower-lite"
mobile_mode = true

[tower]
enabled = true
node_id = "tower-pixel8a"
```

**Use Cases**:
- Full mesh participant
- Local AI inference (Squirrel-lite)
- Portable compute node
- Emergency federation backup

### Deployment Commands

```bash
# Step 1: Cross-compile BearDog for Android
cargo build --release \
  --target aarch64-linux-android \
  --bin beardog-server

# Step 2: Push to Pixel via ADB
adb shell mkdir -p /data/local/tmp/biomeOS/primals
adb push target/aarch64-linux-android/release/beardog-server \
  /data/local/tmp/biomeOS/primals/
adb shell chmod +x /data/local/tmp/biomeOS/primals/beardog-server

# Step 3: Launch HSM on Pixel
adb shell "cd /data/local/tmp/biomeOS && \
  FAMILY_ID=nat0 NODE_ID=pixel_hsm BEARDOG_HSM_MODE=hardware \
  ./primals/beardog-server"

# Step 4: Test from desktop (if network accessible)
curl http://192.168.1.XXX:8080/health
```

---

## 🔑 SoloKey: Physical Lineage Root

### What is SoloKey?

SoloKey is an **open-source FIDO2/U2F hardware security key**:
- Open firmware (can audit/modify)
- USB + NFC interfaces
- Challenge-response cryptography
- Physical button press required

### Integration with biomeOS

```
SoloKey Role in Lineage:
├── Ultimate Root of Trust
│   • Physical possession = trust
│   • Button press = human in loop
│
├── Key Derivation Root
│   • Master seed locked to SoloKey
│   • All family keys derived from SoloKey-protected root
│
└── Attestation for New Devices
    • New LiveSpore needs SoloKey touch to join family
    • Proves physical control during bootstrapping
```

### Integration Architecture

```
┌─────────────────┐       ┌─────────────────┐
│    SoloKey      │◄─────►│   Pixel 8a      │
│  (USB-C/NFC)    │ CTAP2 │ (USB OTG)       │
└─────────────────┘       └────────┬────────┘
                                   │
                                   │ JSON-RPC
                                   ▼
                          ┌────────────────┐
                          │    BearDog     │
                          │ (HSM Handler)  │
                          └────────┬───────┘
                                   │
                                   ▼
                          ┌────────────────┐
                          │  Family Keys   │
                          │   Derived      │
                          └────────────────┘
```

### SoloKey JSON-RPC Methods (Proposed)

```
solokey.* methods for BearDog:
├── solokey.challenge_sign     # Sign challenge with FIDO2 credential
├── solokey.get_assertion      # Get attestation for operation
├── solokey.verify_presence    # Confirm button press (human in loop)
└── solokey.derive_key         # Use SoloKey as HKDF root
```

### Pixel 8a + SoloKey Hierarchy

```toml
[beardog_hsm.solokey]
enabled = true
use_for_master_seed = true    # SoloKey protects master seed
usb_otg_enabled = true        # Connect via USB-C OTG
require_for_deployment = true # SoloKey touch for new nodes
```

**Lineage Flow**:
```
SoloKey (Physical Root)
    │
    ├── Challenge-Response: Unlock master seed
    │
    ▼
Pixel 8a Android Keystore (Mobile HSM)
    │
    ├── Derives: family_key = HKDF(master_seed, "biomeOS/family/nat0")
    │
    ▼
Tower BearDog (Local Compute)
    │
    ├── Derives: node_key = HKDF(family_key, "node/tower-alpha")
    │
    └── All crypto operations use derived keys
```

---

## 🌐 STUN/Relay Gateway Use Case

### Problem: NAT Traversal

USB LiveSpores behind different NATs can't directly connect:
```
LiveSpore A (192.168.1.x) ─── NAT ───┐
                                      │ ?
LiveSpore B (10.0.0.x) ────── NAT ───┘
```

### Solution: Pixel as STUN Gateway

```
LiveSpore A                  Pixel 8a                  LiveSpore B
(Home WiFi)              (Mobile Network)              (Coffee Shop)
    │                          │                           │
    │  ◄── UDP Beacon ──►      │     ◄── UDP Beacon ──►   │
    │                          │                           │
    │  "I'm 192.168.1.5"       │     "I'm 10.0.0.42"      │
    │                          │                           │
    │                          │                           │
    │  ──── Connect via Pixel's public IP ────            │
    │                          │                           │
    └──────────────────────────┴───────────────────────────┘
```

**Why Pixel Works**:
- Mobile network provides public IP (or carrier-grade NAT with STUN)
- Pixel can relay discovery beacons
- Once peers know each other, direct UDP hole-punch possible
- Pixel becomes "rendezvous server"

### Configuration

```toml
[pixel.stun_gateway]
enabled = true
public_relay = true           # Relay beacons for other nodes
mobile_network_stun = true    # Use mobile network for STUN
rendezvous_port = 2300        # Same as Songbird discovery
max_relay_bandwidth_mbps = 1  # Conservative for mobile data
```

---

## 🚀 LiveSpore Bootstrap Flow

### Scenario: New USB LiveSpore Needs Trust

```
New LiveSpore        SoloKey           Pixel 8a          Tower
(untrusted)         (physical)        (mobile HSM)      (verified)
     │                  │                  │                │
     │ "I want to join family nat0"        │                │
     │ ─────────────────────────────────────────────────────►
     │                  │                  │                │
     │                  │     "Touch SoloKey to approve"    │
     │                  │ ◄────────────────┤                │
     │                  │                  │                │
     │            [User touches SoloKey]   │                │
     │                  │                  │                │
     │                  │     Signed challenge              │
     │                  │ ────────────────►│                │
     │                  │                  │                │
     │                  │     Derive family key             │
     │                  │                  │────────────────►
     │                  │                  │                │
     │ "Here's your derived node_key"     │                │
     │ ◄────────────────────────────────────────────────────┤
     │                  │                  │                │
     │ [Now trusted member of family]     │                │
     ▼                  ▼                  ▼                ▼
```

### Bootstrap Script

```bash
#!/bin/bash
# bootstrap_livespore_with_solokey.sh

NEW_SPORE_PATH="/media/eastgate/biomeOS_new"
FAMILY_ID="cf7e8729dc4ff05f"
PIXEL_HSM="http://192.168.1.XXX:8080"  # Pixel IP

echo "=== LiveSpore Bootstrap with SoloKey ==="

# 1. Request bootstrap approval from Pixel HSM
echo "Requesting approval... (touch SoloKey when prompted)"
CHALLENGE=$(curl -s "$PIXEL_HSM" -d '{
  "jsonrpc":"2.0",
  "method":"bootstrap.request",
  "params":{"family_id":"'$FAMILY_ID'"},
  "id":1
}' | jq -r '.result.challenge')

# 2. User touches SoloKey (on Pixel via USB-C)
# Pixel displays: "Touch SoloKey to approve new LiveSpore"

# 3. Derive node key for new spore
NODE_KEY=$(curl -s "$PIXEL_HSM" -d '{
  "jsonrpc":"2.0",
  "method":"bootstrap.derive_node_key",
  "params":{"challenge":"'$CHALLENGE'","node_id":"new-spore"},
  "id":2
}' | jq -r '.result.node_key')

# 4. Write family seed to new LiveSpore
echo "$NODE_KEY" | xxd -r -p > "$NEW_SPORE_PATH/.family.seed"

echo "✅ LiveSpore bootstrapped with hardware-attested lineage"
```

---

## 📋 Implementation Checklist

### Pixel 8a Deployment

- [x] `aarch64-linux-android` target installed
- [x] Pixel 8a deployment config exists (`deployments/basement-hpc/pixel8a.toml`)
- [x] Deployment guide exists (`archive/.../PIXEL_DEPLOYMENT_GUIDE.md`)
- [ ] Cross-compile BearDog for Android
- [ ] Test on Pixel via ADB
- [ ] Validate Android Keystore integration
- [ ] Test Titan M2 StrongBox (if available)
- [ ] Test HSM operations from desktop

### SoloKey Integration

- [ ] Research SoloKey CTAP2/FIDO2 protocol
- [ ] Design `solokey.*` JSON-RPC methods for BearDog
- [ ] Implement SoloKey challenge-sign in BearDog
- [ ] Test with Pixel via USB-C OTG
- [ ] Bootstrap flow with SoloKey attestation

### STUN Gateway

- [ ] Wait for Songbird STUN method exposure
- [ ] Design Pixel relay architecture
- [ ] Test mobile network → LAN relay
- [ ] UDP hole-punch validation

---

## 🔗 Related Documents

| Document | Description |
|----------|-------------|
| `deployments/basement-hpc/pixel8a.toml` | Pixel 8a deployment config |
| `archive/.../PIXEL_DEPLOYMENT_GUIDE.md` | Cross-compile and deploy guide |
| `specs/SPORE_DEPLOYMENT_ARCHITECTURE.md` | HSM-Anchored vs HPC-Anchored |
| `docs/handoffs/SONGBIRD_STUN_RENDEZVOUS_HANDOFF.md` | STUN method requirements |

---

## 🎯 Next Steps

### Immediate (This Session)

1. **Connect Pixel 8a via USB**
2. **Enable USB debugging** (Settings → Developer Options)
3. **Cross-compile BearDog** for `aarch64-linux-android`
4. **Push and launch** BearDog HSM on Pixel
5. **Test hardware-backed operations** from desktop

### Short-Term (Next Sessions)

1. **Full Tower Lite** deployment on Pixel
2. **SoloKey integration** research and prototyping
3. **STUN gateway** once Songbird exposes methods
4. **LiveSpore bootstrap** with hardware attestation

### Long-Term

1. **Native Android app** with UI
2. **iOS support** (iPhone HSM)
3. **Multi-SoloKey** for redundancy
4. **Offline attestation** (air-gapped signing)

---

**Status**: Architecture Complete  
**Dependencies**: ADB access to Pixel, SoloKey for physical root  
**Next**: Cross-compile BearDog and deploy to Pixel! 📱🔑


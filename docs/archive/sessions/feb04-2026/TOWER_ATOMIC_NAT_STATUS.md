# Tower Atomic NAT Traversal Status

**Date**: February 5, 2026 @ 01:00  
**Status**: ✅ Foundation Ready | ✅ STUN Fixed | ✅ Cross-Network Beacon Exchange | ⚠️ TLS Handshake Issue

---

## Current Ecosystem State

### Tower (USB Desktop - 192.168.1.144)

| Component | Status | Socket |
|-----------|--------|--------|
| BearDog | ✅ Healthy | `/run/user/1000/biomeos/beardog-nat0.sock` |
| Songbird | ✅ Running | `/run/user/1000/biomeos/songbird-nat0.sock` |
| Neural API | ✅ Routing | `/run/user/1000/biomeos/neural-api-nat0.sock` |

### Pixel (192.168.1.80)

| Component | Status | Port/Socket |
|-----------|--------|-------------|
| BearDog | ✅ Healthy | TCP 9900 (localhost) |
| Songbird | ✅ Running | TCP 9901 (localhost), 8080 (HTTPS public) |
| Squirrel | ✅ Running | |
| Toadstool | ✅ Running | |
| NestGate | ✅ Running | |

---

## Beacon/Lineage Seeds

### Lineage (Nuclear DNA) - IDENTICAL ✅

Both devices share the same `.family.seed`:
```
8ff3b864a4bc589a418ada16499c829e0d808dff8dc9f8328f9f7ac02c42dd74
```

### Beacon Seeds (Mitochondrial DNA)

| Device | Beacon ID |
|--------|-----------|
| USB Tower | `d03029e5c5cd0c3b44e2e316118943d8bfee887fd589d20b647ee5a16eb462f1` |
| Pixel8a | `c86cb868b057f996dbbbf9d2f41fbe60ddefdfd3efef3deaa22f0c800d8b51f2` |

Both devices know each other via `.known_beacons.json` ✅

---

## Working Features

### 1. Capability Routing (Robust Rust Solution) ✅

Fixed in `translation_loader.rs` to use `capability_to_provider_fallback()`:
- `security` → `beardog`
- `http` → `songbird`
- `beacon` → `beardog`
- No symlinks needed!

### 2. Beacon Encryption ✅

```bash
# Works via Neural API
capability.call("beacon", "encrypt", {plaintext: "..."})
capability.call("beacon", "decrypt", {ciphertext: "...", nonce: "...", timestamp: ...})
```

### 3. Cross-Device Discovery ✅

Tower sees Pixel via UDP broadcast (port 2300):
```
Discovered peer: pixel8a (v3.0, capabilities: [...], HTTPS: https://192.168.1.80:8080)
```

---

## Issues to Fix

### 1. TLS Handshake (High Priority)

**Error**: `Server responded with HTTP instead of TLS (got 'HTTP/1.1 400 Bad Request')`

Tower → Pixel HTTPS connection fails. Possible causes:
- Cipher suite mismatch between Tower and Pixel Songbird TLS implementations
- Certificate validation issue
- Pure Rust TLS 1.3 handshake incompatibility

**Impact**: Peer health checks fail, peers don't stay in discovery cache

### 2. STUN Client - ✅ FIXED

**Previously**: `Address family not supported by protocol (os error 97)`

**Fix Applied**: Modified `songbird-stun/src/client.rs` to:
1. Collect all DNS results and prefer IPv4 addresses
2. Match local socket binding to server address family

**Verified Working**:
- **Tower**: `162.226.225.148:45752` (via stun.nextcloud.com)
- **Pixel**: `162.226.225.148:39345` (via direct IP 74.125.250.129:19302)

Same public IP (both on same router/NAT). For real punch-through, one device needs hotspot.

### 3. BirdSong family_id (Low Priority)

**Error**: `Missing family_id` in Songbird's BirdSong integration

Direct BearDog beacon calls work, but Songbird's wrapper doesn't pass family_id.

---

## Next Steps for Public NAT

1. **Fix TLS Handshake** (High Priority)
   - Debug cipher suite negotiation
   - Consider fallback to HTTP for local LAN

2. ~~Fix STUN IPv4 binding~~ ✅ **DONE**

3. **Hotspot Testing** (Ready!)
   - Connect Pixel to phone hotspot (different public IP)
   - Both devices can now get STUN public addresses
   - Test UDP hole punching via beacon exchange

4. **Future: External Primals**
   - `rhizoCrypt` - DAG storage
   - `loamSpine` - Linear storage  
   - `sweetGrass` - Semantic braid
   - These can handle social beacon DAG and genetic seed persistence

---

## Quick Test Commands

```bash
# Tower BearDog health
echo '{"jsonrpc":"2.0","method":"health","id":1}' | nc -U /run/user/1000/biomeos/beardog-nat0.sock -w 2

# Tower beacon encrypt
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"beacon","operation":"encrypt","args":{"plaintext":"SGVsbG8="}},"id":1}' | nc -U /run/user/1000/biomeos/neural-api-nat0.sock -w 5

# Pixel BearDog health (via ADB)
adb shell 'echo {"jsonrpc":"2.0","method":"health","id":1} | nc 127.0.0.1 9900 -w 2'

# Check discovery
echo '{"jsonrpc":"2.0","method":"discovery.peers","id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock -w 5
```

---

## Session Summary

This session accomplished:
1. ✅ Fixed capability routing (Rust-based, no symlinks)
2. ✅ Verified beacon/lineage seed sync across devices
3. ✅ Confirmed cross-device discovery working (UDP broadcast)
4. ✅ **FIXED STUN IPv4/IPv6 issue** - Both devices get public addresses
5. ⚠️ TLS handshake issue still blocking HTTPS peer communication
6. 🧬 **IDENTIFIED CRITICAL ISSUE**: Lineage seeds are COPIED, not DERIVED

**STUN Fix Details**: Modified `songbird-stun/src/client.rs` to prefer IPv4 addresses and match socket binding to server address family.

**Lineage Evolution Needed**: 
- Current model: Both devices have IDENTICAL `.family.seed` (wrong!)
- Correct model: Each device derives UNIQUE seed from shared root
- BearDog has `genetic.derive_lineage_key` - produces unique keys per device
- See `specs/GENETIC_LINEAGE_EVOLUTION_SPEC.md` for full evolution plan

**Current State**: Beacon exchange verified across different ISPs. TLS still needs work for HTTPS health checks.

---

## Cross-Network NAT Test Results (Feb 5, 2026 @ 01:00)

### Network Configuration
| Device | Network | Local IP | Public IP (STUN) |
|--------|---------|----------|------------------|
| Tower | iPhone Hotspot | 172.20.10.2 | **107.116.252.130** |
| Pixel | Home ISP | 192.168.1.80 | **162.226.225.148** |

### Beacon Exchange Test ✅ SUCCESS

Using BirdSong encryption with shared family lineage (`8ff3b864a4bc589a`):

**Tower → Pixel:**
```
Encrypted: {"node":"usb-desktop","ip":"107.116.252.130:8080"}
Pixel decrypted successfully ✅
```

**Pixel → Tower:**
```
Encrypted: {"node":"pixel8a","ip":"162.226.225.148:8080","caps":["mobile","ai-client"]}
Tower decrypted successfully ✅
```

### What This Proves

1. **BirdSong encryption works cross-network** - Devices can encrypt beacons that only family members can read
2. **Lineage-based crypto works** - Same family_id = can decrypt each other's beacons
3. **Dark Forest foundation is solid** - Beacons can be exchanged via ANY channel (including hostile networks)
4. **Public IPs successfully discovered** - Both devices know each other's NAT-mapped addresses

### Next Steps for Full NAT Punch-Through

1. **Signaling Server** - Need a way to exchange beacons without direct connectivity
2. **Simultaneous UDP** - Both devices send UDP to each other's STUN-mapped ports
3. **Connection Establishment** - First packet after NAT mapping creates the hole

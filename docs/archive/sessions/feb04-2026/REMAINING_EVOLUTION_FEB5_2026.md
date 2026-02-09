# Remaining Evolution - February 5, 2026

**Status**: Lineage derivation IMPLEMENTED ✅  
**Priority**: TLS handshake and remaining Songbird integration

---

## Executive Summary

Cross-network beacon exchange **WORKS** and we **FIXED** the critical lineage issue:

1. ✅ **Lineage seeds now DERIVED** - Each device has unique seed (FIXED Feb 5 @ 03:17 UTC)
2. 🔴 **TLS handshake fails** - Between Songbird instances (still needs investigation)

---

## Issue 1: Lineage Seed Derivation ✅ RESOLVED

### Previous (WRONG) Model
```
USB Tower:  .family.seed = 8ff3b864a4bc589a...
Pixel:      .family.seed = 8ff3b864a4bc589a... (IDENTICAL COPY!)
```

### Current (CORRECT) Model - Implemented Feb 5, 2026 @ 03:17 UTC
```
Family Root: .family.seed = 8ff3b864a4bc589a... (shared genesis)
                            ↓ DERIVE via genetic.derive_lineage_key
USB Tower:   .lineage.seed = 5772c07f24654deb... (UNIQUE)
             .lineage.json = {device_id: "f65cecf5e44b...", method: "Blake3-Lineage-KDF"}
                            ↓
Pixel:       .lineage.seed = 3795d0cac4fb6576... (UNIQUE)
             .lineage.json = {device_id: "a3a85e31-e5b5...", method: "Blake3-Lineage-KDF"}
```

Each device now has UNIQUE derived seed, can prove shared ancestry!

### Implementation Details

**biomeOS CLI Enrollment**:
```bash
# Tower enrollment
biomeos enroll --family-id nat0 --node-id tower --beardog-socket /run/user/1000/biomeos/beardog-nat0.sock

# Pixel enrollment (via TCP since Unix sockets are restricted on Android)
./biomeos enroll --family-id nat0 --node-id pixel8a --beardog-socket 'tcp:127.0.0.1:9900'
```

**New Files Created**:
- `crates/biomeos-spore/src/beacon_genetics/derivation.rs` - LineageDeriver implementation
- `crates/biomeos/src/modes/enroll.rs` - CLI command
- `crates/biomeos-spore/src/beacon_genetics/capability.rs` - DirectBeardogCaller with TCP support

**Validation (Feb 5, 2026 @ 03:17 UTC)**

| Test | Result | Notes |
|------|--------|-------|
| Family seeds identical | ✅ | Correct - shared genetic root |
| Beacon seeds unique | ✅ | Different per device |
| Lineage seed files | ✅ | `.lineage.seed` + `.lineage.json` exist |
| Tower enrollment | ✅ | `V3LAfyRlTeu1hd6zEAtc3iMnNwuquiAgXVJtKse6fA4=` |
| Pixel enrollment | ✅ | `N5XQysT7ZXaedgrguJiWf6jtnfKlF9BbyYxx3ERx2PY=` |
| Seeds are different | ✅ | Cryptographically unique per device |
| Derivation method | ✅ | Blake3-Lineage-KDF via BearDog |

### Remaining Enhancements (Future)

| Feature | Status | Priority |
|---------|--------|----------|
| `genetic.sign_lineage_certificate` | ❌ | Low - for formal PKI |
| `genetic.verify_lineage_certificate` | ❌ | Low - for formal PKI |
| Merkle-style lineage proofs | ❌ | Low - for audit trails |
| Encrypted root seed storage | ❌ | Low - for genesis security |

### Reference
See `specs/GENETIC_LINEAGE_EVOLUTION_SPEC.md` for full design specification.

---

## Issue 2: TLS Handshake Failure (HIGH)

### Symptom
```
Tower Songbird → Pixel Songbird (HTTPS):
ERROR: TLS handshake failed: Server responded with HTTP instead of TLS 
       (got 'HTTP/1.1 400 Bad Request')
```

### Impact
- HTTPS health checks fail between primals
- Peer discovery shows device but can't verify health
- Blocks secure cross-device communication

### Possible Causes

1. **Cipher Suite Mismatch**
   - Tower and Pixel Songbird may have different TLS 1.3 cipher preferences
   - Pure Rust TLS (rustls) compatibility issue

2. **Certificate Validation**
   - Self-signed cert not accepted
   - Missing SNI header

3. **Port Binding**
   - Pixel port 8080 might have HTTP fallback before TLS

### Required Investigation

#### Songbird TLS Configuration
```rust
// Check crates/songbird-http-server/src/tls_config.rs
// Verify cipher suite configuration
// Check certificate generation
```

#### Debug Steps
```bash
# 1. Check Pixel TLS config
adb shell 'cat /data/local/tmp/biomeos/songbird.toml' | grep -i tls

# 2. Test with openssl
openssl s_client -connect 192.168.1.80:8080 -servername pixel8a

# 3. Check if HTTP/HTTPS on same port
curl -v http://192.168.1.80:8080/.well-known/songbird 2>&1 | head -20
```

### Fix Options

1. **Explicit TLS Port** - Separate HTTP (8080) and HTTPS (8443) ports
2. **ALPN Negotiation** - Proper TLS/HTTP2 negotiation
3. **Fallback to HTTP** - For local LAN (less secure but functional)

---

## Issue 3: BirdSong family_id Integration (MEDIUM)

### Symptom
```
birdsong.generate_encrypted_beacon:
ERROR: Encryption failed: BearDog JSON-RPC encrypt failed: Missing family_id
```

### Cause
Songbird's BirdSong wrapper calls BearDog but doesn't pass `family_id` parameter.

### Fix Location
```
songbird/crates/songbird-orchestrator/src/birdsong_integration.rs
```

### Required Change
```rust
// Before (broken)
beardog.call("birdsong.encrypt", json!({
    "plaintext": data,
}))

// After (working)
beardog.call("birdsong.encrypt", json!({
    "plaintext": data,
    "family_id": get_family_id(),  // ADD THIS
}))
```

---

## Issue 4: Songbird Standard Methods (MEDIUM)

### Missing Methods
| Method | Expected | Status |
|--------|----------|--------|
| `health` | Returns primal health | ❌ Not found |
| `identity` | Returns primal identity | ❌ Not found |
| `network.beacon_exchange` | Exchange beacons | ❌ Not found |

### Impact
- Neural API can't health-check Songbird
- Discovery shows "Unknown method: health"

### Fix
Add standard method handlers in Songbird's JSON-RPC dispatcher.

---

## Prioritized Action Plan

### Completed ✅
1. ✅ Document findings (this file)
2. ✅ Implement lineage DERIVATION in biomeOS (Feb 5 @ 03:17 UTC)
3. ✅ Create Songbird handoff document (`docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF_FEB_05_2026.md`)

### Songbird Team (Handoff Delivered)
1. Fix Songbird `family_id` passthrough - Simple fix
2. Add Songbird `health`/`identity`/`rpc.discover` methods - Simple fix
3. Investigate TLS handshake issue - Complex

### Future (Post-Songbird Fixes)
1. Add `genetic.sign_lineage_certificate` to BearDog
2. Update meeting protocol for proof exchange

### Future
1. Signaling server for cross-NAT beacon exchange
2. UDP hole punching with coordinated send
3. Cluster beacons (Phase 2C)

---

## Test Commands Reference

### Beacon Exchange (Working)
```bash
# Tower encrypts
FAMILY_ID="8ff3b864a4bc589a"
BEACON=$(echo -n '{"node":"usb-desktop","ip":"107.116.252.130"}' | base64 -w0)
echo "{\"jsonrpc\":\"2.0\",\"method\":\"birdsong.encrypt\",\"params\":{\"plaintext\":\"$BEACON\",\"family_id\":\"$FAMILY_ID\"},\"id\":1}" | nc -U /run/user/1000/biomeos/beardog-nat0.sock -w 5

# Pixel decrypts (via ADB)
adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"birdsong.decrypt\",\"params\":{\"ciphertext\":\"...\",\"family_id\":\"$FAMILY_ID\"},\"id\":1}' | nc 127.0.0.1 9900"
```

### STUN (Working)
```bash
# Tower
echo '{"jsonrpc":"2.0","method":"stun.get_public_address","params":{},"id":1}' | nc -U /run/user/1000/biomeos/songbird-nat0.sock -w 10

# Pixel
adb shell 'echo "{\"jsonrpc\":\"2.0\",\"method\":\"stun.get_public_address\",\"params\":{\"server\":\"74.125.250.129:19302\"},\"id\":1}" | nc 127.0.0.1 9901'
```

### TLS Debug
```bash
# Test HTTPS to Pixel
curl -k -v https://192.168.1.80:8080/.well-known/songbird 2>&1 | head -30

# Check Songbird logs
tail -50 /tmp/songbird-nat0.log | grep -i tls
```

---

## Summary Table

| Issue | Priority | Owner | Status |
|-------|----------|-------|--------|
| Lineage DERIVATION | ✅ Resolved | biomeOS + BearDog | **IMPLEMENTED** |
| TLS Handshake | 🔴 High | Songbird | Handoff delivered |
| BirdSong family_id | 🔴 High | Songbird | Handoff delivered |
| Songbird `health` | 🔴 High | Songbird | Handoff delivered |
| Signaling Server | 🟢 Low | biomeOS | Design phase |

**Handoff**: `docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF_FEB_05_2026.md`

---

## Recent Updates

### Feb 5, 2026 @ 14:05 UTC - Songbird Reharvest & Resync Complete
- **Rebuilt** Songbird from commit `78e1f7307` (v3.22.0) for both architectures
- **Deployed** new binaries to Tower (`livespore-usb/x86_64/primals/songbird`) and Pixel (`pixel8a-deploy/primals/songbird`)
- **Restarted** Songbird on both devices:
  - Tower: PID 2254733 (Unix socket)
  - Pixel: PID 19749 (TCP 127.0.0.1:9901)
- **Validated** seed architecture:
  - Family seeds: ✅ Match (`8ff3b864...`)
  - Lineage seeds: ✅ Different (Tower `5772c07f...`, Pixel `3795d0ca...`)
  - Beacon seeds: ✅ Different (per-device discovery)
- **Confirmed Issue 1**: TCP socket responds to `health`, Unix socket hangs
- **Updated Handoff**: Added resync validation section with environment variables and PIDs

### Feb 5, 2026 @ 03:35 UTC - Songbird Handoff Crafted
- **Investigated** root causes of Songbird issues
- **Identified** missing method routes in `songbird-universal-ipc/src/service.rs`
- **Found** `family_id` not passed in `birdsong_handler.rs` line 151
- **Created** comprehensive handoff: `docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF_FEB_05_2026.md`
- **Includes**: Code locations, fix suggestions, verification commands

### Feb 5, 2026 @ 03:17 UTC - Lineage Derivation IMPLEMENTED
- **Implemented** `biomeos enroll` CLI command
- **Created** `LineageDeriver` with BearDog genetic method integration
- **Added** `DirectBeardogCaller` supporting both Unix sockets and TCP (`tcp:host:port`)
- **Enrolled** both Tower and Pixel with UNIQUE derived lineage seeds
- **Verified** seeds are cryptographically different:
  - Tower: `5772c07f...` (from Blake3-Lineage-KDF)
  - Pixel: `3795d0ca...` (from Blake3-Lineage-KDF)
- **Critical architecture flaw FIXED** - devices no longer share identical seeds

### Feb 5, 2026 @ 02:57 UTC - Resync & Validation
- **Rebuilt** Songbird v3.33.0 and BearDog v0.9.0 (x86_64 + aarch64-musl)
- **Synced** new binaries to Pixel via ADB
- **Validated** genetic methods work cross-device
- **Identified** lineage derivation architecture gap

---

**Created**: February 5, 2026 @ 01:15 UTC  
**Last Updated**: February 5, 2026 @ 14:05 UTC  
**Author**: AI Assistant + eastgate

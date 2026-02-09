# Beacon Discovery Validation Handoff
## February 4, 2026

**Status**: 🟡 VALIDATED - Gaps Identified for Songbird Evolution  
**Purpose**: Validate USB ↔ Pixel beacon discovery via Neural API capability.call  
**Next Owner**: Songbird primal evolution

---

## Executive Summary

Validation testing on **February 4, 2026** revealed that:

1. ✅ Tower Atomic deploys and runs (BearDog + Songbird)
2. ✅ BearDog basic methods work (`health`, `identity`)
3. ❌ BearDog missing beacon-specific methods (`beacon.get_id`, `beacon.encrypt`, etc.)
4. ❌ Songbird cannot find BearDog (wrong socket path - looking for `/tmp/neural-api-nat0.sock`)
5. ❌ Songbird missing standard methods (`health`, `identity`, `rpc.discover`)

**Root Cause**: Primals are running but missing the methods biomeOS expects via `capability.call`.

---

## Validation Results

### What Works ✅

| Component | Status | Evidence |
|-----------|--------|----------|
| BearDog socket | ✅ | `/run/user/1000/biomeos/beardog-nat0.sock` |
| Songbird socket | ✅ | `/run/user/1000/biomeos/songbird-nat0.sock` |
| BearDog `health` | ✅ | `{"status":"healthy","version":"0.9.0"}` |
| BearDog `identity` | ✅ | `{"family":"nat0","node":"pop-os"}` |
| Beacon genetics files | ✅ | `.known_beacons.json`, `.beacon.seed`, `.family.seed` |
| ADB connected | ✅ | Pixel 8a available with TCP forwards |

### What's Missing ❌

#### BearDog Methods Needed

| Method | biomeOS Expects | BearDog Has | Gap |
|--------|-----------------|-------------|-----|
| `beacon.get_id` | ✅ Defined in capability_translation | ❌ Method not found | **MISSING** |
| `beacon.get_seed` | ✅ Defined | ❌ Method not found | **MISSING** |
| `beacon.encrypt` | ✅ Defined | ❌ Method not found | **MISSING** |
| `beacon.decrypt` | ✅ Defined | ❌ Method not found | **MISSING** |
| `beacon.try_decrypt` | ✅ Defined | ❌ Method not found | **MISSING** |
| `encrypt_discovery` | ✅ Used by Songbird | ❌ Method not found | **MISSING** |
| `verify_lineage` | ✅ For lineage verification | ❌ Method not found | **MISSING** |

#### Songbird Methods Needed

| Method | biomeOS Expects | Songbird Has | Gap |
|--------|-----------------|--------------|-----|
| `health` | Standard method | ❌ Unknown method | **MISSING** |
| `identity` | Standard method | ❌ Unknown method | **MISSING** |
| `rpc.discover` | List available methods | ❌ Unknown method | **MISSING** |
| `beacon_exchange` | For peer discovery | ❌ Not verified | **MISSING** |
| `http.request` | HTTP proxy | ⚠️ Fails (BearDog socket issue) | **BROKEN** |

#### Songbird Configuration Issue

```
ERROR: Failed to connect to BearDog at /tmp/neural-api-nat0.sock: Connection refused

Songbird is looking for: /tmp/neural-api-nat0.sock
BearDog is actually at:   /run/user/1000/biomeos/beardog-nat0.sock
```

---

## Capability Translation Registry (biomeOS Side) ✅

The biomeOS capability translations are correctly defined:

```rust
// crates/biomeos-atomic-deploy/src/capability_translation.rs
("beacon.get_id", "beacon.get_id")
("beacon.get_seed", "beacon.get_seed")
("beacon.encrypt", "beacon.encrypt")
("beacon.decrypt", "beacon.decrypt")
("beacon.try_decrypt", "beacon.try_decrypt")
("network.beacon_exchange", "beacon_exchange")
```

**The translations exist**, but the primals don't implement the methods.

---

## Songbird Evolution Requirements

### Priority 1: Fix BearDog Discovery

Songbird must find BearDog using the standard socket discovery:

```rust
// Current (BROKEN):
let beardog_socket = "/tmp/neural-api-nat0.sock";  // Wrong!

// Required (CORRECT):
// Use $BEARDOG_SOCKET environment variable
// Or XDG_RUNTIME_DIR/biomeos/beardog-{family}.sock
// Or /run/user/$UID/biomeos/beardog-{family}.sock
```

### Priority 2: Implement Standard Methods

```rust
// health method
fn health() -> HealthResponse {
    HealthResponse {
        primal: "songbird",
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    }
}

// identity method
fn identity() -> IdentityResponse {
    IdentityResponse {
        primal: "songbird",
        family_id: get_family_id(),
        capabilities: vec!["http", "discovery", "tls"],
    }
}
```

### Priority 3: Implement beacon_exchange

```rust
// network.beacon_exchange method
async fn beacon_exchange(params: BeaconExchangeParams) -> BeaconExchangeResponse {
    // 1. Connect to peer endpoint
    // 2. Exchange encrypted beacon payloads
    // 3. Return peer's beacon ID and encrypted seed
}
```

---

## BearDog Evolution Requirements

### Add Beacon Methods

```rust
// beacon.get_id - Return this node's beacon ID
fn beacon_get_id() -> BeaconIdResponse {
    BeaconIdResponse {
        beacon_id: self.beacon_seed.derive_id(),
    }
}

// beacon.get_seed - Return this node's beacon seed (for sharing)
fn beacon_get_seed() -> BeaconSeedResponse {
    BeaconSeedResponse {
        seed_hex: self.beacon_seed.to_hex(),
    }
}

// beacon.encrypt - Encrypt with beacon seed
fn beacon_encrypt(params: EncryptParams) -> EncryptResponse {
    let encrypted = self.beacon_seed.encrypt(&params.plaintext)?;
    EncryptResponse { ciphertext: encrypted }
}

// beacon.try_decrypt - Try to decrypt with a specific seed
fn beacon_try_decrypt(params: TryDecryptParams) -> TryDecryptResponse {
    match BeaconSeed::from_hex(&params.seed_hex).try_decrypt(&params.ciphertext) {
        Ok(plaintext) => TryDecryptResponse { decrypted: true, plaintext: Some(plaintext) },
        Err(_) => TryDecryptResponse { decrypted: false, plaintext: None },
    }
}
```

---

## Test Commands for Validation

### After BearDog Evolution

```bash
# Test beacon.get_id
echo '{"jsonrpc":"2.0","method":"beacon.get_id","params":{},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/beardog-nat0.sock

# Test beacon.encrypt
echo '{"jsonrpc":"2.0","method":"beacon.encrypt","params":{"plaintext":"SGVsbG8gV29ybGQ="},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/beardog-nat0.sock
```

### After Songbird Evolution

```bash
# Test health
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird-nat0.sock

# Test beacon_exchange
echo '{"jsonrpc":"2.0","method":"beacon_exchange","params":{"endpoint":"tcp://pixel:9101","payload":"..."},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird-nat0.sock
```

---

## Full Flow After Evolution

```
USB Desktop                        Pixel 8a
    │                                  │
    │  biomeOS BeaconGeneticsManager   │
    │  initiate_meeting()              │
    ▼                                  │
capability.call("beacon.get_id")       │
    │                                  │
    ▼                                  │
BearDog.beacon_get_id()                │
    │ → beacon_id                      │
    ▼                                  │
capability.call("beacon.get_seed")     │
    │                                  │
    ▼                                  │
BearDog.beacon_get_seed()              │
    │ → seed_hex                       │
    ▼                                  │
capability.call("crypto.encrypt")      │
    │                                  │
    ▼                                  │
BearDog.encrypt(seed, context)         │
    │ → encrypted_seed                 │
    ▼                                  │
capability.call("network.beacon_exchange")
    │                                  │
    ▼                                  ▼
Songbird.beacon_exchange() ◄─────────► Songbird.beacon_exchange()
    │         TCP/Dark Forest          │
    ▼                                  ▼
Return peer_beacon_id, encrypted_seed  Return peer_beacon_id, encrypted_seed
    │                                  │
    ▼                                  ▼
capability.call("crypto.decrypt")      capability.call("crypto.decrypt")
    │                                  │
    ▼                                  ▼
Store in .known_beacons.json           Store in .known_beacons.json
    │                                  │
    ▼                                  ▼
✅ Meeting complete!                   ✅ Meeting complete!
```

---

## Files Modified/Created

| File | Purpose |
|------|---------|
| `scripts/validate_beacon_discovery.sh` | Validation script |
| `docs/handoffs/BEACON_DISCOVERY_VALIDATION_FEB04_2026.md` | This handoff |

---

## Next Steps

1. **BearDog Evolution** (wateringHole/btsp/):
   - Add `beacon.*` methods
   - Add `encrypt_discovery` / `verify_lineage`

2. **Songbird Evolution** (wateringHole/songbird/):
   - Fix BearDog socket discovery (use $BEARDOG_SOCKET or XDG paths)
   - Add standard `health`, `identity` methods
   - Implement `beacon_exchange` for Dark Forest peer discovery

3. **Re-validate** after primal evolution:
   ```bash
   cd biomeOS
   FAMILY_ID=nat0 ./scripts/validate_beacon_discovery.sh
   ```

---

**Validation Date**: February 4, 2026  
**Validated By**: AI + biomeOS validation script  
**Handoff To**: BearDog + Songbird primal maintainers

# 🔍 Remaining Debt Before Validation

**Date**: February 6, 2026  
**Purpose**: Complete audit of remaining technical debt before deployment validation  
**Status**: All debt identified and catalogued

---

## Executive Summary

| Component | Debt Items | Effort | Priority |
|-----------|------------|--------|----------|
| Songbird `sovereign-onion` | 9 methods to refactor | ~4.5 hours | HIGH |
| biomeOS | 0 | - | ✅ DONE |
| BearDog | 0 | - | ✅ DONE |

**Total**: ~4.5 hours of Songbird refactoring before validation can proceed.

---

## Debt Breakdown

### ✅ BearDog - No Remaining Debt

All required crypto methods are implemented and tested:
- `crypto.sha3_256` ✅
- `crypto.ed25519_generate_keypair` ✅
- `crypto.sign_ed25519` ✅
- `crypto.verify_ed25519` ✅
- `crypto.x25519_generate_ephemeral` ✅
- `crypto.x25519_derive_secret` ✅
- `crypto.chacha20_poly1305_encrypt` ✅
- `crypto.chacha20_poly1305_decrypt` ✅
- `crypto.hmac_sha256` ✅

---

### ✅ biomeOS - No Remaining Debt

All integration infrastructure is ready:
- Capability translations registered ✅
- Integration tests passing (12/12) ✅
- Deployment graphs updated ✅
- Environment wiring configured ✅

---

### ⚠️ Songbird - TRUE PRIMAL Refactoring Required

**Crate**: `songbird-sovereign-onion`

#### `src/keys.rs` - 4 Methods

| Method | Current Debt | Refactor To |
|--------|--------------|-------------|
| `OnionIdentity::generate()` | Direct `ed25519_dalek` | `generate_via_beardog()` |
| `OnionIdentity::from_stored()` | Direct `SigningKey` | Use raw `[u8; 32]` |
| `EphemeralKeypair::generate()` | Direct `x25519_dalek` | `generate_via_beardog()` |
| `EphemeralKeypair::derive_shared_secret()` | Direct ECDH | `derive_via_beardog()` |
| `SessionKeys::derive()` | Direct `hmac` crate | `derive_via_beardog()` |

**Struct Changes Required**:
```rust
// Current (debt)
pub struct OnionIdentity {
    signing_key: SigningKey,      // ed25519_dalek type
    verifying_key: VerifyingKey,  // ed25519_dalek type
    // ...
}

// Refactored (TRUE PRIMAL)
pub struct OnionIdentity {
    secret_key: [u8; 32],         // raw bytes
    public_key: [u8; 32],         // raw bytes
    // ...
}
```

#### `src/address.rs` - 2 Methods

| Method | Current Debt | Refactor To |
|--------|--------------|-------------|
| `derive_onion_address()` | Direct `sha3::Sha3_256` | `derive_via_beardog()` |
| `validate_onion_address()` | Direct `sha3::Sha3_256` | `validate_via_beardog()` |

#### `src/crypto.rs` - 2 Methods

| Method | Current Debt | Refactor To |
|--------|--------------|-------------|
| `encrypt_data()` | Direct `chacha20poly1305` | `encrypt_via_beardog()` |
| `decrypt_data()` | Direct `chacha20poly1305` | `decrypt_via_beardog()` |

---

## Completed Infrastructure

### BeardogCryptoClient (Ready to Use)

```rust
// Already in songbird-sovereign-onion/src/beardog_crypto.rs
use songbird_sovereign_onion::BeardogCryptoClient;

let client = BeardogCryptoClient::from_env()?;

// All these methods are ready:
client.ed25519_generate_keypair()?;
client.ed25519_sign(&secret, &msg)?;
client.x25519_generate_ephemeral()?;
client.x25519_derive_secret(&our_secret, &their_public)?;
client.chacha20_poly1305_encrypt(&key, &nonce, &plaintext)?;
client.chacha20_poly1305_decrypt(&key, &nonce, &ciphertext)?;
client.sha3_256(&data)?;
client.hmac_sha256(&key, &data)?;
```

### Error Types (Ready)

```rust
// Already in songbird-sovereign-onion/src/error.rs
pub enum OnionError {
    RpcError(String),        // JSON-RPC errors
    ConnectionError(String), // Socket errors
    ConfigError(String),     // Missing env vars
    CryptoError(String),     // Crypto failures
    // ...existing variants...
}
```

### Reference Implementation

See `songbird-tls/src/cert/generator.rs` for the hybrid pattern:

```rust
pub enum CryptoMode {
    Standalone,  // For testing without BearDog
    BearDog,     // TRUE PRIMAL production
    Auto,        // Default: try BearDog, fallback
}
```

---

## Validation Blockers

### Must Complete Before Validation

1. **Songbird Refactoring** (~4.5 hours)
   - Without this, Songbird performs crypto locally
   - Violates TRUE PRIMAL pattern
   - Security audit surface is split

### Ready for Validation (When Debt Cleared)

- [ ] Start BearDog with `crypto.sha3_256` (already done)
- [ ] Start Songbird with `BEARDOG_SOCKET` env var
- [ ] Test .onion address generation via BearDog
- [ ] Test session key exchange via BearDog
- [ ] Test data encryption/decryption via BearDog
- [ ] End-to-end NAT traversal test

---

## Recommended Approach

### Option A: Complete Refactoring First (Recommended)

1. Songbird team completes `sovereign-onion` refactoring (~4.5 hours)
2. Run integration tests
3. Proceed with full validation

**Benefits**: Clean validation, no workarounds needed

### Option B: Parallel Work

1. biomeOS team validates capability translations (can do now)
2. Songbird team refactors in parallel
3. Integration test when both complete

**Benefits**: Faster overall if teams available in parallel

### Option C: Staged Validation

1. Validate biomeOS ↔ BearDog (now)
2. Validate biomeOS ↔ Songbird network (now)
3. Validate crypto delegation (after refactoring)

**Benefits**: Partial progress validation

---

## Handoff Documents Created

1. **Songbird Refactoring**:
   - `songbird/BEARDOG_CRYPTO_REFACTOR_HANDOFF_FEB06_2026.md`
   - Complete with code examples, checklists, testing strategy

2. **biomeOS Integration**:
   - `biomeOS/docs/handoffs/BIOMEOS_SONGBIRD_INTEGRATION_FEB06_2026.md`
   - Updated with debt summary and references

3. **BearDog Crypto**:
   - `biomeOS/docs/handoffs/BEARDOG_ONION_CRYPTO_HANDOFF_FEB06_2026.md`
   - API reference for all crypto methods

---

## Test Commands (After Refactoring)

```bash
# 1. Run Songbird sovereign-onion tests
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo test -p songbird-sovereign-onion --lib

# 2. Run biomeOS integration tests
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo test -p biomeos-atomic-deploy capability_translation_integration

# 3. Full workspace validation
cargo test --workspace --lib
```

---

## Conclusion

**All debt has been identified, catalogued, and handoff documentation delivered.**

The only remaining work before validation is:
- **~4.5 hours of Songbird refactoring** to complete TRUE PRIMAL pattern

biomeOS and BearDog are ready for validation.

---

🧬 biomeOS ✅ | 🐦 Songbird ⚠️ | 🐻🐕 BearDog ✅

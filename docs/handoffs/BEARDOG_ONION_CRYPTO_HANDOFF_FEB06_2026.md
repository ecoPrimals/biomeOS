# 🐻🐕 BearDog Onion Crypto Handoff

**Date**: February 6, 2026  
**From**: Songbird Team  
**To**: BearDog Team + biomeOS Team  
**Purpose**: Add crypto support for Sovereign Onion Service  
**Pattern**: Same as TLS 1.3 delegation

---

## 🎯 Overview

### What We Need

Songbird is evolving a **Sovereign Onion Service** (Pure Rust alternative to Arti) for `.onion` address generation and NAT traversal. Following the **TRUE PRIMAL** pattern established with TLS 1.3, **all crypto operations belong in BearDog**.

### Architecture (Same as TLS 1.3)

```
biomeOS (lifecycle orchestrator)
    ↓ starts & coordinates
BearDog (security primal) ←→ Songbird (network primal)
    ↓ JSON-RPC over Unix socket
    ↓ capability.call("crypto", "sha3_256", ...)
```

**Key Principle**: Songbird has **ZERO crypto primitives**. All crypto is delegated to BearDog via JSON-RPC, just like TLS 1.3.

---

## 📊 Current State

### BearDog Crypto API (Existing, for TLS 1.3)

**Location**: `docs/architecture/BEARDOG_CRYPTO_API_SPEC.md`

**8 Existing Methods**:
1. ✅ `beardog.crypto.sign_ed25519` - Ed25519 signing
2. ✅ `beardog.crypto.verify_ed25519` - Ed25519 verification
3. ✅ `beardog.crypto.x25519_generate_ephemeral` - X25519 key generation
4. ✅ `beardog.crypto.x25519_derive_secret` - X25519 ECDH
5. ✅ `beardog.crypto.chacha20_poly1305_encrypt` - AEAD encryption
6. ✅ `beardog.crypto.chacha20_poly1305_decrypt` - AEAD decryption
7. ✅ `beardog.crypto.blake3_hash` - Blake3 hashing
8. ✅ `beardog.crypto.hmac_sha256` - HMAC-SHA256 (for HKDF)

### What Onion Service Needs

| Operation | BearDog Status | Notes |
|-----------|---------------|-------|
| **Ed25519 identity** | ✅ Already have | For .onion identity keys |
| **X25519 key exchange** | ✅ Already have | For session keys |
| **ChaCha20-Poly1305** | ✅ Already have | For data encryption |
| **HMAC-SHA256** | ✅ Already have | For HKDF |
| **SHA3-256** | ⚠️ **NEED TO ADD** | For .onion address derivation |

**Gap**: Only **SHA3-256** is missing

---

## 🆕 New Method: SHA3-256 Hash

### Method Specification

**Method**: `beardog.crypto.sha3_256`

**Purpose**: Hash data with SHA3-256 (for .onion address derivation per Tor v3 spec)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "beardog.crypto.sha3_256",
  "params": {
    "data": "base64_encoded_data",
    "purpose": "onion_address_checksum"
  },
  "id": 9
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "hash": "base64_encoded_hash_32_bytes"
  },
  "id": 9
}
```

**Performance Target**: < 30 µs / KB (Pure Rust `sha3` crate)

### Use Cases

1. **Onion Address Derivation** (Tor v3 format):
   ```
   checksum = SHA3-256(".onion checksum" || pubkey || version)[0..2]
   address = base32(pubkey || checksum || version).onion
   ```

2. **Onion Address Validation**:
   - Compute expected checksum
   - Compare with embedded checksum

### Implementation (BearDog Side)

**Dependencies** (Pure Rust):
```toml
sha3 = "0.10"  # RustCrypto SHA3 implementation
```

**Handler**:
```rust
// In beardog-crypto-service/src/json_rpc_handlers.rs

use sha3::{Sha3_256, Digest};

pub async fn handle_sha3_256(params: Value) -> Result<Value, JsonRpcError> {
    let data_b64 = params["data"].as_str()
        .ok_or_else(|| JsonRpcError::invalid_params("Missing 'data'"))?;
    
    // Decode base64
    let data = base64::decode(data_b64)
        .map_err(|_| JsonRpcError::invalid_params("Invalid base64"))?;
    
    // Hash with SHA3-256
    let mut hasher = Sha3_256::new();
    hasher.update(&data);
    let hash = hasher.finalize();
    
    // Encode result
    let hash_b64 = base64::encode(&hash);
    
    Ok(json!({
        "hash": hash_b64
    }))
}
```

**Size**: ~20 lines  
**Effort**: ~30 minutes

---

## 🔄 How Songbird Uses BearDog

### Discovery (Same as TLS 1.3)

**Pattern**: Capability-based, runtime discovery

**File**: `crates/songbird-orchestrator/src/crypto/discovery.rs`

**Discovery Order**:
1. `CRYPTO_PROVIDER_SOCKET` (orchestrator-managed via biomeOS)
2. `CRYPTO_PROVIDER` (alternative env var)
3. `BEARDOG_CRYPTO_SOCKET` (compatibility)
4. XDG-compliant: `$XDG_RUNTIME_DIR/biomeos/beardog.sock`
5. Fallback: `/tmp/biomeos/beardog.sock`
6. TCP format: `tcp:host:port` (cross-platform)

### Client (Same as TLS 1.3)

**File**: `crates/songbird-orchestrator/src/crypto/beardog_crypto_client.rs`

**Usage**:
```rust
// Songbird code
let crypto = BeardogCryptoClient::discover().await?;

// Generate .onion identity
let (public_key, secret_key_id) = crypto.ed25519_generate_identity("onion_service").await?;

// Derive .onion address
let checksum_input = format!(".onion checksum{}{}", public_key, version);
let hash = crypto.sha3_256(&checksum_input, "onion_address_checksum").await?;
let checksum = &hash[..2];

// Session key exchange
let (our_public, our_secret_id) = crypto.x25519_generate_ephemeral("onion_handshake").await?;
let shared = crypto.x25519_derive_secret(&our_secret_id, &their_public).await?;

// Derive session keys (HKDF using existing HMAC-SHA256)
let session_keys = derive_keys_via_hkdf(&crypto, &shared, &nonce1, &nonce2).await?;

// Encrypt data
let ciphertext = crypto.chacha20_poly1305_encrypt(&plaintext, &key, &nonce, &[]).await?;
```

**Key Points**:
- Songbird has **ZERO crypto dependencies**
- All crypto via JSON-RPC to BearDog
- Platform-agnostic IPC (Unix/TCP/named pipes)

---

## 🌍 biomeOS Coordination

### Deployment Pattern (Same as TLS 1.3)

**File**: `deployment/graphs/tower_genome.toml`

**Deployment Sequence**:
```toml
# 1. Deploy BearDog first (security foundation)
[[nodes]]
name = "beardog"
primal = "beardog"
genome = "beardog.genome"
health_check = { endpoint = "/health", interval_secs = 10 }

# 2. Deploy Songbird (depends on BearDog)
[[nodes]]
name = "songbird"
primal = "songbird"
genome = "songbird.genome"
depends_on = ["beardog"]  # Wait for BearDog to be healthy

# Wire BearDog socket to Songbird via env var
env = [
    "CRYPTO_PROVIDER_SOCKET=${beardog_socket}",
]
```

### Lifecycle Management

**biomeOS Role**:
1. ✅ Start BearDog first
2. ✅ Wait for BearDog health check
3. ✅ Start Songbird with `CRYPTO_PROVIDER_SOCKET` env var
4. ✅ Monitor both primals
5. ✅ Restart coordination if either fails

**Songbird Role**:
1. ✅ Discover BearDog socket via env var
2. ✅ Establish JSON-RPC connection
3. ✅ Delegate all crypto operations
4. ✅ Report errors to biomeOS if crypto unavailable

**BearDog Role**:
1. ✅ Listen on Unix socket
2. ✅ Handle JSON-RPC requests
3. ✅ Return crypto results
4. ✅ Log all operations (audit trail)

---

## 🔐 Onion Service Crypto Flow

### 1. Generate .onion Identity

**Songbird → BearDog**:
```json
{
  "method": "beardog.crypto.ed25519_generate_identity",
  "params": {"purpose": "onion_service"}
}
```

**BearDog → Songbird**:
```json
{
  "result": {
    "public_key": "base64_32_bytes",
    "secret_key_id": "onion_identity_abc123"
  }
}
```

### 2. Derive .onion Address

**Songbird → BearDog**:
```json
{
  "method": "beardog.crypto.sha3_256",
  "params": {
    "data": "base64_encoded_checksum_input",
    "purpose": "onion_address_checksum"
  }
}
```

**BearDog → Songbird**:
```json
{
  "result": {
    "hash": "base64_32_bytes_sha3_hash"
  }
}
```

**Songbird** (local, no crypto):
- Extract first 2 bytes as checksum
- Assemble: `pubkey || checksum || version`
- Base32 encode
- Append `.onion`

### 3. Session Handshake

**Songbird → BearDog**:
```json
{
  "method": "beardog.crypto.x25519_generate_ephemeral",
  "params": {"purpose": "onion_session"}
}
```

**BearDog → Songbird**:
```json
{
  "result": {
    "public_key": "base64_32_bytes",
    "secret_key_id": "ephemeral_xyz789"
  }
}
```

**Songbird → BearDog**:
```json
{
  "method": "beardog.crypto.x25519_derive_secret",
  "params": {
    "our_secret_key_id": "ephemeral_xyz789",
    "their_public_key": "base64_32_bytes"
  }
}
```

**BearDog → Songbird**:
```json
{
  "result": {
    "shared_secret": "base64_32_bytes"
  }
}
```

### 4. Derive Session Keys (HKDF)

**Using existing `beardog.crypto.hmac_sha256`** (2 calls):

**Extract**:
```json
{
  "method": "beardog.crypto.hmac_sha256",
  "params": {
    "key": "base64_zeros_32_bytes",
    "data": "base64_shared_secret"
  }
}
```

**Expand** (client key):
```json
{
  "method": "beardog.crypto.hmac_sha256",
  "params": {
    "key": "base64_prk",
    "data": "base64_label_client_nonces_counter"
  }
}
```

**Expand** (server key):
```json
{
  "method": "beardog.crypto.hmac_sha256",
  "params": {
    "key": "base64_prk",
    "data": "base64_label_server_nonces_counter"
  }
}
```

### 5. Encrypt/Decrypt Data

**Using existing `beardog.crypto.chacha20_poly1305_encrypt/decrypt`**:

**Encrypt**:
```json
{
  "method": "beardog.crypto.chacha20_poly1305_encrypt",
  "params": {
    "plaintext": "base64_data",
    "key": "base64_session_key",
    "nonce": "base64_nonce_from_sequence",
    "aad": ""
  }
}
```

**Result**: ChaCha20-Poly1305 encrypted data with authentication

---

## 📋 Implementation Checklist

### BearDog Side

**File**: `beardog-crypto-service/src/json_rpc_handlers.rs`

- [ ] Add `sha3 = "0.10"` to `Cargo.toml`
- [ ] Implement `handle_sha3_256(params: Value) -> Result<Value>`
- [ ] Register handler in JSON-RPC router
- [ ] Add unit tests (3 tests):
  - [ ] Test SHA3-256 hash correctness
  - [ ] Test base64 encoding/decoding
  - [ ] Test error handling (invalid input)
- [ ] Update `BEARDOG_CRYPTO_API_SPEC.md` with SHA3-256 method

**Effort**: ~1 hour (very simple, only 1 new method)

### Songbird Side

**File**: `crates/songbird-orchestrator/src/crypto/beardog_crypto_client.rs`

- [ ] Add `sha3_256()` method to `BeardogCryptoClient`
- [ ] Update onion service to use BearDog delegation
- [ ] Remove direct crypto dependencies from `songbird-sovereign-onion/Cargo.toml`:
  - [ ] Remove `ed25519-dalek` (use BearDog)
  - [ ] Remove `x25519-dalek` (use BearDog)
  - [ ] Remove `chacha20poly1305` (use BearDog)
  - [ ] Remove `sha3` (use BearDog)
  - [ ] Remove `hmac` (use BearDog)
  - [ ] Remove `sha2` (use BearDog)
  - [ ] Keep: `sled`, `base32`, `serde`, `tokio` (non-crypto)
- [ ] Refactor `keys.rs`, `crypto.rs`, `address.rs` to use BearDog client
- [ ] Update tests to use mock BearDog (or real BearDog in integration tests)

**Effort**: ~4 hours (refactor existing code to use delegation)

### biomeOS Side

**File**: `deployment/graphs/sovereign_onion_genome.toml`

- [ ] Create deployment graph for onion service
- [ ] Ensure BearDog dependency declared
- [ ] Wire `CRYPTO_PROVIDER_SOCKET` env var
- [ ] Add health checks

**Effort**: ~30 minutes (copy from TLS 1.3 pattern)

---

## 🧪 Testing Strategy

### Unit Tests (BearDog)

**File**: `beardog-crypto-service/tests/sha3_tests.rs`

```rust
#[test]
fn test_sha3_256_hash() {
    // Test against known test vector
    let input = b"test";
    let expected = "36f028580bb02cc8272a9a020f4200e346e276ae664e45ee80745574e2f5ab80";
    let result = sha3_256(input);
    assert_eq!(hex::encode(result), expected);
}
```

### Integration Tests (Songbird)

**File**: `crates/songbird-sovereign-onion/tests/integration_with_beardog.rs`

```rust
#[tokio::test]
async fn test_onion_address_derivation_via_beardog() {
    // Start mock BearDog
    let beardog = MockBeardog::start().await;
    
    // Generate identity via BearDog
    let identity = OnionIdentity::generate_via_beardog(&beardog).await.unwrap();
    
    // Verify .onion address format
    assert!(identity.onion_address().ends_with(".onion"));
    assert_eq!(identity.onion_address().len(), 62);
}
```

### E2E Tests (biomeOS)

**File**: `tests/e2e/onion_service_e2e.rs`

```rust
#[tokio::test]
async fn test_onion_service_full_flow() {
    // 1. biomeOS starts BearDog
    // 2. biomeOS starts Songbird with env var
    // 3. Songbird creates onion service
    // 4. Verify crypto delegation works
    // 5. Test encrypted connection
}
```

---

## 🚀 Migration Path

### Phase 1: Add SHA3-256 to BearDog (1 hour)

**Owner**: BearDog Team

**Tasks**:
1. Add `sha3 = "0.10"` dependency
2. Implement `handle_sha3_256()` handler
3. Register in JSON-RPC router
4. Write 3 unit tests
5. Update API spec

**Deliverable**: BearDog supports SHA3-256 hashing

### Phase 2: Refactor Songbird Onion (4 hours)

**Owner**: Songbird Team

**Tasks**:
1. Add `sha3_256()` to `BeardogCryptoClient`
2. Refactor `OnionIdentity` to use BearDog for Ed25519
3. Refactor `address.rs` to use BearDog for SHA3-256
4. Refactor `keys.rs` to use BearDog for X25519
5. Refactor `crypto.rs` to use BearDog for ChaCha20-Poly1305
6. Remove direct crypto dependencies
7. Update tests

**Deliverable**: Songbird onion service 100% delegated to BearDog

### Phase 3: Integration Testing (2 hours)

**Owner**: Both Teams

**Tasks**:
1. Write integration tests
2. Test full onion service flow
3. Verify all crypto goes through BearDog
4. Performance benchmarks

**Deliverable**: Verified crypto delegation

### Phase 4: biomeOS Coordination (30 minutes)

**Owner**: biomeOS Team

**Tasks**:
1. Create deployment graph
2. Test lifecycle coordination
3. Verify env var wiring

**Deliverable**: biomeOS manages BearDog + Songbird for onion service

---

## 📊 Benefits

### Architecture

✅ **TRUE PRIMAL Compliance**: Crypto in BearDog, network in Songbird  
✅ **Separation of Concerns**: Clean boundaries  
✅ **Reusability**: BearDog crypto used by TLS, onion service, and future features  
✅ **Single Source of Truth**: All crypto in one place

### Security

✅ **Centralized Audit**: All crypto operations in BearDog  
✅ **Key Management**: BearDog owns all secret keys  
✅ **Audit Logging**: All crypto calls logged  
✅ **Rate Limiting**: BearDog can limit crypto operations per client

### Performance

✅ **Unix Socket IPC**: ~10 µs latency per call  
✅ **Batching**: Can batch multiple operations  
✅ **Caching**: BearDog can cache ephemeral keys  
✅ **Async**: Non-blocking crypto operations

### Maintainability

✅ **Single Crypto Codebase**: Updates in one place  
✅ **Version Control**: BearDog API versioning  
✅ **Testing**: Centralized crypto test suite  
✅ **Evolution**: Can swap crypto implementations without changing Songbird

---

## 📈 Comparison: Before → After

### Before (Onion Service with Direct Crypto)

```
songbird-sovereign-onion/
├── Cargo.toml (10 crypto dependencies)
├── src/
│   ├── address.rs (uses sha3 directly)
│   ├── keys.rs (uses ed25519-dalek, x25519-dalek)
│   ├── crypto.rs (uses chacha20poly1305)
```

**Issues**:
- ❌ Violates TRUE PRIMAL (crypto in network primal)
- ❌ Duplicates crypto primitives (also in BearDog)
- ❌ Audit complexity (crypto in multiple places)

### After (Onion Service with BearDog Delegation)

```
songbird-sovereign-onion/
├── Cargo.toml (ZERO crypto dependencies)
├── src/
│   ├── address.rs (calls beardog.crypto.sha3_256)
│   ├── keys.rs (calls beardog.crypto.ed25519_*, x25519_*)
│   ├── crypto.rs (calls beardog.crypto.chacha20_poly1305_*)
```

**Benefits**:
- ✅ TRUE PRIMAL compliant
- ✅ Zero crypto duplication
- ✅ Single audit surface (BearDog only)
- ✅ Same pattern as TLS 1.3

---

## 🎯 Summary

### What BearDog Needs to Add

**1 new method**: `beardog.crypto.sha3_256`  
**Effort**: ~1 hour  
**Dependencies**: `sha3 = "0.10"` (Pure Rust, RustCrypto)  
**Pattern**: Exactly like existing `blake3_hash` method

### What Songbird Needs to Change

**Refactor**: Use BearDog delegation instead of direct crypto  
**Effort**: ~4 hours  
**Pattern**: Exactly like TLS 1.3 crypto delegation  
**Result**: ZERO crypto dependencies in Songbird

### What biomeOS Needs to Do

**Coordinate**: BearDog + Songbird lifecycle  
**Effort**: ~30 minutes  
**Pattern**: Exactly like TLS 1.3 coordination  
**Result**: Seamless crypto delegation

---

## 📚 Reference Documents

1. **BearDog Crypto API**: `docs/architecture/BEARDOG_CRYPTO_API_SPEC.md`
2. **TLS Crypto Delegation**: `crates/songbird-tls/src/crypto.rs`
3. **BearDog Client**: `crates/songbird-orchestrator/src/crypto/beardog_crypto_client.rs`
4. **Primal Coordination**: `specs/PRIMAL_COORDINATION_ARCHITECTURE.md`
5. **Deployment Graphs**: `deployment/graphs/tower_genome.toml`

---

## ✅ Next Steps

1. **BearDog Team**: Implement `sha3_256` method (~1 hour)
2. **Songbird Team**: Refactor onion service for BearDog delegation (~4 hours)
3. **Integration**: Test crypto delegation (~2 hours)
4. **biomeOS**: Add deployment coordination (~30 minutes)

**Total Effort**: ~8 hours  
**Timeline**: 1-2 days  
**Result**: TRUE PRIMAL architecture for onion service

---

**Handoff Complete**: February 6, 2026  
**Pattern**: Same as TLS 1.3 (proven, production-ready)  
**Status**: Ready for implementation

🐻🐕 **BearDog + Songbird** | 🧬 **TRUE PRIMAL** | ✨ **Crypto Delegation**

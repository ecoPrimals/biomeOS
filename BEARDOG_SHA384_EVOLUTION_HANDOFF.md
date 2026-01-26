# 🐻 BearDog SHA-384 Evolution Handoff - January 26, 2026

## 🎯 Goal: Enable 100% TLS 1.3 Validation

**Current Status**: 95% TLS success rate (blocked by SHA-384 requirement)
**Blocker**: Cipher suite 0x1302 (TLS_AES_256_GCM_SHA384) requires SHA-384 for HKDF

---

## 📊 Root Cause Analysis

### The Problem

Some servers **require** cipher suite 0x1302 (TLS_AES_256_GCM_SHA384).
This cipher uses **SHA-384** for:
1. Transcript hashing
2. HKDF key derivation
3. HMAC operations

### Current Implementation (SHA-256 HARDCODED)

**BearDog** (`key_derivation.rs`):
```rust
// Line ~150: HARDCODED to SHA-256!
let hkdf = Hkdf::<Sha256>::new(Some(&salt), &pre_master_secret);
```

**Songbird** (`transcript.rs`):
```rust
// Line ~50: HARDCODED to SHA-256!
let mut hasher = Sha256::new();
```

### Required Implementation (Cipher-Aware)

**BearDog** needs to select hash based on cipher suite:
```rust
let hkdf = match cipher_suite {
    0x1301 => Hkdf::<Sha256>::new(Some(&salt), &pre_master_secret),
    0x1302 => Hkdf::<Sha384>::new(Some(&salt), &pre_master_secret),
    0x1303 => Hkdf::<Sha256>::new(Some(&salt), &pre_master_secret),
    _ => return Err("Unsupported cipher suite"),
};
```

---

## 🔧 BearDog Evolution Required

### Phase 1: Add `crypto.hash_for_cipher` Method (NEW)

**Purpose**: Songbird can request cipher-specific hashing without knowing implementation details.

```json
// Request
{
  "jsonrpc": "2.0",
  "method": "crypto.hash_for_cipher",
  "params": {
    "data": "base64-encoded-data",
    "cipher_suite": 4866  // 0x1302
  },
  "id": 1
}

// Response
{
  "result": {
    "hash": "base64-encoded-48-byte-sha384-hash"
  }
}
```

**Implementation** (`crypto_handlers_hashing.rs`):
```rust
pub fn handle_hash_for_cipher(params: &Value) -> Result<Value, BearDogError> {
    let data = extract_base64(params, "data")?;
    let cipher_suite = params.get("cipher_suite")
        .and_then(|v| v.as_u64())
        .ok_or("Missing cipher_suite")? as u16;
    
    let hash = match cipher_suite {
        0x1301 | 0x1303 => sha2::Sha256::digest(&data).to_vec(),
        0x1302 => sha2::Sha384::digest(&data).to_vec(),
        _ => return Err("Unsupported cipher suite"),
    };
    
    Ok(json!({ "hash": BASE64_STANDARD.encode(&hash) }))
}
```

### Phase 2: Update `tls.derive_handshake_secrets` (MODIFY)

**Current**: Uses HKDF-SHA256 for all cipher suites
**Required**: Use HKDF-SHA384 when cipher_suite = 0x1302

**Files**: `crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto/tls/key_derivation.rs`

```rust
// Update handle_tls_derive_handshake_secrets:

// Select HKDF based on cipher suite
let (early_secret, handshake_secret, derived_secrets) = match cipher_suite {
    0x1301 | 0x1303 => derive_with_sha256(&shared_secret, &transcript_hash),
    0x1302 => derive_with_sha384(&shared_secret, &transcript_hash),
    _ => return Err("Unsupported cipher suite"),
};

// Helper for SHA-384 derivation
fn derive_with_sha384(shared_secret: &[u8], transcript_hash: &[u8]) -> Result<...> {
    let hkdf = Hkdf::<Sha384>::new(Some(b""), shared_secret);
    // ... rest of derivation using SHA-384
}
```

### Phase 3: Update `tls.derive_application_secrets` (MODIFY)

Same pattern as Phase 2 - select HKDF hash based on cipher_suite parameter.

---

## 📝 Graph Changes for capability.call

Add to `tower_atomic_bootstrap.toml`:

```toml
[nodes.beardog.capabilities_provided]
# Existing
sha256 = "crypto.sha256"
sha384 = "crypto.sha384"

# NEW: Cipher-aware hashing for TLS
hash_for_cipher = "crypto.hash_for_cipher"
```

**Songbird can then call**:
```json
{
  "method": "capability.call",
  "params": {
    "capability": "crypto",
    "operation": "hash_for_cipher",
    "args": {
      "data": "...",
      "cipher_suite": 4866
    }
  }
}
```

---

## 🔗 Coordination with Songbird

### Songbird Changes (After BearDog Evolution)

**File**: `crates/songbird-http-client/src/tls/handshake_refactored/transcript.rs`

```rust
// Change from:
pub(super) fn compute_transcript_hash(&self) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(&self.transcript);
    hasher.finalize().to_vec()
}

// To:
pub(super) async fn compute_transcript_hash(&self, cipher_suite: u16) -> Result<Vec<u8>> {
    // Use BearDog's cipher-aware hashing
    self.crypto.hash_for_cipher(&self.transcript, cipher_suite).await
}
```

---

## 📋 Implementation Checklist

### BearDog P0 (Required for 100% TLS)

- [ ] Add `crypto.hash_for_cipher` method
- [ ] Update `tls.derive_handshake_secrets` to use cipher-specific HKDF
- [ ] Update `tls.derive_application_secrets` to use cipher-specific HKDF
- [ ] Add unit tests for SHA-384 derivation
- [ ] Test with 0x1302 cipher suite

### Graph Changes (biomeOS)

- [ ] Add `hash_for_cipher` mapping to `tower_atomic_bootstrap.toml`
- [ ] Validate capability routing

### Songbird (After BearDog)

- [ ] Update transcript hashing to use `hash_for_cipher`
- [ ] Pass cipher_suite to all derivation calls
- [ ] Test against servers requiring 0x1302

---

## 🎯 Success Criteria

| Test | Current | Target |
|------|---------|--------|
| TLS validation success | 95% | 100% |
| Cipher 0x1301 support | ✅ | ✅ |
| Cipher 0x1302 support | ❌ | ✅ |
| Cipher 0x1303 support | ✅ | ✅ |

---

## 📁 Files to Modify

### BearDog

```
beardog/crates/beardog-tunnel/src/unix_socket_ipc/
├── handlers/
│   ├── crypto_handler.rs           # Add hash_for_cipher routing
│   └── crypto/
│       ├── hash.rs                  # (Optional) Add hash_for_cipher
│       └── tls/
│           └── key_derivation.rs   # ⭐ MAIN CHANGES HERE
└── crypto_handlers_hashing.rs       # Add handle_hash_for_cipher
```

### biomeOS

```
biomeOS/graphs/tower_atomic_bootstrap.toml  # Add hash_for_cipher mapping
```

### Songbird (After BearDog)

```
songbird/crates/songbird-http-client/src/
├── tls/handshake_refactored/
│   └── transcript.rs               # Use cipher-aware hashing
└── crypto/
    └── capability.rs               # Add hash_for_cipher to trait
```

---

## 🚀 Estimated Effort

| Task | Effort | Priority |
|------|--------|----------|
| BearDog: hash_for_cipher | 2 hours | P0 |
| BearDog: HKDF-SHA384 for key derivation | 4 hours | P0 |
| biomeOS: Graph update | 15 min | P0 |
| Songbird: Use cipher-aware hashing | 2 hours | P1 |
| Testing & validation | 2 hours | P0 |

**Total**: ~10 hours for 100% TLS validation

---

## 🔬 Why This Architecture?

### TRUE PRIMAL Pattern

By having BearDog provide `hash_for_cipher`:
1. **Songbird doesn't know** about SHA-256 vs SHA-384 internals
2. **BearDog owns** all crypto decisions
3. **Graph defines** the semantic mapping
4. **capability.call** routes transparently

### Future-Proof

When new cipher suites are added:
1. BearDog adds support (SHA-512, etc.)
2. Graph maps new capabilities
3. Songbird unchanged - just passes cipher_suite

---

## 📞 Contact

- **BearDog**: `/home/eastgate/Development/ecoPrimals/phase1/beardog`
- **Songbird**: `/home/eastgate/Development/ecoPrimals/phase1/songbird`
- **biomeOS**: This repository
- **Standards**: `/home/eastgate/Development/ecoPrimals/wateringHole/`

---

**Created**: January 26, 2026  
**Status**: Ready for BearDog Evolution  
**Impact**: 95% → 100% TLS Validation 🎯


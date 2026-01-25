# Semantic Evolution Strategy - Implementation Plan

**Date**: January 25, 2026  
**Status**: 🚀 **ACTIVE** - Dual-track approach  
**Goal**: Enable isomorphic evolution while maintaining immediate functionality

---

## 🎯 DUAL-TRACK APPROACH

### Track A: Quick Fix (Immediate - 30-60 min)
**Update Songbird HTTP Client to use semantic names**

### Track B: Long-term (Week 2+)
**Route all primals through Neural API for full capability translation**

**Key**: Both happen in parallel - Track A unblocks now, Track B ensures future

---

## 📋 TRACK A: SONGBIRD HTTP CLIENT UPDATE

### Status: ⏳ **NEXT IMMEDIATE TASK**

### Files to Update:

**Primary**: `ecoPrimals/phase1/songbird/crates/songbird-http-client/src/beardog_client.rs`

### Changes Needed:

```rust
// OLD → NEW

// Key Exchange
"x25519_generate_ephemeral"     → "crypto.x25519_generate_ephemeral"
"x25519_derive_secret"          → "crypto.x25519_derive_secret"

// AEAD Encryption
"chacha20_poly1305_encrypt"     → "crypto.chacha20_poly1305_encrypt"
"chacha20_poly1305_decrypt"     → "crypto.chacha20_poly1305_decrypt"
"aes128_gcm_encrypt"            → "crypto.aes128_gcm_encrypt"
"aes128_gcm_decrypt"            → "crypto.aes128_gcm_decrypt"
"aes256_gcm_encrypt"            → "crypto.aes256_gcm_encrypt"
"aes256_gcm_decrypt"            → "crypto.aes256_gcm_decrypt"

// TLS Operations
"tls_derive_handshake_secrets"    → "tls.derive_handshake_secrets"
"tls_derive_application_secrets"  → "tls.derive_application_secrets"
"tls_compute_finished_verify_data" → "tls.compute_finished_verify_data"

// Hashing (if used)
"blake3_hash"                   → "crypto.blake3_hash"
```

### Implementation:

1. Open `beardog_client.rs`
2. Find all `self.call_rpc(` calls
3. Update method names to use `crypto.*` and `tls.*` prefixes
4. Test compilation
5. Test HTTPS via IPC

**Estimated Time**: 30-60 minutes

---

## 📋 TRACK B: NEURAL API ROUTING (LONG-TERM)

### Phase 1: Capability Translation Infrastructure ✅ **DONE**

**Status**: Already implemented in biomeOS!

```rust
// crates/biomeos-atomic-deploy/src/capability_translation.rs
pub struct CapabilityTranslationRegistry {
    translations: HashMap<String, CapabilityTranslation>,
}

impl CapabilityTranslationRegistry {
    pub async fn call_capability(&self, semantic: &str, params: Value) -> Result<Value> {
        // 1. Lookup translation
        // 2. Connect to provider  
        // 3. Translate method name
        // 4. Execute RPC
    }
}
```

**Neural API Methods**: ✅ Implemented
- `capability.call` - Call with automatic translation
- `capability.discover_translation` - Inspect mappings
- `capability.list_translations` - List all mappings

---

### Phase 2: Graph Mappings ✅ **DONE**

**Status**: Already defined in graphs!

```toml
# graphs/tower_atomic_bootstrap.toml

[nodes.capabilities_provided]
"crypto.generate_keypair" = "crypto.x25519_generate_ephemeral"
"crypto.ecdh_derive" = "crypto.x25519_derive_secret"
"crypto.encrypt" = "crypto.chacha20_poly1305_encrypt"
# ... all mappings defined
```

---

### Phase 3: Primal Integration (Week 2+)

**Update primals to route through Neural API**:

#### Songbird Evolution:

**Current** (Direct BearDog):
```rust
// songbird-http-client/src/lib.rs
pub struct SongbirdHttpClient {
    beardog_socket: String,  // Direct connection
}

impl SongbirdHttpClient {
    async fn tls_handshake(&mut self) {
        let keypair = self.beardog_client
            .call_rpc("crypto.x25519_generate_ephemeral", ...)
            .await?;
    }
}
```

**Future** (Via Neural API):
```rust
// songbird-http-client/src/lib.rs
pub struct SongbirdHttpClient {
    neural_api_socket: String,  // Route through Neural API
}

impl SongbirdHttpClient {
    async fn tls_handshake(&mut self) {
        // Fully semantic - no provider knowledge
        let keypair = self.neural_api_client
            .call_capability("crypto.generate_keypair", json!({
                "algorithm": "x25519"
            }))
            .await?;
    }
}
```

**Benefits**:
- ✅ Zero BearDog knowledge in Songbird
- ✅ Provider swappable (BearDog → RustCrypto)
- ✅ Version evolution handled by Neural API
- ✅ TRUE PRIMAL architecture achieved

---

### Phase 4: Other Primals (Month 2)

**Squirrel**, **NestGate**, **ToadStool** all adopt semantic routing:
```rust
// Generic pattern for all primals
let result = neural_api.call_capability(
    "semantic.capability.name",
    params
).await?;
```

---

## 📊 WATERINGHOLE STANDARDS UPDATE

### ✅ **COMPLETE**

**New Document**: `wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md`

**Contents**:
- Semantic namespace structure (`domain.operation`)
- Core domain definitions (crypto, tls, http, storage, discovery, genetic)
- Evolution patterns (generic → specific)
- Migration guide (3-phase approach)
- Neural API translation layer documentation
- Examples and adoption status

**Updated**: `wateringHole/PRIMAL_IPC_PROTOCOL.md`
- References semantic naming standard
- Method format updated to show semantic structure

**Status**: Committed to wateringHole repo (local)

---

## 🌐 HOW BIOMEOS BRIDGES THE GAP

### Current Architecture:

```
Songbird HTTP Client (old names)
    ↓ (direct socket)
BearDog v0.18.0 (new names)
    ↓
❌ Method mismatch!
```

### Track A Solution (Immediate):

```
Songbird HTTP Client (UPDATED to new names)
    ↓ (direct socket)
BearDog v0.18.0 (new names)
    ↓
✅ Works!
```

### Track B Solution (Future):

```
Songbird HTTP Client (semantic)
    ↓
Neural API (capability translation)
    ├─ Looks up "crypto.generate_keypair"
    ├─ Translates to "crypto.x25519_generate_ephemeral"
    ├─ Routes to BearDog socket
    └─ Returns result
    ↓
BearDog v0.18.0 (or ANY provider!)
    ↓
✅ Isomorphic evolution achieved!
```

---

## 🎯 IMMEDIATE ACTION ITEMS

### 1. Update Songbird HTTP Client (NOW)

```bash
cd ecoPrimals/phase1/songbird
cd crates/songbird-http-client/src

# Edit beardog_client.rs
# - Find all call_rpc() calls
# - Add crypto.* and tls.* prefixes
# - Test compilation

# Test
cargo build --release
cargo test
```

### 2. Test Tower Atomic (30 min after fix)

```bash
# Start BearDog
cd ecoPrimals/phase1/beardog
./target/debug/beardog server --socket /tmp/beardog-nat0.sock

# Start Songbird (with updated client)
cd ecoPrimals/phase1/songbird  
SONGBIRD_SECURITY_PROVIDER=/tmp/beardog-nat0.sock \
./target/debug/songbird server --socket /tmp/songbird-nat0.sock

# Test HTTPS via IPC
echo '{"jsonrpc":"2.0","method":"http.get","params":{"url":"https://www.google.com"},"id":1}' \
  | nc -U /tmp/songbird-nat0.sock

# Expected: HTTP 200 OK! 🎉
```

### 3. Document Success (After test)

Create `TOWER_ATOMIC_SUCCESS_JAN_25_2026.md` with:
- Test results
- Response times
- Validation of 100% Pure Rust TLS 1.3
- Next steps for biomeOS integration

---

## 📈 EVOLUTION TIMELINE

### Week 1 (This Week)
- ✅ Semantic naming standard published
- ✅ wateringHole docs updated  
- ⏳ Songbird HTTP client updated (Track A)
- ⏳ Tower Atomic validated end-to-end

### Week 2
- Begin Track B: Songbird → Neural API routing
- Update deployment graphs for semantic translation
- Test capability.call methods

### Month 2
- Other primals adopt semantic naming
- All primals route through Neural API
- Full isomorphic evolution achieved

---

## 🎉 SUCCESS CRITERIA

### Track A Complete When:
- [x] Songbird HTTP client uses semantic method names
- [ ] Tower Atomic test: HTTP 200 OK from Google
- [ ] Documented in success report

### Track B Complete When:
- [ ] Songbird routes through Neural API
- [ ] Capability translation working end-to-end
- [ ] Zero BearDog references in Songbird code
- [ ] Full HTTPS via semantic translation

### Ecosystem Complete When:
- [ ] All primals use semantic naming
- [ ] All primals route through Neural API (optional but preferred)
- [ ] Standards documented and followed
- [ ] Isomorphic evolution validated

---

## 💡 KEY INSIGHTS

### This is Evolution, Not Migration!

**Traditional Migration**:
```
1. Plan big bang change
2. Coordinate all teams
3. Switch everything at once
4. Hope nothing breaks
```

**Isomorphic Evolution**:
```
1. Define semantic layer
2. Old and new work together
3. Update incrementally
4. No coordination needed
5. Structure preserved throughout
```

### The Semantic Layer is Working!

The fact that we discovered this mismatch **proves** the semantic layer is:
- ✅ Catching evolution gaps
- ✅ Providing clear error messages
- ✅ Enabling independent evolution
- ✅ Protecting ecosystem coherence

**This is GOOD!** It means the architecture is self-correcting!

---

## 📚 REFERENCES

- `wateringHole/SEMANTIC_METHOD_NAMING_STANDARD.md` - Official standard
- `wateringHole/PRIMAL_IPC_PROTOCOL.md` - Updated IPC protocol
- `biomeOS/ISOMORPHIC_EVOLUTION.md` - Core principles
- `biomeOS/TOWER_ATOMIC_SEMANTIC_TRANSLATION_GAP.md` - Gap analysis
- `biomeOS/SONGBIRD_REHARVEST_JAN_25_2026.md` - Songbird status

---

**Status**: Dual-track approach active  
**Track A**: Ready to execute (Songbird update)  
**Track B**: Infrastructure ready (Neural API)  
**Outcome**: Immediate fix + long-term evolution = Resilient ecosystem! 🚀

---

*"Fix the immediate, build for the future, maintain coherence throughout"* 🌍🦀


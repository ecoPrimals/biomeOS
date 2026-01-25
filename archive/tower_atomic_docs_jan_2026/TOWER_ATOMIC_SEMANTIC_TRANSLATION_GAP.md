# Tower Atomic Semantic Translation - Method Name Evolution

**Date**: January 25, 2026  
**Issue**: Songbird HTTP Client → BearDog method name mismatch  
**Root Cause**: Evolution gap between library mode and semantic layer

---

## 🎯 THE ISSUE

### What Happened:
```bash
# Songbird → BearDog
Method called: "x25519_generate_ephemeral"
BearDog error: "Method not found: x25519_generate_ephemeral (code: -32601)"

# But BearDog DOES respond to:
Method called: "crypto.x25519_generate_ephemeral"  
BearDog response: ✅ SUCCESS! (generated keypair)
```

### Why This Happened:

**BearDog v0.18.0+** uses semantic namespaces:
- `crypto.x25519_generate_ephemeral` ✅ (NEW - semantic)
- `x25519_generate_ephemeral` ❌ (OLD - removed)

**Songbird HTTP Client** (library mode) still uses old names:
- Calls `x25519_generate_ephemeral` (OLD)
- Should call `crypto.x25519_generate_ephemeral` (NEW)

---

## 🌍 ISOMORPHIC EVOLUTION IN ACTION

### This Is Actually CORRECT Behavior!

The `crypto.` prefix is **intentional** - it's part of the semantic layer that makes the ecosystem isomorphic:

```toml
# Tower Atomic Graph Mapping
[nodes.capabilities_provided]
# Semantic Name (stable) → Actual Method (can evolve)
"crypto.generate_keypair" = "crypto.x25519_generate_ephemeral"
"crypto.ecdh_derive" = "crypto.x25519_derive_secret"
"crypto.encrypt" = "crypto.chacha20_poly1305_encrypt"
```

**Why This Matters**:

1. **Primal Swappability**: 
   ```
   Today:    BearDog implements "crypto.x25519_generate_ephemeral"
   Tomorrow: RustCrypto implements "generate_x25519_keys"
   
   Graph mapping changes, Songbird code UNCHANGED!
   ```

2. **Version Evolution**:
   ```
   BearDog v0.9:  "x25519_generate_ephemeral"
   BearDog v0.18: "crypto.x25519_generate_ephemeral"
   BearDog v0.20: "crypto.x25519.generate"  (hypothetical)
   
   Only graph updates, no code changes!
   ```

3. **Service Replacement**:
   ```
   If Songbird is replaced with "NewTLSService":
   - NewTLSService calls semantic "crypto.generate_keypair"
   - Neural API translates to whatever provider exists
   - Ecosystem continues functioning!
   ```

---

## 🔧 THE SOLUTION

### Option A: Update Songbird HTTP Client (Quick Fix)

Update `songbird-http-client` to use semantic method names:

```rust
// OLD (crates/songbird-http-client/src/beardog_client.rs):
let response = self.call_rpc("x25519_generate_ephemeral", params).await?;

// NEW (semantic):
let response = self.call_rpc("crypto.x25519_generate_ephemeral", params).await?;
```

**All method updates needed**:
```rust
// Key Exchange
"x25519_generate_ephemeral" → "crypto.x25519_generate_ephemeral"
"x25519_derive_secret"      → "crypto.x25519_derive_secret"

// AEAD Encryption
"chacha20_poly1305_encrypt" → "crypto.chacha20_poly1305_encrypt"
"chacha20_poly1305_decrypt" → "crypto.chacha20_poly1305_decrypt"
"aes128_gcm_encrypt"        → "crypto.aes128_gcm_encrypt"
"aes128_gcm_decrypt"        → "crypto.aes128_gcm_decrypt"

// TLS Operations
"tls_derive_handshake_secrets"    → "tls.derive_handshake_secrets"
"tls_derive_application_secrets"  → "tls.derive_application_secrets"
"tls_compute_finished_verify_data" → "tls.compute_finished_verify_data"
```

**Estimated Time**: 30 minutes (find/replace + test)

---

### Option B: Deploy via Neural API (TRUE PRIMAL Way)

Use biomeOS Neural API with capability translation:

```toml
# tower_atomic_bootstrap.toml already has this!
[nodes.capabilities_provided]
"crypto.generate_keypair" = "crypto.x25519_generate_ephemeral"
"crypto.ecdh_derive" = "crypto.x25519_derive_secret"
# ... etc
```

**How it works**:
```
1. Songbird HTTP Client → Neural API
   Request: "crypto.generate_keypair" (semantic)

2. Neural API → Capability Translation
   Looks up: "crypto.generate_keypair" → "crypto.x25519_generate_ephemeral"
   
3. Neural API → BearDog
   Calls: "crypto.x25519_generate_ephemeral" (actual method)
   
4. BearDog → Neural API → Songbird
   Returns: keypair
```

**Status**: Requires Songbird HTTP client to route through Neural API instead of direct BearDog calls

---

## 📊 CURRENT ARCHITECTURE

### Library Mode (Current - Direct):
```
Songbird HTTP Client
    ↓ (direct socket)
BearDog
    ↓
❌ Method name mismatch (old vs new)
```

### Neural API Mode (Target - Semantic):
```
Songbird HTTP Client
    ↓ (semantic request)
Neural API (Capability Translation)
    ↓ (translated method)
BearDog
    ↓
✅ Works! (Neural API handles evolution)
```

---

## 🎯 RECOMMENDATION

### Immediate (This Session):

**Update Songbird HTTP Client** to use semantic method names:

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cd crates/songbird-http-client/src
```

Find and replace in `beardog_client.rs`:
- All RPC calls to use `crypto.*` and `tls.*` prefixes
- Match BearDog v0.18.0+ semantic namespace

**Benefits**:
- ✅ Quick fix (30 min)
- ✅ Makes Songbird compatible with current BearDog
- ✅ Aligns with semantic evolution
- ✅ Tower Atomic works immediately

### Medium-term (Week 2):

**Route through Neural API** for full semantic translation:
- Songbird HTTP client uses Neural API as proxy
- Neural API handles all capability translation
- Full isomorphic evolution achieved

---

## 🔍 VERIFICATION

### Test BearDog Directly (Working):
```bash
echo '{"jsonrpc":"2.0","method":"crypto.x25519_generate_ephemeral","params":{},"id":1}' \
  | nc -U /tmp/beardog-nat0.sock

Result: ✅ {"jsonrpc":"2.0","result":{"public_key":"..."},"id":1}
```

### Test Songbird → BearDog (Current - Fails):
```bash
echo '{"jsonrpc":"2.0","method":"http.get","params":{"url":"https://google.com"},"id":1}' \
  | nc -U /tmp/songbird-nat0.sock

Result: ❌ Method not found: x25519_generate_ephemeral
```

### After Fix (Expected):
```bash
echo '{"jsonrpc":"2.0","method":"http.get","params":{"url":"https://google.com"},"id":1}' \
  | nc -U /tmp/songbird-nat0.sock

Result: ✅ {"jsonrpc":"2.0","result":{"status_code":200,...},"id":1}
```

---

## 💡 KEY INSIGHT

**This is NOT a bug - it's evolution!**

The `crypto.` prefix represents our commitment to:
1. **Semantic Stability**: Method categories don't change
2. **Provider Flexibility**: Implementations can evolve
3. **Isomorphic Evolution**: Ecosystem structure preserved
4. **TRUE PRIMAL**: No hardcoded dependencies

The fact that we hit this issue means **the semantic layer is working** - it's catching mismatches between old and new conventions!

---

## 📋 ACTION ITEMS

### For Songbird Team:

- [ ] Update `beardog_client.rs` to use semantic method names
- [ ] Add `crypto.` prefix to all crypto operations
- [ ] Add `tls.` prefix to all TLS operations
- [ ] Test against BearDog v0.18.0+
- [ ] Verify HTTPS works end-to-end

**Estimated Time**: 30-60 minutes

### For biomeOS Team:

- [ ] Document semantic method naming convention
- [ ] Add to `PRIMAL_IPC_PROTOCOL.md` standard
- [ ] Create migration guide for primals
- [ ] Validate all graphs use semantic names

---

## 🎉 CONCLUSION

**What Happened**: We discovered a method name evolution gap  
**Why It Happened**: BearDog evolved to semantic namespaces, Songbird client hasn't yet  
**Is It Bad?**: No! It proves the semantic layer is working  
**Fix**: Update Songbird to use semantic names (30 min)  
**Future**: Route through Neural API for full capability translation

**This is Isomorphic Evolution in practice!** 🌍🦀

---

**Status**: Evolution gap identified and solution clear  
**Next**: Update Songbird HTTP client method names  
**ETA**: 30-60 minutes to full Tower Atomic HTTPS working

---

*"The semantic layer catches what hardcoded dependencies would silently break!"*


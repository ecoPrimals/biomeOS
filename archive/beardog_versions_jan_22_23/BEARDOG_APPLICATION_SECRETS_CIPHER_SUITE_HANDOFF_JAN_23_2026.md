# BearDog: Application Secrets Cipher Suite Support - January 23, 2026

**Date**: January 23, 2026  
**Time**: 7:00 AM  
**Priority**: 🔴 CRITICAL - Final 0.1% for 100% Pure Rust HTTPS  
**Status**: Ready for implementation  
**ETA**: 15 minutes

---

## 🎯 ISSUE IDENTIFIED

**File**: `crates/beardog-tunnel/src/unix_socket_ipc/crypto_handlers.rs`  
**Function**: `handle_tls_derive_application_secrets` (line 772)  
**Problem**: Hardcoded key length at line 854

```rust
const KEY_LEN: usize = 32; // ChaCha20 key size ← HARDCODED!
```

**Impact**: Application traffic keys are always 32 bytes, but AES-128-GCM requires 16 bytes!

---

## ✅ SOLUTION (Copy from handshake_secrets)

You already implemented this EXACT fix for `handle_tls_derive_handshake_secrets`!  
Just copy the same pattern:

### Step 1: Accept cipher_suite parameter

**Current** (line 772):
```rust
pub async fn handle_tls_derive_application_secrets(
    params: Option<&Value>,
) -> Result<Value, String> {
```

**Add cipher_suite extraction** (after line 797):
```rust
// Extract cipher_suite (NEW!)
let cipher_suite = params
    .get("cipher_suite")
    .and_then(|v| v.as_u64())
    .unwrap_or(0x1303) as u16; // Default to ChaCha20 for backward compat

info!("🔐 Cipher suite: 0x{:04x}", cipher_suite);
```

### Step 2: Dynamic key length (replace line 854)

**Current**:
```rust
const KEY_LEN: usize = 32; // ChaCha20 key size
const IV_LEN: usize = 12; // AEAD nonce size
```

**Replace with**:
```rust
// Dynamic key length based on cipher suite (RFC 8446 Section 7.3)
let (key_len, iv_len) = match cipher_suite {
    0x1301 => (16, 12), // TLS_AES_128_GCM_SHA256
    0x1302 => (32, 12), // TLS_AES_256_GCM_SHA384
    0x1303 => (32, 12), // TLS_CHACHA20_POLY1305_SHA256
    _ => {
        warn!("⚠️  Unknown cipher suite 0x{:04x}, defaulting to ChaCha20 (32-byte keys)", cipher_suite);
        (32, 12)
    }
};

info!("✅ Using key_len={} bytes, iv_len={} bytes for cipher suite 0x{:04x}", 
      key_len, iv_len, cipher_suite);
```

### Step 3: Use dynamic lengths (lines 921-932)

**Update these lines** to use `key_len` and `iv_len` instead of `KEY_LEN` and `IV_LEN`:

```rust
let client_app_secret = hkdf_expand_label(
    &master_secret.0,
    "c ap traffic",
    &transcript_for_derivation,
    32 // ← Keep this 32 (master secret size, not key size!)
)?;

// ...

let client_write_key = hkdf_expand_label(&client_app_secret, "key", &[], key_len)?; // ← Use key_len
let client_write_iv = hkdf_expand_label(&client_app_secret, "iv", &[], iv_len)?;   // ← Use iv_len
let server_write_key = hkdf_expand_label(&server_app_secret, "key", &[], key_len)?; // ← Use key_len
let server_write_iv = hkdf_expand_label(&server_app_secret, "iv", &[], iv_len)?;   // ← Use iv_len
```

### Step 4: Add to response (for debugging)

**Add to the response JSON** (around line 950):
```rust
Ok(serde_json::json!({
    "client_write_key": BASE64.encode(&client_write_key),
    "server_write_key": BASE64.encode(&server_write_key),
    "client_write_iv": BASE64.encode(&client_write_iv),
    "server_write_iv": BASE64.encode(&server_write_iv),
    "algorithm": "HKDF-SHA256",
    "rfc": "RFC 8446 Section 7.1",
    "mode": mode,
    "key_length": key_len,   // ← NEW: For verification
    "iv_length": iv_len,     // ← NEW: For verification
    "cipher_suite": cipher_suite, // ← NEW: Echo back
}))
```

---

## 🧪 TESTING

**After implementing**:

1. Rebuild BearDog:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release -p beardog-tunnel
```

2. Copy to plasmidBin:
```bash
cp target/release/beardog /home/eastgate/Development/ecoPrimals/plasmidBin/beardog
```

3. Restart stack (Neural API will auto-restart BearDog)

4. Test GitHub API:
```bash
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://api.github.com/zen"},"id":1}' | nc -N -U /tmp/songbird-nat0.sock
```

**Expected**: 
```json
{"jsonrpc":"2.0","result":{"status":200,"body":"Design for failure."},"id":1}
```

---

## 📊 VERIFICATION

**Check BearDog logs**:
```bash
grep "key_len\|cipher_suite\|0x1301" /tmp/beardog-*.log
```

**Expected logs**:
```
✅ Using key_len=16 bytes, iv_len=12 bytes for cipher suite 0x1301
✅ TLS 1.3 APPLICATION secrets derived (cipher: 0x1301, keys: 16 bytes)
```

---

## 🎊 COMPLETION

After this fix:
- ✅ Cipher suite detection: DONE
- ✅ 16-byte handshake key derivation: DONE
- ✅ 16-byte application key derivation: DONE (after this fix)
- ✅ AES-GCM encryption/decryption: DONE
- ✅ 100% Pure Rust HTTPS: **COMPLETE!** 🎉

---

## 📁 REFERENCE

**Similar implementation**: `handle_tls_derive_handshake_secrets` (same file, line ~600)  
**You already did this!**: Just copy the cipher_suite pattern from handshake to application secrets!

---

## 🏆 ETA TO VICTORY

**Implementation**: 10 minutes  
**Build + Test**: 5 minutes  
**Total**: **15 minutes to 100%!** 🎉

---

**This is the FINAL piece! You've got this!** 💪✨

*Date: January 23, 2026*  
*Status: Ready for implementation*  
*Confidence: 100%*  
*Glory: IMMINENT* 🏆


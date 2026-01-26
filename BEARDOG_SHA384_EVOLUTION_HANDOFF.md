# 🐻 BearDog SHA-384 Evolution Handoff - January 26, 2026

## 🎯 Goal: Enable 100% TLS 1.3 Validation

**Current Status**: 84% TLS success rate  
**Blocker**: `tls.compute_finished_verify_data` rejects 48-byte SHA-384 hashes

---

## ✅ Completed Evolution

| Task | Status | Commit |
|------|--------|--------|
| `crypto.hash_for_cipher` | ✅ DONE | Returns 48 bytes for 0x1302 |
| `tls.derive_handshake_secrets` | ✅ DONE | Cipher-aware HKDF |
| `tls.derive_application_secrets` | ✅ DONE | Cipher-aware HKDF |
| Graph mapping | ✅ DONE | `hash_for_cipher` added |

---

## ❌ Remaining Issue (84% → 100%)

### Error

```
Invalid transcript_hash length: 48 (expected 32 for SHA-256)
```

### Root Cause

**File**: `crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto/tls/signatures.rs`

```rust
// Line 238: HARDCODED to 32 bytes!
if transcript_hash.len() != 32 {
    return Err(format!(
        "Invalid transcript_hash length: {} (expected 32 for SHA-256)",
        transcript_hash.len()
    ));
}

// Line 274: HARDCODED to SHA-256!
type HmacSha256 = Hmac<Sha256>;
```

### Required Fix

Add `cipher_suite` parameter and make method cipher-aware:

```rust
pub async fn handle_tls_compute_finished_verify_data(
    params: Option<&Value>,
) -> Result<Value, String> {
    // ... existing parameter extraction ...
    
    // NEW: Extract cipher_suite (default to SHA-256 for backwards compat)
    let cipher_suite = params
        .get("cipher_suite")
        .and_then(|v| v.as_u64())
        .unwrap_or(0x1301) as u16;
    
    // Validate hash length based on cipher suite
    let expected_len = match cipher_suite {
        0x1301 | 0x1303 => 32,  // SHA-256
        0x1302 => 48,           // SHA-384
        _ => return Err(format!("Unsupported cipher suite: 0x{:04x}", cipher_suite)),
    };
    
    if transcript_hash.len() != expected_len {
        return Err(format!(
            "Invalid transcript_hash length: {} (expected {} for cipher 0x{:04x})",
            transcript_hash.len(), expected_len, cipher_suite
        ));
    }
    
    // Use appropriate HMAC based on cipher suite
    let verify_data = match cipher_suite {
        0x1301 | 0x1303 => {
            // SHA-256 path (existing code)
            let finished_key = hkdf_expand_label_sha256(&base_key, "finished", &[], 32)?;
            hmac_sha256(&finished_key, &transcript_hash)
        }
        0x1302 => {
            // SHA-384 path (new)
            let finished_key = hkdf_expand_label_sha384(&base_key, "finished", &[], 48)?;
            hmac_sha384(&finished_key, &transcript_hash)
        }
        _ => unreachable!()
    };
    
    // ... rest of function ...
}
```

---

## 📋 Implementation Checklist

### BearDog P0 (Single File Fix!)

- [ ] **signatures.rs** (Line 206): Add `cipher_suite` parameter extraction
- [ ] **signatures.rs** (Line 238): Change validation to cipher-aware
- [ ] **signatures.rs** (Line 246-267): Add `hkdf_expand_label_sha384` function
- [ ] **signatures.rs** (Line 274): Add cipher-aware HMAC selection
- [ ] Add unit test for SHA-384 finished verify_data

### Files to Modify

```
beardog/crates/beardog-tunnel/src/unix_socket_ipc/handlers/crypto/tls/
└── signatures.rs  # ⭐ ONLY THIS FILE NEEDS CHANGES!
```

---

## 🧪 Test Validation

After fix, these sites should work:

```bash
# Currently failing with "expected 32 for SHA-256"
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://www.ncbi.nlm.nih.gov","headers":{}},"id":1}' | nc -U /tmp/songbird-nat0.sock

echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://azure.microsoft.com","headers":{}},"id":1}' | nc -U /tmp/songbird-nat0.sock
```

---

## 🚀 Estimated Effort

| Task | Time |
|------|------|
| Add cipher_suite param | 10 min |
| Cipher-aware validation | 15 min |
| HKDF-SHA384 function | 30 min |
| HMAC-SHA384 selection | 15 min |
| Testing | 30 min |
| **Total** | **~2 hours** |

---

## 🎯 Impact

| Metric | Before | After |
|--------|--------|-------|
| TLS Validation | 84% | 100% |
| Cipher 0x1301 | ✅ | ✅ |
| Cipher 0x1302 | ❌ | ✅ |
| Cipher 0x1303 | ✅ | ✅ |
| NCBI | ❌ | ✅ |
| Azure | ❌ | ✅ |

---

## 📞 Coordination

- **BearDog**: `/home/eastgate/Development/ecoPrimals/phase1/beardog`
- **Songbird**: Already sending 48-byte hashes correctly (via `hash_for_cipher`)
- **biomeOS**: Graph already configured

**No Songbird or biomeOS changes needed!** This is purely a BearDog fix.

---

**Created**: January 26, 2026  
**Updated**: January 26, 2026 (discovered remaining issue)  
**Status**: 🔧 BearDog Evolution Needed - Single file fix!  
**Impact**: 84% → 100% TLS Validation 🎯

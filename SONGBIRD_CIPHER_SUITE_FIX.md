# 🎵 Songbird Cipher Suite Fix - January 26, 2026

## 🎯 Issue: Missing `cipher_suite` parameter in `tls_compute_finished_verify_data`

**Current Status**: 85% TLS validation (NCBI, Azure failing)  
**Root Cause**: Songbird doesn't pass `cipher_suite` to `tls_compute_finished_verify_data`  
**BearDog defaults to 0x1301**, rejects 48-byte SHA-384 hashes  

---

## 📊 Error Details

```
Invalid transcript_hash length: 48 (expected 32 for SHA-256 in cipher 0x1301)
```

**Flow**:
1. Server negotiates cipher 0x1302 (SHA-384)
2. Songbird computes 48-byte transcript hash ✅
3. Songbird calls `tls_compute_finished_verify_data(base_key, hash)` **WITHOUT cipher_suite**
4. BearDog defaults to 0x1301, expects 32 bytes, gets 48 → ERROR

---

## 🔧 Fix Required

### 1. Update `CryptoCapability` Trait

**File**: `crates/songbird-http-client/src/crypto/capability.rs`

```rust
// BEFORE (line 259-263)
async fn tls_compute_finished_verify_data(
    &self,
    base_key: &[u8],
    transcript_hash: &[u8],
) -> Result<Vec<u8>>;

// AFTER
async fn tls_compute_finished_verify_data(
    &self,
    base_key: &[u8],
    transcript_hash: &[u8],
    cipher_suite: u16,  // NEW!
) -> Result<Vec<u8>>;
```

### 2. Update `BearDogProvider` Implementation

**File**: `crates/songbird-http-client/src/crypto/beardog_provider.rs`

```rust
// BEFORE (line 720-736)
async fn tls_compute_finished_verify_data(
    &self,
    base_key: &[u8],
    transcript_hash: &[u8],
) -> Result<Vec<u8>> {
    let result = self.call(
        "tls.compute_finished_verify_data",
        json!({
            "base_key": BASE64_STANDARD.encode(base_key),
            "transcript_hash": BASE64_STANDARD.encode(transcript_hash)
            // MISSING: cipher_suite!
        }),
    ).await?;
    ...
}

// AFTER
async fn tls_compute_finished_verify_data(
    &self,
    base_key: &[u8],
    transcript_hash: &[u8],
    cipher_suite: u16,  // NEW!
) -> Result<Vec<u8>> {
    let result = self.call(
        "tls.compute_finished_verify_data",
        json!({
            "base_key": BASE64_STANDARD.encode(base_key),
            "transcript_hash": BASE64_STANDARD.encode(transcript_hash),
            "cipher_suite": cipher_suite  // NEW!
        }),
    ).await?;
    ...
}
```

### 3. Update Callers in Handshake Flow

**File**: `crates/songbird-http-client/src/tls/handshake_refactored/handshake_flow.rs`

Search for `tls_compute_finished_verify_data` and add `self.cipher_suite`:

```rust
// BEFORE
let verify_data = self.crypto.tls_compute_finished_verify_data(
    &base_key,
    &transcript_hash,
).await?;

// AFTER
let verify_data = self.crypto.tls_compute_finished_verify_data(
    &base_key,
    &transcript_hash,
    self.cipher_suite,  // NEW!
).await?;
```

### 4. Also Update Legacy Handshake (if used)

**File**: `crates/songbird-http-client/src/tls/handshake_legacy.rs`

Same pattern - add `self.cipher_suite` parameter.

---

## 📋 Implementation Checklist

- [ ] `crypto/capability.rs`: Add `cipher_suite: u16` to trait
- [ ] `crypto/beardog_provider.rs`: Pass `cipher_suite` in JSON
- [ ] `tls/handshake_refactored/handshake_flow.rs`: Pass `self.cipher_suite`
- [ ] `tls/handshake_legacy.rs`: Pass `self.cipher_suite` (if used)
- [ ] Test against NCBI and Azure

---

## 🧪 Validation

After fix:

```bash
# These should work:
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://www.ncbi.nlm.nih.gov","headers":{}},"id":1}' | nc -U /tmp/songbird-nat0.sock

echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://azure.microsoft.com","headers":{}},"id":1}' | nc -U /tmp/songbird-nat0.sock
```

---

## ⏱️ Estimated Effort

| Task | Time |
|------|------|
| Update trait | 2 min |
| Update BearDogProvider | 5 min |
| Update handshake_flow.rs | 10 min |
| Update handshake_legacy.rs | 10 min |
| Testing | 15 min |
| **Total** | **~45 min** |

---

## 🎯 Impact

| Metric | Before | After |
|--------|--------|-------|
| TLS Validation | 85% | 100% |
| NCBI | ❌ | ✅ |
| Azure | ❌ | ✅ |

---

**BearDog Status**: ✅ Ready (SHA-384 evolution complete)  
**Songbird Status**: 🔧 Needs this fix  
**Impact**: 85% → 100% TLS Validation 🎯


# 🔍 TLS 1.3 Validation Plan - Pure Rust vs OpenSSL & RFC 8446
## January 23, 2026

**Status**: ✅ **READY TO EXECUTE**  
**Goal**: Validate BearDog + Songbird TLS implementation against reference implementations  

---

## 🎯 WHAT WE HAVE (From BearDog v0.19.0)

### Captured Hex Values (example.com test)

**Transcript Hash** (SHA-256 of all handshake messages):
```
fb27b3a2bbd8d422ae5868fbaf5f9cbcf4aa4d34cdc05c22ed309aef975fed25
```

**Master Secret** (first 16 bytes):
```
8dfabcf4eccfef61756c064ee445357f
```

**CLIENT_TRAFFIC_SECRET_0** (32 bytes for application data encryption):
```
af38bd1558833132c711baf130b416c12992205557af3fa5e1286d8ead73699a
```

**SERVER_TRAFFIC_SECRET_0** (32 bytes for application data decryption):
```
4eebb0c23f26bec0a2545bcacb48d34230b6690148564731ce2a523277630bbe
```

**Cipher Suite**: `0x1301` (TLS_AES_128_GCM_SHA256)

---

## 📋 VALIDATION STRATEGIES

### Strategy 1: OpenSSL SSLKEYLOGFILE Comparison ✅

**Method**: Use OpenSSL's key logging to extract the same secrets for the same connection.

**Steps**:
1. Connect to `example.com:443` using OpenSSL with `SSLKEYLOGFILE`
2. Extract `CLIENT_TRAFFIC_SECRET_0` from the log
3. Compare with BearDog's output (should match byte-for-byte)
4. Verify cipher suite matches (0x1301)

**Expected Result**: Exact match (if our implementation is correct)

---

### Strategy 2: RFC 8448 Test Vectors ✅

**Method**: Use RFC 8448 "Example Handshake Traces for TLS 1.3"

**Reference**: https://www.rfc-editor.org/rfc/rfc8448.html

**Test Vectors Available**:
- ClientHello (with extensions)
- ServerHello (with key_share)
- EncryptedExtensions
- Certificate
- CertificateVerify
- Finished messages
- Application data

**What We Can Validate**:
1. Transcript hash computation (SHA-256 of messages)
2. HKDF-Expand-Label implementation
3. Key derivation from shared secret
4. Handshake traffic secrets
5. Application traffic secrets
6. Finished message verify_data

**Expected Result**: Our intermediate values should match RFC 8448 test vectors

---

### Strategy 3: Manual HKDF-Expand-Label Validation ✅

**Method**: Manually compute keys using Python/Rust reference implementation

**Process**:
1. Use our captured `pre_master_secret` (shared secret from ECDH)
2. Use our captured `client_random` and `server_random`
3. Use our captured `transcript_hash`
4. Manually compute:
   - Early Secret (HKDF-Extract with empty salt)
   - Handshake Secret (HKDF-Extract with shared secret)
   - Master Secret (HKDF-Extract with derived secret)
   - Client Application Traffic Secret (HKDF-Expand-Label)
   - Server Application Traffic Secret (HKDF-Expand-Label)
5. Compare with BearDog's output

**Tools**:
- Python: `cryptography` library
- Rust: `hkdf`, `sha2` crates
- OpenSSL CLI: `openssl dgst -sha256`

**Expected Result**: Exact match if our HKDF implementation is correct

---

### Strategy 4: Cross-Site Validation ✅

**Method**: Test multiple sites and compare patterns

**Test Sites**:
1. `example.com` (already tested)
2. `github.com` (TLS 1.3 supported)
3. `cloudflare.com` (modern TLS)
4. `google.com` (may use different cipher suite)
5. `mozilla.org` (TLS reference implementation)

**What to Validate**:
- Cipher suite negotiation (0x1301, 0x1302, 0x1303)
- Key lengths match cipher suite
- Transcript hash length (32 bytes for SHA-256, 48 for SHA-384)
- All secrets derive correctly

**Expected Result**: Consistent behavior across sites

---

### Strategy 5: Known-Answer Tests (KAT) ✅

**Method**: Create test cases with known inputs/outputs

**Test Cases**:
1. **Zero Input Test**:
   - Shared secret: all zeros (32 bytes)
   - Client random: all zeros
   - Server random: all zeros
   - Transcript hash: SHA-256 of empty string
   - Verify deterministic output

2. **RFC 8448 Reproduction**:
   - Use exact values from RFC 8448 Section 3
   - Verify our implementation produces same keys

3. **Incremental Test**:
   - Shared secret: `[0x01, 0x02, 0x03, ...]`
   - Verify against Python reference

**Expected Result**: Deterministic outputs that can be validated

---

## 🔧 TOOLS AND SETUP

### OpenSSL Key Logging

```bash
# Enable key logging
export SSLKEYLOGFILE=/tmp/tls-keys.log

# Connect to example.com
openssl s_client -connect example.com:443 -tls1_3

# Extract keys
grep "CLIENT_TRAFFIC_SECRET_0" /tmp/tls-keys.log
```

**Format**:
```
CLIENT_TRAFFIC_SECRET_0 <client_random_hex> <secret_hex>
```

### Python Validation Script

```python
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.hkdf import HKDF, HKDFExpand
import binascii

def hkdf_expand_label(secret, label, context, length):
    """TLS 1.3 HKDF-Expand-Label (RFC 8446 Section 7.1)"""
    label_bytes = b"tls13 " + label.encode('ascii')
    hkdf_label = (
        length.to_bytes(2, 'big') +
        len(label_bytes).to_bytes(1, 'big') + label_bytes +
        len(context).to_bytes(1, 'big') + context
    )
    return HKDFExpand(
        algorithm=hashes.SHA256(),
        length=length,
        info=hkdf_label,
    ).derive(secret)

# Test with our captured values
shared_secret = bytes.fromhex("...") # From BearDog
transcript_hash = bytes.fromhex("fb27b3a2bbd8d422ae5868fbaf5f9cbcf4aa4d34cdc05c22ed309aef975fed25")

# Derive master secret
# ... (full key schedule)

# Compare
print(f"BearDog: af38bd1558833132c711baf130b416c12992205557af3fa5e1286d8ead73699a")
print(f"Python:  {client_traffic_secret.hex()}")
```

### Rust Validation (Using BearDog's own code)

```rust
// Use BearDog's HKDF implementation directly
use beardog_tunnel::tls::hkdf_expand_label;

#[test]
fn test_client_traffic_secret_validation() {
    let shared_secret = hex::decode("...").unwrap();
    let transcript_hash = hex::decode(
        "fb27b3a2bbd8d422ae5868fbaf5f9cbcf4aa4d34cdc05c22ed309aef975fed25"
    ).unwrap();
    
    // Derive according to RFC 8446
    let master_secret = derive_master_secret(&shared_secret);
    let client_secret = hkdf_expand_label(
        &master_secret,
        "c ap traffic",
        &transcript_hash,
        32
    ).unwrap();
    
    assert_eq!(
        hex::encode(client_secret),
        "af38bd1558833132c711baf130b416c12992205557af3fa5e1286d8ead73699a"
    );
}
```

---

## 📊 VALIDATION MATRIX

### What to Compare

| Item | BearDog Output | OpenSSL | RFC 8448 | Manual Calc | Status |
|------|---------------|---------|----------|-------------|--------|
| Transcript Hash | fb27b3a2... | ⏳ | ⏳ | ⏳ | Pending |
| Master Secret | 8dfabcf4... | ⏳ | ⏳ | ⏳ | Pending |
| CLIENT_TRAFFIC_SECRET_0 | af38bd15... | ⏳ | ⏳ | ⏳ | Pending |
| SERVER_TRAFFIC_SECRET_0 | 4eebb0c2... | ⏳ | ⏳ | ⏳ | Pending |
| Client Write Key | (from log) | ⏳ | ⏳ | ⏳ | Pending |
| Server Write Key | (from log) | ⏳ | ⏳ | ⏳ | Pending |
| Client Write IV | (from log) | ⏳ | ⏳ | ⏳ | Pending |
| Server Write IV | (from log) | ⏳ | ⏳ | ⏳ | Pending |

---

## 🎯 EXECUTION PLAN

### Phase 1: OpenSSL Comparison (30 minutes)
1. ✅ Capture BearDog values (DONE - from v0.19.0 logs)
2. ⏳ Run OpenSSL with SSLKEYLOGFILE to `example.com`
3. ⏳ Extract CLIENT_TRAFFIC_SECRET_0 and SERVER_TRAFFIC_SECRET_0
4. ⏳ Compare byte-for-byte with BearDog
5. ⏳ Document results (match/mismatch)

### Phase 2: RFC 8448 Validation (1 hour)
1. ⏳ Read RFC 8448 test vectors
2. ⏳ Create test harness with RFC values
3. ⏳ Run BearDog's HKDF against test vectors
4. ⏳ Validate intermediate values (handshake secret, master secret)
5. ⏳ Validate final traffic secrets

### Phase 3: Manual Calculation (30 minutes)
1. ⏳ Write Python validation script
2. ⏳ Implement HKDF-Expand-Label
3. ⏳ Derive traffic secrets manually
4. ⏳ Compare with BearDog output
5. ⏳ Document any differences

### Phase 4: Cross-Site Testing (30 minutes)
1. ⏳ Test `github.com`
2. ⏳ Test `cloudflare.com`
3. ⏳ Test `google.com` (may use different cipher)
4. ⏳ Validate consistency across sites
5. ⏳ Document cipher suite behavior

### Phase 5: Known-Answer Tests (30 minutes)
1. ⏳ Create zero-input test
2. ⏳ Create RFC 8448 reproduction test
3. ⏳ Create incremental test
4. ⏳ Add to BearDog test suite
5. ⏳ Validate deterministic behavior

---

## 🚨 WHAT TO LOOK FOR

### Red Flags 🚩

**If Traffic Secrets DON'T Match OpenSSL**:
- ❌ HKDF-Expand-Label implementation bug
- ❌ Transcript hash computation error
- ❌ Wrong key schedule order
- ❌ Byte order issues (big-endian vs little-endian)

**If Handshake Works But Decryption Fails**:
- ❌ Nonce construction error (IV XOR sequence number)
- ❌ AAD mismatch (TLS record header)
- ❌ Ciphertext/tag splitting error
- ❌ Wrong key used for decryption

**If Different Sites Behave Differently**:
- ❌ Cipher suite negotiation bug
- ❌ Key length derivation error (16 vs 32 bytes)
- ❌ Hash algorithm mismatch (SHA-256 vs SHA-384)

### Green Flags ✅

**If Traffic Secrets MATCH OpenSSL**:
- ✅ HKDF implementation is correct
- ✅ Transcript hash is correct
- ✅ Key schedule is correct
- ✅ **Any remaining issues are in AEAD or record layer**

**If RFC 8448 Tests Pass**:
- ✅ Implementation follows RFC 8446 exactly
- ✅ Can be trusted for production use
- ✅ Edge cases handled correctly

**If Cross-Site Tests Consistent**:
- ✅ Cipher negotiation works
- ✅ Dynamic key lengths work
- ✅ Robust implementation

---

## 📋 DELIVERABLES

### Documentation
1. **Validation Results Report** (markdown)
   - OpenSSL comparison (match/mismatch)
   - RFC 8448 validation results
   - Manual calculation verification
   - Cross-site test results

2. **Test Suite** (Rust code)
   - RFC 8448 test cases
   - Known-answer tests
   - OpenSSL compatibility tests

3. **Debug Guide** (for future)
   - How to validate TLS keys
   - Common pitfalls
   - Debugging workflow

### Code
1. **Python validation script** (reference implementation)
2. **Rust test harness** (integrated with BearDog)
3. **OpenSSL comparison script** (bash/Python)

---

## 🎯 SUCCESS CRITERIA

### Minimum Success (85%)
- ✅ OpenSSL comparison shows exact match for traffic secrets
- ✅ Transcript hash validates correctly
- ✅ At least 3 cross-site tests pass

### Target Success (95%)
- ✅ All OpenSSL comparisons match
- ✅ RFC 8448 test vectors pass
- ✅ Manual calculations match
- ✅ All cross-site tests pass

### Full Success (100%)
- ✅ All of the above
- ✅ Known-answer tests added to suite
- ✅ Comprehensive documentation
- ✅ **Production-ready validation infrastructure**

---

## 🚀 LET'S START!

### Immediate Next Steps

1. **Run OpenSSL with SSLKEYLOGFILE** (5 minutes)
   - Connect to `example.com:443`
   - Capture keys
   - Extract CLIENT_TRAFFIC_SECRET_0

2. **Compare with BearDog** (5 minutes)
   - Our value: `af38bd1558833132c711baf130b416c12992205557af3fa5e1286d8ead73699a`
   - OpenSSL value: (to be extracted)
   - Result: Match or Mismatch?

3. **Document Findings** (10 minutes)
   - If match: ✅ Celebrate! Implementation is correct!
   - If mismatch: ❌ Investigate and debug

**Total ETA for Phase 1**: ~20 minutes

---

**Ready to execute?** Let's validate our Pure Rust TLS implementation! 🚀🦀✨


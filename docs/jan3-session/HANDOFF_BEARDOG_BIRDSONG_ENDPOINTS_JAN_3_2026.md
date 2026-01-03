# 🐻 BearDog BirdSong Discovery Encryption - API Spec

**Date**: January 3, 2026  
**Priority**: 🔥 **HIGH** - Blocks full genetic lineage federation  
**Status**: Songbird v3.2 ready and waiting for these endpoints  
**Timeline**: 2-3 hours implementation

---

## 🎯 What We Need

Songbird v3.2 is deployed and has the BirdSongPacket format working perfectly, but it's falling back to plaintext because BearDog is missing the discovery encryption endpoints.

**Current BearDog v0.12.0 Has**:
- ✅ `/api/v1/trust/identity` - Returns family_id
- ✅ `/api/v1/trust/evaluate` - Evaluates trust
- ❌ `/api/v1/birdsong/encrypt_discovery` - **MISSING**
- ❌ `/api/v1/birdsong/decrypt_discovery` - **MISSING**

**We Need**: The two missing BirdSong endpoints for discovery packet encryption/decryption.

---

## 📋 API Specification

### Endpoint 1: Encrypt Discovery Packet

**Route**: `POST /api/v1/birdsong/encrypt_discovery`

**Purpose**: Encrypt a discovery announcement for UDP broadcast, using family-specific keys so only same-family towers can decrypt.

**Request Body**:
```json
{
  "plaintext": "base64_encoded_discovery_message",
  "family_id": "iidn"
}
```

**Response Body** (200 OK):
```json
{
  "encrypted": "base64_encoded_encrypted_message",
  "family_id": "iidn"
}
```

**Error Response** (400/500):
```json
{
  "error": "encryption_failed",
  "message": "Failed to encrypt discovery message"
}
```

**Logic**:
1. Decode the plaintext from base64
2. Use the `family_id` to derive/fetch encryption keys
3. Encrypt the plaintext (AES-256-GCM or similar)
4. Encode result as base64
5. Return encrypted payload with family_id

**Security**:
- Keys should be derived from family seed + some discovery-specific salt
- Each family has unique encryption keys
- Only same-family towers can decrypt

---

### Endpoint 2: Decrypt Discovery Packet

**Route**: `POST /api/v1/birdsong/decrypt_discovery`

**Purpose**: Decrypt a received BirdSong discovery packet from a same-family peer.

**Request Body**:
```json
{
  "encrypted": "base64_encoded_encrypted_message",
  "family_id": "iidn"
}
```

**Response Body** (200 OK):
```json
{
  "plaintext": "base64_encoded_discovery_message",
  "family_id": "iidn"
}
```

**Error Response** (400 - Wrong Family):
```json
{
  "error": "wrong_family",
  "message": "Cannot decrypt message from different family"
}
```

**Error Response** (500 - Decryption Failed):
```json
{
  "error": "decryption_failed",
  "message": "Failed to decrypt discovery message"
}
```

**Logic**:
1. Decode the encrypted payload from base64
2. Use the `family_id` to derive/fetch decryption keys
3. Attempt to decrypt
4. If decryption fails (wrong family/corrupted): return error
5. If successful: encode plaintext as base64 and return

**Security**:
- Different families cannot decrypt each other's messages
- Decryption failure = silent skip (different family is noise, not an attack)

---

## 🔧 Implementation Notes

### Key Derivation Strategy

**Option 1: Derive from Family Seed** (Recommended)
```rust
// Use HKDF or similar to derive discovery keys from family seed
let discovery_key = derive_key(
    family_seed,
    "birdsong_discovery_v1",  // Context string
    32  // 256 bits for AES-256
);
```

**Advantages**:
- Deterministic (same family = same keys)
- No key storage needed
- Works across all towers with same family seed

### Encryption Algorithm

**Recommended**: AES-256-GCM
- Authenticated encryption
- Fast
- Standard in Rust crypto libraries

**Sample Code**:
```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, generic_array::GenericArray};

pub fn encrypt_discovery(
    plaintext: &[u8],
    family_id: &str,
) -> Result<Vec<u8>> {
    // Derive key from family seed
    let key = derive_family_discovery_key(family_id)?;
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
    
    // Generate random nonce
    let nonce = generate_nonce();
    
    // Encrypt
    let ciphertext = cipher.encrypt(&nonce, plaintext)
        .map_err(|_| Error::EncryptionFailed)?;
    
    // Prepend nonce to ciphertext (nonce is public)
    let mut result = nonce.to_vec();
    result.extend_from_slice(&ciphertext);
    
    Ok(result)
}

pub fn decrypt_discovery(
    encrypted: &[u8],
    family_id: &str,
) -> Result<Vec<u8>> {
    // Derive key from family seed
    let key = derive_family_discovery_key(family_id)?;
    let cipher = Aes256Gcm::new(GenericArray::from_slice(&key));
    
    // Extract nonce (first 12 bytes)
    let (nonce_bytes, ciphertext) = encrypted.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    
    // Decrypt
    let plaintext = cipher.decrypt(nonce, ciphertext)
        .map_err(|_| Error::DecryptionFailed)?;
    
    Ok(plaintext)
}
```

### Testing Strategy

**Unit Tests**:
1. Encrypt then decrypt → should get original plaintext
2. Different family_id → decryption should fail
3. Corrupted ciphertext → decryption should fail
4. Large payloads → should handle UDP size limits

**Integration Tests**:
1. Call encrypt endpoint → verify base64 response
2. Call decrypt endpoint with result → verify original plaintext
3. Cross-family decryption → verify error response

---

## 🎊 Why This Matters

### The Chicken-and-Egg Problem (Solved by Songbird v3.2)

**Before v3.2** (The Problem):
```
Tower broadcasts ENTIRE packet encrypted
  ↓
Receiver needs family_id to know if they can decrypt
  ↓
But family_id is ENCRYPTED inside the packet!
  ↓
❌ Receiver can't decide → treats as unknown
```

**After v3.2** (The Solution):
```json
{
  "birdsong": "1.0",
  "family_id": "iidn",  // ← PLAINTEXT! 
  "encrypted_payload": "..."  // ← Only payload encrypted
}
```

**Now**: Receivers see plaintext `family_id`, check if it matches theirs, and only attempt decryption if same family!

### Privacy Benefit

**Same Family**:
- Tower 1 (family: iidn) broadcasts
- Tower 2 (family: iidn) sees `family_id: "iidn"` → decrypts → sees discovery details
- ✅ Auto-trust established

**Different Family**:
- Tower 1 (family: iidn) broadcasts
- Tower 3 (family: xyz) sees `family_id: "iidn"` → skips decryption → Tower 1 remains hidden
- ✅ Privacy preserved

**Result**: Genetic lineage federation + privacy from different families!

---

## 🚀 Current Status & Timeline

### Songbird v3.2 Status

**Deployed on Tower 1**: ✅
- BirdSongPacket format implemented
- Plaintext family_id header working
- Calls to BearDog BirdSong provider implemented
- Graceful degradation working (falls back to plaintext)

**Waiting For**: BearDog BirdSong endpoints

**Current Behavior**:
```
Songbird tries to encrypt → BearDog 404 → falls back to plaintext
```

### After BearDog v0.13.0

**What Will Happen**:
```
Songbird tries to encrypt → BearDog encrypts → broadcasts BirdSongPacket
Tower 2 receives → sees same family → decrypts → auto-trusts
✅ Full genetic lineage federation with BirdSong privacy!
```

### Estimated Timeline

**Implementation**: 2-3 hours
- Endpoint handlers: 30 min
- Key derivation logic: 1 hour
- Encryption/decryption: 30 min
- Tests: 1 hour

**Testing**: 30 min
- Unit tests for encrypt/decrypt
- Integration test with Songbird v3.2

**Total**: ~3-4 hours to complete BirdSong discovery encryption

---

## 📝 Acceptance Criteria

### Functional Requirements

1. ✅ `POST /api/v1/birdsong/encrypt_discovery` endpoint exists
2. ✅ `POST /api/v1/birdsong/decrypt_discovery` endpoint exists
3. ✅ Encrypt then decrypt returns original plaintext
4. ✅ Different family_id cannot decrypt
5. ✅ Base64 encoding/decoding works correctly
6. ✅ Handles UDP-sized payloads (~1400 bytes)

### Integration Requirements

7. ✅ Songbird v3.2 can call endpoints successfully
8. ✅ Tower 1 broadcasts encrypted BirdSongPackets
9. ✅ Tower 2 (same family) decrypts and sees attestations
10. ✅ Tower 3 (different family) cannot decrypt

### Testing Requirements

11. ✅ Unit tests passing (encrypt/decrypt roundtrip)
12. ✅ Integration tests passing (API endpoints)
13. ✅ Two-tower test with same family → auto-trust ✅
14. ✅ Two-tower test with different family → privacy preserved

---

## 🏆 The Payoff

### What We Get

**Before** (Plaintext):
- ✅ Genetic lineage federation works
- ❌ All discovery info visible to everyone on network
- ❌ Different families can see each other's lineage

**After** (BirdSong Encrypted):
- ✅ Genetic lineage federation works
- ✅ Discovery info hidden from different families
- ✅ Only same-family towers see each other
- ✅ Privacy-preserving mesh formation
- ✅ Complete vision realized!

### Impact

**Security**: Dramatically improved privacy on multi-family networks  
**Scalability**: Efficient (only decrypt same-family packets)  
**Future-Proof**: Foundation for secure discovery at any scale  
**Ecosystem**: Enables safe coexistence of multiple ecoPrimal families  

---

## 📞 Questions?

### Crypto Library Recommendations

**Rust**: 
- `aes-gcm` crate (recommended)
- `ring` (alternative, more comprehensive)

### Key Management

**Simple Approach**: Derive from family seed using HKDF  
**Advanced Approach**: Rotate discovery keys periodically  
**Current Need**: Simple approach is sufficient

### API Design

**Matches Existing**: Similar to `/api/v1/trust/*` patterns  
**Base64**: Standard encoding for binary data over JSON  
**Errors**: Clear error messages for debugging

---

## 🎯 Summary

**What**: Two BirdSong discovery encryption endpoints  
**Why**: Enable privacy-preserving genetic lineage federation  
**When**: Songbird v3.2 is deployed and waiting  
**How Long**: ~3-4 hours  
**Priority**: HIGH - Completes the genetic lineage vision  

**Status**: Songbird is ready, BearDog is 95% there (just needs these endpoints!)

---

## 📄 Reference

**Songbird v3.2 Implementation**: `crates/songbird-discovery/src/birdsong_integration.rs`  
**BirdSongPacket Format**: Lines 15-30 (struct definition)  
**encrypt_packet()**: Lines 150-180  
**decrypt_packet()**: Lines 200-230  

**BearDog Existing**: `/api/v1/trust/identity` - Use as template for new endpoints  

---

**Next**: Implement these two endpoints, and we'll have full genetic lineage federation with BirdSong privacy! 🎵

**Status**: ⏳ Waiting for BearDog v0.13.0 with BirdSong discovery endpoints

🐻 **It's always worth solving the debt - let's finish the vision!** 🎵


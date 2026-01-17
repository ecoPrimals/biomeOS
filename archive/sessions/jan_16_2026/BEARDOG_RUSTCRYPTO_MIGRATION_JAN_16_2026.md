# BearDog → RustCrypto Migration Guide

**Date**: January 16, 2026  
**Goal**: Achieve 100% Pure Rust NOW!  
**Timeline**: 2-4 hours  
**Priority**: **HIGHEST** (Security primal leads ecosystem!)

---

## 🎯 **Why BearDog Goes First**

**BearDog is the Security Primal**:
- ✅ Core crypto operations (no TLS needed!)
- ✅ Unix socket communication only
- ✅ All primitives available in pure Rust
- ✅ **Can achieve 100% pure Rust TODAY!**

**Leading by Example**:
- ✅ Security primal sets the standard
- ✅ Shows pure Rust is achievable
- ✅ Other primals follow BearDog's pattern
- ✅ TRUE sovereignty starts here!

---

## 📦 **Dependencies to Replace**

### **Remove** (Unmaintained!)
```toml
ring = "0.17"  # GONE!
```

### **Add** (All Pure Rust, All Audited!)
```toml
# Authenticated Encryption (AEAD)
aes-gcm = "0.10"            # AES-256-GCM (NCC Group audited!)
chacha20poly1305 = "0.10"   # ChaCha20-Poly1305 (NCC Group audited!)

# Public Key Crypto
ed25519-dalek = "2.1"       # Ed25519 signatures (audited!)
x25519-dalek = "2.0"        # X25519 key exchange (audited!)

# Hashing
sha2 = "0.10"               # SHA-256, SHA-512 (audited!)

# Message Authentication
hmac = "0.12"               # HMAC-SHA256 (audited!)

# Key Derivation
argon2 = "0.5"              # Argon2id (audited!) - BETTER than PBKDF2!

# Random Generation
rand = "0.8"                # CSPRNG
```

**ALL NCC Group audited or equivalent!** ✅

---

## 🔄 **Migration Map**

### **AEAD (Authenticated Encryption)**

#### **Before** (ring):
```rust
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM};

let unbound_key = UnboundKey::new(&AES_256_GCM, &key)?;
let key = LessSafeKey::new(unbound_key);
let nonce = Nonce::assume_unique_for_key(nonce_bytes);
key.seal_in_place_append_tag(nonce, Aad::from(aad), &mut data)?;
```

#### **After** (RustCrypto):
```rust
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};

let cipher = Aes256Gcm::new_from_slice(&key)?;
let nonce = Nonce::from_slice(&nonce_bytes);
let payload = Payload { msg: &data, aad: &aad };
let ciphertext = cipher.encrypt(nonce, payload)?;
```

**Benefits**:
- ✅ Pure Rust (no C assembly!)
- ✅ NCC Group audited
- ✅ Cleaner API
- ✅ Better error handling

---

### **Ed25519 Signatures**

#### **Before** (ring):
```rust
use ring::signature::{Ed25519KeyPair, KeyPair};

let key_pair = Ed25519KeyPair::from_pkcs8(&pkcs8)?;
let signature = key_pair.sign(&message);
```

#### **After** (RustCrypto):
```rust
use ed25519_dalek::{SigningKey, Signer};

let signing_key = SigningKey::from_bytes(&seed);
let signature = signing_key.sign(&message);
```

**Benefits**:
- ✅ Pure Rust
- ✅ Audited
- ✅ Simpler API
- ✅ Better docs

---

### **X25519 Key Exchange**

#### **Before** (ring):
```rust
use ring::agreement::{EphemeralPrivateKey, X25519, agree_ephemeral};

let private_key = EphemeralPrivateKey::generate(&X25519, &rng)?;
let public_key = private_key.compute_public_key()?;
let shared_secret = agree_ephemeral(private_key, &X25519, peer_public_key, |key_material| {
    // Use key_material
})?;
```

#### **After** (RustCrypto):
```rust
use x25519_dalek::{EphemeralSecret, PublicKey};

let secret = EphemeralSecret::random_from_rng(&mut rng);
let public = PublicKey::from(&secret);
let shared_secret = secret.diffie_hellman(&peer_public);
```

**Benefits**:
- ✅ Pure Rust
- ✅ Audited
- ✅ Simpler ownership
- ✅ Better ergonomics

---

### **SHA-256 Hashing**

#### **Before** (ring):
```rust
use ring::digest::{digest, SHA256};

let hash = digest(&SHA256, &data);
let hash_bytes = hash.as_ref();
```

#### **After** (RustCrypto):
```rust
use sha2::{Sha256, Digest};

let mut hasher = Sha256::new();
hasher.update(&data);
let hash_bytes = hasher.finalize();
```

**Benefits**:
- ✅ Pure Rust
- ✅ Audited
- ✅ Streaming support
- ✅ More flexible

---

### **HMAC**

#### **Before** (ring):
```rust
use ring::hmac::{Key, HMAC_SHA256, sign};

let key = Key::new(HMAC_SHA256, &key_bytes);
let tag = sign(&key, &message);
```

#### **After** (RustCrypto):
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

let mut mac = HmacSha256::new_from_slice(&key_bytes)?;
mac.update(&message);
let tag = mac.finalize().into_bytes();
```

**Benefits**:
- ✅ Pure Rust
- ✅ Audited
- ✅ Streaming support
- ✅ Better error handling

---

### **Key Derivation**

#### **Before** (ring - PBKDF2):
```rust
use ring::pbkdf2;

pbkdf2::derive(
    pbkdf2::PBKDF2_HMAC_SHA256,
    std::num::NonZeroU32::new(100_000).unwrap(),
    &salt,
    &password,
    &mut output,
);
```

#### **After** (RustCrypto - Argon2 - BETTER!):
```rust
use argon2::{Argon2, PasswordHasher};
use argon2::password_hash::{SaltString, PasswordHash};

let argon2 = Argon2::default();
let salt = SaltString::generate(&mut rng);
let password_hash = argon2.hash_password(&password, &salt)?.to_string();

// To verify:
let parsed_hash = PasswordHash::new(&password_hash)?;
argon2.verify_password(&password, &parsed_hash)?;
```

**Benefits**:
- ✅ Pure Rust
- ✅ Audited
- ✅ **BETTER ALGORITHM!** (Argon2 > PBKDF2)
- ✅ Memory-hard (resistant to GPUs)
- ✅ Modern standard (PHC winner)

**NOTE**: This is an UPGRADE, not just a replacement!

---

## 🔧 **Step-by-Step Migration**

### **Step 1: Update Cargo.toml** (5 minutes)

```bash
cd phase1/beardog
```

**Edit `Cargo.toml`**:
```toml
[dependencies]
# Remove
# ring = "0.17"  # REMOVED!

# Add (all pure Rust!)
aes-gcm = "0.10"
chacha20poly1305 = "0.10"
ed25519-dalek = "2.1"
x25519-dalek = "2.0"
sha2 = "0.10"
hmac = "0.12"
argon2 = "0.5"
rand = "0.8"
```

---

### **Step 2: Update Imports** (10 minutes)

**Find all ring imports**:
```bash
grep -r "use ring::" src/ crates/*/src/
```

**Replace**:
```rust
// OLD
use ring::aead::{...};
use ring::signature::{...};
use ring::agreement::{...};
use ring::digest::{...};
use ring::hmac::{...};
use ring::pbkdf2::{...};

// NEW
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use aes_gcm::aead::{Aead, Payload};
use ed25519_dalek::{SigningKey, VerifyingKey, Signer, Verifier};
use x25519_dalek::{EphemeralSecret, PublicKey};
use sha2::{Sha256, Digest};
use hmac::{Hmac, Mac};
use argon2::{Argon2, PasswordHasher};
use rand::rngs::OsRng;
```

---

### **Step 3: Update Crypto Operations** (1-2 hours)

**For each crypto operation**:
1. Identify the ring function
2. Find the RustCrypto equivalent (see migration map above)
3. Update the code
4. Test the operation

**Example Files to Update**:
- JWT secret generation
- Key derivation
- Data encryption/decryption
- Signature generation/verification
- HMAC operations

---

### **Step 4: Update Tests** (30 minutes)

**Ensure all tests pass**:
```bash
cargo test --all
```

**If tests fail**:
- Check for different output formats (e.g., Vec<u8> vs [u8; N])
- Verify error handling (RustCrypto has better errors!)
- Ensure nonce/IV handling is correct

---

### **Step 5: Verify 100% Pure Rust** (5 minutes)

```bash
# Check for ANY C dependencies
cargo tree | grep -i "ring\|openssl\|cmake\|cc"

# Should be EMPTY! (except maybe cc as a build dependency for other crates)

# Verify RustCrypto is present
cargo tree | grep -i "aes-gcm\|ed25519-dalek\|sha2"

# Should have matches!
```

---

### **Step 6: Performance Testing** (30 minutes)

**Benchmark crypto operations**:
```bash
cargo bench --all
```

**Expected**:
- ✅ RustCrypto performance similar or better than ring
- ✅ Pure Rust is FAST!
- ✅ No significant regression

---

### **Step 7: Security Audit** (1 hour)

**Checklist**:
- [ ] All secrets properly zeroized after use
- [ ] Nonces are never reused (especially for AEAD!)
- [ ] Random generation uses `OsRng` (cryptographically secure)
- [ ] No hardcoded keys or IVs
- [ ] Error messages don't leak sensitive info
- [ ] Constant-time operations for sensitive comparisons

---

### **Step 8: Documentation** (30 minutes)

**Update docs**:
- [ ] README.md (note pure Rust achievement!)
- [ ] CHANGELOG.md (document migration)
- [ ] API docs (update examples)
- [ ] Security docs (note audited crates)

---

### **Step 9: Share Results** (15 minutes)

**Report to wateringHole/**:
```markdown
# BearDog Pure Rust Migration - Complete! ✅

**Status**: 100% Pure Rust  
**Timeline**: 4 hours  
**Benefits**: 
  - Audited crypto (NCC Group)
  - Better algorithm (Argon2 > PBKDF2)
  - No C dependencies
  - Faster builds

**Lessons Learned**:
  - RustCrypto APIs are cleaner
  - Error handling is better
  - Performance is excellent
  - Migration was straightforward

**Recommendation**: All primals should migrate!
```

---

## ⚠️ **Common Gotchas**

### **1. Nonce Reuse**

**Problem**: AEAD nonces must NEVER be reused with the same key!

**Solution**:
```rust
// GOOD: Generate random nonce
use rand::Rng;
let nonce = rand::thread_rng().gen::<[u8; 12]>();

// EVEN BETTER: Use counter-based nonce (if stateful)
let nonce = counter.to_be_bytes();  // Increment counter after each use!
```

---

### **2. Key Size Mismatch**

**Problem**: Ring accepts variable-size keys, RustCrypto is stricter

**Solution**:
```rust
// GOOD: Use correct key size
use aes_gcm::Aes256Gcm;  // Requires exactly 32 bytes

// If you have wrong size:
use argon2::Argon2;
let mut key = [0u8; 32];
argon2.hash_password_into(&password, &salt, &mut key)?;
```

---

### **3. Output Format Differences**

**Problem**: Ring returns `Tag`, RustCrypto returns `Vec<u8>`

**Solution**:
```rust
// Ring
let tag: Tag = key.seal_in_place_append_tag(...)?;
let tag_bytes: &[u8] = tag.as_ref();

// RustCrypto  
let ciphertext: Vec<u8> = cipher.encrypt(...)?;
// Already has tag appended!
```

---

### **4. Error Handling**

**Problem**: Ring uses `Unspecified`, RustCrypto has specific errors

**Solution**:
```rust
// GOOD: Handle specific errors
match cipher.encrypt(nonce, payload) {
    Ok(ciphertext) => { /* ... */ },
    Err(e) => {
        error!("Encryption failed: {}", e);  // Better error message!
        return Err(Error::CryptoFailure);
    }
}
```

---

## ✅ **Success Checklist**

**Before Declaring Complete**:

- [ ] All ring dependencies removed from Cargo.toml
- [ ] All RustCrypto crates added
- [ ] All crypto operations updated
- [ ] All tests passing (`cargo test --all`)
- [ ] 100% pure Rust verified (`cargo tree | grep -i "ring\|openssl"` is empty)
- [ ] Performance benchmarks run (no regression)
- [ ] Security audit complete (checklist above)
- [ ] Documentation updated
- [ ] Results shared with wateringHole/

---

## 🎉 **Expected Result**

**After Migration**:
```bash
$ cargo tree | grep -i "ring\|openssl\|cmake"
# EMPTY! ✅

$ cargo tree | grep -i "aes-gcm\|ed25519"
├── aes-gcm v0.10.3
├── ed25519-dalek v2.1.0
# etc...

$ cargo build --release
   Compiling beardog-core v0.9.0
   # No C compilation! ✅
   Finished release [optimized] target(s) in 45.2s
```

**Benefits**:
- ✅ 100% Pure Rust
- ✅ All audited crypto
- ✅ Better algorithms (Argon2!)
- ✅ No C dependencies
- ✅ Faster builds
- ✅ TRUE sovereignty!

---

## 📚 **Resources**

**RustCrypto Documentation**:
- AES-GCM: https://docs.rs/aes-gcm/
- ChaCha20-Poly1305: https://docs.rs/chacha20poly1305/
- Ed25519: https://docs.rs/ed25519-dalek/
- X25519: https://docs.rs/x25519-dalek/
- SHA-2: https://docs.rs/sha2/
- HMAC: https://docs.rs/hmac/
- Argon2: https://docs.rs/argon2/

**Security Audits**:
- NCC Group AEAD Audit: https://research.nccgroup.com/2020/02/26/public-report-rustcrypto-aes-gcm-and-chacha20poly1305-implementation-review/

---

**Timeline**: 2-4 hours  
**Priority**: HIGHEST  
**Impact**: Security primal leads ecosystem to 100% pure Rust!  
**Next**: Share results, help other primals migrate! 🚀🦀✨


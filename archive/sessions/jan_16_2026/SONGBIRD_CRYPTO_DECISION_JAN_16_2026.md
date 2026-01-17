# Songbird Crypto Decision: Breaking the Circular Dependency

**Date**: January 16, 2026  
**Issue**: Circular dependency between aws-lc-rs (cmake) and ring (unmaintained)  
**Status**: 🚨 **CRITICAL** - Need clear direction  
**For**: Songbird Team

---

## 🚨 **The Situation**

### **What Songbird Team Just Reported**

**Migration Path**:
```
FROM: aws-lc-rs (requires cmake to build)
TO:   ring (self-contained, NO external build tools)
```

**Reasoning**: Avoid cmake dependency, simpler build

**This seems reasonable... BUT there's critical new information!**

---

### **What We Just Discovered**

**ring Status** (RustSec Advisory RUSTSEC-2025-0007):
- ❌ **UNMAINTAINED** since February 2025
- ❌ Author on indefinite hiatus
- ❌ Only security-only patches (no features)
- ❌ **ring is a DEAD END**

**This creates a circular dependency**:
```
aws-lc-rs → Requires cmake ❌
ring      → UNMAINTAINED ❌
```

**We're stuck between two bad options... OR ARE WE?**

---

## 💡 **The Key Question**

### **Does Songbird Actually Need TLS?**

**What We Know About Songbird**:
- ✅ P2P communication primal
- ✅ UDP multicast discovery (BirdSong protocol)
- ✅ Encrypted BTSP tunnels
- ✅ Unix socket JSON-RPC (local IPC)
- ❓ **Does it use HTTPS/TLS for external communication?**

**This is CRITICAL because**:
- If NO TLS needed → Use RustCrypto directly (pure Rust!)
- If TLS needed → Must choose between ring/aws-lc-rs/wait

---

## 🎯 **Option A: Use RustCrypto Directly** (RECOMMENDED!)

### **If Songbird Uses:**
- ✅ Unix sockets (local IPC)
- ✅ UDP (discovery)
- ✅ Custom encrypted tunnels (BTSP)
- ❌ NO HTTPS/TLS to external services

**Then You Can**:
```toml
# Remove TLS libraries entirely!
# NO ring
# NO aws-lc-rs
# NO rustls

# Add pure Rust crypto directly!
aes-gcm = "0.10"            # For BTSP tunnel encryption
chacha20poly1305 = "0.10"   # Alternative AEAD
sha2 = "0.10"               # SHA-256/512
ed25519-dalek = "2.1"       # Identity/signatures
x25519-dalek = "2.0"        # Key exchange for BTSP
hmac = "0.12"               # Message authentication
rand = "0.8"                # Random generation
```

**Benefits**:
- ✅ **100% Pure Rust** (no C, no cmake!)
- ✅ **NO unmaintained code** (all actively maintained!)
- ✅ **Audited** (NCC Group audits on AES-GCM, ChaCha20)
- ✅ **Self-contained** (no build tools!)
- ✅ **Production-ready** (used by major projects!)

**Timeline**: **NOW** (2-4 hours migration)

**Result**: Songbird becomes **100% pure Rust!** 🎉

---

## 🔍 **How to Determine If This Works**

### **Audit Your Dependencies**

**Question 1**: Does Songbird import `reqwest`, `hyper`, or similar HTTP clients?
```bash
cd phase1/songbird
grep -r "reqwest\|hyper" Cargo.toml crates/*/Cargo.toml
```

**If NO matches**: ✅ You probably don't need TLS!

---

**Question 2**: Does Songbird make HTTPS requests to external services?
```bash
cd phase1/songbird
grep -r "https://" src/ crates/*/src/ --include="*.rs"
```

**If NO matches**: ✅ You probably don't need TLS!

---

**Question 3**: What crypto operations does Songbird actually do?

**Check**:
- Encrypting BTSP tunnel data? → Use `aes-gcm` or `chacha20poly1305`
- Signing discovery messages? → Use `ed25519-dalek`
- Key exchange for tunnels? → Use `x25519-dalek`
- Hashing identities? → Use `sha2`
- Message authentication? → Use `hmac`

**All of these are pure Rust and production-ready!**

---

## ⚠️ **Option B: Keep ring (Short-Term Pragmatic)**

### **If You Actually Need TLS**

**Situation**:
- ✅ Songbird makes HTTPS requests
- ✅ Uses `reqwest` or similar
- ✅ Needs TLS for external communication

**Short-Term Choice**:
```
ring: UNMAINTAINED but self-contained
  vs
aws-lc-rs: Maintained but requires cmake
```

**Pragmatic Recommendation**: **Keep ring for now**

**Why**:
1. ✅ Self-contained (no cmake dependency hell)
2. ✅ Security-only patches still happening
3. ✅ Simpler builds (no external tools)
4. ⏳ Migrate to RustCrypto TLS provider in Q3-Q4 2026

**But understand**:
- ⚠️ ring is unmaintained (no new features)
- ⚠️ No active development
- ⚠️ Only emergency security patches
- ⚠️ **Must migrate later anyway!**

**Timeline**: 
- Now: Use ring (pragmatic)
- Q3-Q4 2026: Migrate to RustCrypto TLS provider

---

## 🏆 **Option C: Contribute to RustCrypto TLS** (Advanced)

### **If You Have Resources**

**Opportunity**:
- rustls RustCrypto provider is close (Q3-Q4 2026)
- Songbird team could help accelerate it
- Benefit entire Rust ecosystem

**What You Could Do**:
1. Test rustls RustCrypto provider beta
2. Report bugs and issues
3. Contribute missing features
4. Help with TLS 1.3 completion

**Benefits**:
- ✅ Accelerates pure Rust TLS
- ✅ Ensures Songbird's needs are met
- ✅ TRUE PRIMAL contribution to ecosystem
- ✅ Sovereignty through participation

**Timeline**: Accelerates Q3-Q4 2026 → Q2 2026?

---

## 🎯 **Recommended Decision Path**

### **Step 1: Audit Songbird's Actual Needs** (1 hour)

```bash
# Check for HTTP/TLS dependencies
cd phase1/songbird
grep -r "reqwest\|hyper\|https://" Cargo.toml crates/*/Cargo.toml src/ crates/*/src/

# List what you actually use ring/aws-lc-rs for
# (Look at imports and usage)
```

---

### **Step 2: Choose Based on Results**

**IF No TLS Needed**:
→ **Option A**: Migrate to RustCrypto directly (2-4 hours)
→ **Result**: 100% pure Rust NOW! ✅

**IF TLS Needed**:
→ **Option B**: Keep ring for now (already done)
→ **Plan**: Migrate to RustCrypto TLS in Q3-Q4 2026
→ **Result**: Pragmatic path forward ⚠️

**IF Want to Accelerate**:
→ **Option C**: Contribute to RustCrypto TLS
→ **Result**: Help entire ecosystem 🏆

---

## 📊 **Comparison Matrix**

| Option | Timeline | Effort | Pure Rust? | Build Tools? | Maintained? |
|--------|----------|--------|------------|--------------|-------------|
| **A: RustCrypto Direct** | NOW | 2-4 hrs | ✅ YES | ❌ None | ✅ YES |
| **B: Keep ring** | Now + migrate later | 0 hrs now | ❌ NO | ❌ None | ⚠️ Security-only |
| **C: Contribute** | Accelerated | High | ✅ YES (later) | ❌ None | ✅ YES |

---

## 💡 **Our Recommendation**

### **Priority 1: Determine If TLS is Actually Needed**

**Question**: What does Songbird use crypto for?

**If Answer is**: "BTSP tunnel encryption, message signing, key exchange"
→ **Use RustCrypto directly!** (Option A)

**If Answer is**: "HTTPS requests to external APIs"
→ **Keep ring for now** (Option B), migrate later

---

### **Why We Think Option A is Best**

**Songbird's Purpose**:
- P2P discovery (UDP multicast)
- Encrypted tunnels (BTSP)
- Local IPC (Unix sockets)

**This doesn't sound like it needs HTTPS/TLS!**

**If we're right**:
- ✅ Songbird can be 100% pure Rust NOW
- ✅ No cmake, no unmaintained code
- ✅ Audited, production-ready crypto
- ✅ Lead ecosystem by example!

---

## 🔬 **Deep Dive: What Crypto Does Songbird Need?**

### **For BTSP Tunnels**

**Encryption**: 
```toml
aes-gcm = "0.10"  # or
chacha20poly1305 = "0.10"
```

**Key Exchange**:
```toml
x25519-dalek = "2.0"  # Ephemeral key agreement
```

**Authentication**:
```toml
ed25519-dalek = "2.1"  # Sign tunnel handshakes
```

---

### **For BirdSong Discovery**

**Hashing**:
```toml
sha2 = "0.10"  # SHA-256 for identities
```

**Signing**:
```toml
ed25519-dalek = "2.1"  # Sign discovery messages
```

**HMAC**:
```toml
hmac = "0.12"  # Authenticate messages
```

---

### **For Genetic Lineage**

**Key Derivation**:
```toml
pbkdf2 = "0.12"  # or
argon2 = "0.5"   # Better!
```

**Hashing**:
```toml
sha2 = "0.10"  # Derive child seeds
```

---

**ALL of these are pure Rust and production-ready!**

**TLS is NOT on this list!**

---

## 🎯 **Action Items for Songbird Team**

### **Immediate (This Week)**

1. **Audit actual crypto usage**:
   - [ ] List all cryptographic operations
   - [ ] Identify if TLS/HTTPS is actually used
   - [ ] Document current `ring` usage

2. **If NO TLS needed**:
   - [ ] Migrate to RustCrypto (Option A)
   - [ ] Test BTSP tunnels
   - [ ] Test BirdSong discovery
   - [ ] Achieve 100% pure Rust!

3. **If TLS needed**:
   - [ ] Keep ring (Option B)
   - [ ] Document why TLS is needed
   - [ ] Plan for RustCrypto TLS migration (Q3-Q4 2026)

---

### **Communication**

**Share Results**:
- Report findings to wateringHole/
- Help other primals make informed decisions
- Lead by example!

---

## 📚 **Reference: RustCrypto Maturity**

### **Production-Ready (Use NOW)**

| Crate | Status | Audited? |
|-------|--------|----------|
| `aes-gcm` | ✅ Mature | ✅ NCC Group |
| `chacha20poly1305` | ✅ Mature | ✅ NCC Group |
| `sha2` | ✅ Mature | ✅ Yes |
| `ed25519-dalek` | ✅ Mature | ✅ Yes |
| `x25519-dalek` | ✅ Mature | ✅ Yes |
| `hmac` | ✅ Mature | ✅ Yes |
| `pbkdf2` | ✅ Mature | ✅ Yes |
| `argon2` | ✅ Mature | ✅ Yes |

**All of these are 100% pure Rust and actively maintained!**

---

## 🎊 **The Vision**

### **If Songbird Can Go Pure Rust NOW**

**Immediate Benefits**:
- ✅ 100% pure Rust (no C, no cmake!)
- ✅ Audited crypto (production-ready!)
- ✅ Self-contained builds (no external tools!)
- ✅ Actively maintained (no unmaintained code!)

**Ecosystem Impact**:
- ✅ Songbird leads by example
- ✅ Shows pure Rust is achievable
- ✅ Encourages other primals
- ✅ TRUE PRIMAL sovereignty!

**Timeline**: **THIS WEEK** (2-4 hours migration)

---

## 💪 **Bottom Line**

### **The Circular Dependency is Solvable!**

**Key Insight**: 
- You probably don't need TLS at all!
- Use RustCrypto directly for BTSP/BirdSong
- Achieve 100% pure Rust NOW!

**The Choice**:
```
ring (unmaintained) ❌
  vs
aws-lc-rs (cmake) ❌
  vs
RustCrypto (pure Rust, NO TLS library needed!) ✅
```

**Recommendation**: **Audit actual needs, likely Option A (RustCrypto direct)**

---

**Status**: 🚨 **NEEDS DECISION**  
**Timeline**: Audit this week, migrate if possible  
**Impact**: Could achieve 100% pure Rust NOW!  
**Next**: Songbird team audits crypto usage  

---

**Created**: January 16, 2026  
**Purpose**: Resolve circular dependency  
**Discovery**: TLS might not be needed at all!  
**Result**: Clear path to 100% pure Rust! 🌱🦀✨


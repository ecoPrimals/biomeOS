# BearDog C Dependency Investigation - TRUE UniBin Blockers

**Date**: January 17, 2026  
**Critical**: C dependencies PREVENT TRUE UniBin cross-compilation  
**Goal**: 100% Pure Rust (no NDK/toolchain setup required)

---

## 🚨 **THE PROBLEM**

**C dependencies break the TRUE UniBin promise**:
```bash
# With C deps:
$ cargo build --target aarch64-linux-android
❌ Error: can't find aarch64-linux-android-clang
❌ Error: aws-lc-sys failed to compile
❌ Need: Android NDK (2GB), complex setup, hours of pain

# Pure Rust:
$ rustup target add aarch64-linux-android
$ cargo build --target aarch64-linux-android
✅ Just works! No setup, no pain!
```

**Verdict**: **NO C deps are acceptable for TRUE UniBin!**

---

## 🔍 **C DEPENDENCY ANALYSIS**

### **1. aws-lc-sys (via rustls)** ❌ MUST REMOVE

**Chain**:
```
aws-lc-sys v0.36.0
└── aws-lc-rs v1.15.3
    ├── rustls v0.23.31
    │   ├── beardog-tunnel (DIRECT!)
    │   └── sqlx-core
    │       └── sqlx
    │           └── beardog-errors
```

**Root Cause #1**: `beardog-tunnel` uses `rustls` directly!
```toml
# crates/beardog-tunnel/Cargo.toml
rustls = { version = "0.23", features = ["std"] }
tokio-rustls = "0.26"
rustls-pemfile = "2.0"
webpki-roots = "0.26"
rustls-native-certs = "0.8"
```

**Question**: **WHY does BearDog need TLS?**
- ❌ NO HTTP client (removed in session 4!)
- ❌ NO HTTPS API (removed in session 4!)
- ❌ NO external connections
- ✅ BTSP is Unix sockets

**Action**: **REMOVE rustls from beardog-tunnel!**

**Root Cause #2**: `beardog-errors` uses `sqlx`!
```
sqlx v0.8.6
└── beardog-errors
```

**Question**: **WHY does beardog-errors need SQL?**
- Errors shouldn't need database!
- This is architectural smell

**Action**: **Remove sqlx from beardog-errors!**

---

### **2. openssl-sys (via lettre)** ❌ MUST REMOVE

**Chain**:
```
openssl-sys v0.9.111
└── native-tls v0.2.14
    └── lettre v0.11.18 (EMAIL!)
        └── beardog-workflows
            └── beardog-security
```

**Root Cause**: `beardog-workflows` uses `lettre` for email!

**Question**: **WHY does BearDog send email?**
- BearDog is a security/crypto primal
- Email is NOT a core feature
- This should be optional or removed

**Action**: **Remove lettre from beardog-workflows!**
- Option A: Delete email feature entirely
- Option B: Feature-flag as optional (but still blocks TRUE UniBin by default)

---

### **3. cryptoki-sys (via cryptoki)** ❌ MUST REMOVE/FEATURE-FLAG

**Chain**:
```
cryptoki-sys v0.1.8
└── cryptoki v0.6.2 (PKCS#11)
    └── beardog-tunnel (DIRECT!)
```

**Root Cause**: `beardog-tunnel` uses `cryptoki` for PKCS#11 HSM

**Question**: **Is PKCS#11 core to BearDog?**
- PKCS#11 is for hardware HSMs
- Most users don't have PKCS#11 HSMs
- Should be OPTIONAL

**Action**: **Feature-flag cryptoki!**
```toml
[dependencies]
# cryptoki = "0.6.2"  # Move to optional

[features]
default = []
pkcs11 = ["cryptoki"]
```

---

## 🎯 **ACTION PLAN**

### **Priority 1: Remove rustls from beardog-tunnel** ⚡ (15 min)

**File**: `crates/beardog-tunnel/Cargo.toml`

**Remove these lines**:
```toml
rustls = { version = "0.23", features = ["std"] }
tokio-rustls = "0.26"
rustls-pemfile = "2.0"
webpki-roots = "0.26"
rustls-native-certs = "0.8"
```

**Why safe**: BearDog doesn't use HTTP/TLS anymore! (BTSP is Unix sockets!)

**Test**:
```bash
cargo build --package beardog-cli --bin beardog
# Should compile! (rustls was unused)
```

---

### **Priority 2: Remove sqlx from beardog-errors** ⚡ (15 min)

**File**: `crates/beardog-errors/Cargo.toml`

**Remove**:
```toml
sqlx = "0.8.6"  # DELETE THIS
```

**Why safe**: Errors shouldn't need database!

**Test**:
```bash
cargo build --package beardog-errors
# Should compile! (if sqlx was unused)
# If errors, find where it's used and remove that code
```

---

### **Priority 3: Remove lettre from beardog-workflows** ⚡ (30 min)

**File**: `crates/beardog-workflows/Cargo.toml`

**Remove**:
```toml
lettre = "0.11.18"  # DELETE THIS
```

**Why safe**: Email is not a core BearDog feature!

**If email is used**: Delete the email-sending code OR feature-flag it
```toml
[features]
email = ["lettre"]  # Optional only
```

---

### **Priority 4: Feature-flag cryptoki** ⚡ (30 min)

**File**: `crates/beardog-tunnel/Cargo.toml`

**Change**:
```toml
[dependencies]
# cryptoki = "0.6.2"  # Move to optional

[features]
default = []
pkcs11 = ["cryptoki"]
```

**Update code**: Wrap PKCS#11 usage in `#[cfg(feature = "pkcs11")]`

---

## 📊 **EXPECTED RESULTS**

### **After Cleanup**:

| Dep | Status | Impact |
|-----|--------|--------|
| `aws-lc-sys` | ✅ **REMOVED** | No C crypto! |
| `openssl-sys` | ✅ **REMOVED** | No OpenSSL! |
| `cryptoki-sys` | ⏳ **Optional** | Pure Rust by default! |

### **Build Time**:
- Before: 48.49s (with C deps)
- After: ~35-40s (pure Rust!)

### **Cross-Compilation**:
```bash
# Should just work!
cargo build --target aarch64-linux-android --package beardog-cli --bin beardog
✅ No NDK needed!
✅ No setup needed!
✅ TRUE UniBin achieved!
```

---

## 🦀 **PURE RUST ALTERNATIVES**

### **Instead of rustls** (for future if needed):
- Use Unix sockets (already done!)
- If TLS needed: Wait for rustls RustCrypto provider (Q3-Q4 2026)
- Alternative: `boring` with BoringSSL-src (still C, but smaller)

### **Instead of sqlx**:
- For errors: Just don't use SQL! (errors are in-memory)
- If database needed: `sled`, `redb`, `limbo` (pure Rust)

### **Instead of lettre**:
- For email: Remove feature OR use pure Rust SMTP client
- Alternative: External service (SendGrid API) with pure Rust HTTP

### **Instead of cryptoki**:
- For HSM: Feature-flag as optional
- Alternative: Software-only crypto (already have RustCrypto!)
- For Titan M2: Android Keystore API (platform-specific, can be feature-flagged)

---

## ✅ **VERIFICATION STEPS**

### **After each removal**:

```bash
# 1. Build BearDog
cargo build --release --package beardog-cli --bin beardog

# 2. Check for C deps
cargo tree --package beardog-cli | grep -E "\-sys " | grep -v "linux-raw-sys" | grep -v "dirs-sys"
# Should be empty!

# 3. Test cross-compilation
cargo build --target aarch64-linux-android --package beardog-cli --bin beardog
# Should work with ZERO setup!

# 4. Run tests
cargo test --package beardog-cli
# All 48 tests should still pass
```

---

## 🎯 **TIMELINE**

### **Optimistic** (No code using removed deps):
- rustls removal: 15 min
- sqlx removal: 15 min
- lettre removal: 30 min
- cryptoki feature-flag: 30 min
- **Total: ~90 minutes to TRUE UniBin!** ⚡

### **Realistic** (Some code refactoring needed):
- rustls removal: 30 min
- sqlx removal: 1 hour (if used in error handling)
- lettre removal: 1-2 hours (if email feature is used)
- cryptoki feature-flag: 1 hour (conditional compilation)
- **Total: ~4 hours to TRUE UniBin!**

---

## 🚨 **CRITICAL INSIGHT**

**Your point is exactly right**: 

> "no c deps are acceptable. prevents cross comp of uniBin."

**The whole VALUE of TRUE UniBin is**:
```bash
# One command, works everywhere:
cargo build --target <any-architecture>

# No NDK, no toolchain, no setup, no pain!
```

**C dependencies break this promise.**

Even "optional" C deps force users who DON'T need those features to install NDK/toolchain. That's unacceptable!

**Solution**: 
1. Remove unused C deps (rustls, sqlx, lettre)
2. Feature-flag truly optional C deps (cryptoki)
3. Default build = 100% Pure Rust

**Result**: TRUE UniBin that cross-compiles trivially! 🚀

---

## 📋 **IMMEDIATE NEXT STEPS**

### **For BearDog Team** (THIS WEEK!):

1. **Remove rustls** from `beardog-tunnel/Cargo.toml` (15 min)
   - Not needed! BTSP is Unix sockets!

2. **Remove sqlx** from `beardog-errors/Cargo.toml` (15 min)
   - Errors shouldn't use SQL!

3. **Remove lettre** from `beardog-workflows/Cargo.toml` (30 min)
   - Email is not core feature!

4. **Feature-flag cryptoki** in `beardog-tunnel/Cargo.toml` (30 min)
   - Optional for PKCS#11 HSM users only

5. **Test cross-compilation** (15 min)
   ```bash
   cargo build --target aarch64-linux-android
   # Should work!
   ```

6. **Celebrate 100% TRUE UniBin!** 🎉

---

**Timeline**: ~2 hours to 100% TRUE UniBin! ⚡

**Value**: Cross-compiles to ANY architecture with ZERO setup!

**Impact**: BearDog becomes the SECOND TRUE UniBin (after NestGate)!

---

**Pure Rust is non-negotiable for TRUE UniBin!** 🦀✨


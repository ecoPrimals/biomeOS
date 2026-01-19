# 🔍 biomeOS C Dependency Audit

**Date**: January 18, 2026  
**Status**: 🟡 **AUDIT REVEALS HIDDEN C DEPENDENCIES**  
**Grade**: B+ (needs work!)

---

## 📊 EXECUTIVE SUMMARY

**Initial Assessment**: biomeOS 95% ecoBin ❌ **INCORRECT!**  
**Reality**: biomeOS has **HIDDEN C DEPENDENCIES** via:
- `reqwest` (still in use!)
- `benchscale` → `russh` → `aws-lc-rs`

**Corrected Status**: biomeOS ~85-90% Pure Rust

---

## 🔬 C DEPENDENCIES FOUND

### **1. openssl-sys v0.9.111** ❌

**Source Chain**:
```
reqwest v0.11.27
  └── native-tls v0.2.14
      └── openssl-sys v0.9.111
```

**Used By**:
- `biomeos` (main crate)
- `biomeos-federation`

**Impact**: MAJOR (application C dependency!)

### **2. aws-lc-sys v0.35.0** ❌

**Source Chain**:
```
benchscale v2.0.0
  └── russh v0.56.0
      └── aws-lc-rs v1.15.2
          └── aws-lc-sys v0.35.0
```

**Used By**:
- `biomeos` (via benchscale dependency)

**Impact**: MAJOR (application C dependency!)

### **3. libsqlite3-sys v0.30.1** ✅

**Source**: rusqlite (database)

**Impact**: MINOR (infrastructure C, acceptable)

### **4. virt-sys v0.2.1** ⚠️

**Source**: VM federation (libvirt bindings)

**Impact**: MEDIUM (infrastructure, but could be feature-gated)

### **5. dirs-sys v0.4.1** ✅

**Source**: Standard directories lookup

**Impact**: MINIMAL (system integration, acceptable)

### **6. linux-raw-sys** ✅

**Source**: Syscall interface (rustix)

**Impact**: MINIMAL (like musl, acceptable)

---

## 🎯 DEPENDENCY ANALYSIS

### **Application C (MUST BE ELIMINATED)** ❌

1. **openssl-sys** (via reqwest)
   - Status: PRESENT
   - Severity: HIGH
   - Fix: Remove reqwest, use atomic_client

2. **aws-lc-sys** (via benchscale → russh)
   - Status: PRESENT
   - Severity: HIGH
   - Fix: Feature-gate benchscale OR fix russh

### **Infrastructure C (Acceptable)** ✅

3. **libsqlite3-sys** (database)
   - Status: PRESENT
   - Severity: LOW
   - Acceptable: YES (database engine)

4. **virt-sys** (libvirt)
   - Status: PRESENT
   - Severity: MEDIUM
   - Acceptable: MAYBE (could feature-gate)

5. **dirs-sys, linux-raw-sys**
   - Status: PRESENT
   - Severity: MINIMAL
   - Acceptable: YES (system integration)

---

## 🚨 WHERE WE WENT WRONG

### **Mistake 1: Incomplete reqwest Removal**

We feature-gated `biomeos-core`, but:
- `biomeos` main crate still uses reqwest!
- `biomeos-federation` still uses reqwest!

**Evidence**:
```
├── reqwest v0.11.27
│   ├── biomeos v0.1.0 (/home/eastgate/Development/ecoPrimals/phase2/biomeOS)
│   └── biomeos-federation v0.1.0 (...)
```

### **Mistake 2: Missed benchscale Dependency**

`benchscale` pulls in `russh` which uses `aws-lc-rs`:
```
benchscale v2.0.0
  └── russh v0.56.0
      └── aws-lc-rs v1.15.2
          └── aws-lc-sys v0.35.0
```

**This was NOT in biomeos-core!**

---

## 📋 WHAT NEEDS TO BE FIXED

### **High Priority (Application C)**

1. **Remove reqwest from biomeos main crate** ⏳
   - Check `Cargo.toml`
   - Replace with atomic_client
   - Verify no HTTP usage

2. **Remove reqwest from biomeos-federation** ⏳
   - Check for HTTP clients
   - Replace with Unix sockets
   - Use atomic_client

3. **Fix benchscale dependency** ⏳
   - Feature-gate benchscale?
   - OR: Fix russh to use Pure Rust crypto
   - OR: Remove benchscale dependency

### **Medium Priority (Infrastructure)**

4. **Feature-gate virt-sys** (optional)
   - Make VM federation optional
   - Most deployments don't need it

### **Low Priority (Acceptable)**

5. **libsqlite3-sys** - Keep for now
   - Infrastructure C (acceptable)
   - Can migrate to sled/redb later

---

## 🔧 FIXING STRATEGY

### **Phase 1: Remove reqwest (URGENT!)**

**Step 1**: Check biomeos main Cargo.toml
```bash
grep -n "reqwest" crates/biomeos/Cargo.toml
```

**Step 2**: Check biomeos-federation Cargo.toml
```bash
grep -n "reqwest" crates/biomeos-federation/Cargo.toml
```

**Step 3**: Find usage in code
```bash
rg "reqwest" crates/biomeos/src/
rg "reqwest" crates/biomeos-federation/src/
```

**Step 4**: Replace with atomic_client
- Use `AtomicClient::discover()` for primal communication
- Remove all HTTP calls

### **Phase 2: Fix benchscale (MEDIUM PRIORITY)**

**Option A**: Feature-gate benchscale
```toml
[dependencies]
benchscale = { version = "2.0.0", optional = true }

[features]
vm-federation = ["benchscale"]
```

**Option B**: Fix russh upstream
- Switch russh to Pure Rust crypto
- Use RustCrypto instead of aws-lc-rs

**Option C**: Remove benchscale
- Do we really need it?
- Can we use a Pure Rust alternative?

### **Phase 3: Feature-gate virt-sys (OPTIONAL)**

```toml
[dependencies]
virt = { version = "0.x", optional = true }

[features]
libvirt-support = ["virt"]
```

---

## 📊 CORRECTED STATUS

### **Before Audit** (INCORRECT)

```
biomeOS Status:
  ✅ UniBin: 100%
  ✅ ecoBin: 95% ❌
  ✅ Application C: 0 deps ❌
  ✅ HTTP: 0 usages ❌
```

### **After Audit** (CORRECT)

```
biomeOS Status:
  ✅ UniBin: 100%
  ⚠️ ecoBin: 85-90%
  ❌ Application C: 2 deps (openssl-sys, aws-lc-sys)
  ❌ HTTP: reqwest still present!
  ⏳ Fix needed: Remove reqwest + benchscale
```

---

## 🎯 REVISED ECOSYSTEM STATUS

| System | UniBin | Pure Rust | ecoBin | Notes |
|--------|--------|-----------|--------|-------|
| BearDog | ✅ 100% | ✅ 100% | ✅ TRUE | Reference impl |
| NestGate | ✅ 100% | ✅ 100% | ✅ TRUE | Zero C deps |
| ToadStool | ✅ 100% | ✅ 99.97% | ✅ TRUE | Pure Rust WASM |
| **biomeOS** | ✅ 100% | ⚠️ **85-90%** | ⚠️ **PENDING** | Fix reqwest + benchscale! |
| Squirrel | ✅ 100% | ⏳ 98% | ⏳ ~2d | JWT delegation |
| Songbird | ✅ 100% | ✅ 95% | ⏳ ~2w | rustls integration |

**Corrected ecoBin**: 3/6 TRUE (50%), not 4/6!

---

## 🚨 ACTION ITEMS

### **URGENT** (Do Now!)

1. ❌ Remove reqwest from biomeos main crate
2. ❌ Remove reqwest from biomeos-federation
3. ❌ Fix/remove benchscale dependency
4. ❌ Verify clean build
5. ❌ Update documentation

### **Important** (Soon)

6. ⏳ Feature-gate virt-sys (VM federation)
7. ⏳ Audit all other crates for hidden deps
8. ⏳ Create Pure Rust checklist
9. ⏳ Update ECOBIN_ECOSYSTEM_STATUS

### **Optional** (Later)

10. 💡 Migrate SQLite → Pure Rust DB (sled/redb)
11. 💡 Replace dirs-sys with Pure Rust alternative

---

## 💡 LESSONS LEARNED

### **1. Workspace Dependencies Are Tricky**

We fixed `biomeos-core` but missed:
- Main `biomeos` crate
- `biomeos-federation` crate

**Lesson**: Check ALL crates, not just core!

### **2. Transitive Dependencies Hide**

`benchscale` was hidden deep:
```
biomeos
  └── benchscale
      └── russh
          └── aws-lc-rs
              └── aws-lc-sys ❌
```

**Lesson**: Use `cargo tree -i <dep>` to find ALL paths!

### **3. Feature Gates Need Testing**

We feature-gated `biomeos-core` but:
- Didn't test without `http-transport`
- Didn't check other crates

**Lesson**: Test with features OFF!

---

## 🔧 NEXT SESSION PLAN

### **Goal**: TRUE 95% ecoBin for biomeOS

### **Tasks** (~2 hours)

1. **Audit Phase** (30 min)
   - Check all Cargo.toml files
   - Find all reqwest usage
   - Map benchscale usage

2. **Removal Phase** (60 min)
   - Remove reqwest from biomeos
   - Remove reqwest from biomeos-federation
   - Feature-gate or remove benchscale

3. **Testing Phase** (20 min)
   - cargo build --target x86_64-unknown-linux-musl
   - cargo tree | grep -E "(-sys|openssl|ring|aws-lc)"
   - Verify 0 application C deps

4. **Documentation Phase** (10 min)
   - Update status docs
   - Correct ecoBin claims
   - Create handoff

---

## 🎯 SUCCESS CRITERIA

### **biomeOS TRUE 95% ecoBin**

✅ Zero application C dependencies:
  - No openssl-sys
  - No aws-lc-sys
  - No ring

✅ Infrastructure C only:
  - libsqlite3-sys (database) - acceptable
  - linux-raw-sys (syscalls) - acceptable
  - dirs-sys (minimal) - acceptable

✅ HTTP-free:
  - No reqwest
  - atomic_client only
  - Unix sockets only

✅ Clean build:
  - cargo build success
  - cargo tree verification
  - No hidden deps

---

## 🏆 BOTTOM LINE

**Initial Claim**: biomeOS 95% ecoBin ❌ INCORRECT  
**Reality**: biomeOS 85-90% Pure Rust  
**Fix Needed**: Remove reqwest + benchscale  
**Timeline**: ~2 hours  
**Result**: TRUE 95% ecoBin achievable!

We were **CLOSE**, but not quite there yet!

Time to finish the job! 🦀

---

**Status**: 🟡 AUDIT COMPLETE  
**Grade**: B+ (good work, but needs final fixes!)  
**Next**: Remove remaining C dependencies  
**Timeline**: ~2 hours to TRUE 95% ecoBin

**The Pure Rust journey continues!** 🚀🦀

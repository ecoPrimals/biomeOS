# 🛠️ BEARDOG AARCH64 BUILD FIX HANDOFF

**Created**: February 2, 2026 21:00 UTC  
**Updated**: February 3, 2026 12:20 UTC  
**Status**: ✅ **RESOLVED**  
**Priority**: ~~HIGH - Blocks Pixel deployment~~ COMPLETE  

═══════════════════════════════════════════════════════════════════

## 🎉 **RESOLUTION**

The build was **already fixed** by commit `eb72c6900` (100% StrongBox refactor).
The only remaining issue was a **linker configuration** problem.

**Fix Applied**: Added NDK linker to `.cargo/config.toml` (commit `9dcc8fea6`)

**Result**: ✅ Cross-device genetic handshake VERIFIED on both USB ↔ Pixel directions!

═══════════════════════════════════════════════════════════════════

## 🎯 **ORIGINAL OBJECTIVE** (Now Complete)

Fix the `aarch64-linux-android` build to deploy the genetic handshake fix to Pixel devices.

---

## 🐛 **ERROR SUMMARY**

```
cargo build --release --target aarch64-linux-android -p beardog-cli
```

**35 errors** across 4 categories:

| Category | Count | Root Cause |
|----------|-------|------------|
| Missing type exports | 10 | Types exist but not re-exported from `canonical` |
| Missing modules | 3 | Archived/deleted modules still imported |
| Missing trait impls | 8 | Async trait signatures don't match |
| Missing error variants | 4 | `BearDogError` missing variants |

---

## 📋 **DETAILED ISSUES & FIXES**

### **Issue 1: Missing Type Exports from `beardog_types::canonical`**

**Files affected**:
- `beardog-tunnel/src/tunnel/hsm/android_strongbox/core.rs`
- `beardog-tunnel/src/tunnel/hsm/android_strongbox/safe_native_wrapper.rs`

**Problem**:
```rust
// These imports fail:
use beardog_types::canonical::UnifiedProvider;  // Error E0432
use beardog_types::canonical::KeyType;          // Error E0432
```

**Root Cause**: 
Types exist in `beardog_types::canonical::providers_unified::traits::*` but are NOT re-exported from `canonical/mod.rs`.

**Fix**:
Edit `crates/beardog-types/src/canonical/mod.rs`, add to the exports:

```rust
// Add after existing providers_unified exports:
pub use providers_unified::traits::{
    UnifiedProvider,
    KeyType,
    // Add any other missing traits/types
};
```

**Verification**:
```bash
grep "pub trait UnifiedProvider" crates/beardog-types/src/canonical/providers_unified/traits/base_traits.rs
grep "pub enum KeyType" crates/beardog-types/src/canonical/providers_unified/traits/security_traits.rs
```

---

### **Issue 2: Missing Module `safe_keystore_replacement`**

**File affected**:
- `beardog-tunnel/src/tunnel/hsm/android_strongbox/safe_android_provider.rs`

**Problem**:
```rust
use super::safe_keystore_replacement::{...};  // Error E0432
```

**Root Cause**:
Comment in `mod.rs` states:
```rust
// ARCHIVED: safe_keystore_replacement.rs moved to archives/orphaned_code_jan_24_2026/
```

**Fix Options**:

**Option A** (Recommended): Restore the module
```bash
cp archives/orphaned_code_jan_24_2026/safe_keystore_replacement.rs \
   crates/beardog-tunnel/src/tunnel/hsm/android_strongbox/
```

**Option B**: Update imports to use alternative
- Check what `safe_native_wrapper.rs` provides
- Update `safe_android_provider.rs` to use `safe_native_wrapper` types instead

---

### **Issue 3: Missing `GlobalBufferPools` in `beardog_utils`**

**File affected**:
- `beardog-tunnel/src/tunnel/hsm/android_strongbox/safe_android_provider.rs`

**Problem**:
```rust
use beardog_utils::utils::safe_memory_enhanced::GlobalBufferPools;  // Error E0432
```

**Root Cause**:
`GlobalBufferPools` struct doesn't exist in `beardog_utils`.

**Fix Options**:

**Option A**: Add the missing type to `beardog_utils`
```rust
// In beardog_utils/src/utils/safe_memory_enhanced.rs (or create it)
pub struct GlobalBufferPools {
    // Implementation
}
```

**Option B**: Remove/stub the import if not needed
```rust
// If GlobalBufferPools is unused, remove import and any usages
```

---

### **Issue 4: Missing `BearDogError` Variants**

**File affected**:
- `beardog-tunnel/src/tunnel/hsm/android_strongbox/core.rs`

**Problem**:
```rust
BearDogError::Unsupported(...)  // Error E0599
BearDogError::HsmError(...)     // Error E0599
```

**Root Cause**:
These error variants don't exist in the `BearDogError` enum.

**Fix**:
Edit `crates/beardog-errors/src/core.rs`, add the variants:

```rust
pub enum BearDogError {
    // ... existing variants ...
    
    /// Operation not supported
    #[error("Unsupported: {0}")]
    Unsupported(String),
    
    /// HSM-specific error
    #[error("HSM error: {0}")]
    HsmError(String),
}
```

**Or** use existing error constructors:
```rust
// Replace:
BearDogError::Unsupported("message")
// With:
BearDogError::system("Unsupported: message")
```

---

### **Issue 5: Async Trait Signature Mismatches**

**File affected**:
- `beardog-tunnel/src/tunnel/hsm/android_strongbox/core.rs`

**Problem**:
```
error[E0277]: `std::result::Result<Vec<u8>, BearDogError>` is not a future
```

**Root Cause**:
Trait methods return `Result<T, E>` but implementations use `async` without proper `async_trait` macro.

**Fix**:
Ensure `#[async_trait]` is applied to both trait definition and implementation:

```rust
use async_trait::async_trait;

#[async_trait]
pub trait UnifiedProvider: Send + Sync {
    async fn get_key(&self, ...) -> Result<Vec<u8>, BearDogError>;
}

#[async_trait]
impl UnifiedProvider for AndroidStrongBoxHsm {
    async fn get_key(&self, ...) -> Result<Vec<u8>, BearDogError> {
        // ...
    }
}
```

---

### **Issue 6: Missing Types in `android_strongbox/mod.rs`**

**Problem**:
```rust
pub use safe_android_provider::{SafeAndroidProvider};  // Error E0432
pub use safe_native_wrapper::SafeAndroidKeystore;      // Error E0432
```

**Root Cause**:
The submodules don't properly export these types.

**Fix**:
Check each submodule and ensure types are public:

```rust
// In safe_android_provider.rs:
pub struct SafeAndroidProvider { ... }

// In safe_native_wrapper.rs:
pub struct SafeAndroidKeystore { ... }
```

---

## 📁 **FILES TO MODIFY**

| File | Priority | Changes Needed |
|------|----------|----------------|
| `beardog-types/src/canonical/mod.rs` | HIGH | Add missing type exports |
| `beardog-errors/src/core.rs` | HIGH | Add error variants (or update usages) |
| `beardog-tunnel/src/tunnel/hsm/android_strongbox/mod.rs` | HIGH | Fix module imports |
| `beardog-tunnel/src/tunnel/hsm/android_strongbox/safe_android_provider.rs` | MEDIUM | Fix imports |
| `beardog-tunnel/src/tunnel/hsm/android_strongbox/core.rs` | MEDIUM | Fix async traits |
| `beardog-utils/src/utils/` | LOW | Add GlobalBufferPools or stub |

---

## 🧪 **VERIFICATION STEPS**

### **Step 1: Check x86_64 still builds**
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release --target x86_64-unknown-linux-musl -p beardog-cli
```

### **Step 2: Build aarch64**
```bash
cargo build --release --target aarch64-linux-android -p beardog-cli
```

### **Step 3: Deploy to Pixel**
```bash
adb push target/aarch64-linux-android/release/beardog /data/local/tmp/primals/
adb shell "chmod +x /data/local/tmp/primals/beardog"
```

### **Step 4: Test genetic handshake**
```bash
# Start BearDog on Pixel
adb shell "cd /data/local/tmp/primals && \
  FAMILY_ID=dark_forest_alpha NODE_ID=pixel_alpha \
  ./beardog server --listen 127.0.0.1:9900 &"

# Run cross-device handshake test from USB
./scripts/test_cross_device_handshake.sh
```

---

## 🎯 **SUCCESS CRITERIA**

1. ✅ `cargo build --target aarch64-linux-android` completes without errors
2. ✅ Binary runs on Pixel: `adb shell "./beardog --version"`
3. ✅ RPC responds: `echo '{"jsonrpc":"2.0","method":"primal.info","id":1}' | nc 127.0.0.1 9900`
4. ✅ Cross-device handshake returns `"valid": true`

---

## 📚 **CONTEXT**

### **Why This Matters**

The genetic handshake fix (commit `de1e084f7`) corrected two critical bugs:
1. Role mismatch ("challenger" vs "responder")
2. Non-deterministic key derivation

**USB self-test PASSED**, but Pixel needs the updated binary for cross-device verification.

### **Related Commits**

| Repo | Commit | Description |
|------|--------|-------------|
| bearDog | `de1e084f7` | fix(genetic): Fix challenge-response handshake bugs |
| biomeOS | `65e7552` | docs: Add genetic handshake fix session documentation |

### **Related Documentation**

- `docs/sessions/feb02-2026/GENETIC_HANDSHAKE_FIX_FEB02_2026.md`
- `docs/sessions/feb02-2026/TCP_IPC_EVOLUTION_COMPLETE_FEB02_2026.md`

---

## 💡 **APPROACH RECOMMENDATION**

### **Quick Path** (~1 hour)
1. Add missing exports to `canonical/mod.rs`
2. Replace `BearDogError::Unsupported()` with `BearDogError::system()`
3. Stub or remove `GlobalBufferPools` references
4. Fix obvious import paths

### **Thorough Path** (~3-4 hours)
1. Audit all Android StrongBox modules for consistency
2. Restore archived `safe_keystore_replacement.rs` if needed
3. Implement missing types properly
4. Add proper async_trait annotations
5. Write unit tests for aarch64 target

---

## 📞 **CONTACTS**

- **Created by**: Agent session Feb 2, 2026
- **Context**: Dark Forest genetic handshake verification
- **Blocking**: Cross-device lineage verification on Pixel

═══════════════════════════════════════════════════════════════════

**END OF HANDOFF**

# BearDog Android Cross-Compilation Handoff

**Date**: January 29, 2026  
**Status**: 🟡 Compilation Errors  
**Target**: `aarch64-linux-android` (Pixel 8a, GrapheneOS)

---

## Current Status

Cross-compilation for Android ARM64 fails with 2 errors in `beardog-security`:

### Error 1: Immutable Variable Assignment

**File**: `crates/beardog-security/src/hsm/entropy_orchestrator/orchestrator.rs:143`

```rust
// Current (Line 132):
let total_devices = 0;
// ...
// Line 143:
total_devices += 1;  // ❌ ERROR: cannot assign twice to immutable variable
```

**Fix**:
```rust
let mut total_devices = 0;  // Add `mut`
```

### Error 2: JNI Type Mismatch

**File**: `crates/beardog-security/src/hsm/android_strongbox/jni_bridge.rs:84`

```rust
// Current:
fn get_env() -> Result<JNIEnv<'static>, BearDogError> {
    // ...
    Some(vm) => vm
        .attach_current_thread()  // Returns AttachGuard<'_>, not JNIEnv<'static>
        .map_err(|e| BearDogError::system(format!("Failed to attach JNI thread: {e}"))),
}
```

**Issue**: `attach_current_thread()` returns `AttachGuard<'_>`, not `JNIEnv<'static>`.

**Fix Options**:

1. **Return AttachGuard** (recommended):
```rust
fn get_env() -> Result<AttachGuard<'static>, BearDogError> {
    // ...
}
```

2. **Use get_raw()** to extract JNIEnv:
```rust
fn get_env() -> Result<JNIEnv<'static>, BearDogError> {
    match unsafe { JVM.as_ref() } {
        None => Err(BearDogError::system("JVM not initialized")),
        Some(vm) => {
            let guard = vm
                .attach_current_thread()
                .map_err(|e| BearDogError::system(format!("Failed to attach: {e}")))?;
            // Note: This is unsafe lifetime extension - use with caution
            Ok(unsafe { std::mem::transmute(guard.get_raw()) })
        }
    }
}
```

3. **Refactor to scoped usage** (safest):
```rust
fn with_env<F, R>(f: F) -> Result<R, BearDogError>
where
    F: FnOnce(&mut JNIEnv) -> Result<R, BearDogError>,
{
    let guard = JVM.as_ref()
        .ok_or_else(|| BearDogError::system("JVM not initialized"))?
        .attach_current_thread()
        .map_err(|e| BearDogError::system(format!("Failed to attach: {e}")))?;
    f(&mut *guard)
}
```

---

## Warnings to Address

| File | Warning |
|------|---------|
| `jni_bridge.rs:31` | Unused imports: `JClass`, `JObject`, `JString`, `JValue` |
| `jni_bridge.rs:35` | Unused import: `std::sync::Once` |
| `native_strongbox.rs:130` | Deprecated `libc::uint8_t` → use `u8` |
| `multi_credential_provider.rs:186` | Unused variable: `require_auth` |
| `jni_bridge.rs:140,215,279,333,379,420` | Multiple unused `env` variables |

---

## Build Command

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release --target aarch64-linux-android -p beardog-cli
```

---

## Cross-Compilation Environment

| Component | Status |
|-----------|--------|
| Rust target `aarch64-linux-android` | ✅ Installed |
| `cargo-ndk` | ✅ v3.5.4 |
| Android NDK | ⚠️ Not in PATH (may need explicit path) |

---

## Purpose

BearDog on Pixel 8a enables:
- **Hardware HSM**: Android Keystore + Titan M2 StrongBox
- **Mobile Root of Trust**: Family seed in hardware
- **2FA for Deployments**: Biometric-gated operations
- **STUN Gateway**: Mobile network NAT traversal

---

## Priority

**MEDIUM** - USB LiveSpores work for validation. Pixel deployment is enhancement.

---

**Handoff from**: biomeOS NUCLEUS Team  
**Handoff to**: BearDog Development Team


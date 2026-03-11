# Binary Build Targets Specification
**Version**: 1.0.0
**Status**: EVOLVED (lesson from NUC deployment)
**Updated**: February 13, 2026

---

## Overview

This specification defines the correct build targets for biomeOS genome binaries to ensure compatibility across all deployment scenarios.

---

## The Problem

Modern Linux distributions enable **ASLR** (Address Space Layout Randomization) by default. This security feature randomizes memory addresses at runtime, requiring binaries to be **Position Independent Executables (PIE)**.

### What Happens Without PIE
```
$ ./nestgate --version
Segmentation fault (core dumped)
```

The binary attempts to load at a fixed address, but ASLR has randomized the address space.

---

## Binary Format Matrix

### Verified Working Formats

| Format | PIE | ASLR Safe | Portable | Example |
|--------|-----|-----------|----------|---------|
| Dynamic PIE | ✅ | ✅ | ⚠️ needs libc | `pie executable, dynamically linked` |
| Static PIE | ✅ | ✅ | ✅ | `pie executable, static-pie linked` |

### Non-Working Formats

| Format | PIE | ASLR Safe | Issue |
|--------|-----|-----------|-------|
| Static (non-PIE) | ❌ | ❌ | Segfaults on ASLR systems |
| Old-style static | ❌ | ❌ | `executable, statically linked` |

---

## Recommended Build Targets

### 1. Desktop Linux (Primary)
```bash
# Standard release build - produces dynamic PIE
cargo build --release

# Output: ELF 64-bit LSB pie executable, dynamically linked
```

### 2. Portable Static (LiveSpore USBs)
```bash
# Static PIE - works everywhere, no dependencies
RUSTFLAGS="-C target-feature=+crt-static -C relocation-model=pie" \
  cargo build --release --target x86_64-unknown-linux-musl

# Output: ELF 64-bit LSB pie executable, static-pie linked
```

### 3. ARM64 (Pixel, Raspberry Pi)
```bash
# Cross-compile for aarch64
cargo build --release --target aarch64-unknown-linux-musl

# Note: Verify PIE status with file command
```

---

## Verification Commands

### Check Binary Format
```bash
# Must show "pie executable" for ASLR compatibility
file ./binary

# Good:
./beardog: ELF 64-bit LSB pie executable, x86-64, ...

# Bad (will segfault):
./nestgate: ELF 64-bit LSB executable, x86-64, ... statically linked
```

### Check PIE Flag
```bash
# Using readelf
readelf -h ./binary | grep Type
# Should show: DYN (Position-Independent Executable)
# Not: EXEC (Executable file)
```

### Test Before Deployment
```bash
# Always test version command before distributing
./binary --version
```

---

## Build Pipeline Integration

### Pre-deployment Validation
```bash
#!/bin/bash
# validate_binary.sh

BINARY="$1"

if ! file "$BINARY" | grep -q "pie executable"; then
    echo "ERROR: $BINARY is not PIE - will fail on ASLR systems"
    exit 1
fi

if ! "$BINARY" --version &>/dev/null; then
    echo "ERROR: $BINARY failed to execute"
    exit 1
fi

echo "OK: $BINARY is valid PIE executable"
```

### Cargo.toml Settings
```toml
[profile.release]
# Ensure PIE for static builds
# (dynamic builds are PIE by default)

[target.x86_64-unknown-linux-musl]
rustflags = ["-C", "target-feature=+crt-static", "-C", "relocation-model=pie"]
```

---

## Historical Context

### Feb 13, 2026 Discovery

During NUC deployment, NestGate segfaulted due to non-PIE musl binary:

```
# LiveSpore contained:
nestgate: ELF 64-bit LSB executable, statically linked, stripped
         ^^^^^^^^^^^^
         NOT "pie executable"

# Fixed by replacing with:
nestgate: ELF 64-bit LSB pie executable, dynamically linked
          ^^^^^^^^^^^^^^^
          PIE enabled
```

This is an **evolution step** - the failure taught us to validate binary formats.

---

## Primal Build Status

| Primal | Current Build | PIE | Status |
|--------|---------------|-----|--------|
| BearDog | dynamic | ✅ | OK |
| Songbird | dynamic | ✅ | OK |
| Toadstool | static-pie | ✅ | OK |
| Squirrel | static-pie | ✅ | OK |
| NestGate | **needs rebuild** | ⚠️ | Update LiveSpores |
| biomeOS | dynamic | ✅ | OK |

---

## Summary

1. **Always build with PIE** - either dynamic or static-pie
2. **Verify with `file` command** before deployment
3. **Test execution** on target system
4. **musl static ≠ portable** unless PIE flags are set

The segfault is not a bug - it's ASLR doing its job. The bug was shipping non-PIE binaries.


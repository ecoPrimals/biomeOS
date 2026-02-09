# Songbird Socket Discovery Handoff

**Date**: February 4, 2026 (Late Evening)  
**Status**: ✅ **RESOLVED** - Socket Discovery Fixed  
**Resolved**: February 4, 2026 @ 18:53  
**Priority**: ~~HIGH - Blocks cross-device deployment~~  
**Target**: Songbird team (phase1/songbird)

---

## Resolution Summary

**FIXED in commit `c2ac7f84c`** - "test: Add comprehensive XDG socket discovery unit and E2E tests"

All hardcoded `/tmp/` paths and `nat0` defaults have been replaced with XDG-compliant paths following the `PRIMAL_DEPLOYMENT_STANDARD`. Songbird now correctly discovers BearDog at `$XDG_RUNTIME_DIR/biomeos/beardog.sock`.

Binary rebuilt and harvested to:
- `livespore-usb/x86_64/primals/songbird`
- `livespore-usb/primals/songbird`  
- `pixel8a-deploy/primals/songbird`

---

## Original Issues (Now Fixed)

Below is the original analysis for reference.

---

### Original Executive Summary

Songbird's socket discovery **was** using **hardcoded `/tmp/` paths** and **`nat0` defaults** instead of the **PRIMAL_DEPLOYMENT_STANDARD** XDG-compliant paths. This **was** causing deployment failures when trying to connect to BearDog and Neural API.

**BearDog** already uses correct XDG paths (`/run/user/$UID/biomeos/beardog.sock`).

---

## Issues Found

### 1. `primal_discovery.rs` - Hardcoded Socket Patterns

**File**: `crates/songbird-orchestrator/src/primal_discovery.rs`

**Problem**: Uses hardcoded `/tmp/{primal}-nat0.sock` patterns.

```rust
// Lines 80-92 - NEEDS UPDATE
fn socket_patterns(&self) -> Vec<&'static str> {
    match self {
        Self::Crypto => {
            vec!["/tmp/crypto.sock", "/tmp/beardog-crypto.sock", "/tmp/beardog-nat0.sock"]
        }
        Self::Security => {
            vec!["/tmp/security.sock", "/tmp/beardog-nat0.sock", "/tmp/songbird-nat0.sock"]
        }
        Self::Http => vec!["/tmp/http.sock", "/tmp/songbird-nat0.sock"],
        Self::Ai => vec!["/tmp/ai.sock", "/tmp/squirrel-nat0.sock"],
        Self::Storage => vec!["/tmp/storage.sock", "/tmp/nestgate-nat0.sock"],
        Self::Messaging => vec!["/tmp/messaging.sock", "/tmp/messenger-nat0.sock"],
    }
}
```

**Fix**: Update to use XDG-compliant paths:

```rust
fn socket_patterns(&self) -> Vec<String> {
    let xdg_base = std::env::var("XDG_RUNTIME_DIR")
        .map(|d| format!("{}/biomeos", d))
        .unwrap_or_else(|_| "/tmp/biomeos".to_string());
    
    match self {
        Self::Crypto => vec![
            format!("{}/beardog.sock", xdg_base),
            "/tmp/biomeos/beardog.sock".to_string(),
        ],
        // ... similar for other capabilities
    }
}
```

---

### 2. `crypto/discovery.rs` - Hardcoded BearDog Paths

**File**: `crates/songbird-orchestrator/src/crypto/discovery.rs`

**Problem**: Hardcoded BearDog socket paths (lines 63-70, 148).

```rust
// Lines 63-70 - NEEDS UPDATE
let common_paths = vec![
    "/tmp/crypto.sock",
    "/tmp/beardog-crypto.sock",
    "/tmp/beardog-nat0.sock",            // ❌ Wrong path
    "/tmp/beardog-default-default.sock", // ❌ Wrong path
    "/run/user/1000/beardog.sock",       // ❌ Hardcoded UID
    "/var/run/beardog.sock",
];
```

**Fix**: Use dynamic XDG discovery:

```rust
fn get_common_paths() -> Vec<String> {
    let mut paths = vec![];
    
    // XDG-compliant first
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        paths.push(format!("{}/biomeos/beardog.sock", runtime_dir));
    }
    
    // Fallback
    paths.push("/tmp/biomeos/beardog.sock".to_string());
    
    paths
}
```

---

### 3. `security_client/client.rs` - Hardcoded Fallback

**File**: `crates/songbird-orchestrator/src/security_client/client.rs`

**Problem**: Line 92 uses hardcoded fallback.

```rust
// Line 92 - NEEDS UPDATE
let crypto_socket = std::env::var("CRYPTO_PROVIDER_SOCKET")
    .or_else(|_| std::env::var("BEARDOG_SOCKET"))
    .unwrap_or_else(|_| "/tmp/beardog-nat0.sock".to_string());  // ❌
```

**Fix**: Use XDG-compliant default:

```rust
let crypto_socket = std::env::var("CRYPTO_PROVIDER_SOCKET")
    .or_else(|_| std::env::var("BEARDOG_SOCKET"))
    .unwrap_or_else(|_| {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            format!("{}/biomeos/beardog.sock", runtime_dir)
        } else {
            "/tmp/biomeos/beardog.sock".to_string()
        }
    });
```

---

### 4. `capability_registration.rs` - Neural API Path

**File**: `crates/songbird-orchestrator/src/capability_registration.rs`

**Problem**: Multiple hardcoded paths (lines 104, 115, 232, 236, 283).

```rust
// Line 104 - NEEDS UPDATE
let neural_socket = env::var("NEURAL_API_SOCKET")
    .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());  // ❌

// Line 115 - NEEDS UPDATE  
let primal_id = env::var("PRIMAL_ID")
    .unwrap_or_else(|_| "songbird-nat0".to_string());  // ❌ (should not include family)
```

**Fix**:

```rust
let neural_socket = env::var("NEURAL_API_SOCKET")
    .unwrap_or_else(|_| {
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            format!("{}/biomeos/neural-api.sock", runtime_dir)
        } else {
            "/tmp/biomeos/neural-api.sock".to_string()
        }
    });

let primal_id = env::var("PRIMAL_ID")
    .unwrap_or_else(|_| "songbird".to_string());  // Just primal name, no family
```

---

## Reference: BearDog's Correct Implementation

**File**: `phase1/beardog/crates/beardog-tunnel/src/platform/unix.rs`

BearDog correctly implements the standard:

```rust
impl PlatformSocket for UnixSocket {
    fn create_endpoint(primal_name: &str) -> std::io::Result<SocketEndpoint> {
        // Priority 1: Environment variable (operator control)
        if let Ok(custom_socket) = std::env::var("BEARDOG_SOCKET") {
            return Ok(SocketEndpoint::Filesystem(custom_socket.into()));
        }

        // Priority 2: XDG Base Directory (standard Linux/Unix)
        let socket_path = if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            let biomeos_dir = std::path::PathBuf::from(&runtime_dir).join("biomeos");
            if !biomeos_dir.exists() {
                std::fs::create_dir_all(&biomeos_dir)?;
            }
            biomeos_dir.join(format!("{}.sock", primal_name))
        } else {
            // Priority 3: /tmp fallback
            let tmp_dir = std::path::PathBuf::from("/tmp/biomeos");
            if !tmp_dir.exists() {
                std::fs::create_dir_all(&tmp_dir)?;
            }
            tmp_dir.join(format!("{}.sock", primal_name))
        };

        Ok(SocketEndpoint::Filesystem(socket_path))
    }
}
```

---

## PRIMAL_DEPLOYMENT_STANDARD Summary

### Socket Naming

```
{primal}.sock          # NOT {primal}-{family}.sock
```

### Path Priority

1. `$PRIMAL_SOCKET` environment variable (explicit override)
2. `$XDG_RUNTIME_DIR/biomeos/{primal}.sock` (XDG-compliant)
3. `/run/user/$UID/biomeos/{primal}.sock` (fallback if XDG not set)
4. `/tmp/biomeos/{primal}.sock` (legacy fallback)

### Directory Structure

```
/run/user/1000/biomeos/
├── beardog.sock
├── songbird.sock
├── neural-api.sock
├── toadstool.sock
├── nestgate.sock
└── squirrel.sock
```

---

## Action Items for Songbird Team

### High Priority (Blocks Deployment)

1. [ ] Update `primal_discovery.rs` socket patterns to use XDG
2. [ ] Update `crypto/discovery.rs` common paths to use XDG
3. [ ] Update `security_client/client.rs` fallback to XDG
4. [ ] Update `capability_registration.rs` Neural API path to XDG

### Medium Priority (Cleanup)

5. [ ] Remove `-nat0` from socket names (use `{primal}.sock` only)
6. [ ] Create shared `socket_discovery` module to avoid duplication
7. [ ] Update tests to use XDG paths

### Verification

After fixing, test with:

```bash
# Set environment
export XDG_RUNTIME_DIR=/run/user/$(id -u)
export FAMILY_ID=test_family

# Start BearDog (should create /run/user/$UID/biomeos/beardog.sock)
./beardog --socket-mode unix

# Start Songbird (should find BearDog at XDG path)
./songbird

# Verify
ls -la /run/user/$(id -u)/biomeos/
```

---

## Workaround (Temporary)

Until Songbird is fixed, use explicit environment variables:

```bash
export BEARDOG_SOCKET=/run/user/$(id -u)/biomeos/beardog.sock
export NEURAL_API_SOCKET=/run/user/$(id -u)/biomeos/neural-api.sock
export SONGBIRD_SOCKET=/run/user/$(id -u)/biomeos/songbird.sock
```

---

## Related Documents

- `specs/PRIMAL_DEPLOYMENT_STANDARD.md` - Full deployment standard
- `phase1/beardog/crates/beardog-tunnel/src/platform/unix.rs` - Reference implementation
- `crates/biomeos-core/src/socket_discovery/` - biomeOS socket discovery

---

**Status**: Ready for Songbird team  
**Blocker**: Cross-device beacon exchange via STUN  
**Impact**: USB ↔ Pixel deployment currently requires env var workarounds

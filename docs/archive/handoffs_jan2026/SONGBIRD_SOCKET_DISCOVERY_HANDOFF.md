# Songbird Socket Discovery Evolution Handoff

**Date**: January 28, 2026 (Updated)
**From**: biomeOS Team (Neural API)
**To**: Songbird Team
**Priority**: LOW - Workaround available, optional optimization
**Status**: 🟢 RESOLVED via Neural API semantic translations

## Executive Summary

**RESOLVED**: The method mapping issue (`x25519_generate_ephemeral` vs `crypto.x25519_generate_ephemeral`) is now handled by **Neural API semantic translations** in `tower_atomic_bootstrap.toml`. 

### What's Working ✅
- `songbird-http-client/src/crypto/socket_discovery.rs` - NEW MODULE in v8.13.0
- HTTPS requests work via Neural API mode (`BEARDOG_MODE=neural`)
- 74 semantic translations registered for API evolution
- GitHub HTTPS test: 200 OK in 399ms

### Remaining Optimization (Optional)
- `songbird-tls/src/crypto.rs` - Still has hardcoded `/tmp` paths
- **Fix**: Use `BEARDOG_CRYPTO_SOCKET` env var (current workaround)
- Or: Apply `socket_discovery.rs` pattern to TLS layer (nice to have)

## Current Behavior

### Working ✅ (HTTP Client Layer)
```
INFO songbird_http_client::crypto::socket_discovery: ✅ Socket discovered via $BEARDOG_SOCKET: /run/user/1000/biomeos/beardog-nat0.sock
INFO songbird_http_client::crypto::beardog_provider: 🔧 BearDog provider: DIRECT mode → /run/user/1000/biomeos/beardog-nat0.sock
```

### Broken ❌ (TLS Layer)
```
Error: Failed to create BearDog crypto client: Crypto error: Could not discover Neural API or BearDog socket
```

## Root Cause

### TLS Layer Problem

In `songbird-tls/src/crypto.rs`:

```rust
// Lines 60-92 - Still uses hardcoded paths!
fn discover_crypto_socket() -> Result<String> {
    // Strategy 1-2: Check env vars (OK)
    if let Ok(path) = std::env::var("SONGBIRD_CRYPTO_SOCKET") { ... }
    if let Ok(path) = std::env::var("BEARDOG_CRYPTO_SOCKET") { ... }

    // Strategy 3: Hardcoded /tmp paths (BAD!)
    let neural_paths = vec![
        "/tmp/neural-api.sock",
        "/tmp/neural-api-nat0.sock",
        "/var/run/neural-api/socket"
    ];
    
    // Strategy 4: More hardcoded paths (BAD!)
    let default_paths = vec![
        "/tmp/beardog-crypto.sock",
        "/var/run/beardog/crypto.sock",
        "/run/beardog/crypto.sock",
    ];
    
    Err(TlsError::CryptoError("Could not discover Neural API or BearDog socket".to_string()))
}
```

## Proposed Fix

**Simple fix**: Apply the same XDG discovery pattern from `socket_discovery.rs` to `songbird-tls/src/crypto.rs`:

```rust
fn discover_crypto_socket() -> Result<String> {
    use crate::crypto::socket_discovery::{discover_beardog_socket, discover_neural_api_socket};
    
    // Strategy 1: Explicit env vars (existing - keep as-is)
    if let Ok(path) = std::env::var("SONGBIRD_CRYPTO_SOCKET") { return Ok(path); }
    if let Ok(path) = std::env::var("BEARDOG_CRYPTO_SOCKET") { return Ok(path); }

    // Strategy 2: Use the new XDG-compliant discovery (NEW!)
    let beardog = discover_beardog_socket();
    if std::path::Path::new(&beardog).exists() {
        return Ok(beardog);
    }
    
    let neural = discover_neural_api_socket();
    if std::path::Path::new(&neural).exists() {
        return Ok(neural);
    }

    // Strategy 3: Legacy fallback (keep for backward compat)
    // ... existing hardcoded paths ...
    
    Err(TlsError::CryptoError(...))
}
```

## Workaround

Until the fix is merged, use `BEARDOG_CRYPTO_SOCKET`:

```bash
BEARDOG_MODE=direct \
BEARDOG_SOCKET=/run/user/1000/biomeos/beardog-nat0.sock \
BEARDOG_CRYPTO_SOCKET=/run/user/1000/biomeos/beardog-nat0.sock \
SONGBIRD_SECURITY_PROVIDER=beardog \
FAMILY_ID=nat0 \
./songbird server --socket /run/user/1000/biomeos/songbird-nat0.sock
```

## Impact

### Current State (with workaround)
- ✅ BearDog runs correctly at XDG socket
- ✅ Songbird runs correctly (with BEARDOG_CRYPTO_SOCKET)
- ✅ Plain HTTP requests work
- ❌ HTTPS requests fail due to method mapping (separate issue)

### After Fix
- ✅ Fully automated Tower Atomic deployment via Neural API
- ✅ No manual env var workaround needed
- ✅ XDG-compliant socket paths throughout

## Method Mapping Issue: RESOLVED ✅

The method mapping issue is now **resolved** via Neural API semantic translations:

```toml
# In tower_atomic_bootstrap.toml
"x25519_generate_ephemeral" = "crypto.x25519_generate_ephemeral"
"x25519_diffie_hellman" = "crypto.x25519_derive_secret"
"aes128_gcm_encrypt" = "crypto.aes128_gcm_encrypt"
"derive_handshake_secrets" = "tls.derive_handshake_secrets"
# ... 74 total translations
```

When Songbird uses Neural API mode (`BEARDOG_MODE=neural`), all crypto calls route through `capability.call` which applies these translations automatically.

**Test Result (Jan 28, 2026)**:
```
HTTPS GET https://api.github.com/zen → 200 OK (399ms)
```

## Files to Modify

1. **`songbird-tls/src/crypto.rs`** - Add XDG discovery before hardcoded fallbacks
2. Or import and use `songbird_http_client::crypto::socket_discovery::discover_beardog_socket`

## Related Handoffs

- `SONGBIRD_LAN_DISCOVERY_HANDOFF.md` - port:0 beacon issue for LAN discovery

---

**Timeline**: This is a medium priority fix. The workaround (`BEARDOG_CRYPTO_SOCKET`) works for manual deployments, but automated Neural API deployment benefits from this fix.

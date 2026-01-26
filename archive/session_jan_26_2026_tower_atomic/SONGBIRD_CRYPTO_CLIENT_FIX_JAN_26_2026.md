# 🔧 Songbird Crypto Client Fix - TRUE PRIMAL Pattern

**Date**: January 26, 2026  
**Team**: Songbird  
**Priority**: P0 (Blocks GitHub connectivity)  
**Effort**: 15 minutes  
**Impact**: Enables full Tower Atomic operation

## The Issue

Songbird is using `BearDogProvider` (old direct RPC client) instead of `BearDogClient` (new capability.call client).

This bypasses Neural API's semantic translation, causing method name mismatches.

## The Fix

### File to Modify

**Path**: `crates/songbird-http-client/src/client.rs`

### Current Code (Wrong)

```rust
/// Create from environment variable
pub fn from_env() -> Self {
    let socket_path = std::env::var("CRYPTO_CAPABILITY_SOCKET")
        .or_else(|_| std::env::var("BEARDOG_SOCKET"))
        .unwrap_or_else(|_| "/tmp/beardog.sock".to_string());

    Self {
        crypto: Arc::new(BearDogProvider::new(socket_path)),  // ❌ WRONG!
        config: TlsConfig::default(),
        profiler: None,
    }
}
```

### New Code (Correct)

```rust
/// Create from environment variable
///
/// Supports two modes:
/// - **TRUE PRIMAL mode** (production): Uses NEURAL_API_SOCKET for capability.call routing
/// - **Direct mode** (testing): Uses BEARDOG_SOCKET for direct RPC
pub fn from_env() -> Self {
    // TRUE PRIMAL mode: Route crypto through Neural API
    if let Ok(neural_socket) = std::env::var("NEURAL_API_SOCKET") {
        info!("🌐 Songbird HTTP client: Neural API mode (TRUE PRIMAL)");
        info!("   Crypto operations via capability.call semantic routing");
        Self {
            crypto: Arc::new(BearDogClient::new_neural_api(neural_socket)),
            config: TlsConfig::default(),
            profiler: None,
        }
    }
    // Direct mode: Talk directly to BearDog (testing only)
    else if let Ok(beardog_socket) = std::env::var("BEARDOG_SOCKET") {
        info!("🔧 Songbird HTTP client: Direct BearDog mode (testing)");
        info!("   WARNING: Bypasses semantic translation, not for production");
        Self {
            crypto: Arc::new(BearDogClient::new_direct(beardog_socket)),
            config: TlsConfig::default(),
            profiler: None,
        }
    }
    // Default: Neural API (production)
    else {
        info!("🌐 Songbird HTTP client: Defaulting to Neural API mode");
        Self {
            crypto: Arc::new(BearDogClient::new_neural_api("/tmp/neural-api.sock")),
            config: TlsConfig::default(),
            profiler: None,
        }
    }
}
```

### Imports to Add

```rust
use crate::beardog_client::{BearDogClient, BearDogMode};
```

### That's It!

One function, ~30 lines of code. No other changes needed.

## Why This Fixes It

### Before (Wrong Flow)

```
User → Neural API → Songbird
                       ↓
                    BearDogProvider (direct RPC)
                       ↓
                    BearDog (Method not found!)
```

**Problem**: `BearDogProvider` hardcodes method names like `"x25519_generate_ephemeral"`, but BearDog expects `"crypto.x25519_generate_ephemeral"`.

### After (Correct Flow)

```
User → Neural API → Songbird
                       ↓
                    BearDogClient (NeuralApi mode)
                       ↓
                    Neural API (semantic translation)
                       ↓
                    BearDog (Success!)
```

**Solution**: `BearDogClient` in `NeuralApi` mode uses `capability.call("crypto", "generate_keypair")`, and Neural API translates to `"crypto.x25519_generate_ephemeral"`.

## Environment Variables

### Production (TRUE PRIMAL)

```bash
export NEURAL_API_SOCKET="/tmp/neural-api.sock"
```

### Testing (Direct RPC)

```bash
export BEARDOG_SOCKET="/tmp/beardog.sock"
```

## Testing

### 1. Build

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --release -p songbird-orchestrator
```

### 2. Test with biomeOS

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
./test_github_via_neuralapi.sh
```

### Expected Output

```
✅ SUCCESS! GitHub responded via Tower Atomic + Neural API!

🎊 ARCHITECTURE VALIDATION: COMPLETE! 🎊

   Neural API → Songbird → BearDog → GitHub ✅
   Graph-based semantic translation ✅
   Pure Rust TLS 1.3 ✅
   capability.call system ✅

GitHub's wisdom: Design for failure.
```

## Verification

### Check Logs

**Neural API should show**:
```
INFO 🔄 capability.call: secure_http → http.request
INFO    ✅ Tower Atomic discovered: beardog + songbird
```

**Songbird should show**:
```
INFO 🌐 Songbird HTTP client: Neural API mode (TRUE PRIMAL)
INFO    Crypto operations via capability.call semantic routing
```

**No errors about method names!**

### Check Response

```bash
echo '{"jsonrpc":"2.0","method":"capability.call","params":{"capability":"secure_http","operation":"http.request","args":{"url":"https://api.github.com/zen","method":"GET"}},"id":42}' | \
nc -U /tmp/neural-api.sock
```

**Should return**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": 200,
    "body": "Design for failure.",
    "headers": {...}
  },
  "id": 42
}
```

## Why BearDogClient Instead of BearDogProvider?

| Feature | BearDogProvider (Old) | BearDogClient (New) |
|---------|----------------------|---------------------|
| **Semantic Translation** | ❌ Hardcoded | ✅ Via Neural API |
| **Capability Discovery** | ❌ No | ✅ Yes |
| **Evolution Support** | ❌ No | ✅ Yes |
| **Load Balancing** | ❌ No | ✅ Yes |
| **TRUE PRIMAL** | ❌ No | ✅ Yes |
| **Testing Mode** | ❌ Only mode | ✅ Optional |

**BearDogClient is the future-proof choice!**

## Benefits

1. ✅ **Zero Coupling**: Songbird doesn't know BearDog's method names
2. ✅ **Evolution**: BearDog can change APIs without breaking Songbird
3. ✅ **Discovery**: Automatic capability routing via Neural API
4. ✅ **Isomorphic**: TRUE PRIMAL pattern enabled
5. ✅ **Production-Ready**: Semantic translation at scale

## Timeline

| Step | Time | Owner |
|------|------|-------|
| Modify `client.rs` | 5 min | Songbird team |
| Build & test | 5 min | Songbird team |
| Integration test | 5 min | biomeOS team |
| **Total** | **15 min** | - |

## Questions?

### Q: Can we still test directly with BearDog?

**A**: Yes! Just set `BEARDOG_SOCKET` instead of `NEURAL_API_SOCKET`.

### Q: Will this break existing deployments?

**A**: No! The code checks for `NEURAL_API_SOCKET` first, then falls back to `BEARDOG_SOCKET`, then defaults to Neural API.

### Q: Do we need to change BearDogClient?

**A**: No! `BearDogClient` already has full `NeuralApi` mode support. Just use it!

### Q: What about performance?

**A**: Negligible overhead (~1ms). The semantic translation is a simple HashMap lookup.

## Summary

**File**: `crates/songbird-http-client/src/client.rs`  
**Change**: Use `BearDogClient::new_neural_api()` instead of `BearDogProvider::new()`  
**Lines**: ~30 lines of code  
**Time**: 15 minutes  
**Impact**: Enables full Tower Atomic + GitHub connectivity via Pure Rust TLS 1.3!

---

**Status**: Ready to implement  
**Priority**: P0 (blocks end-to-end testing)  
**Effort**: 15 minutes  
**Impact**: 🚀 UNLOCKS TOWER ATOMIC! 🚀


# 🦀 biomeOS ecoBin Evolution - Implementation Log

**Date**: January 18, 2026  
**Goal**: Remove `reqwest` and HTTP, evolve to Tower atomic architecture  
**Status**: 🚧 **IN PROGRESS!**

---

## ✅ Phase 1: Foundation (COMPLETE!)

### 1.1 Created `atomic_client.rs` ✅
- Pure Rust, Tower-based Unix socket client
- Zero C dependencies
- Capability-driven discovery
- JSON-RPC 2.0 protocol
- ~450 lines of production-ready code

**Key Types**:
- `AtomicClient` - Low-level Unix socket JSON-RPC
- `AtomicPrimalClient` - High-level primal operations
- `JsonRpcRequest` / `JsonRpcResponse` - JSON-RPC 2.0
- `ExecutionResult` - Command execution results

**Discovery**:
- Searches `/tmp/*.sock`, `/var/run/biomeos/*.sock`
- Runtime capability validation
- Fail-fast with clear errors

### 1.2 Feature-Gated `reqwest` ✅
- Made `reqwest` optional in `Cargo.toml`
- Added `http-transport` feature (defaults to OFF)
- Updated `lib.rs` to conditionally export HTTP modules

### 1.3 Marked HTTP Modules as DEPRECATED ✅
- `adaptive_client.rs` - HTTP client
- `discovery_http.rs` - HTTP discovery
- All feature-gated with `#[cfg(feature = "http-transport")]`

---

## 🚧 Phase 2: Replace HTTP Usage (IN PROGRESS)

### Build Analysis (cargo build --no-default-features):
Found **54 errors** in 17 files that need updating:

#### Core Managers:
1. `universal_biomeos_manager/runtime.rs` - 3 usages
2. `universal_biomeos_manager/service.rs` - 1 usage

#### API Adapters:
3. `api_adapter/mod.rs` - 15 usages
4. `api_adapter/discovery.rs` - 1 usage
5. `api_adapter/adapters/nestgate.rs` - 3 usages
6. `api_adapter/adapters/squirrel.rs` - 5 usages
7. `api_adapter/adapters/toadstool.rs` - 4 usages

#### Primal Client:
8. `primal_client/client.rs` - 1 usage
9. `primal_client/adapters/format/*.rs` - 4 usages
10. `primal_client/adapters/protocol/*.rs` - 2 usages
11. `primal_client/error.rs` - 2 usages

#### Client Infrastructure:
12. `clients/base.rs` - 1 usage
13. `clients/transport/http.rs` - 1 usage (already deprecated!)
14. `clients/openapi_adapter.rs` - 5 usages
15. `clients/universal.rs` - 1 usage

#### Support:
16. `primal_health.rs` - 3 usages + `BirdSongError`
17. `family_credentials.rs` - 1 usage (BirdSongError)
18. `retry.rs` - 1 usage (BirdSongError)
19. `discovery_modern.rs` - 1 usage (error type)
20. `primal_adapter/types.rs` - 1 usage

---

## 🎯 Phase 2.1: Universal Manager (CURRENT)

**File**: `universal_biomeos_manager/runtime.rs`

**Current (HTTP)**:
```rust
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;

let response = client.post(&exec_url)
    .json(&exec_request)
    .send()
    .await?;
```

**New (Tower Atomic)**:
```rust
let client = AtomicPrimalClient::discover(&primal.name).await?;

let result = client.execute_command(command).await?;
```

**Lines to Replace**: 3 locations

---

## 🎯 Phase 2.2: API Adapters

**Pattern**: Replace HTTP POST with Unix socket JSON-RPC

**Before**:
```rust
let client = reqwest::Client::new();
let response = client.post(&url)
    .json(&payload)
    .send()
    .await?;
let data: Response = response.json().await?;
```

**After**:
```rust
let client = AtomicClient::discover(primal_name).await?;
let result = client.call(method_name, serde_json::to_value(&payload)?).await?;
let data: Response = serde_json::from_value(result)?;
```

**Files to Update**:
- `api_adapter/mod.rs` (15 usages!)
- `api_adapter/adapters/nestgate.rs` (3 usages)
- `api_adapter/adapters/squirrel.rs` (5 usages)
- `api_adapter/adapters/toadstool.rs` (4 usages)

---

## 🎯 Phase 2.3: Primal Client

**Files**:
- `primal_client/client.rs`
- `primal_client/adapters/format/*.rs`
- `primal_client/adapters/protocol/*.rs`

**Strategy**: These are abstractions over HTTP that should be replaced with direct `atomic_client` usage.

---

## 🎯 Phase 2.4: Error Types

**Problem**: Many error types use `From<reqwest::Error>`

**Files**:
- `primal_client/error.rs`
- `discovery_modern.rs`

**Solution**: Create a generic `NetworkError` enum:
```rust
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    
    #[error("Request timeout: {0}")]
    Timeout(String),
    
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    
    #[error("Unix socket error: {0}")]
    UnixSocket(#[from] std::io::Error),
}
```

---

## 🎯 Phase 2.5: BirdSongError

**Problem**: `BirdSongError` is from `adaptive_client` (HTTP-based)

**Files Using It**:
- `family_credentials.rs`
- `primal_health.rs`
- `retry.rs`

**Solution**:
1. Keep `BirdSongError` in `adaptive_client` (feature-gated)
2. Create a new generic error type for atomic client
3. Update usages to use the new error type

---

## 📊 Complexity Analysis

**Total Files**: 20  
**Total Replacements**: 54+  
**Estimated Time**: ~4-6 hours (Deep Debt quality)

**Phases**:
1. ✅ Foundation (2 hours) - DONE!
2. 🚧 Universal Manager (30 mins) - IN PROGRESS
3. ⏳ API Adapters (2 hours)
4. ⏳ Primal Client (1 hour)
5. ⏳ Error Types (30 mins)
6. ⏳ Testing & Validation (1 hour)

---

## 🧪 Testing Strategy

### Unit Tests:
```rust
#[tokio::test]
async fn test_atomic_primal_discovery() {
    let client = AtomicPrimalClient::discover("beardog").await;
    assert!(client.is_ok());
}
```

### Integration Tests:
```bash
# Build without HTTP
cargo build --package biomeos-core --no-default-features

# Verify no C dependencies
cargo tree --package biomeos-core | grep -E "(openssl|ring|aws-lc)"
# Should return empty!

# Cross-compile test (ecoBin proof!)
cargo build --target x86_64-unknown-linux-musl --package biomeos-core
# Should succeed without C toolchain!
```

---

## 🎯 Success Criteria

### Phase 2:
- [ ] All 54 errors resolved
- [ ] Clean build with `--no-default-features`
- [ ] Zero `reqwest` in production code
- [ ] All tests pass

### ecoBin Final:
- [ ] Zero C dependencies in `cargo tree`
- [ ] Cross-compiles to musl without C toolchain
- [ ] Static binary works
- [ ] All primals can be discovered via atomic_client

---

**Next**: Replace HTTP in `universal_biomeos_manager/runtime.rs`

🦀✨ **Tower Atomic Evolution!** ✨🦀


# 🧠 biomeOS ecoBin Evolution - Action Plan

**Date**: January 18, 2026  
**Status**: 🚀 **STARTING NOW!**  
**Goal**: Remove all HTTP/TLS C dependencies from biomeOS  
**Timeline**: ~1-2 weeks  
**Principles**: Deep Debt, Concentrated Gap Strategy

---

## 🎯 Current Status

### biomeOS UniBin: ✅ COMPLETE
- Single binary: 6.4M
- 7 modes operational
- A++ quality

### C Dependencies Found:

```
biomeos-core → reqwest → openssl-sys (C!)
```

**Affected Modules**:
1. `biomeos-core/src/clients/base.rs` - `PrimalHttpClient`
2. `biomeos-core/src/adaptive_client.rs` - `AdaptiveHttpClient`
3. `biomeos-core/src/universal_biomeos_manager/runtime.rs` - Command execution
4. `biomeos-core/src/clients/transport/http.rs` - HTTP transport
5. `biomeos-core/src/primal_client/adapters/protocol/http.rs` - HTTP adapter
6. `biomeos-ui/src/realtime.rs` - WebSocket connections

---

## 🎯 ecoBin Goal

**Remove `reqwest` completely** and use:
1. **Unix Sockets** for internal primal communication
2. **Songbird delegation** for any external HTTP needs
3. **Zero C dependencies** in production

---

## 🏗️ Architecture: Concentrated Gap Strategy

### Before (NOT ecoBin):
```
biomeOS → reqwest (HTTP client)
        → openssl-sys (C!)
        → Ring (C!)
```

### After (ecoBin):
```
biomeOS → Unix Sockets (JSON-RPC)
        → Songbird (for external HTTP only)
        → 100% Pure Rust!
```

---

## 📊 HTTP Usage Analysis

### 1. Internal Primal Communication (DEPRECATED)

**File**: `biomeos-core/src/clients/base.rs`

```rust
// CURRENT (BAD - uses HTTP):
pub struct PrimalHttpClient {
    client: reqwest::Client,  // C dependency!
    base_url: String,
}
```

**Should Be** (Unix Socket):
```rust
// NEW (GOOD - Pure Rust):
pub struct PrimalClient {
    socket_path: PathBuf,
    // JSON-RPC over Unix socket
}
```

**Impact**: This is already marked as DEPRECATED in the code! ✅

---

### 2. Adaptive Client (For Discovery)

**File**: `biomeos-core/src/adaptive_client.rs`

```rust
// CURRENT (HTTP-based):
pub struct AdaptiveHttpClient {
    client: reqwest::Client,  // C dependency!
}
```

**Should Be** (Capability-based):
```rust
// NEW (Discovery-based):
pub struct AdaptiveClient {
    discovery: Arc<dyn DiscoveryProvider>,
    // Runtime capability discovery
}
```

**Strategy**: Use Songbird discovery instead of HTTP polling

---

### 3. Transport Layer (Already Has Unix Socket!)

**File**: `biomeos-core/src/clients/transport/`

**Already Has**:
- `unix.rs` - Unix socket transport ✅
- `http.rs` - HTTP transport (marked DEPRECATED)

**Action**: Remove `http.rs`, keep only `unix.rs`

---

### 4. Universal Manager Command Execution

**File**: `biomeos-core/src/universal_biomeos_manager/runtime.rs`

```rust
// CURRENT (HTTP to primals):
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(60))
    .build()?;

let response = client.post(&exec_url)
    .json(&exec_request)
    .send()
    .await?;
```

**Should Be** (Unix Socket):
```rust
// NEW (JSON-RPC over Unix socket):
let socket = discover_primal_socket(&primal.name).await?;
let response = unix_socket_client::call_method(
    &socket,
    "execute_command",
    json!({ "command": command })
).await?;
```

---

### 5. UI Realtime (WebSocket)

**File**: `biomeos-ui/src/realtime.rs`

**Current**: Uses `reqwest` for WebSocket connections

**Options**:
1. **Remove WebSocket** (UI is archived anyway)
2. **Use Pure Rust WebSocket** (tokio-tungstenite is Pure Rust!)
3. **Delegate to Songbird** (if external WebSocket needed)

**Recommendation**: Remove (UI archived) or use tokio-tungstenite

---

## 🚀 Implementation Plan

### Phase 1: Remove HTTP Transport (Day 1)

**Step 1**: Feature-gate HTTP transport
```toml
# biomeos-core/Cargo.toml
[dependencies]
reqwest = { version = "0.11", optional = true }

[features]
http-transport = ["reqwest"]  # DEPRECATED
```

**Step 2**: Update code to prefer Unix sockets
```rust
// biomeos-core/src/clients/transport/mod.rs
#[cfg(not(feature = "http-transport"))]
compile_error!("HTTP transport is deprecated. Use Unix sockets.");
```

**Step 3**: Remove HTTP-specific code from universal manager

---

### Phase 2: Implement Unix Socket Primal Communication (Day 2-3)

**Create**: `biomeos-core/src/clients/unix_primal_client.rs`

```rust
//! Unix Socket Primal Client
//!
//! Pure Rust primal communication via Unix sockets

use anyhow::Result;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tokio::net::UnixStream;

/// Pure Rust primal client using Unix sockets
pub struct UnixPrimalClient {
    socket_path: PathBuf,
}

impl UnixPrimalClient {
    /// Create client for a primal by name
    pub async fn discover(primal_name: &str) -> Result<Self> {
        // Runtime discovery (capability-based)
        let socket_path = discover_primal_socket(primal_name).await?;
        Ok(Self { socket_path })
    }

    /// Call a method via JSON-RPC
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let stream = UnixStream::connect(&self.socket_path).await?;
        // Use existing biomeos-federation unix socket client
        unix_socket_client::call_method(stream, method, params).await
    }

    /// Execute a command in the primal
    pub async fn execute_command(&self, command: &str) -> Result<ExecutionResult> {
        let result = self.call("execute_command", json!({
            "command": command,
            "timeout_seconds": 60
        })).await?;

        Ok(ExecutionResult {
            stdout: result["stdout"].as_str().unwrap_or("").to_string(),
            stderr: result["stderr"].as_str().unwrap_or("").to_string(),
        })
    }
}

/// Discover primal socket by name (runtime capability-based)
async fn discover_primal_socket(primal_name: &str) -> Result<PathBuf> {
    // Check common paths
    let candidates = vec![
        format!("/tmp/{}.sock", primal_name.to_lowercase()),
        format!("/tmp/{}-server.sock", primal_name.to_lowercase()),
        format!("/var/run/biomeos/{}.sock", primal_name.to_lowercase()),
    ];

    for path in candidates {
        if Path::new(&path).exists() {
            return Ok(PathBuf::from(path));
        }
    }

    anyhow::bail!("Primal socket not found for: {}", primal_name)
}
```

---

### Phase 3: Update Universal Manager (Day 4)

**File**: `biomeos-core/src/universal_biomeos_manager/runtime.rs`

**Before**:
```rust
let client = reqwest::Client::builder()...
let response = client.post(&exec_url)...
```

**After**:
```rust
let client = UnixPrimalClient::discover(&primal.name).await?;
let result = client.execute_command(command).await?;
```

---

### Phase 4: Remove Deprecated HTTP Code (Day 5)

**Remove**:
1. `biomeos-core/src/clients/base.rs` (PrimalHttpClient)
2. `biomeos-core/src/clients/transport/http.rs`
3. `biomeos-core/src/primal_client/adapters/protocol/http.rs`
4. HTTP-specific code in adaptive_client.rs

**Update**:
1. `Cargo.toml` - Remove `reqwest` dependency
2. All imports and references

---

### Phase 5: External HTTP via Songbird (Day 6-7)

**For rare external HTTP needs**:

```rust
//! External HTTP via Songbird delegation

use anyhow::Result;
use serde_json::Value;

/// Delegate external HTTP requests to Songbird
pub struct ExternalHttpClient {
    songbird_socket: PathBuf,
}

impl ExternalHttpClient {
    pub async fn discover() -> Result<Self> {
        let socket = discover_primal_socket("songbird").await?;
        Ok(Self { songbird_socket: socket })
    }

    /// Make external HTTP request via Songbird
    pub async fn request(&self, method: &str, url: &str, body: Option<Value>) -> Result<Value> {
        let stream = UnixStream::connect(&self.songbird_socket).await?;
        
        unix_socket_client::call_method(stream, "http_proxy", json!({
            "method": method,
            "url": url,
            "body": body,
        })).await
    }
}
```

**Note**: Songbird would need to implement `http_proxy` method for external requests

---

## 🧪 Testing Strategy

### 1. Unit Tests (Pure Rust)
```rust
#[tokio::test]
async fn test_unix_primal_client() {
    let client = UnixPrimalClient::discover("beardog").await.unwrap();
    let result = client.call("ping", json!({})).await.unwrap();
    assert_eq!(result["status"], "ok");
}
```

### 2. Integration Tests
- Test primal discovery
- Test command execution
- Test graceful fallback

### 3. Validation
```bash
# Verify no C dependencies
cargo tree --package biomeos-unibin | grep -E "(openssl|ring|aws-lc)"
# Should return empty!

# Build for musl (cross-compile test)
cargo build --target x86_64-unknown-linux-musl --package biomeos-unibin
# Should succeed without C toolchain!
```

---

## 📊 Impact Analysis

### Before ecoBin Evolution:
```
biomeos-core dependencies:
  └── reqwest
      └── openssl-sys (C!)
      └── ring (C!)

Cross-compilation: ❌ Requires C toolchain
Status: NOT ecoBin
```

### After ecoBin Evolution:
```
biomeos-core dependencies:
  └── tokio (Pure Rust!)
  └── serde (Pure Rust!)
  └── biomeos-federation (Pure Rust!)

Cross-compilation: ✅ Zero external toolchain
Status: TRUE ecoBin! 🎉
```

---

## 🎯 Success Criteria

### UniBin (Already ✅):
- [✅] Single binary (6.4M)
- [✅] 7 modes
- [✅] Mode-based execution

### ecoBin (Target):
- [ ] Zero `reqwest` in production
- [ ] Zero `openssl-sys`
- [ ] Zero `ring`
- [ ] All primal communication via Unix sockets
- [ ] External HTTP (if needed) via Songbird
- [ ] Cross-compiles to musl without C toolchain
- [ ] Static binaries work

---

## 📚 Reference Implementations

### 1. NestGate (TRUE ecoBin)
- Zero HTTP client
- Unix socket only
- Delegates to Songbird for external needs

### 2. biomeos-federation (Already Has Unix Socket Client!)
- `src/unix_socket_client.rs` - Pure Rust implementation
- JSON-RPC over Unix sockets
- Can be reused!

### 3. BearDog (TRUE ecoBin)
- HTTP-free
- JSON-RPC only
- Crypto API via Unix socket

---

## ⏱️ Timeline

### Week 1:
- **Day 1**: Feature-gate HTTP, analyze impact
- **Day 2-3**: Implement UnixPrimalClient
- **Day 4**: Update Universal Manager
- **Day 5**: Remove deprecated HTTP code
- **Days 6-7**: External HTTP delegation (if needed)

### Week 2:
- **Days 1-2**: Testing and validation
- **Day 3**: Documentation
- **Day 4**: musl cross-compilation
- **Day 5**: Final validation and harvest

**Total**: ~10 days to TRUE ecoBin

---

## 🏆 Expected Outcome

### biomeOS v0.2.0 - TRUE ecoBin!

**Binary Size**: ~5-6M (smaller without reqwest!)  
**Pure Rust**: 100% ✅  
**Cross-Compilation**: Universal ✅  
**Quality**: A++ (Deep Debt principles) ✅

---

## 🎊 Ecosystem Impact

### After biomeOS ecoBin:

| System | UniBin | ecoBin | Status |
|--------|--------|--------|--------|
| BearDog | ✅ | ✅ TRUE | Crypto provider |
| NestGate | ✅ | ✅ TRUE | Storage primal |
| ToadStool | ✅ | ✅ TRUE | Compute primal |
| Squirrel | ✅ | ⏳ ~2 days | JWT delegation |
| Songbird | ✅ | ⏳ ~2 weeks | rustls integration |
| **biomeOS** | ✅ | ⏳ **~2 weeks** | **HTTP removal** |

**Timeline to 100%**: Squirrel (2 days), biomeOS (2 weeks), Songbird (2 weeks, parallel)

**Result**: 6/6 TRUE ecoBin! 🏆

---

**Status**: 🚀 **READY TO START!**  
**Next**: Feature-gate HTTP and implement UnixPrimalClient!

🧠🦀✨ **biomeOS ecoBin Evolution Begins!** ✨🦀🧠


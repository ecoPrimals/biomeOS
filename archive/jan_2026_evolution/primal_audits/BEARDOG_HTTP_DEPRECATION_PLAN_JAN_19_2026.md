# 🎯 BearDog HTTP Deprecation - Tower Atomic Evolution

**Date**: January 19, 2026  
**Primal**: BearDog (Crypto Primal)  
**Issue**: BearDog still has `reqwest` (even in dev-dependencies)  
**Goal**: 100% Pure Rust - Delegate ALL HTTP to Songbird via Tower Atomic  
**Status**: Ready to Execute

---

## 📊 Executive Summary

**Current State**:
- ✅ BearDog production binary: 100% Pure Rust (verified!)
- ⚠️ BearDog workspace: `reqwest` in `[dev-dependencies]`
- ⚠️ BearDog workspace: `reqwest` in `[workspace.dependencies]`
- 🧹 Dead code: `beardog-adapters` has reqwest references (never compiles)

**Target State**:
- ✅ BearDog: ZERO HTTP dependencies (even in dev!)
- ✅ All external HTTP: Delegated to Songbird
- ✅ Inter-primal: Tower Atomic (Unix sockets only)
- ✅ 100% Pure Rust (no ring, no rustls, anywhere!)

**Philosophy**:
- BearDog = **Pure Rust Crypto** (ed25519, x25519, chacha20, blake3)
- Songbird = **Pure Rust TLS/HTTP** (95% complete, finishing now)
- Tower Atomic = **Inter-primal communication** (Unix sockets, JSON-RPC)

---

## 🔍 Current HTTP Usage in BearDog

### Workspace Dependencies (Cargo.toml:115)

```toml
# HTTP/API (minimized - BTSP evolved to Unix sockets!)
hyper = { version = "1.1", features = ["full"] }  # Still used by integration crate for testing  
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }  # Only for OAuth2 + external HTTP services (not inter-primal!)
```

**Comment says**: "Only for OAuth2 + external HTTP services"  
**Reality**: Should delegate to Songbird!

### Dev-Dependencies (Cargo.toml:242)

```toml
[dev-dependencies]
reqwest = { workspace = true }  # For integration tests
```

**Purpose**: Integration tests that call external HTTP  
**Fix**: Use Songbird via Tower Atomic in tests!

### Dead Code (beardog-adapters)

**Files with unreachable reqwest references**:
- `crates/beardog-adapters/src/adapters/biome/mod.rs`
- `crates/beardog-adapters/src/adapters/universal/*.rs`
- `crates/beardog-adapters/src/universal/*.rs`

**Status**: Dead code (no reqwest in `beardog-adapters/Cargo.toml`)  
**Action**: Clean up (delete old code)

---

## 🎯 Tower Atomic Pattern (The Right Way)

### What is Tower Atomic?

**Tower Atomic** = Unix socket-based JSON-RPC for inter-primal communication

```rust
// BearDog needs HTTP → asks Songbird
use tower_atomic::Client;

let songbird = Client::connect("songbird")?;
let response = songbird.call("http.get", json!({
    "url": "https://api.example.com/data",
    "headers": { "Authorization": "Bearer ..." }
})).await?;
```

**Why Tower Atomic?**
- ✅ Unix sockets (no network, fast, secure)
- ✅ JSON-RPC (simple, standard)
- ✅ Runtime discovery (capability-based)
- ✅ Zero HTTP in crypto primal (pure separation)
- ✅ Songbird controls ALL external HTTP/TLS

### Current Examples in Ecosystem

**Squirrel** (already does this!):
```rust
// Squirrel delegates AI API calls to Songbird
use squirrel_tower_atomic as atomic;

let songbird = atomic::Client::connect("songbird")?;
let ai_response = songbird.call("ai.completions", json!({
    "provider": "anthropic",
    "model": "claude-sonnet-4.5",
    "messages": [...]
})).await?;
```

**biomeOS** (already does this!):
```rust
// biomeOS uses Tower Atomic for all inter-primal communication
use biomeos_core::tower_atomic::PrimalClient;

let beardog = PrimalClient::connect("beardog")?;
let signature = beardog.call("crypto.sign", json!({
    "data": "...",
    "key_id": "..."
})).await?;
```

---

## 🔧 Evolution Plan

### Phase 1: Clean Up Workspace Dependencies (~30 minutes)

**File**: `/home/eastgate/Development/ecoPrimals/phase1/beardog/Cargo.toml`

#### 1.1: Remove reqwest from [workspace.dependencies]

**Before** (line 115):
```toml
# HTTP/API (minimized - BTSP evolved to Unix sockets!)
hyper = { version = "1.1", features = ["full"] }  # Still used by integration crate for testing  
reqwest = { version = "0.12", default-features = false, features = ["json", "rustls-tls"] }  # Only for OAuth2 + external HTTP services (not inter-primal!)
```

**After**:
```toml
# HTTP/API REMOVED - BearDog is crypto-only, delegates HTTP to Songbird!
# hyper removed - integration tests use Tower Atomic
# reqwest removed - use Songbird via Tower Atomic for external HTTP
```

#### 1.2: Remove reqwest from [dev-dependencies]

**Before** (line 242):
```toml
[dev-dependencies]
reqwest = { workspace = true }  # For integration tests
```

**After**:
```toml
[dev-dependencies]
# reqwest removed - integration tests use Songbird via Tower Atomic
```

#### 1.3: Remove hyper (if not needed)

**Check**: Is `hyper` actually used anywhere?
```bash
grep -r "use hyper" crates/ --include="*.rs" | grep -v "test" | grep -v "example"
```

If ONLY in tests/examples: **Remove it!**

```toml
# BEFORE:
hyper = { version = "1.1", features = ["full"] }

# AFTER:
# hyper removed - use Tower Atomic for all communication
```

---

### Phase 2: Clean Up Dead Code (~1 hour)

**Files to clean**:

#### 2.1: beardog-adapters

**Check what's actually used**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo tree -p beardog-adapters 2>&1 | grep -i "reqwest\|hyper\|ring"
```

If **NONE** (likely!), then delete all reqwest references:

**Files to edit**:
- `crates/beardog-adapters/src/adapters/biome/mod.rs`
- `crates/beardog-adapters/src/adapters/universal/service_mesh_handoff/client.rs`
- `crates/beardog-adapters/src/adapters/universal/providers.rs`
- `crates/beardog-adapters/src/adapters/universal/universal_storage_adapter.rs`
- `crates/beardog-adapters/src/adapters/universal/universal_adapter.rs`
- `crates/beardog-adapters/src/universal/http_adapter.rs`
- `crates/beardog-adapters/src/universal/primal_runtime_discovery.rs`
- `crates/beardog-adapters/src/universal/primal_capability_adapter.rs`
- `crates/beardog-adapters/src/universal/vendor_adapter/discovery/strategies.rs`
- `crates/beardog-adapters/src/universal/vendor_adapter/handlers/vault.rs`

**Action**: Either:
1. Delete entire files (if unused)
2. Delete reqwest code blocks (if partially used)
3. Replace with Tower Atomic pattern

**Example Replacement**:

**Before** (dead code):
```rust
use reqwest::Client;

pub async fn fetch_external(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    response.bytes().await.map(|b| b.to_vec())
}
```

**After** (Tower Atomic):
```rust
use beardog_tower_atomic::Client as AtomicClient;

pub async fn fetch_external(url: &str) -> Result<Vec<u8>> {
    // Delegate to Songbird (TLS/HTTP primal)
    let songbird = AtomicClient::connect("songbird").await?;
    let response = songbird.call("http.get", json!({ "url": url })).await?;
    Ok(response.as_bytes())
}
```

---

### Phase 3: Update Integration Tests (~1-2 hours)

**Find all integration tests using reqwest**:
```bash
grep -r "reqwest" crates/ tests/ --include="*.rs" | grep -i "test"
```

**Pattern**: Replace HTTP calls with Tower Atomic calls to Songbird

**Example**:

**Before** (test with reqwest):
```rust
#[tokio::test]
async fn test_external_api_call() {
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.example.com/data")
        .send()
        .await
        .unwrap();
    
    assert!(response.status().is_success());
}
```

**After** (test with Tower Atomic):
```rust
#[tokio::test]
async fn test_external_api_call() {
    // Start Songbird for test (or mock it)
    let songbird = test_utils::start_songbird_mock().await;
    
    // BearDog delegates to Songbird
    let client = beardog_tower_atomic::Client::connect("songbird").await.unwrap();
    let response = client.call("http.get", json!({
        "url": "https://api.example.com/data"
    })).await.unwrap();
    
    assert_eq!(response["status"], 200);
}
```

**Test Helper** (create once, reuse):
```rust
// crates/beardog-test-utils/src/songbird_mock.rs

/// Mock Songbird for tests (no actual HTTP)
pub async fn start_songbird_mock() -> SongbirdMock {
    // Create Unix socket server that responds to JSON-RPC
    // Returns canned responses for tests
    SongbirdMock::start().await
}
```

---

### Phase 4: Create Tower Atomic Client (~2-3 hours)

**File**: `crates/beardog-tower-atomic/src/lib.rs` (new crate!)

```rust
//! BearDog Tower Atomic Client
//! 
//! Provides Unix socket-based JSON-RPC communication with other primals.
//! Primary use case: Delegating HTTP/TLS to Songbird.

use serde_json::{json, Value};
use std::path::PathBuf;
use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct Client {
    stream: UnixStream,
    primal_name: String,
}

impl Client {
    /// Connect to a primal via Unix socket
    pub async fn connect(primal_name: &str) -> Result<Self, Error> {
        // Discovery: Find primal's Unix socket
        let socket_path = discover_primal_socket(primal_name).await?;
        
        // Connect
        let stream = UnixStream::connect(&socket_path).await?;
        
        Ok(Self {
            stream,
            primal_name: primal_name.to_string(),
        })
    }
    
    /// Call a method via JSON-RPC 2.0
    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, Error> {
        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        
        // Send request
        let request_str = serde_json::to_string(&request)?;
        self.stream.write_all(request_str.as_bytes()).await?;
        self.stream.write_all(b"\n").await?;
        
        // Read response
        let mut buffer = Vec::new();
        self.stream.read_to_end(&mut buffer).await?;
        
        // Parse response
        let response: Value = serde_json::from_slice(&buffer)?;
        
        // Check for error
        if let Some(error) = response.get("error") {
            return Err(Error::JsonRpcError(error.clone()));
        }
        
        // Return result
        Ok(response["result"].clone())
    }
}

/// Discover primal's Unix socket path
async fn discover_primal_socket(primal_name: &str) -> Result<PathBuf, Error> {
    // 1. Check XDG runtime dir
    let xdg_runtime = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/run/user/1000".to_string());
    
    let socket_path = PathBuf::from(format!(
        "{}/ecoPrimals/{}.sock",
        xdg_runtime, primal_name
    ));
    
    if socket_path.exists() {
        return Ok(socket_path);
    }
    
    // 2. Check home dir
    let home = std::env::var("HOME")?;
    let socket_path = PathBuf::from(format!(
        "{}/.local/share/ecoPrimals/{}.sock",
        home, primal_name
    ));
    
    if socket_path.exists() {
        return Ok(socket_path);
    }
    
    Err(Error::PrimalNotFound(primal_name.to_string()))
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Primal not found: {0}")]
    PrimalNotFound(String),
    
    #[error("JSON-RPC error: {0}")]
    JsonRpcError(Value),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Environment error: {0}")]
    Env(#[from] std::env::VarError),
}
```

**Cargo.toml**:
```toml
[package]
name = "beardog-tower-atomic"
version = "0.9.0"
edition = "2021"

[dependencies]
tokio = { version = "1.35", features = ["net", "io-util"] }
serde_json = "1.0"
thiserror = "1.0"
```

---

### Phase 5: Update Examples (~30 minutes)

**Find examples using HTTP**:
```bash
grep -r "reqwest\|hyper" examples/ --include="*.rs"
```

**Replace with Tower Atomic pattern**:

**Before**:
```rust
// examples/external_api_demo.rs
use reqwest;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let response = client.get("https://api.example.com").send().await.unwrap();
    println!("Status: {}", response.status());
}
```

**After**:
```rust
// examples/songbird_delegation_demo.rs
use beardog_tower_atomic::Client;

#[tokio::main]
async fn main() {
    // BearDog delegates HTTP to Songbird
    let mut songbird = Client::connect("songbird").await.unwrap();
    
    let response = songbird.call("http.get", json!({
        "url": "https://api.example.com"
    })).await.unwrap();
    
    println!("Status: {}", response["status"]);
    println!("Body: {}", response["body"]);
}
```

---

## 📊 Expected Results

### Before Evolution

**Workspace Dependencies**:
```toml
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }  # Pulls ring!
hyper = { version = "1.1", features = ["full"] }
```

**Dev Dependencies**:
```toml
reqwest = { workspace = true }  # For tests
```

**Dependency Tree**:
```
beardog
├── reqwest v0.12.23
│   └── rustls v0.23.31
│       └── ring v0.17.14 ❌
```

**Result**: reqwest in dev-deps (OK for production, but not pure)

---

### After Evolution

**Workspace Dependencies**:
```toml
# HTTP removed - BearDog is crypto-only!
# All HTTP delegated to Songbird via Tower Atomic
```

**Dev Dependencies**:
```toml
# reqwest removed - tests use Songbird mock
beardog-tower-atomic = { path = "crates/beardog-tower-atomic" }
```

**Dependency Tree**:
```
beardog
├── ed25519-dalek v2.1 ✅
├── x25519-dalek v2.0 ✅
├── chacha20poly1305 v0.10 ✅
├── blake3 v1.5 (pure) ✅
├── sled v0.34 ✅
└── beardog-tower-atomic v0.9.0 ✅
    ├── tokio v1.35 ✅
    └── serde_json v1.0 ✅
```

**Result**: ZERO HTTP, ZERO ring, 100% Pure Rust (everywhere!)

---

## 🎯 Benefits

### 1. TRUE Separation of Concerns

**BearDog**:
- ✅ Crypto only (ed25519, x25519, chacha20, blake3)
- ✅ HSM integration (FIDO2, TPM, SoloKey)
- ✅ BTSP server (Unix sockets)
- ❌ NO HTTP, NO TLS, NO network

**Songbird**:
- ✅ TLS/HTTP only (Pure Rust, 95% complete)
- ✅ External API gateway
- ✅ AI proxy (Anthropic, OpenAI)
- ✅ Single controlled entry point

### 2. TRUE Pure Rust

**Before**:
- Production: 100% Pure Rust ✅
- Dev-deps: Has ring (via reqwest) ⚠️
- Grade: A (acceptable, but not perfect)

**After**:
- Production: 100% Pure Rust ✅
- Dev-deps: 100% Pure Rust ✅
- Tests: 100% Pure Rust ✅
- Grade: A++ (PERFECT!)

### 3. TRUE ecoBin

**Cross-Compilation**:
- ✅ x86_64-unknown-linux-musl
- ✅ aarch64-unknown-linux-musl
- ✅ armv7-unknown-linux-musleabihf
- ✅ x86_64-apple-darwin
- ✅ aarch64-apple-darwin
- ✅ riscv64gc-unknown-linux-gnu
- ✅ wasm32-wasi

**No Blockers**: Zero C dependencies anywhere!

### 4. Ecosystem Consistency

**All Primals Use Tower Atomic**:
- ✅ biomeOS → BearDog (crypto)
- ✅ biomeOS → Songbird (AI, TLS)
- ✅ Squirrel → Songbird (AI APIs)
- ✅ BearDog → Songbird (HTTP/TLS)
- ✅ ToadStool → BearDog (crypto)
- ✅ NestGate → BearDog (crypto)

**Result**: Unified inter-primal communication!

---

## 🚀 Execution Timeline

### Total Effort: 5-7 hours

**Phase 1: Workspace Cleanup** (~30 min)
- Remove reqwest from [workspace.dependencies]
- Remove reqwest from [dev-dependencies]
- Remove hyper (if unused)
- Test: `cargo build --release`

**Phase 2: Dead Code Cleanup** (~1 hour)
- Delete/replace reqwest code in beardog-adapters
- Verify: `cargo tree | grep ring` (should be empty!)

**Phase 3: Integration Tests** (~1-2 hours)
- Update tests to use Tower Atomic
- Create Songbird mock helper
- Test: `cargo test`

**Phase 4: Tower Atomic Client** (~2-3 hours)
- Create beardog-tower-atomic crate
- Implement Client with discovery
- Add to workspace

**Phase 5: Examples** (~30 min)
- Update examples to use Tower Atomic
- Add new "songbird_delegation_demo.rs"

**Phase 6: Validation** (~30 min)
- Verify: `cargo tree | grep -i "ring\|reqwest\|hyper"`
- Cross-compile: All targets ✅
- Binary analysis: `nm` confirms zero HTTP symbols
- Document: Update BearDog docs

---

## 📚 Documentation Updates

### Files to Update

**1. BearDog README.md**
```markdown
## Architecture

BearDog is a **Pure Rust crypto primal** with ZERO network dependencies.

### Communication

- **Inter-primal**: Tower Atomic (Unix sockets, JSON-RPC)
- **BTSP**: Unix socket server (no HTTP)
- **External HTTP**: Delegated to Songbird

### Dependencies

- ✅ 100% Pure Rust (production AND development!)
- ✅ Zero C dependencies
- ✅ Zero HTTP/TLS (crypto-only!)
```

**2. ARCHITECTURE.md**
```markdown
## Primal Boundaries

### BearDog's Domain
- Cryptography (signing, encryption, hashing)
- HSM integration (FIDO2, TPM, hardware keys)
- BTSP tunnel security
- Genetic lineage and trust

### NOT BearDog's Domain
- ❌ HTTP/HTTPS (use Songbird!)
- ❌ External APIs (use Songbird!)
- ❌ Network protocols (use NestGate!)
- ❌ AI inference (use Squirrel!)

### Tower Atomic Pattern
BearDog uses Unix sockets to delegate:
- HTTP → Songbird
- AI → Squirrel (via Songbird)
- Network → NestGate
```

**3. UNIBIN_ECOBIN_EXPLAINED.md**
```markdown
## BearDog ecoBin Status

**Grade**: A++ (TRUE ecoBin)

**Verification**:
- ✅ Production: 100% Pure Rust
- ✅ Development: 100% Pure Rust
- ✅ Tests: 100% Pure Rust
- ✅ Cross-compilation: ALL targets
- ✅ Binary analysis: Zero HTTP symbols

**Key Principle**: BearDog is crypto-only. All HTTP delegated to Songbird.
```

---

## 🎊 Success Criteria

**Code**:
- ✅ `cargo tree | grep -i ring` → (empty)
- ✅ `cargo tree | grep -i reqwest` → (empty)
- ✅ `cargo tree | grep -i hyper` → (empty)
- ✅ `cargo build --release` → success
- ✅ `cargo test` → all pass

**Cross-Compilation**:
- ✅ `cargo build --target x86_64-unknown-linux-musl` → success
- ✅ `cargo build --target aarch64-unknown-linux-musl` → success
- ✅ `cargo build --target armv7-unknown-linux-musleabihf` → success
- ✅ `cargo build --target x86_64-apple-darwin` → success (on macOS)

**Binary Analysis**:
- ✅ `nm target/release/beardog | grep -i "ring\|reqwest\|hyper"` → (empty)
- ✅ Size appropriate for Pure Rust (~2-3 MB musl static)

**Functionality**:
- ✅ BTSP still works (Unix sockets)
- ✅ Crypto operations work (ed25519, x25519, etc.)
- ✅ HSM integration works (FIDO2, TPM)
- ✅ Tower Atomic delegation works (to Songbird)

---

## 🌍 Ecosystem Impact

### Before This Evolution

**Primals with HTTP**:
- BearDog: reqwest (dev-deps only, but still there)
- Songbird: reqwest (production, for TLS - being replaced)
- Squirrel: Uses Songbird proxy ✅
- ToadStool: Used to have, now uses Tower Atomic ✅

**Status**: 2/7 primals with HTTP dependencies

### After This Evolution

**Primals with HTTP**:
- Songbird: Pure Rust TLS (95% complete, finishing)

**Primals 100% Pure Rust (NO HTTP anywhere)**:
- ✅ BearDog (crypto)
- ✅ biomeOS (orchestrator)
- ✅ Squirrel (AI)
- ✅ ToadStool (compute)
- ✅ NestGate (network)
- ✅ petalTongue (UI, headless/CLI modes)

**Status**: 6/7 primals 100% Pure Rust! (Songbird finishing soon!)

---

## 💡 Key Learnings

### 1. TRUE PRIMAL = Single Domain

**BearDog should ONLY do**:
- Cryptography
- HSM integration
- Trust evaluation
- BTSP security tunnels

**BearDog should NEVER do**:
- HTTP/HTTPS (that's Songbird!)
- AI inference (that's Squirrel!)
- Network protocols (that's NestGate!)

### 2. Tower Atomic = Inter-Primal Glue

**Pattern**:
```rust
// Need capability outside your domain? Ask via Tower Atomic!
let other_primal = TowerAtomic::connect("primal_name").await?;
let result = other_primal.call("method", params).await?;
```

**Benefits**:
- Runtime discovery (no hardcoded deps)
- Unix sockets (fast, secure)
- JSON-RPC (simple, standard)
- Separation of concerns

### 3. Dev-Dependencies Still Matter!

**Old thinking**: "Dev-deps don't affect production, so ring is OK"  
**New thinking**: "Dev-deps should ALSO be Pure Rust for true ecoBin!"

**Why**:
- Cross-compilation for tests
- Consistency across all builds
- A++ grade (perfect)
- Ecosystem purity

---

## 🚀 Next Steps

**Immediate**:
1. Review this plan
2. Approve execution
3. Execute Phase 1-6 (~5-7 hours)
4. Validate with full test suite
5. Update documentation

**Follow-Up**:
1. Apply pattern to any other primals with HTTP
2. Document Tower Atomic as ecosystem standard
3. Create "How to Delegate Capabilities" guide
4. Celebrate TRUE ecoBin achievement!

---

**Date**: January 19, 2026  
**Plan By**: biomeOS Team  
**Status**: Ready to Execute  
**Expected Outcome**: BearDog A++ (100% Pure Rust everywhere!)  
**Timeline**: 5-7 hours

🎯 **Let's make BearDog TRULY Pure Rust!** 🦀

**Key Message**: "BearDog is crypto-only. All HTTP delegated to Songbird via Tower Atomic. This is the TRUE PRIMAL way!"


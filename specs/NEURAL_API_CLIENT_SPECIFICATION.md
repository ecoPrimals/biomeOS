# Neural API Client Specification

**Version**: 1.0.0  
**Date**: January 20, 2026  
**Status**: Specification for primal integration with Neural API routing layer

---

## 🎯 Purpose

Enable primals to communicate with external services (HTTP APIs, other primals) **without**:
- ❌ Direct HTTP dependencies (`reqwest`, `hyper`)
- ❌ C crypto dependencies (`ring`, `openssl-sys`)
- ❌ Knowledge of other primals (TRUE PRIMAL pattern)
- ❌ Hardcoded endpoints or socket paths

---

## 🏗️ Architecture

### Service Mesh Pattern

```text
┌──────────────┐
│   Squirrel   │ (AI Primal)
│              │
│ ┌──────────┐ │
│ │ NeuralAPI│ │ ← Pure Rust client
│ │  Client  │ │ ← Unix socket only
│ └──────────┘ │ ← Capability-based
└──────┬───────┘
       │ Unix socket: /tmp/neural-api.sock
       │ JSON-RPC 2.0
       ↓
┌──────────────────────────────┐
│     Neural API Server        │
│  ┌────────────────────────┐  │
│  │   Neural Router        │  │
│  │  - Discovers primals   │  │
│  │  - Routes requests     │  │
│  │  - Collects metrics    │  │
│  └────────────────────────┘  │
└──────────────────────────────┘
       │
       ├──→ Tower Atomic (BearDog + Songbird)
       │       ↓ HTTP/TLS handling
       │       ↓ Crypto (Pure Rust)
       │
       └──→ External API (e.g., Anthropic)
```

### TRUE PRIMAL Pattern

**Squirrel's Knowledge**:
- ✅ "I need to call an HTTP API"
- ✅ "Neural API is at /tmp/neural-api.sock"
- ❌ Does NOT know Songbird exists
- ❌ Does NOT know BearDog exists
- ❌ Does NOT know how HTTP/TLS works

**Discovery at Runtime**:
1. Squirrel → `neural_api.proxy_http` → Neural API
2. Neural API → discovers Tower Atomic
3. Neural API → routes to Songbird
4. Songbird → uses BearDog for crypto
5. Songbird → makes HTTPS call
6. Response → back to Squirrel

---

## 📋 Client API

### Construction

```rust
use neural_api_client::NeuralApiClient;

// Runtime discovery of Neural API socket
let client = NeuralApiClient::new("/tmp/neural-api.sock")?;

// Or with family_id
let client = NeuralApiClient::discover("nat0")?;
// → Finds /tmp/neural-api-nat0.sock
```

### Methods

#### 1. `proxy_http` - HTTP Proxy

Forward HTTP request through Tower Atomic (Songbird + BearDog).

**Signature**:
```rust
pub async fn proxy_http(
    &self,
    method: &str,           // "GET", "POST", etc.
    url: &str,              // Full URL
    headers: Option<HashMap<String, String>>,
    body: Option<serde_json::Value>,
) -> Result<HttpResponse>
```

**Example**:
```rust
// Call Anthropic API (no reqwest needed!)
let response = client.proxy_http(
    "POST",
    "https://api.anthropic.com/v1/messages",
    Some(HashMap::from([
        ("x-api-key".to_string(), api_key),
        ("anthropic-version".to_string(), "2023-06-01".to_string()),
    ])),
    Some(json!({
        "model": "claude-3-opus-20240229",
        "max_tokens": 1024,
        "messages": [{"role": "user", "content": "Hello!"}]
    }))
).await?;

println!("Status: {}", response.status);
println!("Body: {}", response.body);
```

**No reqwest, no ring, no HTTP knowledge! Just capability-based routing.**

#### 2. `discover_capability` - Capability Discovery

Discover primal(s) providing a capability.

**Signature**:
```rust
pub async fn discover_capability(
    &self,
    capability: &str,  // "secure_http", "secure_storage", etc.
) -> Result<CapabilityInfo>
```

**Example**:
```rust
let info = client.discover_capability("secure_http").await?;
println!("Capability: {}", info.capability);
println!("Atomic type: {:?}", info.atomic_type);
for primal in info.primals {
    println!("  - {} @ {}", primal.name, primal.socket);
}
```

#### 3. `route_to_primal` - Generic Primal Routing

Route arbitrary JSON-RPC request to primal by capability.

**Signature**:
```rust
pub async fn route_to_primal(
    &self,
    capability: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value>
```

**Example**:
```rust
// Call crypto function via capability (no direct BearDog knowledge)
let signature = client.route_to_primal(
    "crypto_sign",
    "ed25519.sign",
    json!({"data": "...", "key_id": "..."})
).await?;
```

#### 4. `get_metrics` - Routing Metrics

Get routing metrics (for debugging/observability).

**Signature**:
```rust
pub async fn get_metrics(&self) -> Result<RoutingMetrics>
```

---

## 🔧 Implementation Details

### Pure Rust, Zero External Deps

**Dependencies**:
```toml
[dependencies]
anyhow = "1.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.35", features = ["net", "io-util"] }
```

**External HTTP/Crypto Deps**:
- ❌ NO `reqwest`
- ❌ NO `hyper`
- ❌ NO `ring`
- ❌ NO `openssl-sys`
- ✅ Only `tokio` for Unix sockets

### Modern Idiomatic Rust

**Principles**:
- ✅ Async/await throughout
- ✅ `Result<T, E>` for all fallible operations
- ✅ `?` operator for error propagation
- ✅ No `.unwrap()` or `.expect()` in production
- ✅ Comprehensive error types

**Error Handling**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum NeuralApiError {
    #[error("Failed to connect to Neural API: {0}")]
    ConnectionError(String),
    
    #[error("JSON-RPC error: {code} - {message}")]
    RpcError { code: i32, message: String },
    
    #[error("Request timeout after {0}ms")]
    Timeout(u64),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

### JSON-RPC 2.0 Protocol

**Request Format**:
```json
{
    "jsonrpc": "2.0",
    "method": "neural_api.proxy_http",
    "params": {
        "method": "POST",
        "url": "https://...",
        "headers": {...},
        "body": {...}
    },
    "id": 1
}
```

**Response Format**:
```json
{
    "jsonrpc": "2.0",
    "result": {
        "status": 200,
        "headers": {...},
        "body": {...}
    },
    "id": 1
}
```

**Error Response**:
```json
{
    "jsonrpc": "2.0",
    "error": {
        "code": -32603,
        "message": "Internal error: ..."
    },
    "id": 1
}
```

---

## 🧪 Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_client_construction() {
        let client = NeuralApiClient::new("/tmp/test.sock");
        assert!(client.is_ok());
    }
    
    #[test]
    fn test_discover_socket_path() {
        let path = NeuralApiClient::discover_socket("nat0");
        assert_eq!(path, PathBuf::from("/tmp/neural-api-nat0.sock"));
    }
    
    #[tokio::test]
    async fn test_proxy_http_serialization() {
        // Test request building without actual connection
    }
}
```

### Integration Tests

**Prerequisites**: Neural API + Tower Atomic running

```rust
#[tokio::test]
async fn test_proxy_http_real() {
    let client = NeuralApiClient::new("/tmp/neural-api-test.sock").unwrap();
    
    let response = client.proxy_http(
        "GET",
        "https://httpbin.org/get",
        None,
        None
    ).await;
    
    assert!(response.is_ok());
    assert_eq!(response.unwrap().status, 200);
}

#[tokio::test]
async fn test_discover_capability() {
    let client = NeuralApiClient::new("/tmp/neural-api-test.sock").unwrap();
    
    let info = client.discover_capability("secure_http").await;
    
    assert!(info.is_ok());
    let info = info.unwrap();
    assert_eq!(info.capability, "secure_http");
    assert!(info.primals.len() >= 2); // BearDog + Songbird
}
```

---

## 📦 Integration into Squirrel

### Before (Current Squirrel)

```rust
// Squirrel's current HTTP code (BAD - uses reqwest + ring)
use reqwest::Client;

async fn call_anthropic_api(prompt: &str) -> Result<String> {
    let client = Client::new();  // ← reqwest (pulls in ring!)
    
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .json(&json!({
            "model": "claude-3-opus-20240229",
            "messages": [{"role": "user", "content": prompt}]
        }))
        .send()  // ← HTTP knowledge, TLS handling
        .await?;
    
    let body = response.json::<Value>().await?;
    Ok(body["content"][0]["text"].as_str().unwrap().to_string())
}
```

**Dependencies**:
```toml
reqwest = { version = "0.11", features = ["json"] }  # ← Pulls in ring!
```

### After (With NeuralApiClient)

```rust
// Squirrel's new HTTP code (GOOD - Pure Rust, capability-based)
use neural_api_client::NeuralApiClient;

async fn call_anthropic_api(prompt: &str) -> Result<String> {
    let client = NeuralApiClient::discover("nat0")?;  // ← Pure Rust!
    
    let response = client
        .proxy_http(  // ← Capability-based routing
            "POST",
            "https://api.anthropic.com/v1/messages",
            Some(HashMap::from([
                ("x-api-key".to_string(), api_key),
                ("anthropic-version".to_string(), "2023-06-01".to_string()),
            ])),
            Some(json!({
                "model": "claude-3-opus-20240229",
                "messages": [{"role": "user", "content": prompt}]
            }))
        )
        .await?;  // ← Neural API handles discovery + routing
    
    let body: Value = serde_json::from_str(&response.body)?;
    Ok(body["content"][0]["text"].as_str().unwrap().to_string())
}
```

**Dependencies**:
```toml
neural-api-client = { path = "../../phase2/biomeOS/crates/neural-api-client" }
# NO reqwest, NO ring, NO HTTP libs!
```

---

## 🚀 Migration Path

### Step 1: Create Client Crate (30 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo new --lib crates/neural-api-client
```

Implement:
- `NeuralApiClient` struct
- `proxy_http()` method
- `discover_capability()` method
- `route_to_primal()` method
- Error types
- Unit tests

### Step 2: Integrate into Squirrel (1 hour)

```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
```

1. Add dependency:
```toml
[dependencies]
neural-api-client = { path = "../../phase2/biomeOS/crates/neural-api-client" }
```

2. Create wrapper in Squirrel:
```rust
// crates/main/src/neural_api.rs
use neural_api_client::NeuralApiClient;

pub struct HttpClient {
    neural_client: NeuralApiClient,
}

impl HttpClient {
    pub fn new(family_id: &str) -> Result<Self> {
        Ok(Self {
            neural_client: NeuralApiClient::discover(family_id)?,
        })
    }
    
    pub async fn post(&self, url: &str, headers: HashMap<String, String>, body: Value) -> Result<HttpResponse> {
        self.neural_client.proxy_http("POST", url, Some(headers), Some(body)).await
    }
}
```

### Step 3: Replace reqwest Calls (1 hour)

Find all `reqwest` usage:
```bash
grep -r "reqwest::" crates/
```

Replace with `neural_api_client`:
```rust
// OLD
let client = reqwest::Client::new();
let response = client.post(url).json(&body).send().await?;

// NEW
let client = HttpClient::new("nat0")?;
let response = client.post(url, headers, body).await?;
```

### Step 4: Remove Dependencies (5 min)

```toml
# DELETE these lines from Cargo.toml:
reqwest = "0.11"
openai = "..."
anthropic-sdk = "..."
```

### Step 5: Test (30 min)

1. Start Tower Atomic:
```bash
# BearDog
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo run --release -- server --socket /tmp/beardog-nat0.sock --family-id nat0

# Songbird
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
SONGBIRD_ORCHESTRATOR_SOCKET=/tmp/songbird-nat0.sock \
SONGBIRD_ORCHESTRATOR_FAMILY_ID=nat0 \
cargo run --release -- orchestrator
```

2. Start Neural API:
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo run --release -- neural-api --family-id nat0
```

3. Test Squirrel:
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
cargo test --release
```

4. Manual test:
```bash
cargo run --release -- server --socket /tmp/squirrel-nat0.sock
# Call Anthropic API via routing
```

### Step 6: Harvest ecoBin (15 min)

```bash
cargo build --release --target x86_64-unknown-linux-musl
strip target/x86_64-unknown-linux-musl/release/squirrel
cp target/x86_64-unknown-linux-musl/release/squirrel \
   /home/eastgate/Development/ecoPrimals/plasmidBin/primals/squirrel/squirrel-x86_64-linux-musl

# Verify
ldd target/x86_64-unknown-linux-musl/release/squirrel  # Should show: not a dynamic executable
cargo tree | grep -i "ring\|reqwest\|hyper"  # Should show: nothing
```

---

## 📊 Expected Impact

### Dependencies

**Before**:
```
Squirrel dependencies:
├── reqwest (HTTP client)
│   ├── hyper (HTTP protocol)
│   │   └── rustls (TLS)
│   │       └── ring ❌ (C crypto)
│   └── ...
├── openai (uses reqwest)
└── anthropic-sdk (uses reqwest)

Total: ~300 dependencies, 2+ C dependencies
```

**After**:
```
Squirrel dependencies:
├── neural-api-client (Pure Rust Unix socket)
│   ├── tokio (async runtime)
│   ├── serde_json (JSON)
│   └── anyhow (errors)
└── ...

Total: ~150 dependencies, 0 C dependencies ✅
```

### Binary Size

**Before**: ~25 MB (with reqwest + ring)  
**After**: ~15 MB (without HTTP deps) (-40%)

### Compile Time

**Before**: ~120 seconds (with reqwest)  
**After**: ~80 seconds (without reqwest) (-33%)

### Architecture

**Before**: Tight coupling, hardcoded  
**After**: TRUE PRIMAL, capability-based ✅

---

## ✅ Success Criteria

1. ✅ Squirrel builds without `reqwest`
2. ✅ Squirrel builds without `ring` or `openssl-sys`
3. ✅ Anthropic API calls work via Neural API routing
4. ✅ No knowledge of Songbird or BearDog in Squirrel code
5. ✅ Socket paths discovered at runtime (no hardcoding)
6. ✅ All tests pass
7. ✅ ecoBin harvest successful (static binary, no C deps)

---

## 🎯 Timeline

**Total Estimated Time**: 3-4 hours

| Task | Duration | Status |
|------|----------|--------|
| Create neural-api-client crate | 30 min | Pending |
| Implement client methods | 1 hour | Pending |
| Integrate into Squirrel | 1 hour | Pending |
| Replace reqwest calls | 1 hour | Pending |
| Testing | 30 min | Pending |
| ecoBin harvest | 15 min | Pending |

**Start**: Day 2 (Tomorrow)  
**Completion**: Day 2 (Tomorrow afternoon)

---

## 🏆 Final Result

**Squirrel will become**:
- ✅ 100% Pure Rust (zero C dependencies)
- ✅ TRUE PRIMAL (zero knowledge of other primals)
- ✅ Capability-based (runtime discovery)
- ✅ Portable (works on any platform)
- ✅ Observable (all HTTP logged via Neural API)
- ✅ Learnable (metrics collected for optimization)

**ecoBin Grade**: A++ (from A+ current)

---

**Status**: Specification complete, ready for Day 2 implementation! 🚀


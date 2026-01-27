# Songbird + BearDog Coupling Status & Evolution Path

**Date**: January 25, 2026  
**Current Status**: ✅ **Library-level coordination** (tight coupling)  
**Goal**: 🎯 **Service-level coordination** (loose coupling via IPC + Neural API)

---

## 📊 Current Architecture: Tight Coupling

### How It Works Today (Library Level)

**Songbird's Current Integration**:
```rust
// In songbird-http-client/src/client.rs
pub struct SongbirdHttpClient {
    crypto: Arc<dyn CryptoCapability>,  // <- Trait-based abstraction
    config: TlsConfig,
    profiler: Option<Arc<ServerProfiler>>,
}

impl SongbirdHttpClient {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            crypto: Arc::new(BearDogProvider::new(socket_path)),  // <- Direct instantiation
            config: TlsConfig::default(),
            profiler: None,
        }
    }
    
    pub fn from_env() -> Self {
        let socket_path = std::env::var("CRYPTO_CAPABILITY_SOCKET")
            .or_else(|_| std::env::var("BEARDOG_SOCKET"))
            .unwrap_or_else(|_| "/tmp/beardog.sock".to_string());
        
        Self {
            crypto: Arc::new(BearDogProvider::new(socket_path)),  // <- Still direct
            config: TlsConfig::default(),
            profiler: None,
        }
    }
}
```

**BearDogProvider** (in `songbird-http-client/src/crypto/beardog_provider.rs`):
```rust
pub struct BearDogProvider {
    socket_path: String,
    request_id: AtomicU64,
}

impl BearDogProvider {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
            request_id: AtomicU64::new(1),
        }
    }
}

#[async_trait]
impl CryptoCapability for BearDogProvider {
    async fn derive_handshake_secrets(...) -> Result<TlsHandshakeSecrets> {
        self.call("tls.derive_handshake_secrets", params).await?  // <- JSON-RPC call
    }
    
    async fn derive_application_secrets(...) -> Result<TlsApplicationSecrets> {
        self.call("tls.derive_application_secrets", params).await?  // <- JSON-RPC call
    }
    
    // ... all other crypto operations
}
```

### What This Means

**Positive** (Why it works):
- ✅ **Trait-based abstraction**: Uses `CryptoCapability` trait (not hardcoded to BearDog)
- ✅ **JSON-RPC communication**: Already using IPC (Unix sockets + JSON-RPC 2.0)
- ✅ **Socket path from environment**: Configurable via `BEARDOG_SOCKET` or `CRYPTO_CAPABILITY_SOCKET`
- ✅ **Loose coupling at library level**: Could swap BearDog for another `CryptoCapability` provider

**Tight Coupling** (Why it needs evolution):
- ❌ **Direct instantiation**: Songbird creates `BearDogProvider` directly (line 59, 77)
- ❌ **Embedded knowledge**: Songbird knows BearDog's socket path and RPC method names
- ❌ **Library-only**: Can't coordinate via Neural API because Songbird itself isn't a service
- ❌ **No semantic translation**: Method names are BearDog-specific (not semantic)

---

## 🎯 Goal Architecture: Loose Coupling via Neural API

### Evolution Path: 3 Phases

```text
┌──────────────────────────────────────────────────────────────────────┐
│                    PHASE 3: NEURAL API ORCHESTRATION                 │
│                                                                        │
│  ┌──────────────┐   JSON-RPC      ┌────────────────┐                │
│  │   Client     │ ──────────────> │   Neural API   │                │
│  │ (any primal) │   "http.request"│  (orchestrator)│                │
│  └──────────────┘                 └────────┬───────┘                │
│                                             │                         │
│                      Semantic Translation   │                         │
│                      "http.request" → finds │ Songbird by capability  │
│                                             ▼                         │
│                  ┌────────────────────────────────────┐              │
│                  │         Songbird Server            │              │
│                  │  Unix socket: /tmp/songbird.sock   │              │
│                  │  Exposes: http.request, http.get   │              │
│                  └──────────────┬─────────────────────┘              │
│                                 │                                     │
│                    Semantic Translation via Neural API                │
│                    "crypto.encrypt" → finds BearDog                   │
│                                 ▼                                     │
│                  ┌────────────────────────────────────┐              │
│                  │         BearDog Server             │              │
│                  │  Unix socket: /tmp/beardog.sock    │              │
│                  │  Exposes: crypto.*, tls.*          │              │
│                  └────────────────────────────────────┘              │
└──────────────────────────────────────────────────────────────────────┘

    ↑ This is the GOAL (Neural API orchestration + semantic translation)

┌──────────────────────────────────────────────────────────────────────┐
│                    PHASE 2: SONGBIRD AS SERVICE                      │
│                                                                        │
│  ┌──────────────┐   JSON-RPC      ┌────────────────┐                │
│  │   Client     │ ──────────────> │ Songbird Server│                │
│  │ (any primal) │   "http.request"│ (Unix socket)  │                │
│  └──────────────┘                 └────────┬───────┘                │
│                                             │                         │
│                             Direct socket path (hardcoded)            │
│                             /tmp/beardog.sock                         │
│                                             ▼                         │
│                  ┌────────────────────────────────────┐              │
│                  │         BearDog Server             │              │
│                  │  Unix socket: /tmp/beardog.sock    │              │
│                  └────────────────────────────────────┘              │
└──────────────────────────────────────────────────────────────────────┘

    ↑ This is NEXT (Songbird IPC evolution) - BLOCKED

┌──────────────────────────────────────────────────────────────────────┐
│                    PHASE 1: LIBRARY COORDINATION                     │
│                          (CURRENT STATE)                             │
│                                                                        │
│  ┌──────────────┐   Rust library   ┌────────────────┐               │
│  │   Client     │ ───────────────> │ Songbird HTTP  │               │
│  │ (example)    │   function call  │    Client      │               │
│  └──────────────┘                  │   (library)    │               │
│                                     └────────┬───────┘               │
│                                              │                        │
│                              JSON-RPC call (direct socket)            │
│                              /tmp/beardog.sock                        │
│                                              ▼                        │
│                  ┌────────────────────────────────────┐              │
│                  │         BearDog Server             │              │
│                  │  Unix socket: /tmp/beardog.sock    │              │
│                  └────────────────────────────────────┘              │
└──────────────────────────────────────────────────────────────────────┘

    ↑ This is NOW (works perfectly, but library-only)
```

---

## 📝 Phase 1: Current State (Library Coordination)

### ✅ What Works

**Songbird Library** (`songbird-http-client`):
- ✅ TLS 1.3 handshake complete
- ✅ Application data encryption
- ✅ HTTP 200 OK from real servers
- ✅ Trait-based crypto abstraction (`CryptoCapability`)
- ✅ JSON-RPC communication with BearDog

**BearDog Server**:
- ✅ Running on Unix socket (`/tmp/beardog-nat0.sock`)
- ✅ Exposes all crypto operations via JSON-RPC 2.0
- ✅ Methods: `crypto.*`, `tls.*`, `genetic.*`

**Usage** (Library Level):
```rust
// In any Rust codebase
use songbird_http_client::SongbirdHttpClient;

let client = SongbirdHttpClient::from_env();
let response = client.get("https://example.com").await?;
// ✅ Works! HTTP 200 OK
```

### ❌ What's Missing

1. **Songbird is not a service**: Can't call it via IPC
2. **Neural API can't orchestrate**: No way to route HTTPS requests
3. **Not primal-independent**: Songbird has embedded BearDog knowledge

---

## 🚧 Phase 2: Songbird as Service (Next Step)

### Goal: Expose Songbird's HTTPS client via Unix socket JSON-RPC

**Required Changes** (Estimated: 6-8 hours):

### 1. Add Unix Socket Server to `songbird server`

**File**: `songbird/src/bin/songbird/main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    Server {
        /// Unix socket path (for IPC)
        #[arg(long)]
        socket: Option<String>,
        
        /// HTTP port (for federation - can run both!)
        #[arg(long, default_value = "8080")]
        port: u16,
        
        /// Family ID (for multi-instance)
        #[arg(long)]
        family_id: Option<String>,
    },
    // ... other commands
}
```

### 2. Create IPC Handler Module

**File**: `songbird/src/ipc/http_handler.rs`

```rust
use songbird_http_client::SongbirdHttpClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct HttpRequestParams {
    method: String,
    url: String,
    headers: Option<HashMap<String, String>>,
    body: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct HttpResponseResult {
    status: u16,
    headers: HashMap<String, String>,
    body: String,
}

/// Handle http.request RPC call
async fn handle_http_request(params: HttpRequestParams) -> Result<HttpResponseResult> {
    // Use existing Songbird HTTP client library (already works!)
    let client = SongbirdHttpClient::from_env();
    
    let response = client.request(
        &params.method,
        &params.url,
        params.headers.unwrap_or_default(),
        params.body,
    ).await?;
    
    Ok(HttpResponseResult {
        status: response.status,
        headers: response.headers,
        body: response.body.to_string(),
    })
}

/// Handle http.get RPC call (convenience)
async fn handle_http_get(params: serde_json::Value) -> Result<HttpResponseResult> {
    let url = params["url"].as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
    
    handle_http_request(HttpRequestParams {
        method: "GET".to_string(),
        url: url.to_string(),
        headers: None,
        body: None,
    }).await
}

/// Handle http.post RPC call (convenience)
async fn handle_http_post(params: HttpRequestParams) -> Result<HttpResponseResult> {
    let mut params = params;
    params.method = "POST".to_string();
    handle_http_request(params).await
}
```

### 3. Wire to JSON-RPC Server

**File**: `songbird/src/bin/songbird/main.rs` (server command)

```rust
async fn run_server(socket_path: Option<String>, port: u16, family_id: Option<String>) -> Result<()> {
    // If socket path provided, start Unix socket server (IPC mode)
    if let Some(socket_path) = socket_path {
        info!("🚀 Starting Songbird server in IPC mode");
        info!("   Socket: {}", socket_path);
        
        // Initialize JSON-RPC server
        let mut rpc_server = JsonRpcServer::new(&socket_path).await?;
        
        // Register HTTP methods
        rpc_server.register("http.request", handle_http_request);
        rpc_server.register("http.get", handle_http_get);
        rpc_server.register("http.post", handle_http_post);
        
        // Start listening
        rpc_server.serve().await?;
    } else {
        // Start HTTP server (federation mode - current behavior)
        info!("🚀 Starting Songbird server in federation mode");
        info!("   Port: {}", port);
        // ... existing HTTP server code ...
    }
    
    Ok(())
}
```

### Result: Songbird as Service

**Now clients can call Songbird via IPC**:
```bash
# Start Songbird server
songbird server --socket /tmp/songbird-nat0.sock

# Make HTTPS request via IPC
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://example.com"}}' \
  | nc -U /tmp/songbird-nat0.sock

# Response:
# {"jsonrpc":"2.0","id":1,"result":{"status":200,"headers":{...},"body":"..."}}
```

**But still tight coupling**: Songbird still knows BearDog's socket path directly.

---

## 🎯 Phase 3: Neural API Orchestration (Final Goal)

### Goal: Songbird discovers BearDog via Neural API semantic translation

**Required Changes**:

### 1. Update BearDogProvider to Support Dual Mode

**File**: `songbird-http-client/src/crypto/beardog_provider.rs`

```rust
pub enum CryptoDiscoveryMode {
    /// Direct socket path (Phase 2 - for testing/simple deployments)
    Direct(String),
    
    /// Via Neural API semantic translation (Phase 3 - production)
    NeuralApi(String),  // Neural API socket path
}

pub struct BearDogProvider {
    discovery_mode: CryptoDiscoveryMode,
    request_id: AtomicU64,
}

impl BearDogProvider {
    /// Create with direct socket path (Phase 2)
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            discovery_mode: CryptoDiscoveryMode::Direct(socket_path.into()),
            request_id: AtomicU64::new(1),
        }
    }
    
    /// Create with Neural API discovery (Phase 3)
    pub fn with_neural_api(neural_api_socket: impl Into<String>) -> Self {
        Self {
            discovery_mode: CryptoDiscoveryMode::NeuralApi(neural_api_socket.into()),
            request_id: AtomicU64::new(1),
        }
    }
    
    async fn call(&self, semantic_method: &str, params: Value) -> Result<Value> {
        match &self.discovery_mode {
            CryptoDiscoveryMode::Direct(socket_path) => {
                // Phase 2: Direct call to BearDog
                let actual_method = self.semantic_to_actual(semantic_method);
                self.call_direct(socket_path, actual_method, params).await
            }
            CryptoDiscoveryMode::NeuralApi(neural_api_socket) => {
                // Phase 3: Route via Neural API (semantic translation!)
                self.call_via_neural_api(neural_api_socket, semantic_method, params).await
            }
        }
    }
    
    async fn call_via_neural_api(
        &self,
        neural_api_socket: &str,
        semantic_method: &str,
        params: Value,
    ) -> Result<Value> {
        // Neural API translates:
        // - "crypto.encrypt" → discovers BearDog by capability
        // - Routes to /tmp/beardog-nat0.sock
        // - Translates to BearDog's actual method name
        
        let request = json!({
            "jsonrpc": "2.0",
            "method": semantic_method,  // <- Semantic name!
            "params": params,
            "id": self.request_id.fetch_add(1, Ordering::SeqCst),
        });
        
        // Connect to Neural API
        let mut stream = UnixStream::connect(neural_api_socket).await?;
        // ... send request, receive response ...
    }
}
```

### 2. Neural API Capability Translation

**Already implemented in biomeOS!**

**File**: `biomeos-atomic-deploy/src/capability_translation.rs`

```rust
// Neural API already has semantic → actual translation
registry.register_translation(
    "crypto.encrypt",                    // <- Semantic name
    "beardog",                            // <- Provider
    "chacha20_poly1305_encrypt",         // <- Actual method
    "/tmp/beardog-nat0.sock",            // <- Socket path
    None,                                 // <- No param mapping
);

// When Songbird calls "crypto.encrypt", Neural API:
// 1. Looks up "crypto.encrypt" capability
// 2. Finds BearDog as provider
// 3. Translates to "chacha20_poly1305_encrypt"
// 4. Routes to /tmp/beardog-nat0.sock
// 5. Returns result to Songbird
```

### 3. Environment Variable for Mode Selection

```bash
# Phase 2: Direct mode (simple deployments)
export CRYPTO_CAPABILITY_SOCKET=/tmp/beardog-nat0.sock

# Phase 3: Neural API mode (production orchestration)
export NEURAL_API_SOCKET=/tmp/neural-api-nat0.sock
export CRYPTO_DISCOVERY_MODE=neural_api
```

### Result: True Primal Independence

**Now Songbird has ZERO embedded knowledge of BearDog**:
- ✅ Doesn't know BearDog's socket path
- ✅ Doesn't know BearDog's method names
- ✅ Only knows semantic capability names ("crypto.encrypt")
- ✅ Discovers providers at runtime via Neural API
- ✅ Can work with ANY crypto provider that implements the capability
- ✅ Can be tested standalone (Phase 2 mode) or orchestrated (Phase 3 mode)

---

## 📊 Summary: Coupling Evolution

| Aspect | Phase 1 (NOW) | Phase 2 (Next) | Phase 3 (Goal) |
|--------|---------------|----------------|----------------|
| **Songbird IPC** | ❌ Library only | ✅ Unix socket JSON-RPC | ✅ Unix socket JSON-RPC |
| **BearDog Discovery** | 🟡 Env var | 🟡 Env var | ✅ Neural API |
| **Method Names** | 🟡 Semantic in code | 🟡 Semantic in code | ✅ Semantic via API |
| **Primal Independence** | ❌ Embedded knowledge | ⚠️ Partially | ✅ Complete |
| **Neural API Orchestration** | ❌ Not possible | ⚠️ Direct only | ✅ Full |
| **Semantic Translation** | ❌ No | ❌ No | ✅ Yes |
| **Testing** | ✅ Library tests | ✅ E2E + library | ✅ E2E + library |
| **Production Ready** | ⚠️ Library only | ⚠️ Direct deployment | ✅ Orchestrated |

**Legend**:
- ✅ Fully implemented
- 🟡 Partially implemented (some coupling remains)
- ⚠️ Works but limited
- ❌ Not implemented

---

## 🎯 Immediate Next Steps

### 1. **Songbird Team** (6-8 hours)
- [ ] Implement Phase 2: Songbird as Service
- [ ] Add Unix socket JSON-RPC server to `songbird server`
- [ ] Wire `http.request`, `http.get`, `http.post` to existing library
- [ ] Test end-to-end: Client → Songbird IPC → BearDog IPC → HTTPS

### 2. **biomeOS Team** (1-2 hours, after Songbird Phase 2)
- [ ] Deploy Tower Atomic via Neural API
- [ ] Test HTTPS requests through orchestration
- [ ] Validate semantic translation (optional at this phase)

### 3. **Joint Evolution** (Future, Phase 3)
- [ ] Implement dual-mode `BearDogProvider` (Direct + Neural API)
- [ ] Test semantic translation end-to-end
- [ ] Document primal independence patterns
- [ ] Apply pattern to other primals (Squirrel, NestGate, etc.)

---

## 📚 References

- **Current HTTPS Success**: `SONGBIRD_100_PERCENT_HTTPS_SUCCESS_JAN_25_2026.md`
- **IPC Evolution Handoff**: `SONGBIRD_IPC_EVOLUTION_REQUIRED_JAN_25_2026.md`
- **Primal Independence Pattern**: `TEAM_HANDOFF_DUAL_MODE_IMPLEMENTATION_JAN_24_2026.md`
- **Integration Spec**: `BIOMEOS_PRIMAL_INTEGRATION_SPEC.md`
- **Capability Translation**: `crates/biomeos-atomic-deploy/src/capability_translation.rs`

---

*Last Updated: January 25, 2026*


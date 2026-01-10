# 🎯 Wave 2 REAL Deep Debt - Protocol Evolution

**Date**: January 10, 2026  
**Priority**: **HIGH** - Security & Architecture  
**Status**: Analysis & Planning

---

## 🚨 **The Real Problem**

### **Current State: HTTP Fallback Everywhere** ❌

```
ALL primal clients currently use HTTP:
├── beardog.rs → PrimalHttpClient (116 HTTP references!)
├── songbird.rs → PrimalHttpClient
├── toadstool.rs → PrimalHttpClient
├── nestgate.rs → PrimalHttpClient
├── squirrel.rs → PrimalHttpClient
└── base.rs → PrimalHttpClient (base implementation)

Issues:
❌ HTTP is insecure (cleartext by default)
❌ HTTP is non-isomorphic (request/response, not bidirectional)
❌ HTTP adds overhead (headers, parsing, etc.)
❌ HTTP is meant to be FALLBACK ONLY
```

### **Target State: tarpc + JSON-RPC Primary** ✅

```
Primal clients should prefer:
1. tarpc (Rust-native RPC) - Primary
2. JSON-RPC over Unix sockets - Secondary  
3. HTTP REST - Fallback only (legacy, dev)

Priorities:
✅ tarpc: Fast, type-safe, bidirectional
✅ JSON-RPC: Language-agnostic, standard
✅ Unix sockets: Secure, fast, local
❌ HTTP: Insecure, slow, fallback only
```

---

## 📊 **Current Architecture Analysis**

### **What We Have**

**File: `crates/biomeos-core/src/clients/base.rs`**
```rust
/// HTTP client for primal communication ← WRONG!
pub struct PrimalHttpClient {
    client: Client,        // reqwest::Client
    base_url: String,      // "http://localhost:9000"
    timeout: Duration,
}

impl PrimalHttpClient {
    pub async fn get(&self, path: &str) -> Result<Value> {
        // HTTP GET request
    }
    
    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        // HTTP POST request
    }
}
```

**Used everywhere**:
- `beardog.rs`: 19 HTTP references
- `songbird.rs`: 12 HTTP references
- `toadstool.rs`: 9 HTTP references
- `nestgate.rs`: 7 HTTP references
- `squirrel.rs`: 7 HTTP references
- `upa.rs`: 24 HTTP references
- `openapi_adapter.rs`: 15 HTTP references

**Total**: 116 HTTP references across 10 files!

---

## 🎯 **Evolution Strategy**

### **Phase 1: Create Protocol Abstraction Layer**

Instead of `PrimalHttpClient`, create a **protocol-agnostic** client:

```rust
// NEW: crates/biomeos-core/src/clients/transport/mod.rs

/// Transport layer for primal communication
pub enum PrimalTransport {
    /// tarpc (Rust-native RPC) - PRIMARY
    Tarpc(TarpcClient),
    
    /// JSON-RPC over Unix socket - SECONDARY
    JsonRpc(JsonRpcClient),
    
    /// HTTP REST - FALLBACK ONLY
    Http(HttpClient),
}

/// Unified primal client (protocol-agnostic)
pub struct PrimalClient {
    transport: PrimalTransport,
    timeout: Duration,
}

impl PrimalClient {
    /// Discover and connect to a primal (capability-based!)
    pub async fn discover(capability: CapabilityTaxonomy) -> Result<Self> {
        // 1. Discover primal via PrimalRegistry
        let registry = PrimalRegistry::new("../plasmidBin");
        let primal = registry.get_best_for_capability(capability)?
            .ok_or_else(|| anyhow!("No provider for {:?}", capability))?;
        
        // 2. Detect available transports (in order of preference)
        if let Some(socket) = primal.unix_socket() {
            // Try JSON-RPC over Unix socket (preferred!)
            if Self::test_jsonrpc_socket(&socket).await.is_ok() {
                return Ok(Self::new_jsonrpc(socket));
            }
        }
        
        if let Some(endpoint) = primal.http_endpoint() {
            // Fallback to HTTP (legacy/dev only)
            tracing::warn!("Using HTTP fallback for {:?} - insecure!", capability);
            return Ok(Self::new_http(endpoint));
        }
        
        Err(anyhow!("No available transport for {:?}", capability))
    }
    
    /// Call RPC method (transport-agnostic)
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        match &self.transport {
            PrimalTransport::Tarpc(client) => client.call(method, params).await,
            PrimalTransport::JsonRpc(client) => client.call(method, params).await,
            PrimalTransport::Http(client) => client.call_http(method, params).await,
        }
    }
}
```

---

## 🏗️ **Implementation Plan**

### **Step 1: Create Transport Abstraction** (2-3 hours)

```
crates/biomeos-core/src/clients/transport/
├── mod.rs          # PrimalClient, PrimalTransport enum
├── tarpc.rs        # TarpcClient (future - stub for now)
├── jsonrpc.rs      # JsonRpcClient (Unix socket)
├── http.rs         # HttpClient (legacy fallback)
└── discovery.rs    # Transport discovery logic
```

### **Step 2: Implement JSON-RPC over Unix Socket** (3-4 hours)

**Priority**: This is what Songbird and BearDog already support!

```rust
// transport/jsonrpc.rs

use tokio::net::UnixStream;
use serde_json::Value;

pub struct JsonRpcClient {
    socket_path: String,
    stream: Option<UnixStream>,
}

impl JsonRpcClient {
    pub async fn new(socket_path: String) -> Result<Self> {
        let stream = UnixStream::connect(&socket_path).await?;
        Ok(Self {
            socket_path,
            stream: Some(stream),
        })
    }
    
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1,
        });
        
        // Send request over Unix socket
        // Parse response
        // Return result
    }
}
```

### **Step 3: Migrate Clients to Use PrimalClient** (4-5 hours)

**Example: BearDog**

```rust
// BEFORE: beardog.rs (HTTP only)
pub struct BearDogClient {
    http: PrimalHttpClient,  // ← Hardcoded to HTTP!
    endpoint: String,
}

impl BearDogClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }
    
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        let body = serde_json::json!({ /* ... */ });
        let response = self.http.post("/api/v1/security/encrypt", body).await?;
        // ...
    }
}

// AFTER: beardog.rs (protocol-agnostic!)
pub struct BearDogClient {
    client: PrimalClient,  // ← Protocol-agnostic!
}

impl BearDogClient {
    /// Discover BearDog via capability (uses best transport!)
    pub async fn discover() -> Result<Self> {
        let client = PrimalClient::discover(CapabilityTaxonomy::Encryption).await?;
        Ok(Self { client })
    }
    
    /// Create from explicit endpoint (legacy, for testing)
    pub fn new_http(endpoint: impl Into<String>) -> Self {
        Self {
            client: PrimalClient::new_http(endpoint),
        }
    }
    
    pub async fn encrypt(&self, data: &str, key_id: &str) -> Result<EncryptedData> {
        let params = serde_json::json!({
            "data": data,
            "key_id": key_id,
        });
        
        // Transport-agnostic RPC call!
        let response = self.client.call("security.encrypt", params).await?;
        
        serde_json::from_value(response)
            .context("Failed to parse encrypt response")
    }
}
```

### **Step 4: Add tarpc Support** (Future - stub for now)

Create stub for tarpc, implement when primals add tarpc servers:

```rust
// transport/tarpc.rs

pub struct TarpcClient {
    // Future: tarpc client implementation
}

impl TarpcClient {
    pub async fn call(&self, method: &str, params: Value) -> Result<Value> {
        // TODO: Implement when primals support tarpc
        Err(anyhow!("tarpc not yet implemented"))
    }
}
```

### **Step 5: Deprecate HTTP** (Mark as legacy)

```rust
// transport/http.rs

/// HTTP transport - LEGACY FALLBACK ONLY
/// 
/// ⚠️ WARNING: HTTP is insecure and non-isomorphic!
/// This transport should ONLY be used as a fallback for:
/// - Development/testing
/// - Legacy deployments
/// - Primals that don't yet support Unix sockets
/// 
/// Prefer JSON-RPC over Unix sockets or tarpc instead.
#[deprecated(
    since = "0.2.0",
    note = "HTTP is a legacy fallback. Use JSON-RPC over Unix sockets or tarpc instead."
)]
pub struct HttpClient {
    client: reqwest::Client,
    base_url: String,
}
```

---

## 📊 **Migration Timeline**

### **Wave 2A: Transport Abstraction** (1 week)
- Create transport module structure
- Implement JSON-RPC over Unix socket
- Create PrimalClient abstraction
- Add transport discovery logic
- Stub out tarpc for future

### **Wave 2B: Migrate Clients** (2-3 weeks)
- Migrate beardog.rs (security client)
- Migrate songbird.rs (discovery client)
- Migrate toadstool.rs (compute client)
- Migrate nestgate.rs (storage client)
- Migrate squirrel.rs (AI client)

### **Wave 2C: Testing & Validation** (1 week)
- E2E tests with Unix sockets
- Fallback testing (HTTP still works)
- Performance benchmarks
- Security validation

**Total**: ~4-5 weeks

---

## 🎯 **Success Criteria**

### **Must Have**
1. ✅ JSON-RPC over Unix socket working
2. ✅ All clients use PrimalClient (protocol-agnostic)
3. ✅ HTTP marked as deprecated/fallback
4. ✅ Transport auto-discovery working
5. ✅ All tests passing

### **Nice to Have**
1. ⏳ tarpc stub (for future)
2. ⏳ Performance benchmarks showing Unix socket speedup
3. ⏳ Security analysis of transport layer

---

## 💡 **Why This Matters**

### **Security**
```
HTTP:          cleartext, vulnerable to MITM
Unix sockets:  local only, kernel-enforced permissions
tarpc:         type-safe, encrypted if needed
```

### **Performance**
```
HTTP:          ~10ms overhead (headers, parsing)
Unix sockets:  ~0.1ms overhead (direct IPC)
tarpc:         ~0.05ms overhead (zero-copy)
```

### **Architecture**
```
HTTP:          request/response (non-isomorphic)
Unix sockets:  bidirectional streams (isomorphic!)
tarpc:         bidirectional RPC (isomorphic!)
```

---

## 🚀 **Immediate Next Steps**

1. **Create transport module structure** (30 min)
2. **Implement JsonRpcClient** (2-3 hours)
3. **Update BearDogClient to use PrimalClient** (1-2 hours)
4. **Test with BearDog Unix socket** (30 min)
5. **Document migration pattern** (30 min)

**Estimated**: ~1 day for initial proof-of-concept

---

## 🎊 **Bottom Line**

**The file naming (`beardog.rs`) is fine - it's semantically correct.**

**The REAL issue**: We're using HTTP everywhere when we should be using:
1. **JSON-RPC over Unix sockets** (primary)
2. **tarpc** (future, when available)
3. **HTTP** (fallback only, deprecated)

This is a **security and architecture issue**, not a naming issue!

**Ready to proceed with transport evolution?** 🎯


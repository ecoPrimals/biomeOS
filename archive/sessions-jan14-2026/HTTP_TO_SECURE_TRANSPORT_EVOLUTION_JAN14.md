# 🔒 HTTP → Secure Transport Evolution Plan

**Date**: January 14, 2026  
**Status**: 🚨 **CRITICAL - HTTP STILL IN USE**  
**Goal**: Eliminate HTTP, evolve to JSON-RPC over Unix sockets + tarpc

---

## 🚨 The Problem

**User's Critical Insight**:
> "biomeOS still has systems not tied into JSON-RPC and tarpc?  
> We need to get those more evolved as they are our systems for  
> intercommunication and more secure than HTTP."

**Answer**: ✅ **CORRECT! We have the abstraction but aren't using it everywhere!**

---

## 📊 Audit Results

### **HTTP References Found**: 85 files! 🚨

| Category | Files | Status |
|----------|-------|--------|
| **Client Transport** | 10 | ⚠️ Has abstraction, but HTTP fallback used |
| **biomeOS API** | 5 | 🚨 HTTP-only (port 3000) |
| **neuralAPI (planned)** | 1 | 🚨 Spec uses HTTP! |
| **Discovery** | 8 | ⚠️ Mixed (HTTP fallback) |
| **Tests** | 20 | ⚠️ Many use HTTP mocks |
| **Federation** | 10 | ⚠️ HTTP fallback exists |
| **Dependencies (Cargo)** | 8 | ⚠️ reqwest, hyper |
| **Legacy/Archive** | 23 | ✅ OK (archived) |

### **JSON-RPC/tarpc Usage**: 27 files (good foundation!)

**We have the infrastructure but aren't using it consistently!**

---

## 🏗️ Current Architecture (Mixed State)

### **✅ What's GOOD (Secure)**

1. **Transport Abstraction Built** (`crates/biomeos-core/src/clients/transport/`)
   - ✅ `PrimalTransport` with auto-discovery
   - ✅ JSON-RPC over Unix sockets (PRIMARY)
   - ✅ tarpc stub (FUTURE)
   - ✅ HTTP (FALLBACK - deprecated)

2. **Priority Order**:
   ```rust
   TransportPreference::Auto => {
       // ✅ Priority: Unix socket > tarpc > HTTP
       Self::try_unix_socket(primal_name, family_id)
           .await
           .or_else(|_| Self::try_http(primal_name))  // ⚠️ FALLBACK!
   }
   ```

3. **Primals Using Unix Sockets**:
   - ✅ BearDog: `/run/user/{uid}/beardog-{family}.sock`
   - ✅ Songbird: `/run/user/{uid}/songbird-{family}.sock`
   - ✅ ToadStool: `/run/user/{uid}/toadstool-{family}.sock`
   - ✅ NestGate: `/run/user/{uid}/nestgate-{family}.sock`

---

### **🚨 What's BAD (Insecure)**

#### **1. biomeOS API Server** 🚨 HIGH PRIORITY

**Current**: HTTP-only (port 3000)

**Location**: `crates/biomeos-api/src/main.rs`

```rust
// 🚨 PROBLEM: HTTP server on TCP port!
let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
let listener = TcpListener::bind(addr).await?;
axum::serve(listener, app).await?;
```

**Why it's a problem**:
- ❌ Uses TCP port (not port-free!)
- ❌ HTTP (not encrypted by default)
- ❌ Not aligned with TRUE PRIMAL architecture
- ❌ Doesn't use genetic lineage for auth

**What it SHOULD be**:
```rust
// ✅ CORRECT: Unix socket + JSON-RPC
let socket_path = format!("/run/user/{}/biomeos-api.sock", getuid());
let listener = UnixListener::bind(socket_path)?;
// Serve JSON-RPC 2.0 over Unix socket
```

---

#### **2. neuralAPI Server (Planned)** 🚨 CRITICAL

**My Spec (1 hour ago)**: Used HTTP! 🚨

**From `specs/NEURAL_API_SERVER_IMPLEMENTATION_SPEC.md`**:
```markdown
NEURAL_API_BIND_ADDR = "127.0.0.1:8000"  # ❌ WRONG!
```

**What it SHOULD be**:
```markdown
NEURAL_API_SOCKET_PATH = "/run/user/{uid}/neural-api.sock"  # ✅ CORRECT!
```

**The spec already mentions Unix sockets but defaults to HTTP!** 🚨

---

#### **3. HTTP Fallback in Transport** ⚠️ MODERATE

**Location**: `crates/biomeos-core/src/clients/transport/mod.rs`

```rust
// ⚠️ PROBLEM: Tries HTTP when Unix socket fails
TransportPreference::Auto => {
    Self::try_unix_socket(primal_name, family_id)
        .await
        .or_else(|_| Self::try_http(primal_name))  // ⚠️ This!
}
```

**Why it's a problem**:
- Silently falls back to insecure HTTP
- Doesn't fail fast when primal isn't running
- Allows HTTP to "just work" (bad for security!)

**What it SHOULD be**:
```rust
TransportPreference::Auto => {
    Self::try_unix_socket(primal_name, family_id)
        .await
        .or_else(|_| {
            // ✅ Log warning, THEN try tarpc, THEN fail!
            warn!("Unix socket failed, trying tarpc...");
            Self::try_tarpc(primal_name, family_id)
                .await
                .map_err(|e| anyhow!("No secure transport available: {}", e))
        })
}
```

---

#### **4. Discovery HTTP Fallback** ⚠️ MODERATE

**Locations**:
- `crates/biomeos-core/src/discovery_http.rs` - Entire file!
- `crates/biomeos-core/src/discovery_bootstrap.rs` - HTTP references

**Problem**: Discovery can fall back to HTTP instead of failing

---

## 🎯 Evolution Plan

### **Phase 1: Fix biomeOS API** (HIGH PRIORITY - 4-6h)

**Goal**: Make biomeOS API use Unix socket + JSON-RPC (port-free!)

**Changes**:

1. **Create Unix Socket Server**

```rust
// crates/biomeos-api/src/main.rs
use tokio::net::UnixListener;
use std::path::Path;

#[tokio::main]
async fn main() -> Result<()> {
    // ✅ Port-free Unix socket!
    let socket_path = format!("/run/user/{}/biomeos-api.sock", nix::unistd::getuid());
    
    // Remove old socket if exists
    let _ = std::fs::remove_file(&socket_path);
    
    // Create Unix listener
    let listener = UnixListener::bind(&socket_path)?;
    info!("📡 biomeOS API listening on Unix socket: {}", socket_path);
    
    // Set permissions (0600 - owner only)
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))?;
    }
    
    // Serve JSON-RPC 2.0
    let app = create_jsonrpc_router(state);
    serve_over_unix_socket(listener, app).await
}
```

2. **Add HTTP Bridge (Optional, Temporary)**

```rust
// ONLY if PetalTongue can't use Unix sockets yet
// This is a TEMPORARY bridge, to be removed!

#[cfg(feature = "http-bridge")]
{
    warn!("⚠️ HTTP bridge enabled (temporary - remove in production!)");
    tokio::spawn(async move {
        let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
        let listener = TcpListener::bind(addr).await?;
        
        // Proxy HTTP → Unix socket
        http_to_unix_proxy(listener, socket_path).await
    });
}
```

**Timeline**: 4-6 hours

---

### **Phase 2: Fix neuralAPI Server Spec** (IMMEDIATE - 30min)

**Goal**: Update spec to use Unix socket + JSON-RPC by default

**Changes to `specs/NEURAL_API_SERVER_IMPLEMENTATION_SPEC.md`**:

```markdown
## 🔧 Configuration

### **Environment Variables**

| Variable | Purpose | Default |
|----------|---------|---------|
| `NEURAL_API_SOCKET_PATH` | Unix socket path | `/run/user/{uid}/neural-api.sock` ✅ |
| `NEURAL_API_PROTOCOL` | Protocol (jsonrpc, tarpc) | `jsonrpc` ✅ |
| ~~`NEURAL_API_BIND_ADDR`~~ | ❌ REMOVED (HTTP deprecated!) | ❌ |

### **Transport**

**PRIMARY**: JSON-RPC 2.0 over Unix socket
- Path: `/run/user/{uid}/neural-api.sock`
- Protocol: JSON-RPC 2.0
- Security: Owner-only (0600 permissions)

**FUTURE**: tarpc over Unix socket
- Type-safe, bidirectional
- Genetic lineage verification

**DEPRECATED**: ~~HTTP~~ (removed for security!)
```

**Timeline**: 30 minutes (spec update)

---

### **Phase 3: Implement tarpc Support** (MEDIUM PRIORITY - 8-12h)

**Goal**: Add tarpc as the type-safe, bidirectional transport

**Why tarpc**:
- ✅ Type-safe (compile-time API verification)
- ✅ Bidirectional (server can call client!)
- ✅ Efficient (binary protocol)
- ✅ Already used by BearDog!

**Changes**:

1. **Add tarpc Module**

```rust
// crates/biomeos-core/src/clients/transport/tarpc.rs
use tarpc::{client, context, server};
use std::path::Path;

#[tarpc::service]
pub trait PrimalService {
    /// Call a primal method
    async fn call_method(method: String, params: serde_json::Value) -> anyhow::Result<serde_json::Value>;
    
    /// Check health
    async fn health_check() -> HealthStatus;
    
    /// Subscribe to events (bidirectional!)
    async fn subscribe_events() -> EventStream;
}

pub struct TarpcClient {
    client: PrimalServiceClient,
}

impl TarpcClient {
    pub async fn new<P: AsRef<Path>>(socket_path: P) -> Result<Self> {
        // Connect to Unix socket via tarpc
        let transport = tarpc::serde_transport::unix::connect(socket_path).await?;
        let client = PrimalServiceClient::new(client::Config::default(), transport).spawn();
        Ok(Self { client })
    }
    
    pub async fn call_method(&self, method: &str, params: Option<serde_json::Value>) -> Result<serde_json::Value> {
        self.client.call_method(context::current(), method.to_string(), params.unwrap_or(serde_json::Value::Null)).await?
    }
}
```

2. **Update Transport Enum**

```rust
// crates/biomeos-core/src/clients/transport/mod.rs
enum Transport {
    UnixSocket(jsonrpc::JsonRpcUnixClient),
    Tarpc(tarpc::TarpcClient),  // ✅ Add this!
    // ❌ Remove: Http(http::HttpClient),  // Deprecated!
}
```

**Timeline**: 8-12 hours

---

### **Phase 4: Remove HTTP Fallback** (FINAL - 2-4h)

**Goal**: Make secure transport mandatory (fail fast if not available)

**Changes**:

1. **Update TransportPreference**

```rust
pub enum TransportPreference {
    /// JSON-RPC over Unix socket (PRIMARY)
    UnixSocket,
    /// tarpc (TYPE-SAFE)
    Tarpc,
    /// Auto-select secure transport (Unix socket → tarpc)
    Auto,
    // ❌ REMOVED: Http (no longer supported!)
}

impl Default for TransportPreference {
    fn default() -> Self {
        Self::Auto  // ✅ Auto-selects ONLY secure transports!
    }
}
```

2. **Update Auto Discovery**

```rust
TransportPreference::Auto => {
    // ✅ Try Unix socket first
    Self::try_unix_socket(primal_name, family_id)
        .await
        .or_else(|e1| {
            debug!("Unix socket failed: {}, trying tarpc...", e1);
            // ✅ Then try tarpc
            Self::try_tarpc(primal_name, family_id)
                .await
                .map_err(|e2| {
                    // ❌ FAIL FAST - no HTTP fallback!
                    anyhow!(
                        "No secure transport available for {}: \
                         Unix socket: {}, tarpc: {}",
                        primal_name, e1, e2
                    )
                })
        })
}
```

3. **Remove HTTP Module**

```bash
# Delete HTTP transport entirely
rm crates/biomeos-core/src/clients/transport/http.rs

# Update mod.rs
# - Remove: pub mod http;
# + Add: // HTTP removed - use Unix socket + tarpc only!
```

**Timeline**: 2-4 hours

---

## 📊 Success Criteria

### **Phase 1 Complete** ✅
- [ ] biomeOS API uses Unix socket (`/run/user/{uid}/biomeos-api.sock`)
- [ ] No TCP ports in use for biomeOS API
- [ ] JSON-RPC 2.0 protocol
- [ ] PetalTongue still works (via Unix socket or HTTP bridge)

### **Phase 2 Complete** ✅
- [ ] neuralAPI spec updated (Unix socket default)
- [ ] HTTP removed from spec
- [ ] tarpc mentioned as future transport

### **Phase 3 Complete** ✅
- [ ] tarpc module implemented
- [ ] tarpc transport preference works
- [ ] Type-safe primal calls
- [ ] Bidirectional communication (events)

### **Phase 4 Complete** ✅
- [ ] HTTP completely removed from codebase
- [ ] All inter-primal communication via Unix socket or tarpc
- [ ] Zero TCP ports (except temporary HTTP bridge if needed)
- [ ] Secure by default

---

## 🔧 Implementation Priority

| Phase | Priority | Hours | Blocking |
|-------|----------|-------|----------|
| **Phase 2** (Spec Fix) | 🚨 CRITICAL | 0.5h | neuralAPI implementation |
| **Phase 1** (biomeOS API) | 🔴 HIGH | 4-6h | PetalTongue integration |
| **Phase 3** (tarpc) | 🟡 MEDIUM | 8-12h | Type-safe calls |
| **Phase 4** (Remove HTTP) | 🟢 FINAL | 2-4h | Security hardening |

**Total**: 15-23 hours (2-3 work days)

---

## 🎯 Port-Free Architecture (Final State)

### **Before (Current - 🚨)**

```
biomeOS API:      HTTP port 3000        ❌
neuralAPI:        HTTP port 8000        ❌
Primals:          Unix sockets          ✅
PetalTongue:      HTTP client           ❌
Discovery:        HTTP fallback         ❌
```

### **After (Goal - ✅)**

```
biomeOS API:      Unix socket           ✅
neuralAPI:        Unix socket           ✅
Primals:          Unix sockets          ✅
PetalTongue:      Unix socket client    ✅
Discovery:        Unix socket only      ✅
Fallback:         tarpc (if needed)     ✅
HTTP:             ❌ REMOVED             ✅
```

**Result**: TRUE PRIMAL architecture - port-free, secure, fast!

---

## 📚 Related Specifications

- **[GENETIC_LINEAGE_ARCHITECTURE_SPEC.md](specs/GENETIC_LINEAGE_ARCHITECTURE_SPEC.md)** - Auth via lineage
- **[TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)** - Port-free goals
- **[NEURAL_API_SERVER_IMPLEMENTATION_SPEC.md](specs/NEURAL_API_SERVER_IMPLEMENTATION_SPEC.md)** - Needs update!
- **[ENCRYPTION_STRATEGY_SPEC.md](specs/ENCRYPTION_STRATEGY_SPEC.md)** - Transport encryption

---

## 🎊 Conclusion

**User is 100% correct!**

We have:
- ✅ The secure transport abstraction built
- ✅ JSON-RPC over Unix sockets working
- ✅ Primals using Unix sockets

But we still have:
- 🚨 biomeOS API using HTTP (port 3000)
- 🚨 neuralAPI spec defaulting to HTTP
- ⚠️ HTTP fallback everywhere
- ⚠️ 85 files with HTTP references

**Next Actions**:
1. Fix neuralAPI spec (30min) ← DO THIS NOW!
2. Evolve biomeOS API to Unix socket (4-6h)
3. Implement tarpc (8-12h)
4. Remove HTTP entirely (2-4h)

**Total**: 15-23 hours to achieve TRUE PRIMAL port-free architecture!

---

**Created**: January 14, 2026  
**Status**: 🚨 CRITICAL EVOLUTION NEEDED  
**Priority**: HIGH (Security & Architecture alignment)

**"Port-free, secure, fast - the TRUE PRIMAL way!"** 🔒🚀✨


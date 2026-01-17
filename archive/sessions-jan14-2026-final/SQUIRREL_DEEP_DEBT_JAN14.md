# 🐿️ Squirrel Deep Debt - TRUE PRIMAL Violation

**Date**: January 14, 2026 19:25 UTC  
**Status**: 🔴 **DEEP DEBT IDENTIFIED**  
**Priority**: HIGH (blocks full NUCLEUS integration)

---

## 🚨 **Issue**

Squirrel is running an HTTP API server on port 9010 and attempting to register with Songbird via HTTP, violating TRUE PRIMAL architecture.

### **What's Wrong**
```rust
// In squirrel/crates/main/src/biomeos_integration/ecosystem_client.rs:
let songbird_url = std::env::var("SERVICE_MESH_ENDPOINT")
    .unwrap_or_else(|_| "http://localhost:8080".to_string());

Self {
    songbird_url,
    client: reqwest::Client::new(), // ❌ HTTP client!
    ...
}
```

### **TRUE PRIMAL Violations**
1. ❌ **HTTP for inter-primal communication** - Should use Unix sockets
2. ❌ **Hardcoded HTTP URLs** - Should use capability discovery
3. ❌ **reqwest HTTP client** - Should use JSON-RPC over Unix socket
4. ❌ **Port 9010** - Should be port-free for inter-primal comm

### **What's Working**
✅ Squirrel exposes JSON-RPC Unix socket: `/tmp/squirrel-squirrel.sock`  
✅ biomeOS can discover and connect via this socket  
✅ HTTP API can remain for external/human access (optional)

---

## 🎯 **Solution**

### **Short-Term (Now)**
✅ **biomeOS discovers Squirrel directly via Unix socket**
- Use `SquirrelClient::discover()` in biomeOS
- Connect to `/tmp/squirrel-squirrel.sock`
- Skip Squirrel's self-registration with Songbird

### **Long-Term (After Phase 2)**
🔄 **Evolve Squirrel's Songbird client to use Unix sockets**
```rust
// Evolution needed in Squirrel:
// FROM: HTTP reqwest client
// TO:   JSON-RPC Unix socket client (like biomeOS clients)

impl EcosystemClient {
    pub fn new() -> Self {
        // Use TransportClient or equivalent
        let songbird_socket = std::env::var("SONGBIRD_SOCKET")
            .unwrap_or_else(|_| "/run/user/1000/songbird-nat0.sock".to_string());
        
        Self {
            transport: UnixSocketTransport::new(&songbird_socket),
            ...
        }
    }
}
```

---

## 📋 **Action Items**

### **Immediate (biomeOS Side)**
- [x] Stop HTTP-based Squirrel instance
- [ ] Use `SquirrelClient::discover()` for Unix socket connection
- [ ] Test AI inference via JSON-RPC socket
- [ ] Document workaround

### **Future (Squirrel Side)**
- [ ] Evolve `EcosystemClient` to use Unix sockets
- [ ] Replace `reqwest` with JSON-RPC client
- [ ] Make HTTP API optional (for external access only)
- [ ] Add Songbird Unix socket discovery
- [ ] Add BearDog security integration

---

## 🧬 **TRUE PRIMAL Compliance**

### **Current State**
| Component | Status | Notes |
|-----------|--------|-------|
| JSON-RPC socket | ✅ Working | `/tmp/squirrel-squirrel.sock` |
| Songbird discovery | ❌ HTTP | Should use socket |
| BearDog security | ❌ Missing | No genetic lineage |
| Inter-primal comm | ⚠️ Mixed | Socket exists, but HTTP used |

### **Target State**
| Component | Status | Notes |
|-----------|--------|-------|
| JSON-RPC socket | ✅ Working | Primary IPC |
| Songbird discovery | 🎯 Socket | `/run/user/1000/songbird-nat0.sock` |
| BearDog security | 🎯 Integrated | Genetic key validation |
| Inter-primal comm | 🎯 Port-free | 100% Unix sockets |

---

## 💡 **Why This Matters**

1. **Security**: HTTP is less secure than Unix sockets + BearDog
2. **Performance**: Unix sockets are faster (no TCP/IP overhead)
3. **Architecture**: TRUE PRIMAL requires capability-based discovery
4. **Consistency**: All primals should use same transport patterns

---

## 🚀 **Workaround for Now**

Since Squirrel already exposes a JSON-RPC Unix socket, biomeOS can:

```bash
# biomeOS discovers Squirrel directly
export SQUIRREL_SOCKET=/tmp/squirrel-squirrel.sock

# biomeOS connects via SquirrelClient
# No need for Squirrel to self-register with Songbird
# Songbird discovery happens from biomeOS side
```

This allows full NUCLEUS integration while documenting the deep debt for future evolution.

---

## 📖 **References**

- `crates/biomeos-core/src/clients/squirrel.rs` - biomeOS Squirrel client
- `squirrel/crates/main/src/biomeos_integration/ecosystem_client.rs` - HTTP client issue
- `squirrel/crates/main/src/rpc/server.rs` - Working Unix socket server
- TRUE PRIMAL principles: Port-free, capability-based, secure-by-default

---

**Status**: ✅ Documented, ⏳ Workaround in progress  
**Next**: Use biomeOS-side discovery to integrate Squirrel  
**Future**: Evolve Squirrel to full TRUE PRIMAL compliance


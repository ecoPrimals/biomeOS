# 🔴 CRITICAL: BearDog Still Binding HTTP Port (Not Port-Free)

**Date**: January 8, 2026  
**Status**: 🔴 **BLOCKER** - Prevents proper port-free architecture  
**For**: BearDog Team (URGENT)

---

## 🎯 The Problem

BearDog v0.15.0 is **still binding to HTTP port 9000** even though:
1. ✅ Tower configuration specifies Unix socket only
2. ✅ No `HTTP_PORT` or `BEARDOG_BIND_ADDR` environment variables set
3. ✅ Unix socket path is correctly specified
4. ❌ BearDog ignores this and binds HTTP anyway

---

## 📋 Evidence

### Tower Configuration (tower.toml)
```toml
# BearDog v0.15.0 - Security Primal (Port-Free!)
[[primals]]
binary = "./primals/beardog-server"
provides = ["Security", "Encryption", "Trust"]
requires = []

[primals.env]
# ✅ SECURE: File-based seed (BearDog v0.15.0 reads the file)
BEARDOG_FAMILY_SEED_FILE = "./.family.seed"
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_NODE_ID = "node-alpha"
RUST_LOG = "info"

# NO HTTP_PORT
# NO BEARDOG_BIND_ADDR
# ✅ Unix socket should be auto-created at /tmp/beardog-nat0-node-alpha.sock
```

### BearDog Startup Logs
```log
🚀 Step 4: Configuring API Server...
⚠️  No BEARDOG_BIND_ADDR or HTTP_PORT set, using default: 0.0.0.0:9000
   Bind Address: 0.0.0.0:9000
   CORS Enabled: true

╔════════════════════════════════════════════════════════════════╗
║  HTTP API: http://0.0.0.0:9000                                ║
║  Unix Socket: /tmp/primals/beardog-{node}.sock                ║
╚════════════════════════════════════════════════════════════════╝

🚀 BearDog API Server starting on 0.0.0.0:9000
```

**Problem**: BearDog is **falling back to HTTP port 9000** when it should be **Unix socket ONLY**.

---

## ✅ Correct Architecture (User's Vision)

### What Should Happen:
```
BearDog starts with NO environment variables
  ↓
  ├─ NO HTTP_PORT set → ✅ DON'T bind HTTP
  ├─ NO BEARDOG_BIND_ADDR set → ✅ DON'T bind HTTP
  └─ Unix socket ONLY: /tmp/beardog-{family}-{node}.sock
  
Songbird can assign port IF NEEDED (but it isn't!)
```

### Communication Flow:
```
Tower → BearDog
  ↓ Unix Socket: /tmp/beardog-nat0-node-alpha.sock
  ↓ Protocol: JSON-RPC / tarpc
  ✅ ZERO HTTP ports

Songbird → BearDog
  ↓ Unix Socket: /tmp/beardog-nat0-node-alpha.sock
  ↓ Protocol: JSON-RPC / tarpc
  ✅ ZERO HTTP ports

BearDog ↔ BearDog (remote towers)
  ↓ UDP + BTSP tunnel
  ↓ Encrypted P2P
  ✅ ZERO HTTP ports
```

---

## 🔧 Root Cause in BearDog Code

### Current Buggy Code (beardog-server.rs:126-133)
```rust
// Determine bind address from environment
let bind_addr = std::env::var("BEARDOG_BIND_ADDR")
    .or_else(|_| {
        std::env::var("HTTP_PORT").map(|port| format!("0.0.0.0:{}", port))
    })
    .unwrap_or_else(|_| {
        warn!("   No BEARDOG_BIND_ADDR or HTTP_PORT set, using default: 0.0.0.0:9000");
        "0.0.0.0:9000".to_string()  // ❌ WRONG! Should not bind HTTP at all!
    })
    .parse()
    .map_err(|e| {
        error!("Invalid bind address: {}", e);
        BearDogError::configuration(&format!("Invalid bind address: {}", e))
    })?;
```

**Problem**: The `unwrap_or_else` fallback **always creates an HTTP server** even when not needed!

---

## ✅ Required Fix

### Option 1: Make HTTP Optional (Recommended)

```rust
// Read HTTP configuration (OPTIONAL)
let http_enabled = std::env::var("BEARDOG_HTTP_ENABLED")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(false);  // ✅ Default: NO HTTP

if http_enabled {
    info!("🌐 HTTP API enabled");
    
    let bind_addr = std::env::var("BEARDOG_BIND_ADDR")
        .or_else(|_| std::env::var("HTTP_PORT").map(|port| format!("0.0.0.0:{}", port)))
        .unwrap_or_else(|_| "0.0.0.0:9000".to_string())
        .parse()?;
    
    let config = BearDogApiServerConfig {
        bind_addr,
        enable_cors: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    let server = BearDogApiServer::new(config, btsp_provider).await?;
    // ... start HTTP server
} else {
    info!("🔌 Unix socket ONLY (port-free mode)");
    info!("   Socket: /tmp/beardog-{}-{}.sock", family_id, node_id);
    // ✅ NO HTTP SERVER CREATED
}

// ✅ Unix socket IPC server (ALWAYS runs)
let socket_path = format!("/tmp/beardog-{}-{}.sock", family_id, node_id);
let unix_server = UnixSocketIpcServer::new(socket_path, btsp_provider).await?;
unix_server.serve().await?;
```

### Option 2: Disable HTTP by Default (Simpler)

```rust
// Read HTTP configuration
let http_port = std::env::var("HTTP_PORT")
    .ok()
    .and_then(|p| p.parse::<u16>().ok());

let bind_addr = std::env::var("BEARDOG_BIND_ADDR")
    .ok()
    .and_then(|a| a.parse().ok());

// ✅ Only bind HTTP if explicitly requested
if let Some(addr) = bind_addr.or_else(|| {
    http_port.map(|p| SocketAddr::from(([0, 0, 0, 0], p)))
}) {
    info!("🌐 HTTP API: {}", addr);
    // Create and start HTTP server
} else {
    info!("🔌 Unix socket ONLY (no HTTP)");
}

// ✅ Unix socket always available
let socket_path = format!("/tmp/beardog-{}-{}.sock", family_id, node_id);
// ... Unix socket server
```

---

## 🎯 Impact

### Current (Broken):
- ❌ BearDog always binds HTTP port 9000
- ❌ Multiple instances conflict
- ❌ Not truly port-free
- ❌ Unnecessary HTTP attack surface

### After Fix:
- ✅ BearDog uses Unix socket only by default
- ✅ HTTP only if explicitly enabled
- ✅ True port-free architecture
- ✅ Minimal attack surface
- ✅ Multiple instances work (different sockets)

---

## 📝 Testing Checklist

After fix, verify:

1. **No HTTP port bound by default**:
   ```bash
   BEARDOG_FAMILY_ID=test \
   BEARDOG_NODE_ID=test1 \
   ./beardog-server &
   
   netstat -tuln | grep 9000  # Should be EMPTY
   ls /tmp/beardog-test-test1.sock  # Should EXIST
   ```

2. **HTTP only when requested**:
   ```bash
   BEARDOG_HTTP_ENABLED=true \
   HTTP_PORT=9000 \
   BEARDOG_FAMILY_ID=test \
   BEARDOG_NODE_ID=test2 \
   ./beardog-server &
   
   netstat -tuln | grep 9000  # Should show LISTENING
   ```

3. **Multiple instances work**:
   ```bash
   # Start 2 instances with different node IDs
   BEARDOG_NODE_ID=alpha ./beardog-server &
   BEARDOG_NODE_ID=beta ./beardog-server &
   
   ls /tmp/beardog-*-alpha.sock  # Should EXIST
   ls /tmp/beardog-*-beta.sock   # Should EXIST
   netstat -tuln  # Should show NO beardog ports
   ```

---

## 🚦 Priority

**CRITICAL** - This blocks:
- ✅ Multi-spore deployment (port conflicts)
- ✅ True port-free architecture
- ✅ Secure default configuration
- ✅ biomeOS genetic lineage testing

---

## 📋 Files to Modify

1. **`crates/beardog-tunnel/src/bin/beardog-server.rs`**
   - Lines 126-150 (HTTP binding logic)
   - Make HTTP optional, Unix socket primary

2. **Environment Variables** (update docs):
   - `BEARDOG_HTTP_ENABLED` - Enable HTTP (default: false)
   - `HTTP_PORT` - HTTP port (only if enabled)
   - `BEARDOG_BIND_ADDR` - Full bind address (only if enabled)

---

## 🎊 User Feedback

> "port 900 is a sign we are NOT setting this up properly. should be udp ipc unix sockets. songbird can assign for beardog"

**Translation**:
- BearDog should NOT bind HTTP ports by default
- Unix socket IPC is the primary transport
- If HTTP is needed, Songbird coordinates port assignment
- Current behavior (always binding 9000) is incorrect

---

**Next**: BearDog team implements fix, then we can test genetic lineage verification! 🌱


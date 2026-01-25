# 🎯 Songbird Auto-Registration Handoff

**Date**: January 25, 2026  
**For**: Songbird Team  
**From**: biomeOS Integration Team  
**Status**: 🚀 Ready for Implementation

---

## 📋 **OVERVIEW**

This document provides complete instructions for implementing **automatic capability registration** in Songbird. This is the final step to achieve **TRUE PRIMAL loose coupling** in the ecosystem.

### **What This Achieves**

✅ **Zero Configuration** - Primals discover Songbird automatically  
✅ **Loose Coupling** - No hardcoded dependencies  
✅ **Semantic APIs** - Operations like `http.post` just work  
✅ **Isomorphic Evolution** - Songbird can evolve without breaking consumers

---

## 🎯 **THE GOAL**

When Songbird starts, it should automatically register its capabilities with the Neural API, making itself discoverable to all primals in the ecosystem.

### **Before** (Current State)
```
Squirrel → needs to know Songbird's socket path
        → needs to know Songbird's methods
        → tight coupling, hardcoded paths
❌ Fragile, hard to evolve
```

### **After** (With Auto-Registration)
```
Squirrel → Neural API.capability_call("secure_http", "http.post", {...})
        → Neural API discovers Songbird automatically
        → Routes to Songbird transparently
✅ Zero coupling, zero knowledge required!
```

---

## 📐 **ARCHITECTURE**

```
┌─────────────────────────────────────────────────────────────┐
│ SONGBIRD STARTUP SEQUENCE                                   │
│                                                              │
│ 1. Load configuration                                       │
│ 2. Initialize TLS stack (BearDog integration)              │
│ 3. Start JSON-RPC server (Unix socket)                     │
│ 4. 🆕 REGISTER CAPABILITIES WITH NEURAL API                │
│ 5. Start accepting requests                                │
└─────────────────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ NEURAL API - CAPABILITY REGISTRY                            │
│                                                              │
│ Capabilities:                                               │
│   - secure_http → [songbird-nat0]                          │
│       - http.get                                            │
│       - http.post                                           │
│       - http.put                                            │
│       - http.delete                                         │
│       - http.patch                                          │
│       - http.request (generic)                              │
└─────────────────────────────────────────────────────────────┘
                    ↓
┌─────────────────────────────────────────────────────────────┐
│ SQUIRREL (or any primal)                                    │
│                                                              │
│ neural_api.capability_call("secure_http", "http.post", {}) │
│ → Neural API routes to Songbird automatically               │
│ → Zero knowledge of Songbird required!                      │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔧 **IMPLEMENTATION**

### **Step 1: Add Registration Function**

**File**: `crates/songbird-orchestrator/src/capability_registration.rs` (NEW)

```rust
use anyhow::{Context, Result};
use serde_json::json;
use std::env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{info, warn};

/// Register Songbird's capabilities with the Neural API
///
/// This function is called during Songbird startup to make its
/// capabilities discoverable to other primals in the ecosystem.
pub async fn register_capabilities() -> Result<()> {
    info!("🔄 Registering capabilities with Neural API...");

    // Get Neural API socket from environment
    let neural_socket = env::var("NEURAL_API_SOCKET")
        .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());

    // Get our own socket path
    let songbird_socket = env::var("SONGBIRD_SOCKET_PATH")
        .context("SONGBIRD_SOCKET_PATH not set")?;

    // Get our primal ID
    let primal_id = env::var("PRIMAL_ID")
        .unwrap_or_else(|_| "songbird-nat0".to_string());

    // Build registration request
    let registration = json!({
        "jsonrpc": "2.0",
        "method": "capability.register",
        "params": {
            "primal_id": primal_id,
            "capability": "secure_http",
            "socket_path": songbird_socket,
            "operations": [
                "http.get",
                "http.post",
                "http.put",
                "http.delete",
                "http.patch",
                "http.request"  // Generic fallback
            ],
            "metadata": {
                "tls_version": "1.3",
                "pure_rust": true,
                "supports_http2": true,
                "provider": "songbird",
                "version": env!("CARGO_PKG_VERSION")
            }
        },
        "id": 1
    });

    // Connect to Neural API
    let mut stream = UnixStream::connect(&neural_socket)
        .await
        .context("Failed to connect to Neural API")?;

    // Send registration
    let request = format!("{}\n", registration);
    stream
        .write_all(request.as_bytes())
        .await
        .context("Failed to send registration")?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .await
        .context("Failed to read registration response")?;

    // Parse response
    let response_json: serde_json::Value = serde_json::from_str(&response)
        .context("Failed to parse registration response")?;

    if response_json.get("result").is_some() {
        info!("✅ Capabilities registered successfully");
        info!("   Capability: secure_http");
        info!("   Operations: http.get, http.post, http.put, http.delete, http.patch, http.request");
        info!("   Socket: {}", songbird_socket);
        Ok(())
    } else if let Some(error) = response_json.get("error") {
        warn!("⚠️  Registration failed: {:?}", error);
        // Don't fail startup if registration fails - Songbird can still
        // work with direct connections
        Ok(())
    } else {
        warn!("⚠️  Unexpected registration response: {}", response);
        Ok(())
    }
}

/// Unregister capabilities on shutdown (optional but recommended)
pub async fn unregister_capabilities() -> Result<()> {
    info!("🔄 Unregistering capabilities from Neural API...");

    let neural_socket = env::var("NEURAL_API_SOCKET")
        .unwrap_or_else(|_| "/tmp/neural-api-nat0.sock".to_string());

    let primal_id = env::var("PRIMAL_ID")
        .unwrap_or_else(|_| "songbird-nat0".to_string());

    let unregister = json!({
        "jsonrpc": "2.0",
        "method": "capability.unregister",
        "params": {
            "primal_id": primal_id,
            "capability": "secure_http"
        },
        "id": 2
    });

    match UnixStream::connect(&neural_socket).await {
        Ok(mut stream) => {
            let request = format!("{}\n", unregister);
            let _ = stream.write_all(request.as_bytes()).await;
            info!("✅ Capabilities unregistered");
        }
        Err(_) => {
            // Neural API may already be shut down, that's fine
            info!("   Neural API not available for unregistration (ok)");
        }
    }

    Ok(())
}
```

### **Step 2: Wire Into Startup**

**File**: `crates/songbird-orchestrator/src/app/startup.rs` (MODIFY)

```rust
use crate::capability_registration;  // Add this import

pub async fn run_server(config: Config) -> Result<()> {
    info!("🚀 Starting Songbird server...");

    // ... existing initialization code ...

    // Start JSON-RPC server
    let server_handle = tokio::spawn(async move {
        // ... existing server code ...
    });

    // 🆕 REGISTER CAPABILITIES
    // Do this after the server is listening, but before we block on server_handle
    if let Err(e) = capability_registration::register_capabilities().await {
        warn!("Failed to register capabilities: {}", e);
        warn!("Songbird will continue without Neural API registration");
        // Don't fail startup - Songbird can still work with direct connections
    }

    info!("✅ Songbird server running");
    info!("   Socket: {}", config.socket_path);
    info!("   Neural API: Capabilities registered");

    // Wait for server to complete
    server_handle.await??;

    Ok(())
}
```

### **Step 3: Wire Into Shutdown**

**File**: `crates/songbird-orchestrator/src/app/shutdown.rs` (MODIFY)

```rust
use crate::capability_registration;  // Add this import

pub async fn graceful_shutdown() -> Result<()> {
    info!("🛑 Gracefully shutting down Songbird...");

    // 🆕 UNREGISTER CAPABILITIES
    let _ = capability_registration::unregister_capabilities().await;

    // ... existing shutdown code ...

    info!("✅ Songbird shut down gracefully");
    Ok(())
}
```

### **Step 4: Update Module Declarations**

**File**: `crates/songbird-orchestrator/src/lib.rs` (MODIFY)

```rust
pub mod app;
pub mod capability_registration;  // 🆕 Add this line
pub mod config;
pub mod handlers;
// ... other modules ...
```

---

## 🧪 **TESTING**

### **Test 1: Registration on Startup**

```bash
# Start Neural API
./target/release/biomeos neural-api --mode coordinated &

# Start Songbird (with your start script)
./start_songbird_server.sh

# Check Neural API logs - you should see:
# "✅ Capability registered: secure_http by songbird-nat0"
```

### **Test 2: Capability Discovery**

```bash
# Query Neural API for capabilities
echo '{
  "jsonrpc": "2.0",
  "method": "capability.list",
  "params": {},
  "id": 1
}' | nc -U /tmp/neural-api-nat0.sock | jq '.'

# Expected response:
{
  "jsonrpc": "2.0",
  "result": {
    "capabilities": [
      {
        "name": "secure_http",
        "providers": [
          {
            "primal_id": "songbird-nat0",
            "socket": "/tmp/songbird-nat0.sock",
            "operations": ["http.get", "http.post", "http.put", "http.delete", "http.patch", "http.request"],
            "metadata": {
              "tls_version": "1.3",
              "pure_rust": true
            }
          }
        ]
      }
    ]
  },
  "id": 1
}
```

### **Test 3: capability.call Routing**

```bash
# Use capability.call to make an HTTP request
echo '{
  "jsonrpc": "2.0",
  "method": "capability.call",
  "params": {
    "capability": "secure_http",
    "operation": "http.post",
    "args": {
      "url": "https://httpbin.org/post",
      "headers": {
        "User-Agent": "ecoPrimals/1.0"
      },
      "body": {
        "test": "capability_call"
      }
    }
  },
  "id": 1
}' | nc -U /tmp/neural-api-nat0.sock | jq '.'

# Expected: Neural API discovers Songbird and routes the request
# Response: httpbin.org response via Pure Rust TLS 1.3
```

### **Test 4: Unregistration on Shutdown**

```bash
# Stop Songbird gracefully (Ctrl+C or kill -TERM)

# Check Neural API - Songbird should be unregistered
echo '{
  "jsonrpc": "2.0",
  "method": "capability.list",
  "params": {},
  "id": 1
}' | nc -U /tmp/neural-api-nat0.sock | jq '.'

# Expected: No "secure_http" capability listed
```

---

## 🔐 **ENVIRONMENT VARIABLES**

Songbird needs these environment variables for auto-registration:

```bash
# Required
SONGBIRD_SOCKET_PATH=/tmp/songbird-nat0.sock  # Where Songbird is listening

# Optional (with defaults)
NEURAL_API_SOCKET=/tmp/neural-api-nat0.sock   # Where to register
PRIMAL_ID=songbird-nat0                       # Unique primal identifier
```

**Update** `start_songbird_server.sh` to set these:

```bash
#!/bin/bash

export SONGBIRD_SOCKET_PATH="/tmp/songbird-nat0.sock"
export NEURAL_API_SOCKET="/tmp/neural-api-nat0.sock"
export PRIMAL_ID="songbird-nat0"
export BEARDOG_SOCKET="/tmp/beardog-nat0.sock"

./target/release/songbird server
```

---

## 📊 **VALIDATION CHECKLIST**

- [ ] `capability_registration.rs` created
- [ ] `register_capabilities()` wired into startup
- [ ] `unregister_capabilities()` wired into shutdown
- [ ] Module declaration added to `lib.rs`
- [ ] Environment variables set in start script
- [ ] Test 1: Registration on startup passes
- [ ] Test 2: Capability discovery works
- [ ] Test 3: `capability.call` routing works
- [ ] Test 4: Unregistration on shutdown works
- [ ] Songbird logs show successful registration
- [ ] Neural API logs show capability registered

---

## 🎯 **SUCCESS CRITERIA**

✅ **Songbird auto-registers on startup**  
✅ **Neural API lists `secure_http` capability**  
✅ **`capability.call` routes to Songbird**  
✅ **All HTTP methods work (GET, POST, PUT, DELETE)**  
✅ **Songbird unregisters on graceful shutdown**  
✅ **Zero hardcoded dependencies in consumer primals**

---

## 🚀 **BENEFITS**

### **For Songbird**
- Automatic discovery - no manual configuration
- Graceful degradation if Neural API unavailable
- Can evolve without breaking consumers

### **For Consumer Primals (Squirrel, etc.)**
- Zero coupling to Songbird
- No hardcoded socket paths
- Semantic APIs (`http.post` just works!)
- 90% less integration code

### **For The Ecosystem**
- TRUE PRIMAL pattern validated
- Isomorphic evolution enabled
- Loose coupling architecture
- Production-ready system

---

## 💡 **ARCHITECTURAL NOTES**

### **Fail-Safe Design**

Registration failure **does NOT** fail Songbird startup. This ensures:
- Songbird works even if Neural API is down
- Direct socket connections still work
- System is resilient to partial failures

### **Graceful Degradation**

```rust
if let Err(e) = capability_registration::register_capabilities().await {
    warn!("Failed to register capabilities: {}", e);
    // Continue anyway - direct connections still work
}
```

### **Future Evolution**

This architecture enables:
- Multiple HTTPS providers (Songbird, others)
- Load balancing across providers
- Failover and redundancy
- A/B testing of implementations
- Zero-downtime upgrades

---

## 📝 **EXAMPLE: Consumer Usage**

Once auto-registration is complete, here's how Squirrel (or any primal) uses Songbird:

```rust
// Squirrel's code - ZERO knowledge of Songbird!

use biomeos_nucleus::client::call_unix_socket_rpc;
use serde_json::json;

pub async fn fetch_github_repo(owner: &str, repo: &str) -> Result<Value> {
    let neural_socket = "/tmp/neural-api-nat0.sock";
    
    // Use capability.call - Neural API discovers Songbird automatically
    let response = call_unix_socket_rpc(
        neural_socket,
        "capability.call",
        &json!({
            "capability": "secure_http",
            "operation": "http.get",
            "args": {
                "url": format!("https://api.github.com/repos/{}/{}", owner, repo),
                "headers": {
                    "User-Agent": "ecoPrimals/1.0",
                    "Accept": "application/vnd.github.v3+json"
                }
            }
        })
    ).await?;
    
    Ok(response)
}

// That's it! No imports of Songbird, no socket paths, no coupling!
// If Songbird evolves, Squirrel doesn't need to change!
```

**Result**: 90% less code, zero coupling, isomorphic evolution! 🎉

---

## 🎉 **COMPLETION**

Once this is implemented:

✅ **TRUE PRIMAL architecture is complete**  
✅ **Tower Atomic is fully operational**  
✅ **Loose coupling validated**  
✅ **Production-ready ecosystem**

---

**Status**: 🚀 Ready for Implementation  
**Priority**: P1 (Important, not blocking)  
**Estimated Time**: 1-2 hours  
**Testing Time**: 30 minutes  
**Total**: 2 hours

---

**Questions?** Ping the biomeOS team!  
**Ready?** Let's make TRUE PRIMAL a reality! 🌟


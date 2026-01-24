# 🎯 Songbird IPC Evolution Required for biomeOS HTTPS

**Date**: January 25, 2026  
**Status**: 🔴 **ARCHITECTURAL GAP IDENTIFIED**  
**Priority**: High - Blocks biomeOS HTTPS via Neural API  
**Owner**: Songbird Team (Handoff from biomeOS)

---

## 📋 Executive Summary

### **Achievement**: ✅ 100% Pure Rust HTTPS (Library Level)
Songbird's `songbird-http-client` crate successfully achieved 100% Pure Rust HTTPS:
- TLS 1.3 handshake ✅
- BearDog integration ✅  
- HTTP 200 OK from real servers ✅

### **Gap**: ❌ Not IPC Protocol Compliant
The HTTPS functionality is a **library**, NOT a **service**:
- Current: `cargo run --example test_https` (library call)
- Required: `echo '{"method":"http.request"...}' | nc -U /primal/songbird` (IPC)

### **Impact**: ⚠️ biomeOS Cannot Deploy HTTPS via Neural API
Without IPC, biomeOS cannot orchestrate HTTPS requests through Songbird.

---

## 🔍 Standards Compliance Analysis

### **1. UniBin Standard**: ✅ COMPLIANT
```bash
songbird server    # ✅ Single binary with subcommands
songbird doctor    # ✅ Professional CLI
songbird --help    # ✅ Comprehensive help
```

### **2. ecoBin Standard**: ⏳ N/A (Intentional TLS role)
Songbird is the "Concentrated Gap" - the ONE primal allowed TLS/HTTP dependencies.

### **3. Primal IPC Protocol**: ❌ **NOT COMPLIANT**

**Required by Standard** (`PRIMAL_IPC_PROTOCOL.md`):
```
1. Listen on Unix socket: /primal/songbird (or /tmp/songbird-nat0.sock)
2. Accept JSON-RPC 2.0 requests
3. Expose capabilities via method calls (e.g., "http.request")
4. Register capabilities with discovery service
```

**Current Songbird**:
- `songbird server` → HTTP server on port 8080 (federation/discovery)
- No Unix socket JSON-RPC listener
- HTTPS functionality not exposed via IPC

---

## 🎯 What Songbird Needs to Evolve

### **Option A: Add Unix Socket Mode to `songbird server`** (Recommended)

```rust
// In songbird/src/bin/songbird/main.rs

#[derive(Subcommand)]
enum Commands {
    Server {
        #[arg(long, default_value = "8080")]
        port: u16,
        
        // NEW: Unix socket for IPC
        #[arg(long)]
        socket: Option<String>,  // e.g., /tmp/songbird-nat0.sock
        
        #[arg(long)]
        family_id: Option<String>,
    },
    // ...
}
```

When `--socket` is provided:
1. Start Unix socket listener alongside HTTP server
2. Accept JSON-RPC 2.0 requests
3. Handle `http.request` by calling `songbird-http-client` internally

### **Option B: New `songbird ipc-server` Subcommand**

```bash
songbird ipc-server --socket /tmp/songbird-nat0.sock
```

Dedicated subcommand for IPC-only mode.

---

## 📐 IPC Implementation Spec

### **1. Socket Path Convention**
```
Standard: /primal/songbird
biomeOS:  /tmp/songbird-{family_id}.sock  (e.g., /tmp/songbird-nat0.sock)
```

### **2. Required JSON-RPC Methods**

#### **`http.request`** - Core HTTPS Method
```json
// Request
{
    "jsonrpc": "2.0",
    "method": "http.request",
    "params": {
        "url": "https://cloudflare.com",
        "method": "GET",
        "headers": {"User-Agent": "Songbird/5.x"},
        "body": null,
        "timeout_ms": 30000
    },
    "id": 1
}

// Response (Success)
{
    "jsonrpc": "2.0",
    "result": {
        "status_code": 200,
        "headers": {"content-type": "text/html", ...},
        "body": "<!doctype html>...",
        "elapsed_ms": 150
    },
    "id": 1
}

// Response (Error)
{
    "jsonrpc": "2.0",
    "error": {
        "code": -32001,
        "message": "TLS handshake failed",
        "data": {"reason": "certificate validation failed"}
    },
    "id": 1
}
```

#### **`http.get`** - Convenience Method
```json
{
    "jsonrpc": "2.0",
    "method": "http.get",
    "params": {
        "url": "https://api.example.com/data"
    },
    "id": 2
}
```

#### **`http.post`** - Convenience Method
```json
{
    "jsonrpc": "2.0",
    "method": "http.post",
    "params": {
        "url": "https://api.example.com/submit",
        "body": "{\"key\": \"value\"}",
        "content_type": "application/json"
    },
    "id": 3
}
```

### **3. Capability Registration**

On startup, Songbird should register with discovery:
```json
{
    "jsonrpc": "2.0",
    "method": "ipc.register",
    "params": {
        "name": "songbird",
        "endpoint": "/tmp/songbird-nat0.sock",
        "capabilities": ["http", "https", "tls", "discovery"],
        "version": "5.21.0"
    },
    "id": 1
}
```

---

## 🔧 Implementation Guide

### **Step 1: Add IPC Handler Module**

```rust
// crates/songbird-ipc/src/handler.rs

use songbird_http_client::{HttpsClient, BearDogClient};
use tokio::net::UnixStream;
use serde_json::{json, Value};

pub async fn handle_rpc(
    request: Value,
    beardog_socket: &str,
) -> Value {
    let method = request["method"].as_str().unwrap_or("");
    let params = &request["params"];
    let id = &request["id"];
    
    match method {
        "http.request" | "http.get" => {
            let url = params["url"].as_str().unwrap_or("");
            
            // Use existing songbird-http-client
            let beardog = BearDogClient::connect(beardog_socket).await?;
            let client = HttpsClient::new(beardog);
            
            match client.get(url).await {
                Ok(response) => json!({
                    "jsonrpc": "2.0",
                    "result": {
                        "status_code": response.status_code,
                        "body": String::from_utf8_lossy(&response.body)
                    },
                    "id": id
                }),
                Err(e) => json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32001, "message": e.to_string()},
                    "id": id
                })
            }
        }
        _ => json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "Method not found"},
            "id": id
        })
    }
}
```

### **Step 2: Add Socket Listener**

```rust
// In songbird server startup

async fn start_ipc_server(socket_path: &str, beardog_socket: &str) {
    // Remove old socket if exists
    let _ = std::fs::remove_file(socket_path);
    
    let listener = tokio::net::UnixListener::bind(socket_path).await?;
    info!("✅ IPC listening on {}", socket_path);
    
    loop {
        let (stream, _) = listener.accept().await?;
        let beardog = beardog_socket.to_string();
        
        tokio::spawn(async move {
            handle_connection(stream, &beardog).await;
        });
    }
}
```

### **Step 3: Update CLI**

```rust
// In main.rs

Commands::Server { port, socket, family_id, .. } => {
    // Start HTTP server for federation (existing)
    let http_handle = tokio::spawn(start_http_server(port));
    
    // Start IPC server for HTTPS requests (NEW)
    if let Some(socket_path) = socket {
        let beardog_socket = std::env::var("BEARDOG_SOCKET")
            .unwrap_or_else(|_| format!("/tmp/beardog-{}.sock", family_id.unwrap_or("nat0")));
        
        let ipc_handle = tokio::spawn(start_ipc_server(&socket_path, &beardog_socket));
        
        tokio::select! {
            _ = http_handle => {},
            _ = ipc_handle => {},
        }
    } else {
        http_handle.await?;
    }
}
```

---

## 📊 Timeline Estimate

| Task | Effort | Owner |
|------|--------|-------|
| IPC handler module | 2-3 hours | Songbird Team |
| Socket listener | 1-2 hours | Songbird Team |
| CLI updates | 1 hour | Songbird Team |
| Testing | 2 hours | Songbird Team |
| Documentation | 1 hour | Songbird Team |
| **Total** | **7-9 hours** | Songbird Team |

---

## 🔗 biomeOS Integration (After Songbird Evolution)

Once Songbird exposes `http.request` via IPC:

### **1. Neural API Graph**
```toml
[[nodes]]
id = "test_https"
depends_on = ["germinate_songbird"]

[nodes.operation]
name = "http.request"
target_primal = "songbird"

[nodes.operation.params]
url = "https://cloudflare.com"
method = "GET"
```

### **2. Neural API Routing**
biomeOS will route `http.request` to Songbird's socket automatically.

### **3. Semantic Translation**
```json
// Capability translation already in tower_atomic_bootstrap.toml
"http.request" = "http.request"  // 1:1 mapping
```

---

## ✅ Success Criteria

### **Songbird Side**:
- [ ] `songbird server --socket /tmp/songbird-nat0.sock` starts IPC listener
- [ ] JSON-RPC `http.request` method works
- [ ] Returns proper response with status_code, headers, body
- [ ] Connects to BearDog via `BEARDOG_SOCKET` env var
- [ ] Graceful error handling

### **biomeOS Side**:
- [ ] Neural API can spawn Songbird with `--socket` flag
- [ ] Neural API can route `http.request` to Songbird
- [ ] HTTPS request returns HTTP 200 OK

---

## 🎯 Summary

**Current State**:
- ✅ UniBin compliant
- ✅ HTTPS library works (100% Pure Rust TLS 1.3!)
- ❌ Not IPC Protocol compliant
- ❌ biomeOS cannot deploy HTTPS via Neural API

**Required Evolution**:
- Add `--socket` option to `songbird server`
- Implement JSON-RPC handler for `http.request`
- Wire `songbird-http-client` to IPC handler

**Effort**: ~7-9 hours  
**Blocker**: biomeOS HTTPS via Neural API  
**Owner**: Songbird Team

---

**"The HTTPS success is in the library - now expose it via IPC!"** 📚→🌐  
**"UniBin + IPC Protocol = TRUE PRIMAL compliance!"** ✅🦀

---

*Document created: January 25, 2026*  
*Author: biomeOS Team*  
*For: Songbird Team Handoff*


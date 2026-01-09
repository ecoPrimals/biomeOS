# 🍄 Toadstool Unix Socket Evolution - Handoff Document

**Date**: January 9, 2026  
**From**: biomeOS Team  
**To**: Toadstool Team  
**Priority**: 🔥 **HIGH - BLOCKS NODE NICHE DEVELOPMENT**

---

## 🎯 TL;DR - What We Need

Toadstool currently runs on **HTTP REST API (port 8084)**. To enable the **Neural API Node niche**, Toadstool needs to evolve to **Unix socket JSON-RPC** following the BearDog/Songbird pattern.

**Current State**: ❌ HTTP on port 8084  
**Target State**: ✅ Unix socket JSON-RPC  
**Impact**: Unblocks Node niche from 30% → 100%

---

## 🚨 Why This Matters

### **Current Blocker**

The Neural API's **Node niche** (compute orchestration) is **blocked at 30%** because Toadstool doesn't follow the port-free architecture:

```bash
# Current Toadstool (WRONG - uses HTTP)
$ ./toadstool daemon
🌐 HTTP API server listening on 0.0.0.0:8084
📊 Endpoints:
   POST   /api/v1/workload/submit
   GET    /api/v1/workload/:id
   ...

# What we need (RIGHT - Unix socket)
$ ls /tmp/toadstool*.sock
srwxrwxr-x 1 user user 0 Jan 9 /tmp/toadstool-neural-test-node.sock
```

### **Architecture Vision**

**Port-Free Federation**: All primals communicate via Unix sockets + JSON-RPC
- BearDog: ✅ Complete
- Songbird: ✅ Complete  
- Toadstool: ❌ Still HTTP
- NestGate: ❓ Unknown

---

## 📊 Current vs. Target Architecture

### **Current (HTTP REST)**

```
┌─────────────┐
│  biomeOS    │
│  Neural API │
└──────┬──────┘
       │ HTTP POST to localhost:8084
       ↓
┌─────────────────┐
│   Toadstool     │
│   HTTP Server   │
│   Port: 8084    │
└─────────────────┘
```

**Problems**:
- Port conflicts in multi-node deployments
- Harder to secure (network exposure)
- Doesn't follow capability discovery pattern
- Breaks port-free architecture
- Can't use Neural API's graph orchestration

---

### **Target (Unix Socket JSON-RPC)**

```
┌─────────────┐
│  biomeOS    │
│  Neural API │
└──────┬──────┘
       │ Unix socket scan: /tmp/toadstool-*.sock
       ↓
┌─────────────────────────────┐
│   Toadstool                 │
│   Unix Socket JSON-RPC      │
│   /tmp/toadstool-{id}.sock  │
└─────────────────────────────┘
       ↓
   capabilities: ["compute", "workload-manager", ...]
```

**Benefits**:
- ✅ No port conflicts
- ✅ Better security (filesystem permissions)
- ✅ Automatic discovery by Neural API
- ✅ Follows port-free architecture
- ✅ Enables graph-based orchestration

---

## 🎨 Reference Implementation - BearDog Pattern

### **1. Unix Socket Creation**

```rust
// BearDog example: Create Unix socket on startup
use tokio::net::UnixListener;
use std::path::Path;

pub async fn start_unix_socket_server(node_id: &str) -> Result<()> {
    // Socket path pattern: /tmp/{primal}-{node_id}.sock
    let socket_path = format!("/tmp/toadstool-{}.sock", node_id);
    
    // Clean up old socket if exists
    if Path::new(&socket_path).exists() {
        std::fs::remove_file(&socket_path)?;
    }
    
    // Bind to Unix socket
    let listener = UnixListener::bind(&socket_path)?;
    info!("🍄 Toadstool Unix socket listening on {}", socket_path);
    
    // Accept connections
    loop {
        let (stream, _) = listener.accept().await?;
        tokio::spawn(handle_connection(stream));
    }
}
```

---

### **2. JSON-RPC Server**

```rust
// Handle incoming JSON-RPC requests
async fn handle_connection(mut stream: UnixStream) -> Result<()> {
    let mut buffer = vec![0u8; 4096];
    
    loop {
        match stream.read(&mut buffer).await {
            Ok(0) => break, // Connection closed
            Ok(n) => {
                let request: serde_json::Value = serde_json::from_slice(&buffer[..n])?;
                let response = handle_jsonrpc_request(request).await?;
                let response_bytes = serde_json::to_vec(&response)?;
                stream.write_all(&response_bytes).await?;
            }
            Err(e) => {
                warn!("Connection error: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}
```

---

### **3. Capability Advertisement**

```rust
// Respond to capability queries
async fn handle_jsonrpc_request(request: Value) -> Result<Value> {
    let method = request["method"].as_str().ok_or("Missing method")?;
    
    match method {
        "get_capabilities" => {
            Ok(json!({
                "jsonrpc": "2.0",
                "result": {
                    "primal_id": "toadstool",
                    "capabilities": [
                        "compute",
                        "workload-manager",
                        "universal-runtime",
                        "biome-execution"
                    ],
                    "version": "1.0.0"
                },
                "id": request["id"]
            }))
        }
        "submit_workload" => {
            // Handle workload submission
            handle_submit_workload(request["params"].clone()).await
        }
        "get_workload" => {
            // Handle workload status query
            handle_get_workload(request["params"].clone()).await
        }
        "list_workloads" => {
            // Handle workload listing
            handle_list_workloads().await
        }
        _ => {
            Err(anyhow!("Unknown method: {}", method))
        }
    }
}
```

---

## 🔌 Required APIs

### **1. get_capabilities** (REQUIRED)

**Purpose**: Advertise what Toadstool can do

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_capabilities",
  "params": {},
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "primal_id": "toadstool-neural-test-node",
    "capabilities": [
      "compute",
      "workload-manager",
      "universal-runtime",
      "biome-execution"
    ],
    "version": "1.0.0",
    "status": "healthy"
  },
  "id": 1
}
```

---

### **2. submit_workload** (REQUIRED)

**Purpose**: Submit a workload for execution

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "submit_workload",
  "params": {
    "workload_id": "workload-123",
    "workload_type": "biome",
    "manifest": {
      "image": "alpine:latest",
      "command": ["echo", "hello"],
      "resources": {
        "cpu": "1",
        "memory": "512M"
      }
    }
  },
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "workload_id": "workload-123",
    "status": "submitted",
    "submitted_at": "2026-01-09T12:00:00Z"
  },
  "id": 2
}
```

---

### **3. get_workload_status** (REQUIRED)

**Purpose**: Query status of a workload

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_workload_status",
  "params": {
    "workload_id": "workload-123"
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "workload_id": "workload-123",
    "status": "running",
    "started_at": "2026-01-09T12:00:01Z",
    "resources": {
      "cpu_usage": "0.5",
      "memory_usage": "256M"
    }
  },
  "id": 3
}
```

---

### **4. list_workloads** (OPTIONAL - but recommended)

**Purpose**: List all workloads

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "list_workloads",
  "params": {},
  "id": 4
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "workloads": [
      {
        "workload_id": "workload-123",
        "status": "running",
        "started_at": "2026-01-09T12:00:01Z"
      },
      {
        "workload_id": "workload-456",
        "status": "completed",
        "completed_at": "2026-01-09T11:50:00Z"
      }
    ],
    "total": 2
  },
  "id": 4
}
```

---

### **5. stop_workload** (OPTIONAL - but recommended)

**Purpose**: Stop a running workload

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "stop_workload",
  "params": {
    "workload_id": "workload-123",
    "graceful": true,
    "timeout_seconds": 30
  },
  "id": 5
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "workload_id": "workload-123",
    "status": "stopped",
    "stopped_at": "2026-01-09T12:05:00Z"
  },
  "id": 5
}
```

---

## 🔍 How biomeOS Will Discover Toadstool

### **1. Socket Scanning**

```rust
// biomeOS will scan for Toadstool sockets
use glob::glob;

async fn discover_toadstool() -> Result<Vec<String>> {
    let mut discovered = Vec::new();
    
    // Scan for Toadstool sockets
    for entry in glob("/tmp/toadstool-*.sock")? {
        let socket_path = entry?;
        
        // Verify it's a socket
        if socket_path.metadata()?.file_type().is_socket() {
            discovered.push(socket_path.display().to_string());
        }
    }
    
    Ok(discovered)
}
```

---

### **2. Capability Query**

```rust
// biomeOS will query capabilities
async fn query_capabilities(socket_path: &str) -> Result<Vec<String>> {
    let stream = UnixStream::connect(socket_path).await?;
    
    let request = json!({
        "jsonrpc": "2.0",
        "method": "get_capabilities",
        "params": {},
        "id": 1
    });
    
    // Send request, read response
    let response: Value = send_jsonrpc_request(stream, request).await?;
    
    let capabilities = response["result"]["capabilities"]
        .as_array()
        .ok_or("Missing capabilities")?
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    
    Ok(capabilities)
}
```

---

### **3. Registration**

```rust
// biomeOS registers Toadstool in its primal registry
let primal_id = infer_id_from_socket("/tmp/toadstool-neural-test-node.sock");
// Result: "toadstool-neural-test-node"

let capabilities = query_capabilities(socket_path).await?;
// Result: ["compute", "workload-manager", "universal-runtime", "biome-execution"]

registry.register(primal_id, capabilities, socket_path).await;
```

---

## 📝 Implementation Checklist

### **Phase 1: Unix Socket Server** ✅ REQUIRED

- [ ] Create Unix socket on startup
  - Pattern: `/tmp/toadstool-{node_id}.sock`
  - Clean up old sockets
  - Handle permission errors gracefully

- [ ] Implement JSON-RPC server
  - Parse JSON-RPC 2.0 requests
  - Handle connection lifecycle
  - Support concurrent connections

- [ ] Add `get_capabilities` method
  - Return primal ID
  - List all capabilities
  - Include version info

---

### **Phase 2: Core Workload APIs** ✅ REQUIRED

- [ ] Migrate `submit_workload` from HTTP to JSON-RPC
  - Same logic, new protocol
  - Return proper JSON-RPC responses

- [ ] Migrate `get_workload_status` from HTTP to JSON-RPC
  - Query by workload ID
  - Return current status

- [ ] Migrate `list_workloads` from HTTP to JSON-RPC
  - List all workloads
  - Support filtering (optional)

---

### **Phase 3: Advanced Features** ⭐ RECOMMENDED

- [ ] Add `stop_workload` method
  - Graceful shutdown support
  - Timeout handling

- [ ] Add `delete_workload` method
  - Clean up completed workloads
  - Return confirmation

- [ ] Add health check support
  - Report system resources
  - Active workload count
  - Status (healthy/degraded/unhealthy)

---

### **Phase 4: Deprecation** ⏳ FUTURE

- [ ] Mark HTTP API as deprecated
  - Add deprecation warnings
  - Set sunset date

- [ ] Support dual mode (HTTP + Unix socket)
  - Allow gradual migration
  - Document migration path

- [ ] Remove HTTP API
  - Only after all clients migrated
  - Major version bump

---

## 🧪 Testing Strategy

### **Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_unix_socket_creation() {
        let socket_path = "/tmp/toadstool-test.sock";
        let result = create_unix_socket(socket_path).await;
        assert!(result.is_ok());
        assert!(Path::new(socket_path).exists());
    }
    
    #[tokio::test]
    async fn test_get_capabilities() {
        let socket_path = start_test_server().await.unwrap();
        let capabilities = query_capabilities(&socket_path).await.unwrap();
        assert!(capabilities.contains(&"compute".to_string()));
    }
    
    #[tokio::test]
    async fn test_submit_workload_via_jsonrpc() {
        let socket_path = start_test_server().await.unwrap();
        let result = submit_workload_jsonrpc(&socket_path, test_workload()).await;
        assert!(result.is_ok());
    }
}
```

---

### **Integration Tests**

```bash
# Test 1: Socket creation
$ ./toadstool daemon --node-id test-node
$ ls /tmp/toadstool-test-node.sock
✅ Socket exists

# Test 2: Capability query
$ echo '{"jsonrpc":"2.0","method":"get_capabilities","params":{},"id":1}' | \
  socat - UNIX-CONNECT:/tmp/toadstool-test-node.sock
{"jsonrpc":"2.0","result":{"capabilities":["compute","workload-manager"]},"id":1}
✅ Capabilities returned

# Test 3: Workload submission
$ echo '{"jsonrpc":"2.0","method":"submit_workload","params":{"workload_id":"test"},"id":2}' | \
  socat - UNIX-CONNECT:/tmp/toadstool-test-node.sock
{"jsonrpc":"2.0","result":{"workload_id":"test","status":"submitted"},"id":2}
✅ Workload submitted
```

---

### **E2E Tests with Neural API**

```bash
# Test 4: Neural API discovery
$ cargo run --bin biomeos -- deploy --graph --manifest niches/compute-node.toml --validate-only
🔍 Discovered primals:
  • toadstool-test-node → ["compute", "workload-manager"]
✅ Discovery works

# Test 5: Graph-based deployment
$ cargo run --bin biomeos -- deploy --graph --manifest niches/compute-node.toml
📊 Executing graph 'node_deploy'...
  ✅ node: start_toadstool (2.1s)
  ✅ node: register_capabilities (0.3s)
  ✅ node: health_check (0.5s)
🎉 Deployment complete!
✅ Full deployment works
```

---

## 🎯 Success Criteria

### **Minimum Viable Implementation** (Phase 1-2)

- ✅ Unix socket created on startup
- ✅ JSON-RPC server operational
- ✅ `get_capabilities` working
- ✅ `submit_workload` migrated
- ✅ `get_workload_status` migrated
- ✅ biomeOS can discover and register Toadstool
- ✅ Neural API Node niche graphs execute successfully

---

### **Production Ready** (Phase 1-3)

- All Minimum Viable criteria
- ✅ `list_workloads` implemented
- ✅ `stop_workload` implemented
- ✅ Comprehensive test coverage (>80%)
- ✅ Error handling and retry logic
- ✅ Performance benchmarks pass
- ✅ Documentation complete

---

## 📦 Dependencies

### **Required Crates**

```toml
[dependencies]
tokio = { version = "1.35", features = ["full", "net"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
```

---

## 🚀 Timeline Estimate

| Phase | Effort | Duration |
|-------|--------|----------|
| Phase 1 (Unix Socket) | Medium | 1-2 days |
| Phase 2 (Core APIs) | Low | 1 day |
| Phase 3 (Advanced) | Medium | 2-3 days |
| Testing & Polish | Medium | 1-2 days |
| **Total** | | **5-8 days** |

---

## 📚 Reference Documentation

### **Existing Implementations**

1. **BearDog Unix Socket**
   - Path: `ecoPrimals/phase1/beardog/`
   - File: `src/server/unix_socket.rs` (likely)
   - Status: ✅ Complete, production-ready

2. **Songbird Unix Socket**
   - Path: `ecoPrimals/phase1/songbird/`
   - File: `src/server/unix_socket.rs` (likely)
   - Status: ✅ Complete, production-ready

3. **biomeOS Discovery**
   - Path: `biomeOS/crates/biomeos-core/src/graph_deployment.rs`
   - Lines: 47-130 (PrimalRegistry)
   - Status: ✅ Complete, awaiting Toadstool

---

### **Specifications**

1. **JSON-RPC 2.0 Spec**
   - URL: https://www.jsonrpc.org/specification
   - Required reading

2. **Unix Socket in Rust**
   - Tokio docs: https://docs.rs/tokio/latest/tokio/net/struct.UnixListener.html

3. **Neural API Graph Orchestration**
   - Path: `biomeOS/specs/NEURAL_API_IMPLEMENTATION_PHASES.md`

---

## 🤝 Support & Questions

### **Contact**

- **Team**: biomeOS Neural API Team
- **Status**: Awaiting Toadstool Unix socket implementation
- **Priority**: 🔥 HIGH - Blocks Node niche

### **What We'll Provide**

- ✅ Integration testing support
- ✅ Graph definitions for Node niche
- ✅ E2E testing once APIs ready
- ✅ Documentation updates

### **What We Need from You**

- Unix socket JSON-RPC server
- `get_capabilities` method
- Core workload APIs migrated
- Confirmation when ready for testing

---

## 🎊 Impact

**Once complete, this will:**
- ✅ Unblock Node niche (30% → 100%)
- ✅ Enable graph-based compute orchestration
- ✅ Support multi-node deployments
- ✅ Complete port-free architecture
- ✅ Enable Neural API Milestone 2

**This is a critical path item for biomeOS Neural API!**

---

**Status**: ⏳ **AWAITING TOADSTOOL TEAM**  
**Priority**: 🔥 **HIGH**  
**Timeline**: 5-8 days estimated

🍄 **Let's make Toadstool truly universal!** 🚀


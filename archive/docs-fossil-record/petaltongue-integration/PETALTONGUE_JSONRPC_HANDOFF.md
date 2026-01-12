# 🌸 petalTongue Integration Handoff - JSON-RPC Protocol Evolution

**From**: biomeOS Team  
**To**: petalTongue Team  
**Date**: January 11, 2026  
**Priority**: High (Blocking full UI integration)  

---

## 🎯 **Issue Summary**

petalTongue GUI is currently **unable to connect** to biomeOS because of a **protocol mismatch**:

- ✅ **biomeOS**: JSON-RPC 2.0 over Unix socket (line-delimited)
- ❌ **petalTongue**: HTTP/REST over Unix socket (reqwest client)

**Error observed:**
```
ERROR petal_tongue_discovery: ❌ Failed to connect to biomeOS at unix:///run/user/1000/biomeos-device-management.sock: 
  Health check failed: builder error for url (unix:///run/user/1000/biomeos-device-management.sock/api/v1/health): 
  URL scheme is not allowed
```

---

## 🌍 **ecoPrimals Philosophy: JSON-RPC & tarpc First**

**Core Principle:**
> ecoPrimals is a **JSON-RPC 2.0 and tarpc-first ecosystem**. HTTP/REST should be an **optional, enableable fallback** for external integrations, not the primary protocol.

### Why JSON-RPC & tarpc?

1. **Port-Free Architecture**: Unix sockets eliminate port conflicts
2. **100x Faster**: Local IPC without TCP/IP stack overhead  
3. **Language Agnostic**: JSON-RPC 2.0 works with any language
4. **Secure by Default**: File permissions control access
5. **True Primal Sovereignty**: No dependency on HTTP servers
6. **Bi-directional**: Supports notifications and streaming

### Current Ecosystem Status

| Primal | JSON-RPC | tarpc | HTTP | Status |
|--------|----------|-------|------|--------|
| **Songbird** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **BearDog** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **ToadStool** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **NestGate** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **Squirrel** | ✅ Primary | ✅ Yes | ❌ No | Production |
| **biomeOS** | ✅ Primary | ⏳ Planned | ⚠️ Minimal | Production |
| **petalTongue** | ⚠️ **Missing** | ⏳ Planned | ✅ Primary | ⚠️ **Needs Evolution** |

---

## 🔍 **Technical Investigation**

### What biomeOS Provides (Correct!)

**File**: `crates/biomeos-ui/src/bin/device_management_server.rs`

**Protocol**: JSON-RPC 2.0 over Unix socket (line-delimited)

```rust
// biomeOS device_management_server
async fn handle_connection(stream: UnixStream, ...) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        // Read line-delimited JSON-RPC 2.0
        reader.read_line(&mut line).await?;
        
        // Parse JSON-RPC request
        let request: JsonRpcRequest = serde_json::from_str(&line)?;
        
        // Handle method
        let response = handle_method(request, &bridge).await;
        
        // Send response
        let response_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(response_json.as_bytes()).await?;
    }
}
```

**Supported Methods:**
1. `get_devices` → List system devices (CPU, GPU, storage, network)
2. `get_primals_extended` → List discovered primals with health
3. `get_niche_templates` → List available niche templates
4. `assign_device` → Assign device to primal
5. `validate_niche` → Validate niche configuration
6. `deploy_niche` → Deploy a complete niche via Neural API

**Socket Path**: `/run/user/{uid}/biomeos-device-management.sock`

### What petalTongue Is Doing (Incorrect)

**File**: `crates/petal-tongue-discovery/src/http_provider.rs`

**Protocol**: HTTP/REST over Unix socket (reqwest client)

```rust
// petalTongue http_provider
async fn health_check(&self) -> anyhow::Result<String> {
    let url = format!("{}/api/v1/health", self.endpoint);
    // ❌ Tries to send HTTP GET request
    match self.client.get(&url).send().await {
        // ...
    }
}
```

**Problem**: reqwest expects HTTP semantics (request line, headers, body), but biomeOS socket expects line-delimited JSON-RPC.

---

## 💡 **Solution: Create JsonRpcProvider**

### Required Implementation

**New File**: `crates/petal-tongue-discovery/src/jsonrpc_provider.rs`

```rust
//! JSON-RPC 2.0 visualization data provider
//!
//! PRIMARY protocol for ecoPrimals ecosystem. Connects to JSON-RPC 2.0
//! servers over Unix sockets for fast, secure, port-free communication.

use crate::traits::{ProviderMetadata, VisualizationDataProvider};
use async_trait::async_trait;
use petal_tongue_core::{PrimalHealthStatus, PrimalInfo, TopologyEdge};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

/// JSON-RPC 2.0 provider for primals that expose Unix socket JSON-RPC servers
pub struct JsonRpcProvider {
    socket_path: String,
}

#[derive(Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<serde_json::Value>,
    id: u64,
}

#[derive(Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
    id: u64,
}

#[derive(Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<serde_json::Value>,
}

impl JsonRpcProvider {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    async fn call(&self, method: &str, params: Option<serde_json::Value>) -> anyhow::Result<serde_json::Value> {
        // Connect to Unix socket
        let stream = UnixStream::connect(&self.socket_path).await?;
        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send JSON-RPC request
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        };
        let request_json = serde_json::to_string(&request)? + "\n";
        writer.write_all(request_json.as_bytes()).await?;
        writer.flush().await?;

        // Read JSON-RPC response
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        let response: JsonRpcResponse = serde_json::from_str(&line)?;

        // Check for errors
        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("JSON-RPC error {}: {}", error.code, error.message));
        }

        Ok(response.result.unwrap_or(serde_json::Value::Null))
    }
}

#[async_trait]
impl VisualizationDataProvider for JsonRpcProvider {
    async fn get_primals(&self) -> anyhow::Result<Vec<PrimalInfo>> {
        let result = self.call("get_primals_extended", None).await?;
        let primals: Vec<PrimalInfo> = serde_json::from_value(result)?;
        Ok(primals)
    }

    async fn get_topology(&self) -> anyhow::Result<Vec<TopologyEdge>> {
        // Optional: if biomeOS implements get_topology
        Ok(Vec::new())
    }

    async fn health_check(&self) -> anyhow::Result<String> {
        // Try calling get_primals as a health check
        self.call("get_primals_extended", None).await?;
        Ok(format!("JSON-RPC provider at {} is healthy", self.socket_path))
    }

    fn metadata(&self) -> ProviderMetadata {
        ProviderMetadata {
            id: "jsonrpc".to_string(),
            name: "JSON-RPC Provider".to_string(),
            version: "1.0.0".to_string(),
            description: "JSON-RPC 2.0 over Unix sockets (PRIMARY protocol)".to_string(),
            capabilities: vec!["primals".to_string(), "devices".to_string(), "niches".to_string()],
        }
    }
}
```

### Discovery Integration

**Update**: `crates/petal-tongue-discovery/src/lib.rs`

```rust
// Add to discovery chain (BEFORE HttpProvider)
pub async fn discover_all_providers() -> Vec<Box<dyn VisualizationDataProvider>> {
    let mut providers = Vec::new();

    // 1. PRIORITY: Songbird discovery (JSON-RPC + NUCLEUS)
    if let Ok(provider) = SongbirdProvider::discover().await {
        providers.push(Box::new(provider) as Box<dyn VisualizationDataProvider>);
    }

    // 2. Environment hints (JSON-RPC first!)
    if let Ok(url) = std::env::var("BIOMEOS_URL") {
        if url.starts_with("unix://") {
            let socket_path = url.strip_prefix("unix://").unwrap();
            providers.push(Box::new(JsonRpcProvider::new(socket_path)));
        } else if url.starts_with("http://") || url.starts_with("https://") {
            // HTTP as fallback only
            providers.push(Box::new(HttpProvider::new(url)));
        }
    }

    // 3. Standard Unix socket locations (auto-discovery)
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let standard_paths = vec![
        format!("/run/user/{}/biomeos-device-management.sock", uid),
        format!("/run/user/{}/biomeos-ui.sock", uid),
        "/tmp/biomeos.sock".to_string(),
    ];

    for path in standard_paths {
        if tokio::fs::metadata(&path).await.is_ok() {
            providers.push(Box::new(JsonRpcProvider::new(path)));
            break; // Use first available
        }
    }

    // 4. mDNS discovery (optional)
    // 5. HTTP fallback (if explicitly configured)

    providers
}
```

---

## 📊 **Implementation Checklist**

### For petalTongue Team

- [ ] **Create `JsonRpcProvider`** (PRIMARY protocol)
  - [ ] Unix socket connection
  - [ ] Line-delimited JSON-RPC 2.0
  - [ ] Methods: `get_primals_extended`, `get_devices`, `get_niche_templates`
  - [ ] Error handling

- [ ] **Update Discovery Chain**
  - [ ] JSON-RPC before HTTP
  - [ ] Auto-detect Unix sockets
  - [ ] Environment variable: `BIOMEOS_URL=unix:///path/to/socket`

- [ ] **Deprecate HTTP as Primary**
  - [ ] Mark `HttpProvider` as "fallback only"
  - [ ] Add warning if HTTP is used
  - [ ] Document HTTP as "external integration only"

- [ ] **Add tarpc Support** (future)
  - [ ] For streaming updates
  - [ ] For bidirectional communication
  - [ ] For performance-critical paths

- [ ] **Testing**
  - [ ] Unit tests for `JsonRpcProvider`
  - [ ] Integration tests with biomeOS
  - [ ] E2E tests with full UI

### For biomeOS Team (Already Complete! ✅)

- [x] JSON-RPC 2.0 server over Unix socket
- [x] Line-delimited protocol
- [x] Methods: `get_devices`, `get_primals_extended`, `get_niche_templates`, `assign_device`, `validate_niche`, `deploy_niche`
- [x] Socket path: `/run/user/{uid}/biomeos-device-management.sock`
- [x] Graceful error handling
- [x] Concurrent connection support

---

## 🎯 **Expected Outcome**

Once `JsonRpcProvider` is implemented:

```bash
# Launch biomeOS device_management_server
$ cargo run --bin device_management_server
🌸 Starting biomeOS Device Management Server
📡 Binding to Unix socket: /run/user/1000/biomeos-device-management.sock
✅ biomeOS Device Management Server ready

# Launch petalTongue
$ BIOMEOS_URL=unix:///run/user/1000/biomeos-device-management.sock petaltongue
🌸 Starting petalTongue
🔍 Discovering visualization providers...
✅ Found JSON-RPC provider at unix:///run/user/1000/biomeos-device-management.sock
📊 Fetching primals...
✅ Discovered 6 primals
🎨 Rendering UI...
✅ Full UI with live data!
```

---

## 🚀 **Timeline**

**Estimated Effort**: 4-6 hours

1. **Hour 1-2**: Create `JsonRpcProvider` struct and basic RPC client
2. **Hour 2-3**: Implement `VisualizationDataProvider` trait
3. **Hour 3-4**: Update discovery chain and auto-detection
4. **Hour 4-5**: Integration testing with biomeOS
5. **Hour 5-6**: E2E testing and polish

---

## 📚 **References**

- **JSON-RPC 2.0 Spec**: https://www.jsonrpc.org/specification
- **biomeOS RPC Server**: `crates/biomeos-ui/src/bin/device_management_server.rs`
- **biomeOS RPC Bridge**: `crates/biomeos-ui/src/petaltongue_bridge.rs`
- **ToadStool Example**: `ecoPrimals/phase1/toadstool/` (production JSON-RPC + tarpc)
- **Songbird Example**: `ecoPrimals/phase1/songbird/` (production JSON-RPC + tarpc)

---

## 💬 **Questions?**

**Contact**: biomeOS coordination channel  
**Status**: Ready for handoff ✅  
**Blocking**: Full UI integration

---

**Different orders of the same architecture.** 🍄🐸🌸

Let's evolve petalTongue to be a TRUE primal with JSON-RPC + tarpc first!



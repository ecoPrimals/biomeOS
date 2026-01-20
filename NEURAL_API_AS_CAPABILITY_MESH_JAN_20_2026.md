# Neural API as Capability Mesh - Architecture Correction
## January 20, 2026 15:10 UTC

---

## 🎯 Key Insight

**User Feedback**: "Both need to evolve to be more robust on their own. BUT outside of squirrel delegating the work for the http, neuralAPI should be the infra we use to navigate slight differences in primal behavior"

---

## ❌ What We Were Doing Wrong

### Problem: Capability Discovery at Primal Layer
We were making **Squirrel** responsible for:
1. Scanning all sockets in `/tmp`, `/var/run`, etc.
2. Probing each socket with different methods (`health`, `ping`, `http.post`)
3. Handling different response formats from different primals
4. Building its own capability map

**Result**: 
- 30+ sockets scanned (vscode, docker, LibreOffice...)
- 15+ seconds of probing
- Brittle (breaks when primals change methods)
- Duplicated logic (every primal needs discovery code)
- Hangs when encountering unexpected responses

---

## ✅ Correct Architecture: Neural API as Capability Mesh

### Neural API's Role

**Neural API is the mesh ON TOP of primals**. It should:

1. **Know the topology**: Which primals are deployed and where
2. **Track capabilities**: Who provides what
3. **Handle routing**: Translate requests between primals with different interfaces
4. **Manage discovery**: Primals ask Neural API, not each other

### How It Should Work

```
┌─────────────────────────────────────────────────────┐
│              Neural API (Capability Mesh)            │
│                                                      │
│  Knows:                                              │
│  - BearDog @ /tmp/beardog-nat0.sock                  │
│    Capabilities: [crypto.sign, security.verify]      │
│                                                      │
│  - Songbird @ /tmp/songbird-nat0.sock                │
│    Capabilities: [http.post, http.get, security.*]   │
│                                                      │
│  - Squirrel @ /tmp/squirrel-nat0.sock                │
│    Capabilities: [ai.routing, tool.orchestration]    │
│                                                      │
└──────────┬───────────────────────────────────────────┘
           │
           │ (Discovery queries go UP to mesh)
           │
     ┌─────┴─────┐
     ▼           ▼           ▼
 ┌─────┐    ┌─────────┐   ┌─────────┐
 │Bear │    │Songbird │   │Squirrel │
 │Dog  │    │         │   │         │
 └─────┘    └─────────┘   └─────────┘
  Simple     Simple        Simple
  (focused)  (focused)     (focused)
```

---

## 🔧 Implementation Changes

### 1. Squirrel: Query Neural API for Capabilities

**Instead of socket scanning**:
```rust
// ❌ OLD: Scan every socket, try every method
async fn discover_http_provider() -> Result<String> {
    for dir in ["/tmp", "/var/run", "/run/user/1000"] {
        for socket in scan_dir(dir) {
            if try_method(socket, "health").await.is_ok() { return Ok(socket); }
            if try_method(socket, "ping").await.is_ok() { return Ok(socket); }
            if try_method(socket, "http.post").await.is_ok() { return Ok(socket); }
        }
    }
    Err("Not found")
}
```

**✅ NEW: Ask Neural API**:
```rust
async fn discover_http_provider() -> Result<String> {
    // Connect to Neural API (known location from environment)
    let neural_api = env::var("NEURAL_API_SOCKET")
        .unwrap_or("/tmp/neural-api-nat0.sock".to_string());
    
    // Ask for capability
    let request = json!({
        "jsonrpc": "2.0",
        "method": "capability.discover",
        "params": {
            "capability": "http.request",
            "requester": "squirrel"
        },
        "id": 1
    });
    
    let response = send_rpc(&neural_api, request).await?;
    
    // Neural API responds with provider info
    // {
    //   "result": {
    //     "provider": "songbird",
    //     "socket": "/tmp/songbird-nat0.sock",
    //     "methods": ["http.post", "http.get"],
    //     "version": "2.0.0"
    //   }
    // }
    
    Ok(response["result"]["socket"].as_str().unwrap().to_string())
}
```

### 2. Neural API: Capability Registry + Discovery

**New RPC Methods for Neural API**:

```rust
// Neural API RPC interface
pub enum NeuralApiMethod {
    // Existing
    "graph.deploy",
    "graph.status",
    "primal.health",
    
    // NEW: Capability discovery
    "capability.discover",      // Find who provides a capability
    "capability.register",      // Primal announces its capabilities
    "capability.list",          // List all available capabilities
    "primal.route",            // Route request between primals (with translation)
}
```

**Implementation**:
```rust
// In neural_api_server.rs or new capability_registry.rs

struct CapabilityRegistry {
    // Map: capability name -> provider info
    capabilities: HashMap<String, Vec<ProviderInfo>>,
}

struct ProviderInfo {
    primal_name: String,
    socket_path: String,
    methods: Vec<String>,
    version: String,
    family_id: String,
}

impl CapabilityRegistry {
    async fn discover(&self, capability: &str) -> Result<ProviderInfo> {
        // Look up who provides this capability
        self.capabilities
            .get(capability)
            .and_then(|providers| providers.first())
            .cloned()
            .ok_or_else(|| PrimalError::NotFound(
                format!("No provider for capability: {}", capability)
            ))
    }
    
    async fn register(&mut self, primal: String, capabilities: Vec<String>, socket: String) {
        for cap in capabilities {
            self.capabilities
                .entry(cap)
                .or_insert_with(Vec::new)
                .push(ProviderInfo {
                    primal_name: primal.clone(),
                    socket_path: socket.clone(),
                    methods: vec![], // TODO: get from primal
                    version: "unknown".to_string(),
                    family_id: "unknown".to_string(),
                });
        }
    }
}
```

### 3. Deployment: Neural API Learns Topology

**When deploying via graph** (e.g., `tower_atomic.toml`):

```rust
// In neural_executor.rs - after starting each primal

async fn start_primal(&mut self, node: &DeploymentNode) -> Result<()> {
    // Start the primal
    let socket = self.start_process(node).await?;
    
    // Register its capabilities with Neural API
    self.capability_registry.register(
        node.primal.clone(),
        node.capabilities.clone(), // From TOML
        socket
    ).await?;
    
    // Set environment variable for the primal
    // So it knows how to find Neural API
    self.env.insert(
        "NEURAL_API_SOCKET",
        "/tmp/neural-api-nat0.sock"
    );
    
    Ok(())
}
```

**Updated TOML format**:
```toml
# graphs/tower_atomic.toml

[nodes.beardog]
primal = "beardog"
socket = "/tmp/beardog-nat0.sock"
capabilities = ["crypto.sign", "crypto.verify", "security.jwt"]  # NEW!

[nodes.songbird]
primal = "songbird"
socket = "/tmp/songbird-nat0.sock"
capabilities = ["http.post", "http.get", "http.request", "security.*"]  # NEW!
depends_on = ["beardog"]

[nodes.squirrel]
primal = "squirrel"
socket = "/tmp/squirrel-nat0.sock"
capabilities = ["ai.routing", "ai.text_generation", "tool.orchestration"]  # NEW!
depends_on = ["songbird"]  # For HTTP delegation
```

### 4. Primal Startup: Announce to Neural API

**Every primal on startup**:
```rust
// In primal main.rs startup

async fn run_server(...) -> Result<()> {
    // ... setup ...
    
    // Announce capabilities to Neural API
    if let Ok(neural_api) = env::var("NEURAL_API_SOCKET") {
        announce_to_neural_api(&neural_api).await?;
    }
    
    // Start JSON-RPC server
    server.start().await?;
    
    Ok(())
}

async fn announce_to_neural_api(neural_api_socket: &str) -> Result<()> {
    let request = json!({
        "jsonrpc": "2.0",
        "method": "capability.register",
        "params": {
            "primal": env!("CARGO_PKG_NAME"),
            "version": env!("CARGO_PKG_VERSION"),
            "socket": env::var("SQUIRREL_SOCKET")?,
            "capabilities": PRIMAL_CAPABILITIES  // Constant in each primal
        },
        "id": 1
    });
    
    send_rpc(neural_api_socket, request).await?;
    info!("✅ Registered capabilities with Neural API");
    Ok(())
}

// Define capabilities as constant (self-knowledge)
const PRIMAL_CAPABILITIES: &[&str] = &[
    "ai.routing",
    "ai.text_generation",
    "tool.orchestration",
];
```

---

## 🎯 Benefits of This Architecture

### 1. Primals Stay Simple ✅
- Each primal focuses on its core function
- No complex discovery logic
- No probing multiple methods
- Just asks Neural API: "who does X?"

### 2. Neural API Handles Complexity ✅
- Central capability registry
- Knows the full topology
- Can route/translate between primals
- Handles different primal versions/interfaces

### 3. Graceful Evolution ✅
- When Songbird adds new methods: Update capability list
- When new primal joins: Registers its capabilities
- When primal upgrades: Neural API knows about version differences
- No changes needed in other primals

### 4. Better Error Messages ✅
```rust
// Instead of: "Connection refused" (which socket?)
// Neural API can say:
"Capability 'http.request' requires Songbird, but Songbird is not deployed.
Deploy Tower Atomic first: biomeos deploy tower_atomic"
```

### 5. Multi-System Ready ✅
```rust
// Neural API knows about remote systems too
let providers = neural_api.discover("http.request").await?;
// Returns:
// [
//   { primal: "songbird", socket: "/tmp/songbird-nat0.sock", location: "local" },
//   { primal: "songbird", socket: "tcp://friend-node:8080", location: "remote" }
// ]
```

---

## 📋 Implementation Plan

### Phase 1: Neural API Capability Registry (2-3 hours)
1. Add `CapabilityRegistry` struct to Neural API
2. Implement `capability.discover` RPC method
3. Implement `capability.register` RPC method
4. Implement `capability.list` RPC method
5. Test with hardcoded capabilities

### Phase 2: Graph-Based Registration (1-2 hours)
1. Add `capabilities` field to TOML node definition
2. Update `neural_executor.rs` to register after starting primals
3. Set `NEURAL_API_SOCKET` environment variable for all primals
4. Update example graphs

### Phase 3: Primal Announcement (2-3 hours per primal)
1. Add `PRIMAL_CAPABILITIES` constant to each primal
2. Implement `announce_to_neural_api()` on startup
3. Replace socket scanning with Neural API queries
4. Test discovery flow

### Phase 4: Testing & Validation (1-2 hours)
1. Test capability discovery: Squirrel finds Songbird
2. Test multi-provider: Multiple Songbirds for load balancing
3. Test missing capability: Clear error messages
4. Test version differences: Neural API routes correctly

**Total Estimate**: 6-10 hours

---

## 🔄 Migration Path

### Immediate (Today)
**Squirrel Team**: Add timeout + graceful degradation (1-2 hours)
- Fixes immediate hang issue
- Uses existing `AI_PROVIDER_SOCKETS` environment variable
- Gets us to working state

**biomeOS Team**: Document Neural API capability registry design
- Write specs
- Update architecture docs
- Create implementation plan

### This Week
**Neural API Team**: Implement capability registry (6-10 hours)
- Core registry implementation
- Graph-based registration
- RPC methods

### Next Week
**All Primal Teams**: Migrate to Neural API discovery
- Replace socket scanning with Neural API queries
- Add capability announcement on startup
- Test integration

---

## 📊 Comparison

### Before (Socket Scanning)
```
Squirrel startup:
1. Scan /tmp (10 sockets, 500ms each) = 5s
2. Scan /var/run (15 sockets, 500ms each) = 7.5s
3. Scan /run/user/1000 (20 sockets, 500ms each) = 10s
4. Try health, then ping, then http.post on each
Total: 22.5+ seconds ❌

Brittleness:
- Breaks if Songbird changes method names
- Breaks if socket locations change
- Scans irrelevant sockets (docker, vscode)
- Every primal duplicates discovery logic
```

### After (Neural API Mesh)
```
Squirrel startup:
1. Connect to Neural API (known location) = 1ms
2. Query: "who provides http.request?" = 1ms
3. Get response: "songbird @ /tmp/songbird-nat0.sock"
Total: 2ms ✅

Robustness:
- Neural API knows topology (deployed it!)
- Method differences handled by Neural API
- Clear errors if capability missing
- Single source of truth
- Versioning support
```

---

## 🎯 Handoff Updates

### For Squirrel Team (Short-term fix)
Keep the current plan but **simplify**:
1. ✅ Add timeout (2s per attempt)
2. ✅ Try `http.post` for detection
3. ✅ Start server even if discovery fails
4. ❌ **Skip** complex multi-method probing
5. ❌ **Skip** directory scanning optimization
6. ✅ Use `AI_PROVIDER_SOCKETS` as primary method

**Rationale**: Get it working now, but know we'll replace with Neural API discovery soon.

### For Neural API Team (Long-term solution)
1. Design capability registry schema
2. Implement RPC methods
3. Integrate with graph deployment
4. Document migration path for all primals

---

## ✅ Architectural Principles Reinforced

1. **TRUE PRIMAL**: Each primal has self-knowledge only
   - Knows its own capabilities
   - Discovers others via Neural API (not direct probing)

2. **Neural API as Mesh**: Orchestration layer
   - Knows topology
   - Handles routing
   - Manages differences

3. **Separation of Concerns**:
   - Primals: Do one thing well
   - Neural API: Orchestrate many primals
   - Deployment: Configure topology

4. **Evolution-Friendly**:
   - Primals can change methods
   - Neural API handles translation
   - No brittle point-to-point dependencies

---

## 📁 Next Documents to Create

1. `NEURAL_API_CAPABILITY_REGISTRY_SPEC.md` - Detailed specification
2. `PRIMAL_CAPABILITY_STANDARD.md` - How primals declare capabilities
3. `CAPABILITY_DISCOVERY_PROTOCOL.md` - RPC interface spec

---

## 🎉 Summary

**User Insight**: "Neural API should be the infra we use to navigate slight differences in primal behavior"

**Impact**: 
- ✅ Simpler primals
- ✅ Centralized capability management
- ✅ Better error messages
- ✅ Evolution-friendly
- ✅ Multi-system ready

**Short-term**: Fix Squirrel timeout (1-2 hours)  
**Long-term**: Neural API capability mesh (6-10 hours)

---

*The mesh knows the way - primals just ask* 🕸️🧬✨



# 🧠 Neural API JSON-RPC Server Implementation Spec

**Version**: 2.0.0  
**Date**: January 14, 2026  
**Status**: 🟢 **READY FOR IMPLEMENTATION**  
**Priority**: HIGH (Enables PetalTongue View 6, Squirrel Integration, NUCLEUS Coordination)

---

## 🎯 Executive Summary

**neuralAPI is biomeOS's graph-based orchestration engine.**

**Current Status**:
- ✅ **Graph Engine**: Complete (`biomeos-atomic-deploy/src/neural_*.rs`)
- ✅ **Graph Parser**: Complete (TOML → execution)
- ✅ **Node Executors**: Complete (7 types)
- ⏳ **JSON-RPC Server**: NOT YET IMPLEMENTED

**This Spec**: How to build the JSON-RPC server for external access

---

## 🏗️ Architecture

### **Three Layers**

```
┌─────────────────────────────────────────────────────────┐
│ Layer 3: External Clients (PetalTongue, Squirrel, CLI) │
│   - Query graph status                                  │
│   - Trigger graph execution                             │
│   - Stream events (WebSocket)                           │
└────────────────────┬────────────────────────────────────┘
                     │ JSON-RPC over Unix Socket/HTTP
┌────────────────────┴────────────────────────────────────┐
│ Layer 2: neuralAPI JSON-RPC Server (TO BE BUILT)       │
│   - JSON-RPC 2.0 request handling                       │
│   - Graph execution coordination                        │
│   - Event streaming (Server-Sent Events)                │
│   - Error handling & validation                         │
└────────────────────┬────────────────────────────────────┘
                     │ Direct function calls
┌────────────────────┴────────────────────────────────────┐
│ Layer 1: Graph Engine (ALREADY BUILT)                   │
│   - GraphExecutor                                        │
│   - TopologicalSorter                                    │
│   - Node executors (7 types)                            │
│   - Dependency resolution                                │
└─────────────────────────────────────────────────────────┘
```

**This spec focuses on Layer 2 (the server).**

---

## 📡 JSON-RPC API Specification

### **1. Graph Management**

#### **Method**: `neural_api.list_graphs`

**Description**: List all available graph files

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.list_graphs",
  "params": {},
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "graphs": [
      {
        "id": "nucleus_deployment",
        "name": "NUCLEUS Full Deployment",
        "path": "graphs/genetic_lineage_full_nucleus.toml",
        "node_count": 12,
        "description": "Deploy complete NUCLEUS with genetic lineage",
        "last_executed": "2026-01-14T08:30:00Z",
        "status": "idle"
      },
      {
        "id": "tower_atomic",
        "name": "Tower Atomic Deployment",
        "path": "graphs/tower_deploy.toml",
        "node_count": 5,
        "description": "Deploy Tower (BearDog + Songbird)",
        "status": "running",
        "execution_id": "exec-abc123"
      }
    ],
    "count": 2
  },
  "id": 1
}
```

---

#### **Method**: `neural_api.get_graph`

**Description**: Get detailed information about a specific graph

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.get_graph",
  "params": {
    "graph_id": "nucleus_deployment"
  },
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "id": "nucleus_deployment",
    "name": "NUCLEUS Full Deployment",
    "nodes": [
      {
        "id": "create_usb_seed",
        "type": "filesystem",
        "operation": "create_directory",
        "dependencies": [],
        "status": "pending"
      },
      {
        "id": "deploy_beardog",
        "type": "primal",
        "operation": "start",
        "dependencies": ["create_usb_seed"],
        "status": "pending",
        "params": {
          "primal": "beardog",
          "family_id": "nat0"
        }
      }
    ],
    "total_nodes": 12,
    "execution_order": ["create_usb_seed", "deploy_beardog", "..."]
  },
  "id": 2
}
```

---

### **2. Graph Execution**

#### **Method**: `neural_api.execute_graph`

**Description**: Execute a graph (async, returns execution ID)

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.execute_graph",
  "params": {
    "graph_id": "nucleus_deployment",
    "environment": {
      "FAMILY_ID": "nat0",
      "USB_SEED_PATH": "/tmp/livespore-nat0/.family.seed"
    }
  },
  "id": 3
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "execution_id": "exec-def456",
    "status": "running",
    "started_at": "2026-01-14T08:35:00Z",
    "graph_id": "nucleus_deployment",
    "total_nodes": 12,
    "completed_nodes": 0
  },
  "id": 3
}
```

---

#### **Method**: `neural_api.get_execution_status`

**Description**: Get current status of a running execution

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.get_execution_status",
  "params": {
    "execution_id": "exec-def456"
  },
  "id": 4
}
```

**Response** (Running):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "execution_id": "exec-def456",
    "graph_id": "nucleus_deployment",
    "status": "running",
    "started_at": "2026-01-14T08:35:00Z",
    "total_nodes": 12,
    "completed_nodes": 5,
    "failed_nodes": 0,
    "current_node": "deploy_toadstool",
    "progress_percent": 41.7,
    "estimated_completion": "2026-01-14T08:40:00Z",
    "node_statuses": {
      "create_usb_seed": "completed",
      "deploy_beardog": "completed",
      "deploy_songbird": "completed",
      "verify_tower": "completed",
      "deploy_toadstool": "running",
      "deploy_nestgate": "pending"
    }
  },
  "id": 4
}
```

**Response** (Completed):
```json
{
  "jsonrpc": "2.0",
  "result": {
    "execution_id": "exec-def456",
    "status": "completed",
    "started_at": "2026-01-14T08:35:00Z",
    "completed_at": "2026-01-14T08:42:00Z",
    "duration_seconds": 420,
    "total_nodes": 12,
    "completed_nodes": 12,
    "failed_nodes": 0,
    "success": true
  },
  "id": 4
}
```

---

#### **Method**: `neural_api.cancel_execution`

**Description**: Cancel a running execution

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.cancel_execution",
  "params": {
    "execution_id": "exec-def456"
  },
  "id": 5
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "execution_id": "exec-def456",
    "status": "cancelled",
    "nodes_completed": 5,
    "nodes_cancelled": 7
  },
  "id": 5
}
```

---

### **3. Event Streaming**

#### **Endpoint**: `GET /api/v1/neural/events` (SSE)

**Description**: Stream real-time execution events

**Response** (Server-Sent Events):
```
event: execution_started
data: {"execution_id":"exec-def456","graph_id":"nucleus_deployment","started_at":"2026-01-14T08:35:00Z"}

event: node_started
data: {"execution_id":"exec-def456","node_id":"create_usb_seed","started_at":"2026-01-14T08:35:01Z"}

event: node_completed
data: {"execution_id":"exec-def456","node_id":"create_usb_seed","status":"success","duration_ms":150}

event: node_started
data: {"execution_id":"exec-def456","node_id":"deploy_beardog","started_at":"2026-01-14T08:35:02Z"}

event: node_completed
data: {"execution_id":"exec-def456","node_id":"deploy_beardog","status":"success","duration_ms":2500}

event: execution_completed
data: {"execution_id":"exec-def456","status":"completed","duration_seconds":420}
```

---

## 🏗️ Implementation Plan

### **Phase 1: Core JSON-RPC Server** (4-6 hours)

**Files to Create**:
```
crates/biomeos-neural-api/
  ├── Cargo.toml
  ├── src/
  │   ├── lib.rs
  │   ├── server.rs          # JSON-RPC server (Unix socket!)
  │   ├── handlers.rs         # RPC method handlers
  │   ├── execution_manager.rs # Track running executions
  │   ├── graph_registry.rs   # Scan & index graphs/
  │   ├── events.rs           # Event streaming (Unix socket)
  │   ├── types.rs            # Request/response types
  │   └── error.rs            # Error handling
  └── tests/
      └── integration_tests.rs
```

**Dependencies**:
```toml
[dependencies]
# JSON-RPC over Unix socket (PRIMARY)
jsonrpc-core = "18"
jsonrpc-stdio-server = "18"  # For Unix socket support

tokio = { version = "1", features = ["full", "net"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
thiserror = "1"
anyhow = "1"
nix = "0.29"  # For Unix socket permissions

# biomeOS internal
biomeos-atomic-deploy = { path = "../biomeos-atomic-deploy" }
biomeos-types = { path = "../biomeos-types" }

# FUTURE: tarpc for type-safe RPC
# tarpc = { version = "0.34", features = ["tokio1", "serde-transport"] }
```

**Note**: ❌ **Removed** `axum`, `tower`, `tower-http` (HTTP-specific!)

---

### **Phase 2: Graph Registry** (2-3 hours)

**Responsibility**: Scan `graphs/*.toml` and index them

```rust
pub struct GraphRegistry {
    graphs: HashMap<String, GraphMetadata>,
    graphs_dir: PathBuf,
}

impl GraphRegistry {
    pub fn new(graphs_dir: PathBuf) -> Result<Self>;
    pub fn scan_graphs(&mut self) -> Result<()>;
    pub fn get_graph(&self, id: &str) -> Option<&GraphMetadata>;
    pub fn list_graphs(&self) -> Vec<&GraphMetadata>;
}

pub struct GraphMetadata {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub node_count: usize,
    pub description: String,
    pub last_executed: Option<DateTime<Utc>>,
}
```

---

### **Phase 3: Execution Manager** (3-4 hours)

**Responsibility**: Track and manage running graph executions

```rust
pub struct ExecutionManager {
    executions: Arc<RwLock<HashMap<String, ExecutionState>>>,
}

impl ExecutionManager {
    pub async fn start_execution(
        &self,
        graph_id: String,
        environment: HashMap<String, String>,
    ) -> Result<String>; // Returns execution_id
    
    pub async fn get_status(&self, execution_id: &str) -> Option<ExecutionStatus>;
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<()>;
    pub fn subscribe_events(&self, execution_id: &str) -> EventStream;
}

pub struct ExecutionState {
    pub id: String,
    pub graph_id: String,
    pub status: ExecutionStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub node_statuses: HashMap<String, NodeStatus>,
    pub event_sender: broadcast::Sender<ExecutionEvent>,
}
```

---

### **Phase 4: SSE Event Streaming** (2-3 hours)

**Responsibility**: Stream real-time events to clients

```rust
pub async fn events_handler(
    State(manager): State<Arc<ExecutionManager>>,
    Query(params): Query<EventsQuery>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut event_rx = manager.subscribe_events(&params.execution_id);
    
    let stream = async_stream::stream! {
        while let Ok(event) = event_rx.recv().await {
            let json = serde_json::to_string(&event).unwrap();
            yield Ok(Event::default().event(event.event_type()).data(json));
        }
    };
    
    Sse::new(stream)
}
```

---

### **Phase 5: Integration Testing** (2-3 hours)

**Tests**:
1. List graphs
2. Execute graph (simple)
3. Get execution status
4. Cancel execution
5. SSE event streaming
6. Error handling (invalid graph, missing params)
7. Concurrent executions

---

## 🎯 Integration with Existing Systems

### **1. PetalTongue (View 6)**

**Current**: View 6 (Neural Graph Management) is built but needs endpoints

**Integration**:
```javascript
// PetalTongue queries neuralAPI via Unix socket JSON-RPC client
// (PetalTongue needs to use a Unix socket JSON-RPC library)

const neuralClient = new UnixSocketJsonRpcClient('/run/user/1000/neural-api.sock');

// List graphs
const graphs = await neuralClient.call('neural_api.list_graphs', {});
renderGraphsList(graphs.graphs);

// Execute graph
const result = await neuralClient.call('neural_api.execute_graph', {
  graph_id: 'nucleus_deployment',
  environment: { FAMILY_ID: 'nat0' }
});

// Stream events (via Unix socket SSE or WebSocket)
const eventStream = await neuralClient.subscribe_events(result.execution_id);
eventStream.on('data', (event) => updateProgress(event));
```

**Alternative (Temporary HTTP Bridge)**:
```javascript
// ONLY if PetalTongue can't use Unix sockets yet
// This is TEMPORARY and should be removed!

fetch('http://localhost:3000/api/v1/neural/graphs')  // ⚠️ Via biomeOS API proxy
  .then(res => res.json())
  .then(data => renderGraphsList(data.graphs));
```

**Status**: ✅ PetalTongue ready, waiting for server!

---

### **2. Squirrel (AI Integration)**

**Current**: Squirrel MCP server can coordinate graph execution

**Integration**:
```rust
// Squirrel queries neuralAPI
let graphs = neural_api.list_graphs().await?;

// AI decides which graph to run
let graph_id = ai_advisor.recommend_graph(graphs, user_intent)?;

// Execute
let execution_id = neural_api.execute_graph(graph_id, env).await?;

// Monitor
let status = neural_api.get_execution_status(&execution_id).await?;
```

**Status**: ⏳ Waiting for neuralAPI server

---

### **3. biomeOS CLI**

**Current**: `biomeos-cli` can execute graphs directly (no server)

**Future**:
```bash
# Query server
biomeos-cli neural list-graphs

# Execute via server
biomeos-cli neural execute nucleus_deployment

# Stream events
biomeos-cli neural watch exec-abc123
```

**Status**: ✅ Direct execution works, server integration planned

---

## 🔧 Configuration

### **Environment Variables**

| Variable | Purpose | Default |
|----------|---------|---------|
| `NEURAL_API_SOCKET_PATH` | Unix socket path (PRIMARY) | `/run/user/{uid}/neural-api.sock` |
| `NEURAL_API_PROTOCOL` | Protocol (jsonrpc, tarpc) | `jsonrpc` |
| `NEURAL_API_GRAPHS_DIR` | Graphs directory | `./graphs` |
| `NEURAL_API_MAX_CONCURRENT` | Max concurrent executions | `5` |
| `RUST_LOG` | Log level | `info` |

### **⚠️ HTTP Deprecated!**

**IMPORTANT**: This spec originally used HTTP but has been updated to use **Unix sockets only** for security!

- ❌ **REMOVED**: `NEURAL_API_BIND_ADDR` (HTTP is deprecated!)
- ✅ **PRIMARY**: Unix socket (`/run/user/{uid}/neural-api.sock`)
- ✅ **FUTURE**: tarpc over Unix socket (type-safe)

---

## 📊 Success Criteria

**Phase 1 Complete When**:
- ✅ Server starts and listens on Unix socket
- ✅ JSON-RPC 2.0 requests handled correctly
- ✅ Can list graphs
- ✅ Can get graph details
- ✅ Can execute a simple graph
- ✅ Can query execution status
- ✅ Integration tests passing

**Phase 2 Complete When**:
- ✅ PetalTongue View 6 fully functional
- ✅ SSE events streaming
- ✅ Squirrel can coordinate graphs
- ✅ Documentation complete

---

## 🎯 Timeline

**Estimated**: 12-16 hours (2 work days)

| Phase | Hours | Dependencies |
|-------|-------|--------------|
| 1. Core Server | 4-6h | None |
| 2. Graph Registry | 2-3h | Phase 1 |
| 3. Execution Manager | 3-4h | Phases 1 & 2 |
| 4. SSE Events | 2-3h | Phases 1-3 |
| 5. Testing | 2-3h | Phases 1-4 |

**Target**: Complete by January 16, 2026

---

## 📚 Related Specifications

- **[GRAPH_BASED_ORCHESTRATION_SPEC.md](GRAPH_BASED_ORCHESTRATION_SPEC.md)** - Graph engine (already built)
- **[COLLABORATIVE_INTELLIGENCE_SPEC.md](COLLABORATIVE_INTELLIGENCE_SPEC.md)** - AI integration
- **[INTERACTIVE_UI_SPEC.md](INTERACTIVE_UI_SPEC.md)** - PetalTongue integration
- **[NEURAL_API_IMPLEMENTATION_PHASES.md](NEURAL_API_IMPLEMENTATION_PHASES.md)** - Overall roadmap

---

## 🎊 Conclusion

**The graph engine is already built! We just need a JSON-RPC wrapper!**

**This will enable**:
- ✅ PetalTongue View 6 (Neural Graph Management)
- ✅ Squirrel AI coordination
- ✅ Remote graph execution
- ✅ Real-time progress monitoring
- ✅ CLI integration

**Ready to proceed!** 🧠🚀

---

**Created**: January 14, 2026  
**Status**: 🟢 READY FOR IMPLEMENTATION  
**Priority**: HIGH  
**Estimated**: 12-16 hours

**"Infrastructure for infinite composition!"** 🌳🐸✨


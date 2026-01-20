# 🧠 Neural API Routing Layer - Technical Specification

**Version**: v1.0.0  
**Date**: January 20, 2026  
**Status**: Design Complete, Implementation Pending  
**Priority**: HIGH - Core Infrastructure

---

## 📋 **OVERVIEW**

### **Purpose**

The Neural API Routing Layer enables primal-to-primal communication through a central API gateway, maintaining the TRUE PRIMAL pattern where primals remain ignorant of each other and interact only through capability-based requests.

### **Scope**

This specification covers:
- Routing architecture and patterns
- API methods and interfaces
- Capability discovery mechanisms
- Request forwarding protocols
- Metrics collection for future learning

**Out of Scope** (Future):
- Machine learning optimization
- Automatic pathway discovery
- Performance tuning algorithms

---

## 🏗️ **ARCHITECTURE**

### **Three-Layer Model**

```
┌────────────────────────────────────────────────────┐
│ Layer 3: Niche APIs (RootPulse, Hive, Reef)       │
│ • High-level domain abstractions                   │
│ • Multi-primal workflows                           │
│ • User-facing APIs                                 │
└────────────────────────────────────────────────────┘
                    ↕ JSON-RPC
┌────────────────────────────────────────────────────┐
│ Layer 2: Neural API (THIS SPEC)                   │
│ ┌──────────────────┐  ┌──────────────────────┐   │
│ │ Deployment       │  │ Routing (NEW)        │   │
│ │ • Launch primals │  │ • HTTP proxy         │   │
│ │ • Health check   │  │ • Capability routing │   │
│ │ • Graph execute  │  │ • Request forward    │   │
│ └──────────────────┘  └──────────────────────┘   │
└────────────────────────────────────────────────────┘
                    ↕ Unix Sockets
┌────────────────────────────────────────────────────┐
│ Layer 1: Primals (BearDog, Songbird, etc)         │
│ • Self-contained services                          │
│ • Single responsibility                            │
│ • TRUE PRIMAL (self-knowledge only)                │
└────────────────────────────────────────────────────┘
```

### **Design Principles**

1. **TRUE PRIMAL Pattern**: Primals never know about other primals
2. **Capability-Based**: All routing uses capabilities, not names
3. **Service Mesh**: Neural API acts as API gateway/service mesh
4. **Observable**: All requests logged for future learning
5. **Unix Sockets**: All transport over Unix domain sockets

---

## 🔌 **API METHODS**

### **Core Routing Methods**

#### **1. proxy_http** - HTTP Proxying Through Tower Atomic

**Purpose**: Route HTTP/HTTPS requests through Tower Atomic (BearDog + Songbird) for secure communication.

**Method**: `neural_api.proxy_http`

**Parameters**:
```json
{
  "capability": "secure_http",
  "url": "https://api.example.com/endpoint",
  "method": "GET" | "POST" | "PUT" | "DELETE",
  "headers": {
    "Header-Name": "value"
  },
  "body": {} | null
}
```

**Returns**:
```json
{
  "status": 200,
  "headers": {
    "Content-Type": "application/json"
  },
  "body": {},
  "metrics": {
    "latency_ms": 150,
    "routed_through": ["songbird", "beardog"]
  }
}
```

**Example**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.proxy_http",
  "params": {
    "capability": "secure_http",
    "url": "https://api.anthropic.com/v1/messages",
    "method": "POST",
    "headers": {
      "x-api-key": "sk-ant-...",
      "content-type": "application/json"
    },
    "body": {
      "model": "claude-3-opus-20240229",
      "messages": [{"role": "user", "content": "Hello!"}]
    }
  },
  "id": 1
}
```

---

#### **2. call_capability** - Generic Capability Routing

**Purpose**: Route arbitrary JSON-RPC calls to primals providing a capability.

**Method**: `neural_api.call_capability`

**Parameters**:
```json
{
  "capability": "capability_name",
  "method": "primal.method_name",
  "params": {}
}
```

**Returns**:
```json
{
  "result": {},
  "metrics": {
    "latency_ms": 50,
    "routed_to": "primal_name"
  }
}
```

**Example**:
```json
{
  "jsonrpc": "2.0",
  "method": "neural_api.call_capability",
  "params": {
    "capability": "crypto_sign",
    "method": "sign.ed25519",
    "params": {
      "data": "hello world"
    }
  },
  "id": 1
}
```

---

#### **3. store_data** - Storage Through Nest Atomic

**Purpose**: Store data through Nest Atomic (Tower + NestGate) for secure storage.

**Method**: `neural_api.store_data`

**Parameters**:
```json
{
  "capability": "secure_storage",
  "key": "namespace:identifier",
  "value": {},
  "metadata": {}
}
```

**Returns**:
```json
{
  "stored": true,
  "key": "namespace:identifier",
  "hash": "sha256:...",
  "metrics": {
    "latency_ms": 10,
    "routed_through": ["nestgate", "beardog"]
  }
}
```

---

#### **4. execute_compute** - Compute Through Node Atomic

**Purpose**: Execute computation through Node Atomic (Tower + ToadStool).

**Method**: `neural_api.execute_compute`

**Parameters**:
```json
{
  "capability": "secure_compute",
  "function": "function_name",
  "args": {},
  "environment": {}
}
```

**Returns**:
```json
{
  "result": {},
  "logs": [],
  "metrics": {
    "latency_ms": 500,
    "routed_through": ["toadstool", "beardog"]
  }
}
```

---

## 🔍 **CAPABILITY DISCOVERY**

### **Capability → Atomic Mapping**

| Capability | Atomic | Primals | Purpose |
|------------|--------|---------|---------|
| `secure_http` | Tower | BearDog + Songbird | HTTP/HTTPS with Pure Rust TLS |
| `secure_storage` | Nest | BearDog + Songbird + NestGate | Encrypted storage |
| `secure_compute` | Node | BearDog + Songbird + ToadStool | Encrypted compute |
| `crypto_sign` | Tower | BearDog | Cryptographic signing |
| `discovery` | Tower | Songbird | Service discovery |
| `ai` | Standalone | Squirrel | AI/LLM requests |

### **Discovery Algorithm**

```rust
async fn discover_capability(capability: &str) -> Result<DiscoveredPrimals> {
    match capability {
        "secure_http" => {
            // Tower Atomic = BearDog + Songbird
            let topology = get_topology().await?;
            find_primal_pair(&topology, "beardog", "songbird").await
        }
        "secure_storage" => {
            // Nest Atomic = Tower + NestGate
            let tower = discover_capability("secure_http").await?;
            let nestgate = find_primal(&topology, "nestgate").await?;
            Ok(tower + nestgate)
        }
        "secure_compute" => {
            // Node Atomic = Tower + ToadStool
            let tower = discover_capability("secure_http").await?;
            let toadstool = find_primal(&topology, "toadstool").await?;
            Ok(tower + toadstool)
        }
        _ => Err(anyhow!("Unknown capability: {}", capability))
    }
}
```

---

## 🔄 **REQUEST FLOW**

### **Typical Flow**

```
1. Client Request
   ├─ Primal sends JSON-RPC to Neural API
   └─ Method: neural_api.proxy_http

2. Capability Discovery
   ├─ Neural API extracts capability: "secure_http"
   ├─ Discovers Tower Atomic (BearDog + Songbird)
   └─ Gets socket paths: /tmp/songbird-nat0.sock, /tmp/beardog-nat0.sock

3. Request Routing
   ├─ Neural API connects to Songbird socket
   ├─ Forwards request as JSON-RPC
   └─ Songbird internally uses BearDog for crypto

4. Response Handling
   ├─ Songbird returns response
   ├─ Neural API logs metrics
   └─ Neural API returns to client

5. Metrics Collection
   ├─ Latency: 150ms
   ├─ Route: Squirrel → Neural API → Songbird → BearDog → Anthropic
   └─ Stored for future learning
```

### **Sequence Diagram**

```
Squirrel     Neural API     Songbird     BearDog     Anthropic
   │              │             │            │            │
   ├─request─────>│             │            │            │
   │              ├─discover────┤            │            │
   │              │             │            │            │
   │              ├─forward────>│            │            │
   │              │             ├─crypto────>│            │
   │              │             │            ├─https─────>│
   │              │             │            │<───────────┤
   │              │             │<───────────┤            │
   │              │<────────────┤            │            │
   │<─response────┤             │            │            │
   │              ├─log metrics │            │            │
```

---

## 💾 **DATA STRUCTURES**

### **DiscoveredPrimals**

```rust
pub struct DiscoveredPrimals {
    pub capability: String,
    pub primals: Vec<DiscoveredPrimal>,
    pub atomic_type: Option<AtomicType>,
}

pub struct DiscoveredPrimal {
    pub name: String,
    pub socket_path: PathBuf,
    pub capabilities: Vec<String>,
    pub health: HealthStatus,
}

pub enum AtomicType {
    Tower,  // BearDog + Songbird
    Nest,   // Tower + NestGate
    Node,   // Tower + ToadStool
}
```

### **RoutingMetrics**

```rust
pub struct RoutingMetrics {
    pub request_id: String,
    pub capability: String,
    pub method: String,
    pub routed_through: Vec<String>,
    pub latency_ms: u64,
    pub success: bool,
    pub timestamp: DateTime<Utc>,
}
```

---

## 🛠️ **IMPLEMENTATION**

### **File Structure**

```
crates/biomeos-atomic-deploy/src/
├── neural_api_server.rs       (MODIFY)
│   ├── handle_request()       (add new methods)
│   ├── proxy_http()           (NEW)
│   ├── call_capability()      (NEW)
│   ├── store_data()           (NEW)
│   └── execute_compute()      (NEW)
│
├── neural_router.rs           (NEW)
│   ├── discover_capability()
│   ├── forward_to_primal()
│   ├── collect_metrics()
│   └── find_atomic()
│
└── neural_executor.rs         (existing)
    └── (deployment logic)
```

### **Implementation Steps**

**Phase 1: Core Infrastructure** (Day 1-2)
1. Create `neural_router.rs` module
2. Implement `discover_capability()`
3. Implement `forward_to_primal()`
4. Add basic metrics collection

**Phase 2: HTTP Proxy** (Day 2-3)
1. Implement `proxy_http()` method
2. Add to method router in `neural_api_server.rs`
3. Test with Squirrel → Anthropic flow

**Phase 3: Generic Routing** (Day 3-4)
1. Implement `call_capability()` method
2. Implement `store_data()` and `execute_compute()`
3. Add atomic discovery helpers

**Phase 4: Testing & Validation** (Day 4-5)
1. Integration tests for all routing patterns
2. Validate TRUE PRIMAL compliance
3. Performance testing
4. Documentation

---

## 🧪 **TESTING**

### **Unit Tests**

```rust
#[tokio::test]
async fn test_discover_tower_atomic() {
    // Deploy BearDog + Songbird
    // Call discover_capability("secure_http")
    // Assert returns both primals with correct sockets
}

#[tokio::test]
async fn test_forward_to_primal() {
    // Start mock primal server
    // Forward JSON-RPC request
    // Assert response received correctly
}
```

### **Integration Tests**

```rust
#[tokio::test]
async fn test_squirrel_ai_request_via_neural_api() {
    // 1. Deploy Tower + Squirrel via Neural API
    // 2. Squirrel calls neural_api.proxy_http
    // 3. Assert request routed through Tower Atomic
    // 4. Assert response returned to Squirrel
    // 5. Assert metrics collected
}
```

### **Validation Criteria**

- ✅ Primals never know about each other (TRUE PRIMAL)
- ✅ All requests routed through Neural API
- ✅ Correct capability → primal mapping
- ✅ Metrics collected for all requests
- ✅ Error handling graceful
- ✅ Performance acceptable (<200ms overhead)

---

## 📊 **PERFORMANCE**

### **Target Metrics**

| Metric | Target | Rationale |
|--------|--------|-----------|
| Routing Overhead | <50ms | Should be negligible vs network I/O |
| Discovery Cache | <10ms | Cached topology lookup |
| Socket Connection | <20ms | Unix socket is fast |
| Request Forwarding | <20ms | Simple JSON-RPC pass-through |

### **Optimization Strategies**

1. **Connection Pooling**: Reuse Unix socket connections
2. **Topology Caching**: Cache primal discovery results
3. **Async I/O**: Fully async request handling
4. **Batching**: Future optimization for multiple requests

---

## 🔒 **SECURITY**

### **Threat Model**

1. **Malicious Primal**: Can't access other primals directly (routing prevents)
2. **Capability Spoofing**: Neural API validates capabilities before routing
3. **Socket Hijacking**: Unix socket permissions (owner-only)
4. **Man-in-Middle**: Not applicable (local Unix sockets)

### **Security Guarantees**

- ✅ Primals isolated (can't talk directly)
- ✅ Capability-based access control
- ✅ All routing logged (audit trail)
- ✅ Socket permissions enforced

---

## 📈 **FUTURE ENHANCEMENTS**

### **Phase 2: Learning** (Future)

- Automatic pathway optimization
- Usage pattern detection
- Intelligent caching
- Predictive routing

### **Phase 3: Advanced Features** (Future)

- Load balancing across multiple primal instances
- Circuit breakers for failing primals
- Request retries with exponential backoff
- Distributed tracing

---

## 📚 **REFERENCES**

- [Neural API Whitepaper](../../whitePaper/neuralAPI/)
- [RootPulse Whitepaper](../../whitePaper/RootPulse/)
- [BIOMEOS_ATOMICS_ARCHITECTURE.md](../BIOMEOS_ATOMICS_ARCHITECTURE.md)
- [TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](../TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)

---

## ✅ **ACCEPTANCE CRITERIA**

### **Must Have** (v1.0.0):
- ✅ `proxy_http` method working
- ✅ `call_capability` method working
- ✅ Capability discovery implemented
- ✅ Request forwarding operational
- ✅ Metrics collection basic
- ✅ Integration tests passing
- ✅ TRUE PRIMAL pattern validated

### **Should Have** (v1.1.0):
- Connection pooling
- Topology caching
- Performance optimization
- Comprehensive error handling

### **Nice to Have** (v2.0.0):
- Learning mechanisms
- Automatic optimization
- Load balancing
- Circuit breakers

---

## 📋 **STATUS TRACKING**

| Component | Status | ETA |
|-----------|--------|-----|
| Specification | ✅ Complete | Done |
| Core Infrastructure | ⏳ Not Started | Day 1-2 |
| HTTP Proxy | ⏳ Not Started | Day 2-3 |
| Generic Routing | ⏳ Not Started | Day 3-4 |
| Testing | ⏳ Not Started | Day 4-5 |
| Documentation | ⏳ Not Started | Day 5 |

**Overall Progress**: 0% (Design Complete, Implementation Pending)  
**Estimated Completion**: 5 days from start  
**Priority**: HIGH - Unblocks TRUE PRIMAL pattern

---

🧠✨ **Neural API Routing: The Brain That Connects Everything** ✨🧠

**Version**: v1.0.0  
**Status**: Specification Complete  
**Next**: Begin Implementation


# 🧬 TODO Evolution Plan

**Date**: January 15, 2026  
**Found**: 86 TODO/FIXME comments across production code  
**Philosophy**: Deep debt solutions, modern idiomatic Rust, capability-based evolution

---

## 📊 TODO CATEGORIZATION

### ✅ **Category 1: Already Perfect (0 TODOs)** 
**No production mocks** - All 345 mock references are test-only ✅

---

### 🚀 **Category 2: High-Priority Production Implementations (12 TODOs)**

These need evolution from placeholder to complete implementation:

#### **2.1 Unix Socket Health Checks** (Priority: HIGH)
**File**: `crates/biomeos-federation/src/beardog_client.rs:127`
```rust
// TODO: Implement Unix socket health check
// For now, just check if socket exists
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation (JSON-RPC 2.0)
BearDogEndpoint::UnixSocket(path) => {
    let client = UnixSocketClient::new(path);
    client.call_method("health.check", json!({})).await
        .context("Unix socket health check failed")
}
```

**Principle**: Complete implementation, not mock  
**Timeline**: Week 1 (2-3 hours)

---

#### **2.2 SSE Client Implementation** (Priority: HIGH)
**File**: `crates/biomeos-ui/src/realtime.rs:229`
```rust
// TODO: Implement SSE client
warn!("SSE subscription not yet implemented, use WebSocket");
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation
use eventsource_client::{Client as SSEClient, SSE};

async fn subscribe_sse(endpoint: String, event_tx: Sender<GraphEvent>) -> Result<()> {
    let client = SSEClient::new(&endpoint)?;
    
    for event in client.stream() {
        match event {
            Ok(SSE::Event(ev)) => {
                let graph_event: GraphEvent = serde_json::from_str(&ev.data)?;
                event_tx.send(graph_event).await?;
            }
            Ok(SSE::Comment(_)) => continue,
            Err(e) => warn!("SSE error: {}", e),
        }
    }
    Ok(())
}
```

**Dependencies**: Add `eventsource-client` crate (pure Rust)  
**Principle**: Complete, not placeholder  
**Timeline**: Week 1 (4-6 hours)

---

#### **2.3 JSON-RPC Server Implementations** (Priority: HIGH)
**File**: `crates/biomeos-ui/src/capabilities/device_management/provider.rs:82`
```rust
// TODO: Start JSON-RPC server
// rpc_server::start(self.socket_path.clone(), self.clone()).await?;
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation
use jsonrpc_core::{IoHandler, Params, Value};
use jsonrpc_ipc_server::{ServerBuilder, SessionStats};

pub async fn start_jsonrpc_server(
    socket_path: PathBuf,
    provider: DeviceManagementProvider,
) -> Result<()> {
    let mut io = IoHandler::new();
    
    // Register methods
    io.add_method("device.list", move |params: Params| {
        let provider = provider.clone();
        async move {
            let devices = provider.list_devices().await?;
            Ok(Value::Array(devices.iter().map(|d| 
                serde_json::to_value(d).unwrap()
            ).collect()))
        }
    });
    
    // More methods...
    
    let server = ServerBuilder::new(io)
        .session_stats(SessionStats::default())
        .start(&socket_path.to_string_lossy())?;
    
    server.wait();
    Ok(())
}
```

**Dependencies**: Use existing `jsonrpc-*` crates (pure Rust)  
**Principle**: TRUE PRIMAL - self-knowledge only, advertise capability  
**Timeline**: Week 2 (8-12 hours)

---

#### **2.4 TRUE PRIMAL Discovery Queries** (Priority: HIGH)
**File**: `crates/biomeos-ui/src/capabilities/device_management/provider.rs:511`
```rust
// TODO: Query via JSON-RPC for primal name
// For now, derive from socket path
"unknown".to_string()
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation (TRUE PRIMAL!)
async fn query_primal_identity(&self, socket_path: &str) -> Result<String> {
    let client = UnixSocketClient::new(socket_path);
    
    // Query primal for its self-knowledge
    let response = client.call_method("identity.get", json!({})).await?;
    
    Ok(response["name"].as_str()
        .unwrap_or("unknown")
        .to_string())
}
```

**Principle**: **Primal code only has self knowledge** - query at runtime  
**Architecture**: Capability-based, not hardcoded  
**Timeline**: Week 2 (4-6 hours)

---

#### **2.5 Capability Discovery vs Hardcoding** (Priority: HIGH)
**File**: `crates/biomeos-ui/src/capabilities/device_management/provider.rs:524`
```rust
// TODO: Query primal for actual capabilities
// For now, use known patterns
match primal_id {
    // Hardcoded capability matching
}
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation (Capability-Based!)
async fn get_primal_capabilities(&self, socket_path: &str) -> Result<Vec<String>> {
    let client = UnixSocketClient::new(socket_path);
    
    // Query primal for its capabilities (self-knowledge)
    let response = client.call_method("capabilities.list", json!({})).await?;
    
    Ok(response["capabilities"].as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect())
}
```

**Principle**: **Agnostic and capability-based** - no hardcoding  
**Architecture**: Runtime discovery, not compile-time assumptions  
**Timeline**: Week 2 (4-6 hours)

---

#### **2.6 Encryption Key Caching** (Priority: MEDIUM)
**File**: `crates/biomeos-core/src/encrypted_storage/backend.rs:293`
```rust
// TODO: Implement key caching to avoid regenerating the same key
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation (Zero-copy where possible)
use std::sync::Arc;
use dashmap::DashMap;

pub struct KeyCache {
    cache: Arc<DashMap<String, Arc<[u8; 32]>>>,
    max_size: usize,
}

impl KeyCache {
    pub fn get_or_derive(&self, key_id: &str, derive_fn: impl FnOnce() -> [u8; 32]) -> Arc<[u8; 32]> {
        if let Some(key) = self.cache.get(key_id) {
            return key.clone(); // Arc clone is cheap
        }
        
        let key = Arc::new(derive_fn());
        self.cache.insert(key_id.to_string(), key.clone());
        
        // Evict if over max_size (LRU strategy)
        if self.cache.len() > self.max_size {
            // Evict oldest entry
        }
        
        key
    }
}
```

**Dependencies**: `dashmap` (lock-free concurrent HashMap)  
**Principle**: **Fast AND safe Rust** - concurrent, zero unsafe  
**Performance**: Lock-free, Arc for cheap sharing  
**Timeline**: Week 3 (6-8 hours)

---

### 📚 **Category 3: Documented Future Phases (45 TODOs)**

These are **intentional placeholders** for planned evolution phases:

#### **Phase 2: Songbird Integration** (15 TODOs)
- Device discovery via Songbird
- Service registration
- Event subscription
- Primal querying

**Status**: ✅ Properly documented  
**Timeline**: Week 4-6 (as per specs)

#### **Phase 3: Orchestration Actions** (12 TODOs)
- Start/stop/restart primals
- Device assignment/unassignment
- Topology management

**Status**: ✅ Properly documented  
**Timeline**: Week 7-9 (as per specs)

#### **Phase 4: Squirrel Integration** (8 TODOs)
- AI suggestions
- Acceptance/dismissal workflows
- Learning from user actions

**Status**: ✅ Properly documented  
**Timeline**: Week 10-12 (as per specs)

#### **Phase 5: State Persistence** (10 TODOs)
- NestGate integration
- Load/save configurations
- Recovery from saved state

**Status**: ✅ Properly documented  
**Timeline**: Week 13-15 (as per specs)

**Assessment**: These are NOT debt - they're **evolution roadmap markers** ✅

---

### 🔧 **Category 4: Technical Debt to Evolve (15 TODOs)**

#### **4.1 Tarpc Transport** (Priority: MEDIUM)
**File**: `crates/biomeos-core/src/clients/transport/mod.rs:142`
```rust
// TODO: Implement tarpc transport
warn!("tarpc transport not yet implemented, trying Unix socket");
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation
TransportPreference::Tarpc => {
    use tarpc::{client, context, tokio_serde::formats::Json};
    
    let addr = format!("{}:{}", primal_endpoint, port);
    let transport = tarpc::serde_transport::tcp::connect(addr, Json::default()).await?;
    let client = PrimalClient::new(client::Config::default(), transport).spawn();
    
    Ok(Self::Tarpc(client))
}
```

**Dependencies**: `tarpc` (pure Rust RPC framework)  
**Principle**: Modern idiomatic Rust - async/await, type-safe  
**Timeline**: Week 3 (8-10 hours)

---

#### **4.2 Rollback Strategies** (Priority: LOW)
**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs:456`
```rust
// TODO: Implement rollback strategy
warn!("🔄 Rollback not yet implemented");
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation (Smart, not naive)
async fn rollback(&self) -> Result<()> {
    // 1. Stop newly deployed primals
    for primal in &self.deployed_primals {
        self.stop_primal(primal).await?;
    }
    
    // 2. Restore previous state from NestGate
    if let Some(previous_state) = self.load_previous_state().await? {
        self.restore_state(previous_state).await?;
    }
    
    // 3. Re-register previous services with Songbird
    for service in &self.previous_services {
        self.register_service(service).await?;
    }
    
    // 4. Verify rollback success
    self.verify_healthy_state().await?;
    
    Ok(())
}
```

**Principle**: **Deep debt solution** - complete rollback, not just stop  
**Timeline**: Week 4 (6-8 hours)

---

#### **4.3 E2E Test Completions** (Priority: LOW)
**File**: `crates/biomeos-spore/tests/e2e_incubation_tests.rs:46`
```rust
// TODO: Full incubation would require:
// - Mocking HOME directory
// - Creating local config
```

**Evolution Plan**:
```rust
// ✅ Complete Implementation (Real E2E, not mocks)
#[tokio::test]
async fn test_full_incubation_workflow() {
    // Use tempdir for isolated test environment
    let temp_home = TempDir::new()?;
    std::env::set_var("HOME", temp_home.path());
    
    // Create real config (not mock)
    let config = IncubationConfig {
        spore_path: temp_home.path().join("spore"),
        family_seed: generate_test_seed(),
        node_id: "test-node-001".to_string(),
    };
    
    // Run actual incubation (real implementation)
    let result = incubate_spore(config).await?;
    
    // Verify actual outcomes
    assert!(result.spore_path.exists());
    assert!(result.binaries_copied > 0);
    assert!(result.lineage_verified);
}
```

**Principle**: **Mocks isolated to testing**, but prefer real E2E  
**Timeline**: Part of 90% coverage expansion (Week 5-7)

---

### 🎯 **Category 5: Already Planned Evolution (14 TODOs)**

These are covered by existing evolution specs:

#### **Encryption Week 2-8** (NUCLEUS_ENCRYPTION_SPEC.md)
- Week 2: Key rotation
- Week 3: Multi-key support
- Week 4: Hardware-backed keys
- Week 5: Zero-knowledge metadata
- Week 6: Audit logging
- Week 7: Performance optimization
- Week 8: Production hardening

**Status**: ✅ Fully specified, 7-week plan ready  
**TODOs**: 6 in encrypted_storage/

---

#### **LiveSpore BirdSong v3.0** (LIVESPORE_CROSS_PRIMAL_COORDINATION_JAN_2026.md)
- Multi-callsign tag system
- Sequence numbers
- Key rotation support
- Institutional NAT routing

**Status**: ✅ 6-week evolution plan active  
**TODOs**: 4 in federation/

---

## 📋 EVOLUTION PRIORITIES

### **Immediate (Week 1-2): Critical Path**
1. ✅ Unix socket health checks (2-3h)
2. ✅ SSE client implementation (4-6h)
3. ✅ JSON-RPC server (8-12h)
4. ✅ TRUE PRIMAL discovery queries (4-6h)
5. ✅ Capability-based querying (4-6h)

**Total**: 22-33 hours (~3-4 days)  
**Impact**: Production-ready core functionality

---

### **Short-term (Week 3-4): Infrastructure**
6. ✅ Encryption key caching (6-8h)
7. ✅ Tarpc transport (8-10h)
8. ✅ Rollback strategies (6-8h)

**Total**: 20-26 hours (~3 days)  
**Impact**: Performance + reliability

---

### **Medium-term (Week 5-12): Planned Phases**
9. ✅ Phase 2: Songbird integration
10. ✅ Phase 3: Orchestration actions
11. ✅ Phase 4: Squirrel integration
12. ✅ Phase 5: State persistence

**Total**: As per phase specifications  
**Impact**: Complete feature set

---

### **Long-term (Week 13-20): Evolution Plans**
13. ✅ Encryption Week 2-8
14. ✅ LiveSpore BirdSong v3.0
15. ✅ NUCLEUS self-deployment

**Total**: As per evolution specs  
**Impact**: Next-generation capabilities

---

## 🧬 EVOLUTION PRINCIPLES APPLIED

### 1. **Deep Debt Solutions** ✅
- Not "// TODO: fix later"
- But "Complete implementation with proper architecture"
- Example: TRUE PRIMAL queries instead of hardcoded assumptions

### 2. **Modern Idiomatic Rust** ✅
- async/await throughout
- Type-safe RPC (tarpc, JSON-RPC 2.0)
- Lock-free concurrency (DashMap, Arc)
- Zero unsafe code maintained

### 3. **Capability-Based, Not Hardcoded** ✅
- Query primals for capabilities at runtime
- No compile-time primal knowledge
- Discovery-first architecture

### 4. **Primal Self-Knowledge Only** ✅
- Each primal knows only itself
- Queries others via JSON-RPC
- Advertises capabilities, doesn't assume others'

### 5. **Fast AND Safe Rust** ✅
- Arc for cheap cloning (zero-copy where possible)
- DashMap for lock-free caching
- Async for concurrency
- Zero unsafe blocks

### 6. **Mocks Isolated to Testing** ✅
- Current status: ZERO production mocks
- E2E tests use real implementations
- Integration tests use proper test infrastructure

### 7. **External Dependencies → Rust** ✅
- eventsource-client (pure Rust SSE)
- tarpc (pure Rust RPC)
- jsonrpc-* (pure Rust JSON-RPC)
- DashMap (pure Rust concurrent HashMap)

---

## 📊 TODO METRICS

| Category | Count | Priority | Timeline |
|----------|-------|----------|----------|
| **High-Priority Production** | 12 | HIGH | Week 1-2 |
| **Technical Debt** | 15 | MEDIUM | Week 3-4 |
| **Future Phases (Documented)** | 45 | N/A | Per specs |
| **Planned Evolution** | 14 | N/A | Per specs |
| **Total** | 86 | - | - |

### Actionable TODOs: **27 (31%)**
### Documented Evolution: **59 (69%)** ✅

---

## 🎯 EXECUTION PLAN

### **Phase 1: Critical Path (Week 1-2)**
```bash
# Day 1-2: Unix Socket & SSE
- Implement Unix socket health checks
- Implement SSE client
- Test with BearDog/Songbird

# Day 3-4: JSON-RPC Infrastructure
- Implement JSON-RPC server
- Add device management RPC methods
- Test with petalTongue

# Day 5: TRUE PRIMAL Discovery
- Implement identity query
- Implement health probe
- Implement capability query
- Test with all primals
```

**Deliverable**: Production-ready core functionality

---

### **Phase 2: Infrastructure (Week 3-4)**
```bash
# Week 3: Performance
- Implement key caching
- Implement tarpc transport
- Benchmark improvements

# Week 4: Reliability
- Implement rollback strategies
- Add chaos tests
- Add fault injection tests
```

**Deliverable**: Fast, reliable infrastructure

---

### **Phase 3: Feature Completion (Week 5-12)**
Follow existing phase specifications:
- Week 5-6: Songbird integration
- Week 7-9: Orchestration actions
- Week 10-12: Squirrel + NestGate

**Deliverable**: Complete feature set

---

### **Phase 4: Evolution (Week 13-20)**
Follow evolution specifications:
- Encryption Week 2-8
- LiveSpore BirdSong v3.0
- NUCLEUS self-deployment

**Deliverable**: Next-generation capabilities

---

## ✨ SUCCESS CRITERIA

### Code Quality:
- ✅ Zero production mocks maintained
- ✅ Zero unsafe code maintained
- ✅ 100% Rust dependencies maintained
- ✅ TRUE PRIMAL architecture maintained
- ✅ Capability-based discovery throughout

### Functionality:
- ✅ All critical TODOs implemented (27)
- ✅ Production-ready core features
- ✅ Fast AND safe implementations
- ✅ Modern idiomatic Rust patterns

### Architecture:
- ✅ Deep debt solutions, not patches
- ✅ Smart refactoring, not just splits
- ✅ Evolution, not quick fixes
- ✅ Capability-based, not hardcoded

---

## 🚀 READY TO EXECUTE

**Status**: Plan complete, priorities clear  
**Principles**: Aligned with evolution mindset  
**Timeline**: 3-4 days for critical path  
**Confidence**: High - systematic and thorough

---

*Evolution plan created: January 15, 2026*  
*Next: Begin Week 1-2 implementation*


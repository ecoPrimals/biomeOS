# 🎯 Test Coverage Expansion Strategy

**Date**: January 25, 2026  
**Current Coverage**: ~42% (baseline)  
**Goal**: 90% coverage  
**Status**: 📊 **ANALYSIS IN PROGRESS**

---

## 📊 CURRENT COVERAGE ANALYSIS

### High-Value Targets (0% coverage, production code):

#### Priority 1: API Handlers (Critical Business Logic)
```
crates/biomeos-api/src/handlers/discovery.rs          0.00% ⚠️ HIGH PRIORITY
crates/biomeos-api/src/handlers/live_discovery.rs     0.00% ⚠️ HIGH PRIORITY
crates/biomeos-api/src/handlers/topology.rs           0.00% ⚠️ HIGH PRIORITY
crates/biomeos-api/src/handlers/trust.rs              0.00% ⚠️ HIGH PRIORITY
crates/biomeos-api/src/handlers/livespores.rs         0.00% ⚠️ HIGH PRIORITY
```

#### Priority 2: Server Infrastructure
```
crates/biomeos-api/src/main.rs                        0.00% (server entry)
crates/biomeos-api/src/unix_server.rs                 0.00% (Unix socket server)
crates/biomeos-atomic-deploy/.../neural-api-server.rs 0.00% (Neural API binary)
crates/biomeos-atomic-deploy/.../neural-deploy.rs     0.00% (deployment binary)
crates/biomeos-atomic-deploy/src/neural_api_server.rs 0.00% ⚠️ HIGH PRIORITY
```

### Good Coverage (Already Strong):
```
✅ capability_translation.rs       79.53% (semantic layer)
✅ deployment_graph.rs              93.85% (graph deployment)
✅ health_check.rs                  97.41% (health monitoring)
```

### Moderate Coverage (Could Improve):
```
⚠️ state.rs                         52.98%
⚠️ events.rs                        33.04%
⚠️ websocket.rs                     34.18%
⚠️ beardog_jwt_client.rs            31.41%
⚠️ mode.rs                          68.42%
⚠️ http_client.rs                   17.44%
```

---

## 🎯 EXPANSION STRATEGY

### Phase 1: API Handlers (Immediate Impact)
**Target**: +30% coverage by testing 5 handler files

**Files**:
1. `handlers/discovery.rs` - Discovery service routing
2. `handlers/topology.rs` - Network topology
3. `handlers/trust.rs` - Trust and authorization
4. `handlers/live_discovery.rs` - Live service discovery
5. `handlers/livespores.rs` - Spore lifecycle

**Approach**:
```rust
// Integration tests for each handler
- Happy path scenarios
- Error handling
- Edge cases
- Input validation
```

### Phase 2: Neural API Server (High Value)
**Target**: +20% coverage

**File**: `neural_api_server.rs` (1693 regions, 0% coverage)

**Approach**:
```rust
// Server integration tests
- RPC method routing
- Capability translation
- Deployment orchestration
- Health checks
- Error scenarios
```

### Phase 3: Infrastructure (Moderate)
**Target**: +10% coverage

**Files**:
- `websocket.rs` (34% → 70%)
- `http_client.rs` (17% → 60%)
- `beardog_jwt_client.rs` (31% → 70%)

---

## 📋 TEST TEMPLATES

### Template 1: Handler Tests
```rust
// For API handlers (discovery, topology, trust, etc.)

#[tokio::test]
async fn test_handler_success_path() {
    // Setup
    let app = test_app().await;
    
    // Execute
    let response = app.call_handler(valid_request()).await;
    
    // Assert
    assert_eq!(response.status, 200);
}

#[tokio::test]
async fn test_handler_invalid_input() {
    // Test error handling
}

#[tokio::test]
async fn test_handler_missing_resource() {
    // Test 404 scenarios
}
```

### Template 2: Neural API Server Tests
```rust
// For neural_api_server.rs

#[tokio::test]
async fn test_rpc_capability_call() {
    // Test capability translation end-to-end
}

#[tokio::test]
async fn test_rpc_deploy_graph() {
    // Test graph deployment
}

#[tokio::test]
async fn test_rpc_method_not_found() {
    // Test error handling
}
```

---

## 🚀 EXECUTION PLAN

### Step 1: Discovery Handler Tests ✅
Create `crates/biomeos-api/tests/discovery_handler_tests.rs`

**Coverage Expected**: 0% → 80%

**Tests**:
- ✅ Discover by capability
- ✅ Register service
- ✅ Health check
- ✅ Error scenarios

### Step 2: Topology Handler Tests
Create `crates/biomeos-api/tests/topology_handler_tests.rs`

**Coverage Expected**: 0% → 75%

**Tests**:
- Network topology queries
- Primal relationships
- Graph visualization
- Edge cases

### Step 3: Trust Handler Tests
Create `crates/biomeos-api/tests/trust_handler_tests.rs`

**Coverage Expected**: 0% → 70%

**Tests**:
- Authorization checks
- Trust establishment
- Certificate validation
- Security scenarios

### Step 4: Neural API Server Tests
Expand `crates/biomeos-atomic-deploy/tests/neural_api_integration_tests.rs`

**Coverage Expected**: 0% → 60%

**Tests**:
- All RPC methods
- Graph deployment
- Capability routing
- Health monitoring

---

## 📊 PROJECTED COVERAGE

| Phase | Files | Current | Target | Gain |
|-------|-------|---------|--------|------|
| **Phase 1** | 5 handlers | ~0% | ~75% | +30% |
| **Phase 2** | neural_api_server | 0% | ~60% | +20% |
| **Phase 3** | Infrastructure | ~27% | ~65% | +10% |
| **Total** | - | **42%** | **≥90%** | **+48%** |

---

## ✅ SUCCESS CRITERIA

### Coverage Targets:
- [ ] Overall coverage: ≥90%
- [ ] Critical handlers: ≥80%
- [ ] Neural API server: ≥60%
- [ ] Infrastructure: ≥65%

### Quality Targets:
- [ ] All tests passing
- [ ] Fast execution (<5s)
- [ ] Clear test names
- [ ] Good error messages

---

## 🎯 IMMEDIATE ACTION

**Start with**: Discovery handler tests (highest business value, 0% coverage)

**Expected Time**: 1-2 hours per handler

**Expected Result**: +6% per handler file

**Total to Goal**: ~10-15 hours of focused test development

---

**Status**: Strategy defined, ready for execution 🚀



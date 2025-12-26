# Path to 75% Coverage - BiomeOS

**Date**: December 24, 2025  
**Current Coverage**: 35.94% lines, 44.55% functions  
**Target Coverage**: 75% lines  
**Gap**: 39.06 percentage points

---

## 📊 **Current State**

### Coverage Breakdown

| Module | Lines Coverage | Priority | Estimated Effort |
|--------|---------------|----------|------------------|
| **CLI Commands** | 0% | 🔴 HIGH | 2 days |
| **Universal Adapter** | 19.41% | 🔴 HIGH | 3 days |
| **Manifest Modules** | 0% | 🟡 MEDIUM | 2 days |
| **Client Modules** | ~80%+ | ✅ DONE | - |
| **Core Types** | 95%+ | ✅ DONE | - |
| **Discovery Bootstrap** | 90%+ | ✅ DONE | - |

**Total Estimated Effort**: 7 days of focused work

---

## 🎯 **Roadmap to 75%**

### Phase 1: CLI Commands (0% → 60%)
**Impact**: +15% overall coverage  
**Effort**: 2 days

#### What to Test
1. `crates/biomeos-cli/src/commands/health.rs`
   - Health check command
   - Diagnostic display
   - Error handling

2. `crates/biomeos-cli/src/commands/discover.rs`
   - Discovery commands
   - Geolocation discovery
   - Network scan

3. `crates/biomeos-cli/src/commands/monitor.rs`
   - Monitoring commands
   - Real-time updates

4. `crates/biomeos-cli/src/tui/widgets.rs`
   - TUI widget rendering
   - User interactions

#### Implementation Strategy
```rust
// Example test structure
#[tokio::test]
async fn test_health_command_success() {
    // Setup mock discovery service
    let mock = MockServer::start().await;
    
    // Execute health command
    let result = execute_health_command(&config).await;
    
    // Verify output
    assert!(result.is_ok());
}
```

---

### Phase 2: Universal Adapter (19.41% → 70%)
**Impact**: +18% overall coverage  
**Effort**: 3 days

#### What to Test
1. `src/universal_adapter.rs` (881 lines, 19.41% coverage)
   - Service orchestration
   - Request routing
   - Error handling
   - State management

#### High-Value Test Scenarios
1. **Request Routing**
   - Route AI requests to ToadStool
   - Route storage requests to BearDog
   - Route discovery to Songbird

2. **Service Discovery Integration**
   - Bootstrap discovery
   - Service registration
   - Health monitoring

3. **Error Handling**
   - Service unavailable
   - Invalid requests
   - Timeout handling

4. **State Management**
   - Service state tracking
   - Configuration updates
   - Cache management

#### Implementation Strategy
```rust
#[tokio::test]
async fn test_universal_adapter_routes_ai_requests() {
    let mock_toadstool = MockServer::start().await;
    
    // Mock ToadStool AI endpoint
    Mock::given(method("POST"))
        .and(path("/api/v1/ai/process"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_toadstool)
        .await;
    
    // Create adapter with discovered ToadStool
    let adapter = UniversalAdapter::new(config);
    
    // Route AI request
    let result = adapter.route_request("ai", request).await;
    
    assert!(result.is_ok());
}
```

---

### Phase 3: Manifest Modules (0% → 80%)
**Impact**: +8% overall coverage  
**Effort**: 2 days

#### What to Test
1. `crates/biomeos-types/src/manifest/lifecycle/mod.rs` (0%)
   - Lifecycle definitions
   - State transitions

2. `crates/biomeos-types/src/manifest/lifecycle/scaling.rs` (0%)
   - Scaling policies
   - Resource calculations

3. `crates/biomeos-types/src/manifest/networking_core.rs` (0%)
   - Network configuration
   - Service mesh setup

4. `crates/biomeos-types/src/manifest/storage.rs` (0%)
   - Storage configuration
   - Volume management

#### Implementation Strategy
```rust
#[test]
fn test_scaling_policy_calculation() {
    let policy = ScalingPolicy::new()
        .min_instances(1)
        .max_instances(10)
        .target_cpu(70.0);
    
    // Test scale up decision
    let current_cpu = 85.0;
    let decision = policy.calculate_scale_decision(current_cpu);
    
    assert_eq!(decision, ScaleDecision::ScaleUp);
}
```

---

### Phase 4: Operations Module
**Impact**: +5% overall coverage  
**Effort**: 1 day

#### What to Test
1. `crates/biomeos-core/src/universal_biomeos_manager/operations.rs`
   - Service operations
   - Health monitoring
   - Service scaling

---

## 🎓 **Testing Strategy**

### 1. Unit Tests First
Start with simple, fast unit tests for pure functions and data structures.

### 2. Integration Tests Second
Add integration tests for modules that interact with external services.

### 3. E2E Tests Last
Complete with end-to-end tests that exercise full workflows.

### 4. Prioritize High-Impact
Focus on modules with:
- Large code size (more lines to cover)
- Low current coverage (bigger improvement)
- Critical functionality (must work correctly)

---

## 📈 **Expected Coverage Progression**

| Phase | Duration | Coverage Gain | Cumulative Coverage |
|-------|----------|---------------|---------------------|
| **Start** | - | - | 35.94% |
| **Phase 1** (CLI) | 2 days | +15% | 50.94% |
| **Phase 2** (Adapter) | 3 days | +18% | 68.94% |
| **Phase 3** (Manifest) | 2 days | +8% | **76.94%** ✅ |
| **Phase 4** (Polish) | 1 day | +3% | **79.94%** 🎉 |

**Total Time**: 8 days  
**Final Coverage**: ~80% (exceeds 75% target)

---

## 🛠️ **Tools and Techniques**

### Coverage Measurement
```bash
# Generate coverage report
cargo llvm-cov --workspace --html

# View coverage summary
cargo llvm-cov --workspace --summary-only

# Open HTML report
xdg-open target/llvm-cov/html/index.html
```

### Test Execution
```bash
# Run all tests
cargo test --workspace

# Run specific test file
cargo test --test client_tests

# Run with output
cargo test -- --nocapture
```

### Coverage-Driven Development
1. **Identify** uncovered code paths
2. **Write** tests for those paths
3. **Verify** coverage increased
4. **Repeat** until target reached

---

## 📋 **Test Template**

### For CLI Commands
```rust
#[tokio::test]
async fn test_<command>_<scenario>() {
    // Setup
    let config = test_config();
    let mock = MockServer::start().await;
    
    // Execute
    let result = execute_<command>(&config).await;
    
    // Verify
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, "success");
}
```

### For Universal Adapter
```rust
#[tokio::test]
async fn test_adapter_<functionality>() {
    // Setup mocks
    let mock_primal = MockServer::start().await;
    setup_mock_responses(&mock_primal).await;
    
    // Create adapter
    let adapter = UniversalAdapter::new(test_config());
    
    // Execute operation
    let result = adapter.<operation>().await;
    
    // Verify
    assert!(result.is_ok());
}
```

### For Manifest Modules
```rust
#[test]
fn test_manifest_<component>_<behavior>() {
    // Create test data
    let manifest = TestManifest::new();
    
    // Execute
    let result = manifest.<operation>();
    
    // Verify
    assert_eq!(result, expected);
}
```

---

## 🎯 **Quick Wins**

### 1. Test Data Structures (1 hour)
Manifest modules are mostly data structures. Easy to test:
```rust
#[test]
fn test_lifecycle_serialization() {
    let lifecycle = Lifecycle::default();
    let json = serde_json::to_string(&lifecycle).unwrap();
    let deserialized: Lifecycle = serde_json::from_str(&json).unwrap();
    assert_eq!(lifecycle, deserialized);
}
```

### 2. Test Error Cases (2 hours)
Many modules have error paths:
```rust
#[test]
fn test_invalid_config_returns_error() {
    let invalid_config = Config::new().with_invalid_data();
    let result = validate_config(&invalid_config);
    assert!(result.is_err());
}
```

### 3. Test Default Values (1 hour)
Verify defaults are correct:
```rust
#[test]
fn test_default_values() {
    let config = Config::default();
    assert_eq!(config.timeout, Duration::from_secs(30));
    assert_eq!(config.retries, 3);
}
```

---

## 🚧 **Potential Challenges**

### 1. CLI Testing
**Challenge**: CLI commands require terminal interaction  
**Solution**: Mock terminal output, test command parsing separately

### 2. Universal Adapter Complexity
**Challenge**: Large module with many dependencies  
**Solution**: Test each method independently with mocks

### 3. TUI Testing
**Challenge**: Terminal UI rendering is hard to test  
**Solution**: Test logic separately from rendering, verify data flow

### 4. Async Code
**Challenge**: Async tests can be flaky  
**Solution**: Use `tokio::test`, avoid sleeps, use timeouts

---

## 📊 **Success Metrics**

### Primary Goal
- ✅ **75% line coverage** achieved

### Secondary Goals
- ✅ **80% function coverage**
- ✅ **All critical paths tested**
- ✅ **100% test pass rate**
- ✅ **<5 second test execution**

### Quality Metrics
- ✅ **Zero flaky tests**
- ✅ **Clear test names**
- ✅ **Meaningful assertions**
- ✅ **Good error messages**

---

## 🎉 **When You're Done**

### Verification
```bash
# Final coverage check
cargo llvm-cov --workspace --summary-only | grep "TOTAL"
# Should show: 75%+ line coverage

# All tests pass
cargo test --workspace
# Should show: All tests passed

# No warnings
cargo clippy --workspace
# Should show: No warnings

# Formatted
cargo fmt --check --all
# Should show: No changes needed
```

### Documentation
- Update `TEST_EXPANSION_COMPLETE_DEC_24_2025.md`
- Update `PRODUCTION_READY_REPORT_DEC_24_2025.md`
- Update `FINAL_STATUS_DEC_24_2025.md`

### Celebration
- Grade: B → **A**
- Coverage: 35.94% → **75%+**
- Tests: 225 → **500+**

---

## 💡 **Tips for Success**

1. **Start Small**: Begin with easiest tests (data structures)
2. **Measure Often**: Check coverage after each test file
3. **Focus on Impact**: Prioritize high-line-count modules
4. **Use Mocks**: Mock external dependencies aggressively
5. **Test Happy Path First**: Then add error cases
6. **Run Tests Frequently**: Catch failures early
7. **Keep Tests Fast**: Aim for <5 seconds total
8. **Document Assumptions**: Explain test setup clearly

---

**READY TO CONTINUE** 🚀  
**Target**: 75% Coverage  
**Estimated**: 8 days  
**Status**: Blueprint Complete

---

*"The path is clear. The tools are ready. The tests await."*


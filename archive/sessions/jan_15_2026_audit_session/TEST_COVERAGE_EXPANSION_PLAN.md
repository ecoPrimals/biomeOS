# 🧪 Test Coverage Expansion Plan (60% → 90%)

**Date**: January 15, 2026  
**Current**: ~60% (estimated from VALIDATION_GOALS.md)  
**Target**: 90% line coverage  
**Gap**: 30% additional coverage needed  
**Timeline**: 2-3 weeks systematic expansion

---

## 📊 CURRENT STATE

### Coverage Breakdown (Estimated):
```
Module                        Current    Target    Gap
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
biomeos-core                    65%       90%      25%
biomeos-graph                   70%       90%      20%
biomeos-types                   80%       90%      10%
biomeos-spore                   55%       90%      35%
biomeos-federation              50%       90%      40%
biomeos-ui                      45%       90%      45%
biomeos-atomic-deploy           60%       90%      30%
biomeos-boot                    40%       90%      50%
biomeos-cli                     75%       90%      15%
biomeos-nucleus                 60%       90%      30%
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
OVERALL                         60%       90%      30%
```

### Test Types:
- ✅ **Unit Tests**: 2,200+ tests (good coverage)
- ✅ **Integration Tests**: 800+ tests (good coverage)
- ⚠️ **E2E Tests**: 194 files, but many skipped/disabled
- ⚠️ **Chaos Tests**: Present but limited
- ⚠️ **Fault Injection**: Present but limited

---

## 🎯 PRIORITY MODULES (Security & Critical Path)

### Priority 1: Security Code (Weeks 1-2)
**Target**: 95% coverage (higher than normal for security)

#### **1.1 Encryption (biomeos-core/src/encrypted_storage/)**
```rust
// Current: ~65%
// Target: 95%
// Gap: 30%

// Missing Coverage:
- Error paths in key derivation
- Edge cases in metadata encryption
- Concurrent access patterns
- Key rotation scenarios
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_key_derivation_with_invalid_seed() {
    // Test error handling
}

#[tokio::test]
async fn test_concurrent_encryption_operations() {
    // Test thread safety
}

#[tokio::test]
async fn test_key_rotation_during_operation() {
    // Test edge case
}

#[tokio::test]
async fn test_metadata_encryption_roundtrip() {
    // Test complete flow
}
```

**Estimated**: 20-30 new tests, 8-12 hours

---

#### **1.2 Genetic Lineage (biomeos-federation/)**
```rust
// Current: ~50%
// Target: 95%
// Gap: 45%

// Missing Coverage:
- Multi-family verification
- Invalid lineage detection
- Concurrent lineage checks
- BearDog integration errors
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_invalid_lineage_rejection() {
    // Test security boundary
}

#[tokio::test]
async fn test_multi_family_verification() {
    // Test complex scenarios
}

#[tokio::test]
async fn test_beardog_unavailable_fallback() {
    // Test error handling
}
```

**Estimated**: 25-35 new tests, 10-14 hours

---

### Priority 2: Orchestration Code (Week 2)
**Target**: 90% coverage

#### **2.1 Graph Execution (biomeos-graph/src/executor.rs)**
```rust
// Current: ~70%
// Target: 90%
// Gap: 20%

// Missing Coverage:
- Complex dependency graphs
- Circular dependency detection
- Timeout scenarios
- Retry logic edge cases
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_circular_dependency_detection() {
    // Test error case
}

#[tokio::test]
async fn test_graph_execution_timeout() {
    // Test timeout handling
}

#[tokio::test]
async fn test_partial_graph_failure_recovery() {
    // Test resilience
}
```

**Estimated**: 15-20 new tests, 6-8 hours

---

#### **2.2 Neural Executor (biomeos-atomic-deploy/src/neural_executor.rs)**
```rust
// Current: ~60%
// Target: 90%
// Gap: 30%

// Missing Coverage:
- Health check failures
- Rollback scenarios
- Primal startup errors
- Network partition scenarios
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_primal_startup_failure_rollback() {
    // Test rollback logic
}

#[tokio::test]
async fn test_health_check_timeout_handling() {
    // Test timeout paths
}

#[tokio::test]
async fn test_network_partition_recovery() {
    // Test resilience
}
```

**Estimated**: 20-25 new tests, 8-10 hours

---

### Priority 3: Discovery & Federation (Week 3)
**Target**: 90% coverage

#### **3.1 NUCLEUS Discovery (biomeos-nucleus/)**
```rust
// Current: ~60%
// Target: 90%
// Gap: 30%

// Missing Coverage:
- Multi-layer discovery paths
- Discovery failures and retries
- Malformed discovery packets
- Trust establishment edge cases
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_5_layer_discovery_sequence() {
    // Test complete flow
}

#[tokio::test]
async fn test_discovery_packet_corruption() {
    // Test error handling
}

#[tokio::test]
async fn test_trust_establishment_timeout() {
    // Test timeout handling
}
```

**Estimated**: 20-25 new tests, 8-10 hours

---

### Priority 4: UI & API (Week 3)
**Target**: 90% coverage

#### **4.1 Interactive UI Orchestrator (biomeos-ui/src/orchestrator.rs)**
```rust
// Current: ~45%
// Target: 90%
// Gap: 45%

// Missing Coverage:
- Device assignment flows
- Authorization checks
- State persistence
- Error propagation
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_device_assignment_authorization_failure() {
    // Test security boundary
}

#[tokio::test]
async fn test_state_persistence_and_recovery() {
    // Test reliability
}

#[tokio::test]
async fn test_concurrent_device_assignments() {
    // Test concurrency
}
```

**Estimated**: 30-40 new tests, 12-16 hours

---

### Priority 5: LiveSpore & Boot (Week 4)
**Target**: 90% coverage

#### **5.1 Spore Management (biomeos-spore/)**
```rust
// Current: ~55%
// Target: 90%
// Gap: 35%

// Missing Coverage:
- Incubation edge cases
- USB device detection errors
- Manifest verification failures
- Genetic seed validation
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_usb_device_removal_during_incubation() {
    // Test fault tolerance
}

#[tokio::test]
async fn test_invalid_manifest_rejection() {
    // Test validation
}

#[tokio::test]
async fn test_genetic_seed_mismatch_detection() {
    // Test security
}
```

**Estimated**: 25-30 new tests, 10-12 hours

---

#### **5.2 Boot System (biomeos-boot/)**
```rust
// Current: ~40%
// Target**: 90%
// Gap: 50%

// Missing Coverage:
- Boot failure scenarios
- Hardware initialization errors
- Filesystem mount failures
- Network initialization edge cases
```

**Tests to Add**:
```rust
#[tokio::test]
async fn test_boot_failure_recovery() {
    // Test resilience
}

#[tokio::test]
async fn test_hardware_initialization_timeout() {
    // Test timeout handling
}

#[tokio::test]
async fn test_filesystem_mount_failure_fallback() {
    // Test error handling
}
```

**Estimated**: 35-45 new tests, 14-18 hours

---

## 🧪 TEST CATEGORIES TO EXPAND

### 1. **Error Path Testing** (Highest Priority)
**Current Coverage**: ~30%  
**Target**: 90%

**Strategy**:
- Test every `Result::Err` path
- Test every timeout scenario
- Test every validation failure
- Test every network error

**Example**:
```rust
#[tokio::test]
async fn test_network_timeout_handling() {
    let client = create_test_client_with_timeout(Duration::from_millis(1));
    let result = client.call_method("slow_method", json!({})).await;
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::Timeout));
}
```

---

### 2. **Edge Case Testing**
**Current Coverage**: ~40%  
**Target**: 90%

**Strategy**:
- Test boundary conditions
- Test empty inputs
- Test maximum values
- Test invalid combinations

**Example**:
```rust
#[tokio::test]
async fn test_empty_graph_execution() {
    let graph = Graph::new();
    let result = executor.execute(graph).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().nodes_executed, 0);
}
```

---

### 3. **Concurrency Testing**
**Current Coverage**: ~50%  
**Target**: 90%

**Strategy**:
- Test concurrent operations
- Test race conditions
- Test deadlock scenarios
- Test resource contention

**Example**:
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_concurrent_encryption_operations() {
    let backend = EncryptionBackend::new();
    
    let handles: Vec<_> = (0..100).map(|i| {
        let backend = backend.clone();
        tokio::spawn(async move {
            backend.encrypt(&format!("data_{}", i)).await
        })
    }).collect();
    
    for handle in handles {
        assert!(handle.await.unwrap().is_ok());
    }
}
```

---

### 4. **Integration Testing**
**Current Coverage**: ~60%  
**Target**: 90%

**Strategy**:
- Test cross-module interactions
- Test primal-to-primal communication
- Test full workflows
- Test state persistence

**Example**:
```rust
#[tokio::test]
async fn test_full_discovery_and_encryption_workflow() {
    // 1. Start BearDog
    let beardog = start_test_beardog().await?;
    
    // 2. Start Songbird (discovers BearDog)
    let songbird = start_test_songbird().await?;
    
    // 3. Verify encrypted discovery
    let discovered = songbird.discover_primals().await?;
    assert!(discovered.contains(&beardog.identity()));
    
    // 4. Verify auto-trust
    let trust = songbird.check_trust(&beardog.identity()).await?;
    assert!(trust.is_trusted);
}
```

---

### 5. **Chaos & Fault Injection Testing**
**Current Coverage**: ~20%  
**Target**: 60% (lower target, specialized tests)

**Strategy**:
- Random primal failures
- Network partitions
- Disk full scenarios
- Memory pressure

**Example**:
```rust
#[tokio::test]
async fn test_chaos_random_primal_failures() {
    let mut primals = start_test_nucleus().await?;
    
    for _ in 0..10 {
        // Randomly kill a primal
        let victim = primals.choose_random();
        victim.kill().await?;
        
        // Verify system continues functioning
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert!(system_is_healthy(&primals).await?);
        
        // Restart victim
        victim.restart().await?;
    }
}
```

---

## 📋 WEEKLY BREAKDOWN

### **Week 1: Security & Critical Path**
**Monday-Tuesday**: Encryption tests (8-12h)  
**Wednesday-Thursday**: Genetic lineage tests (10-14h)  
**Friday**: Graph execution tests (6-8h)

**Target**: +15% coverage (60% → 75%)  
**Tests Added**: ~60-85 tests  
**Hours**: 24-34 hours

---

### **Week 2: Orchestration & Discovery**
**Monday-Tuesday**: Neural executor tests (8-10h)  
**Wednesday-Thursday**: NUCLEUS discovery tests (8-10h)  
**Friday**: Integration tests (6-8h)

**Target**: +10% coverage (75% → 85%)  
**Tests Added**: ~55-70 tests  
**Hours**: 22-28 hours

---

### **Week 3: UI & Federation**
**Monday-Wednesday**: UI orchestrator tests (12-16h)  
**Thursday-Friday**: Error path expansion (8-12h)

**Target**: +5% coverage (85% → 90%)  
**Tests Added**: ~45-60 tests  
**Hours**: 20-28 hours

---

### **Week 4: Spore & Boot (Optional, if needed)**
**Monday-Tuesday**: Spore management tests (10-12h)  
**Wednesday-Thursday**: Boot system tests (14-18h)  
**Friday**: Polish and verification (4-6h)

**Target**: Maintain 90%+  
**Tests Added**: ~60-75 tests  
**Hours**: 28-36 hours

---

## 🎯 SUCCESS METRICS

### Coverage Targets:
- ✅ **Overall**: 90% line coverage
- ✅ **Security Modules**: 95% coverage
- ✅ **Critical Path**: 90% coverage
- ✅ **Error Paths**: 90% coverage
- ✅ **Edge Cases**: 90% coverage

### Quality Metrics:
- ✅ **Zero Flaky Tests**: All tests deterministic
- ✅ **Fast Execution**: Test suite < 5 minutes
- ✅ **Clear Assertions**: Each test has clear pass/fail
- ✅ **Isolated Tests**: No test dependencies

---

## 🛠️ TOOLS & INFRASTRUCTURE

### Coverage Measurement:
```bash
# Generate baseline
cargo llvm-cov --workspace --lib --lcov --output-path lcov-baseline.info

# Generate HTML report
cargo llvm-cov --workspace --html --open

# Check specific module
cargo llvm-cov --package biomeos-core --html --open
```

### Test Execution:
```bash
# Run all tests
cargo test --workspace --all-features

# Run specific module
cargo test --package biomeos-core --all-features

# Run with coverage
cargo llvm-cov --workspace --all-features
```

### CI Integration:
```yaml
# .github/workflows/coverage.yml
name: Coverage
on: [push, pull_request]
jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install cargo-llvm-cov
      - run: cargo llvm-cov --workspace --lcov --output-path lcov.info
      - run: |
          COVERAGE=$(cargo llvm-cov --workspace --summary-only | grep -oP 'TOTAL.*\K\d+\.\d+')
          if (( $(echo "$COVERAGE < 90.0" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 90% threshold"
            exit 1
          fi
```

---

## ✨ TEST QUALITY PRINCIPLES

### 1. **Deterministic**
```rust
// ❌ BAD: Flaky test
#[tokio::test]
async fn test_flaky() {
    tokio::time::sleep(Duration::from_millis(100)).await;
    // Race condition - sometimes fails
}

// ✅ GOOD: Deterministic test
#[tokio::test]
async fn test_deterministic() {
    let (tx, rx) = oneshot::channel();
    tokio::spawn(async move { tx.send(42).unwrap(); });
    assert_eq!(rx.await.unwrap(), 42); // Always works
}
```

### 2. **Isolated**
```rust
// ❌ BAD: Test depends on global state
#[tokio::test]
async fn test_dependent() {
    GLOBAL_STATE.lock().unwrap().set(42);
    // Other tests might interfere
}

// ✅ GOOD: Isolated test
#[tokio::test]
async fn test_isolated() {
    let state = State::new();
    state.set(42);
    assert_eq!(state.get(), 42);
}
```

### 3. **Clear Assertions**
```rust
// ❌ BAD: Unclear what's being tested
#[tokio::test]
async fn test_unclear() {
    let result = do_something().await;
    assert!(result.is_ok());
}

// ✅ GOOD: Clear assertion
#[tokio::test]
async fn test_clear() {
    let result = encrypt_data("test").await;
    assert!(result.is_ok(), "Encryption should succeed for valid input");
    assert_eq!(result.unwrap().len(), 32, "Encrypted data should be 32 bytes");
}
```

### 4. **Fast Execution**
```rust
// ❌ BAD: Slow test
#[tokio::test]
async fn test_slow() {
    for i in 0..1000 {
        heavy_operation(i).await; // Takes 5 minutes
    }
}

// ✅ GOOD: Fast test
#[tokio::test]
async fn test_fast() {
    let result = heavy_operation(42).await; // Test one case
    assert!(result.is_ok());
    // Property-based testing for bulk validation
}
```

---

## 🚀 READY TO EXECUTE

**Status**: Plan complete, tools ready  
**Timeline**: 2-3 weeks (66-118 hours)  
**Tests to Add**: ~220-290 tests  
**Coverage Gain**: +30% (60% → 90%)  
**Confidence**: High - systematic and thorough

---

**Next Step**: Begin Week 1 - Security & Critical Path testing

*Test coverage expansion plan created: January 15, 2026*


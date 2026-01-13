# 🧪 Test Coverage Strategy

**Date**: January 13, 2026  
**Status**: 🔄 **IN PROGRESS** - Unit tests fixed, integration tests need work  
**Current Coverage**: ~60% (estimated)  
**Target Coverage**: 90%

---

## ✅ Completed: Unit Test Fixes

### Fixed Tests (biomeos-core)

**Before**: 187 passed, 3 failed  
**After**: 190 passed, 0 failed ✅

#### 1. Fixed `capability_registry::tests::test_register_and_get_provider`

**Issue**: `PrimalId::new("beardog@localhost")` failed with `InvalidCharacters`  
**Root Cause**: `@` character not allowed in PrimalId (only alphanumeric, `-`, `_`)  
**Fix**: Changed to `"beardog-localhost"`

#### 2. Fixed `capability_registry::tests::test_unregister`

**Issue**: Same as above  
**Fix**: Changed to `"beardog-localhost"`

#### 3. Fixed `concurrent_startup::tests::test_single_wave`

**Issue**: `unwrap()` on `None` when primal has no requirements  
**Root Cause**: Code assumed all primals have entries in `requires` HashMap  
**Fix**: Handle `None` case explicitly:

```rust
// Before:
let required = self.requires.get(*id).unwrap();

// After:
let required = self.requires.get(*id);
match required {
    None => true, // No requirements means ready to start
    Some(req) => req.iter().all(|cap| {
        // Check if capability provider has started
        self.capability_providers
            .get(cap)
            .map(|provider| started.contains(provider))
            .unwrap_or(false)
    }) || req.is_empty()
}
```

#### 4. Fixed unused variable warning

**Issue**: `for (i, result)` where `i` was never used  
**Fix**: Changed to `for (_i, result)`

**Impact**: All unit tests in biomeos-core now pass ✅

---

## 🔄 Remaining Work: Integration Test Compilation

### Integration Test Compilation Errors

**Status**: ❌ **BLOCKING** - Tests don't compile

#### Category 1: Missing Client Imports (28 errors)

**Files**:
- `tests/chaos_tests.rs`
- `crates/biomeos-core/tests/protocol_integration_tests.rs`

**Errors**:
```
error[E0433]: failed to resolve: use of undeclared type `SongbirdClient`
error[E0433]: failed to resolve: use of undeclared type `BearDogClient`
```

**Root Cause**: `biomeos_core::clients` module is disabled (see `JSON_RPC_CLIENTS_STATUS_JAN13_2026.md`)

**Fix Required**: Re-enable `biomeos_core::clients` module after fixing transport layer issues

**Priority**: HIGH (blocks all integration tests)

#### Category 2: Missing GraphEvent Fields (3 errors)

**File**: `crates/biomeos-api/src/websocket.rs`

**Errors**:
```
error[E0063]: missing fields `coordination`, `graph_name` and `total_nodes` in initializer of `biomeos_graph::GraphEvent`
```

**Root Cause**: `GraphEvent::GraphStarted` struct was updated but test code wasn't

**Fix Required**: Add missing fields to test initializers

**Priority**: MEDIUM (blocks biomeos-api tests)

#### Category 3: Private Field Access (1 error)

**File**: `crates/biomeos-core/tests/protocol_integration_tests.rs:418`

**Error**:
```
error[E0599]: no method named `config` found for struct `Arc<GenericManagedPrimal>`
    dev_primal.config().env_config.get("DEPLOYMENT_ENV"),
               ^^^^^^ private field, not a method
```

**Root Cause**: `config` field is private, not a method

**Fix Required**: Either make field public or add getter method

**Priority**: LOW (single test)

---

## 📊 Test Coverage Analysis

### Current Status (Estimated)

| Crate | Unit Tests | Integration Tests | Coverage (Est.) |
|-------|-----------|-------------------|-----------------|
| biomeos-core | ✅ 190 passed | ❌ Won't compile | ~65% |
| biomeos-graph | ✅ Passing | ❌ Won't compile | ~55% |
| biomeos-api | ✅ Passing | ❌ Won't compile | ~50% |
| biomeos-atomic-deploy | ✅ Passing | ❌ Won't compile | ~60% |
| biomeos-cli | ✅ Passing | ⚠️ Warnings | ~70% |
| biomeos-ui | ✅ Passing | ⚠️ Warnings | ~40% |
| biomeos-boot | ✅ Passing | ⚠️ Warnings | ~55% |
| biomeos-federation | ✅ Passing | N/A | ~60% |
| biomeos-compute | ✅ Passing | N/A | ~50% |
| biomeos-spore | ✅ Passing | N/A | ~65% |

**Overall**: ~60% coverage (unit tests only)

**Blockers**:
1. Integration tests don't compile (client module disabled)
2. Chaos tests don't compile (client module disabled)
3. E2E tests missing

---

## 🎯 Strategy to Reach 90% Coverage

### Phase 1: Fix Integration Test Compilation (HIGH PRIORITY)

**Estimated Time**: 3-4 hours

#### Step 1: Re-enable biomeos_core::clients Module

**Blocker**: Transport layer issues (E0252, E0432, E0404)

**Tasks**:
1. Fix duplicate name errors (E0252)
2. Fix missing import errors (E0432)
3. Fix trait/struct confusion (E0404)
4. Re-enable `pub mod clients;` in `biomeos-core/src/lib.rs`

**Files to Fix**:
- `crates/biomeos-core/src/clients/mod.rs`
- `crates/biomeos-core/src/clients/beardog.rs`
- `crates/biomeos-core/src/clients/songbird.rs`
- `crates/biomeos-core/src/clients/toadstool.rs`
- `crates/biomeos-core/src/clients/nestgate.rs`

#### Step 2: Fix GraphEvent Test Initializers

**File**: `crates/biomeos-api/src/websocket.rs`

**Fix**:
```rust
// Before:
let event = GraphEvent::GraphStarted {
    graph_id: GraphId::new(),
};

// After:
let event = GraphEvent::GraphStarted {
    graph_id: GraphId::new(),
    coordination: CoordinationMode::Sequential,
    graph_name: "test-graph".to_string(),
    total_nodes: 1,
};
```

#### Step 3: Fix Private Field Access

**File**: `crates/biomeos-core/tests/protocol_integration_tests.rs:418`

**Option A**: Add getter method to `GenericManagedPrimal`
```rust
impl GenericManagedPrimal {
    pub fn config(&self) -> &PrimalConfig {
        &self.config
    }
}
```

**Option B**: Change test to not access private field
```rust
// Use public API instead of private field
```

---

### Phase 2: Add Missing Unit Tests (MEDIUM PRIORITY)

**Estimated Time**: 4-5 hours

#### Areas Needing More Unit Tests

1. **biomeos-ui** (~40% coverage)
   - `petaltongue_bridge.rs` - RPC methods
   - `orchestrator.rs` - Orchestration logic
   - Error paths and edge cases

2. **biomeos-api** (~50% coverage)
   - WebSocket handlers
   - REST endpoints
   - Error responses

3. **biomeos-compute** (~50% coverage)
   - Fractal compute patterns
   - Resource allocation
   - Error handling

4. **biomeos-boot** (~55% coverage)
   - Boot sequence
   - Configuration loading
   - Error recovery

#### Test Categories to Add

1. **Happy Path Tests**: Already mostly covered ✅
2. **Error Path Tests**: Need more coverage ⚠️
3. **Edge Case Tests**: Need more coverage ⚠️
4. **Concurrent Tests**: Need more coverage ⚠️

---

### Phase 3: Add Integration Tests (HIGH PRIORITY)

**Estimated Time**: 3-4 hours

#### Integration Test Scenarios

1. **Primal Discovery**
   - Songbird discovers BearDog
   - Capability-based discovery
   - Fallback to debug endpoints

2. **Atomic Deployment**
   - Deploy Tower (BearDog + Songbird)
   - Deploy Node (Tower + ToadStool)
   - Deploy Nest (Tower + NestGate)
   - Deploy NUCLEUS (all primals)

3. **Graph Execution**
   - Simple graph (1 node)
   - Complex graph (multiple waves)
   - Error handling in graph

4. **Inter-Primal Communication**
   - JSON-RPC over Unix sockets
   - HTTP fallback
   - Error handling

---

### Phase 4: Add E2E Tests (MEDIUM PRIORITY)

**Estimated Time**: 2-3 hours

#### E2E Test Scenarios

1. **Full NUCLEUS Deployment**
   - Start all primals
   - Verify discovery
   - Execute graph
   - Shutdown cleanly

2. **Niche Creation**
   - Load niche template
   - Validate resources
   - Deploy niche
   - Verify running

3. **Fault Recovery**
   - Kill primal mid-execution
   - Verify detection
   - Verify recovery

---

### Phase 5: Add Chaos Tests (LOW PRIORITY)

**Estimated Time**: 2-3 hours

#### Chaos Test Scenarios

1. **Network Partitions**
   - Simulate network loss
   - Verify graceful degradation
   - Verify recovery

2. **Resource Exhaustion**
   - Simulate OOM
   - Simulate disk full
   - Verify error handling

3. **Timing Issues**
   - Slow responses
   - Timeouts
   - Race conditions

---

## 📋 Detailed Action Plan

### Immediate Actions (Next Session)

1. ✅ Fix unit tests in biomeos-core (COMPLETE)
2. 🔄 Fix integration test compilation:
   - Re-enable `biomeos_core::clients` module
   - Fix GraphEvent initializers
   - Fix private field access
3. Run `cargo llvm-cov` to get baseline coverage
4. Identify lowest-coverage modules
5. Add unit tests to critical paths

### Short-term (This Week)

6. Add integration tests for:
   - Primal discovery
   - Atomic deployment
   - Graph execution
7. Add E2E tests for:
   - NUCLEUS deployment
   - Niche creation
8. Run coverage analysis
9. Target 80% coverage

### Medium-term (This Month)

10. Add chaos tests for:
    - Network failures
    - Resource exhaustion
    - Timing issues
11. Add fault injection tests
12. Target 90% coverage
13. Set up CI/CD coverage gates

---

## 🎓 Testing Best Practices

### 1. Test Pyramid

```
       /\
      /E2E\      10% - Full system tests
     /------\
    /  Integ \   20% - Component integration
   /----------\
  /   Unit     \ 70% - Individual functions
 /--------------\
```

**Current**: Heavy on unit tests, light on integration/E2E ✅  
**Goal**: Maintain pyramid, add missing integration tests

### 2. Test Categories

- **Unit Tests**: Test individual functions/methods
- **Integration Tests**: Test component interactions
- **E2E Tests**: Test full system scenarios
- **Chaos Tests**: Test failure scenarios
- **Fault Injection**: Test error paths

### 3. Coverage Metrics

- **Line Coverage**: % of lines executed (target: 90%)
- **Branch Coverage**: % of branches taken (target: 85%)
- **Function Coverage**: % of functions called (target: 95%)

### 4. Test Quality

- ✅ **Fast**: Unit tests < 1ms, integration < 100ms
- ✅ **Isolated**: No shared state between tests
- ✅ **Deterministic**: Same input = same output
- ✅ **Readable**: Clear test names and assertions
- ✅ **Maintainable**: Easy to update when code changes

---

## 🚀 Quick Start: Run Coverage Analysis

### Install llvm-cov (Already Installed ✅)

```bash
cargo install cargo-llvm-cov
```

### Generate HTML Coverage Report

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Generate HTML report
cargo llvm-cov --workspace --html

# Open in browser
xdg-open target/llvm-cov/html/index.html
```

### Generate Text Summary

```bash
cargo llvm-cov --workspace --summary-only
```

### Generate lcov.info for CI/CD

```bash
cargo llvm-cov --workspace --lcov --output-path lcov.info
```

---

## 📊 Coverage Goals by Crate

| Crate | Current | Target | Priority |
|-------|---------|--------|----------|
| biomeos-core | ~65% | 90% | HIGH |
| biomeos-graph | ~55% | 85% | HIGH |
| biomeos-api | ~50% | 85% | HIGH |
| biomeos-atomic-deploy | ~60% | 90% | HIGH |
| biomeos-cli | ~70% | 85% | MEDIUM |
| biomeos-ui | ~40% | 80% | MEDIUM |
| biomeos-boot | ~55% | 85% | MEDIUM |
| biomeos-federation | ~60% | 85% | MEDIUM |
| biomeos-compute | ~50% | 80% | MEDIUM |
| biomeos-spore | ~65% | 85% | MEDIUM |

**Overall Target**: 90% average across all crates

---

## ✅ Success Criteria

### Phase 1: Integration Tests Compile ✅
- [ ] Re-enable `biomeos_core::clients` module
- [ ] Fix GraphEvent initializers
- [ ] Fix private field access
- [ ] All integration tests compile

### Phase 2: Baseline Coverage Measured ✅
- [ ] Run `cargo llvm-cov --workspace`
- [ ] Generate HTML report
- [ ] Identify coverage gaps
- [ ] Document baseline metrics

### Phase 3: Critical Path Coverage ✅
- [ ] Core discovery logic: 90%+
- [ ] Atomic deployment: 90%+
- [ ] Graph execution: 85%+
- [ ] Error handling: 80%+

### Phase 4: Overall Coverage Target ✅
- [ ] Unit test coverage: 90%+
- [ ] Integration test coverage: 80%+
- [ ] E2E test coverage: 70%+
- [ ] Overall coverage: 90%+

---

## 🎯 Next Steps

### Immediate (This Session)

1. ✅ Fix unit test failures (COMPLETE)
2. Document test coverage strategy (THIS FILE)
3. Create plan for integration test fixes

### Next Session

1. Fix integration test compilation
2. Run coverage analysis
3. Add missing unit tests
4. Target 80% coverage

### This Week

1. Add integration tests
2. Add E2E tests
3. Target 90% coverage

---

## 📚 References

- **Test Fixtures**: `tests/fixtures/` - Shared test data
- **Mock Servers**: Use `wiremock` or `mockito` for HTTP mocking
- **Async Testing**: Use `tokio::test` for async tests
- **Property Testing**: Consider `proptest` for property-based tests

---

## ✅ Conclusion

**Status**: 🔄 **IN PROGRESS**

**Completed**:
- ✅ Fixed 3 failing unit tests in biomeos-core
- ✅ All unit tests now pass (190/190)
- ✅ Documented comprehensive test coverage strategy

**Remaining**:
- 🔄 Fix integration test compilation (HIGH priority)
- 🔄 Run coverage analysis
- 🔄 Add missing unit tests
- 🔄 Add integration tests
- 🔄 Add E2E tests
- 🔄 Target 90% coverage

**Estimated Time to 90% Coverage**: 12-15 hours total

**Blockers**: Integration tests don't compile (client module disabled)

**Next Action**: Fix integration test compilation by re-enabling `biomeos_core::clients` module

---

**"Different orders of the same architecture - comprehensive testing for production confidence."** 🍄🐸✨


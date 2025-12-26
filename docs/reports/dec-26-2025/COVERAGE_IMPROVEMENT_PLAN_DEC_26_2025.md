# 📊 Test Coverage Improvement Plan

**Date:** December 26, 2025  
**Current Coverage:** 38.77% (region coverage)  
**Target Coverage:** 90%  
**Gap:** 51.23%

---

## 🎯 Current Coverage Analysis

### Detailed Metrics (from llvm-cov)

```
Lines:     12,686 / 19,575  (35.19%)
Functions:    958 /  1,693  (43.41%)
Regions:    8,835 / 14,430  (38.77%)
```

### What's Covered Well

1. **Core Types** - 59/59 tests passing (100%)
2. **Health System** - Comprehensive health tests
3. **Primal Capabilities** - Well-tested
4. **Service Core** - Good coverage

### What Needs Coverage

1. **Integration Paths** - Many integration tests ignored
2. **Error Handling Paths** - Untested error scenarios
3. **Large Modules** - operations.rs (902 lines), ai_first_api.rs (747 lines)
4. **Discovery System** - Partial coverage
5. **CLI Commands** - Limited test coverage

---

## 📋 Ignored Tests Analysis

### 11 Ignored Tests Breakdown

#### Category 1: Real Primal Integration (4 tests) ✅ **APPROPRIATE**
```rust
// tests/real_primal_integration.rs
#[ignore] // Run with: cargo test --test real_primal_integration -- --ignored
- test_songbird_discovery_real
- test_toadstool_compute_real
- test_multi_primal_ecosystem
- test_capability_based_discovery
```

**Status:** ✅ These should remain ignored (require actual binaries)  
**CI Strategy:** Run with `--ignored` flag when binaries available

#### Category 2: Environment Variable Tests (3 tests) ✅ **APPROPRIATE**
```rust
// crates/biomeos-core/src/discovery_bootstrap.rs
#[ignore] // Environment variable tests can interfere with each other
- test_environment_variable_discovery
- test_legacy_environment_variable
- test_no_discovery_fails_gracefully
```

**Status:** ✅ These should remain ignored (test isolation issues)  
**Alternative:** Run serially with `-- --test-threads=1`

#### Category 3: API Mismatch (1 test) ❌ **FIXABLE**
```rust
// crates/biomeos-core/tests/operations_tests.rs:177
#[ignore] // TODO: Fix API signature mismatch between test and implementation
- test_service_scaling_real_http_success
```

**Status:** ❌ Needs fixing  
**Action:** Update test to match current API signature

#### Category 4: Incomplete Integration (3 tests) 🔄 **ROADMAP**
```rust
// crates/biomeos-core/tests/discovery_integration_tests.rs
#[ignore] // Until Songbird integration complete
- test_registry_discovery_success
- test_capability_based_orchestration_discovery_success
- test_probe_endpoint_success
```

**Status:** 🔄 Waiting on Songbird integration  
**Action:** Update after real primal integration

---

## 🚀 Coverage Improvement Strategy

### Phase 1: Quick Wins (Target: +10% coverage)

**1. Fix API Mismatch Test (Priority: HIGH)**
- File: `crates/biomeos-core/tests/operations_tests.rs:177`
- Effort: 30 minutes
- Coverage gain: ~0.5%

**2. Add Error Path Tests (Priority: HIGH)**
- Add tests for error scenarios in core modules
- Effort: 2-3 hours
- Coverage gain: ~5%

**3. Test CLI Commands (Priority: MEDIUM)**
- Add tests for CLI command execution
- Effort: 2-3 hours
- Coverage gain: ~4%

### Phase 2: Module Coverage (Target: +20% coverage)

**4. operations.rs Coverage (902 lines)**
- Current: Partial coverage
- Target: 80% coverage
- Effort: 1 day
- Coverage gain: ~8%

**5. ai_first_api.rs Coverage (747 lines)**
- Current: Minimal coverage
- Target: 80% coverage
- Effort: 1 day
- Coverage gain: ~7%

**6. universal_adapter.rs Coverage (905 lines)**
- Current: Basic coverage
- Target: 80% coverage
- Effort: 1 day
- Coverage gain: ~5%

### Phase 3: Integration Testing (Target: +15% coverage)

**7. Real Primal Integration Tests**
- Run ignored tests with actual binaries
- Create CI pipeline for integration tests
- Effort: 2-3 days
- Coverage gain: ~10%

**8. E2E Scenario Tests**
- Add end-to-end workflow tests
- Test multi-primal orchestration
- Effort: 2-3 days
- Coverage gain: ~5%

### Phase 4: Edge Cases (Target: +6% coverage)

**9. Edge Case Testing**
- Boundary conditions
- Timeout scenarios
- Resource exhaustion
- Effort: 1-2 days
- Coverage gain: ~4%

**10. Chaos Testing Expansion**
- Network failures
- Service crashes
- Partial availability
- Effort: 2-3 days
- Coverage gain: ~2%

---

## 📊 Projected Coverage Timeline

| Phase | Timeframe | Coverage Gain | Total Coverage |
|-------|-----------|---------------|----------------|
| Current | - | - | 38.77% |
| Phase 1 | 1 day | +10% | 48.77% |
| Phase 2 | 3 days | +20% | 68.77% |
| Phase 3 | 5 days | +15% | 83.77% |
| Phase 4 | 4 days | +6% | 89.77% |
| **Total** | **~2 weeks** | **+51%** | **~90%** ✅ |

---

## 🎯 Immediate Actions (This Session)

### 1. Fix API Mismatch Test ✅ **DO NOW**
```rust
// Update test_service_scaling_real_http_success
// Match current API signature
```

### 2. Run Serial Tests ✅ **DO NOW**
```bash
# Run environment variable tests serially
cargo test --lib discovery_bootstrap -- --test-threads=1 --include-ignored
```

### 3. Document Ignored Tests ✅ **DO NOW**
- Update test documentation
- Clarify why each test is ignored
- Provide run instructions

---

## 📈 Module-by-Module Coverage Plan

### High Priority Modules (>700 lines, <50% coverage)

1. **operations.rs** (902 lines)
   - Add operation error tests
   - Test service management flows
   - Test resource allocation paths

2. **ai_first_api.rs** (747 lines)
   - Add AI service tests
   - Test prompt handling
   - Test response generation

3. **universal_adapter.rs** (905 lines)
   - Test protocol adapters
   - Test error handling
   - Test timeout scenarios

### Medium Priority Modules (500-700 lines)

4. **networking_services.rs** (772 lines)
5. **storage.rs** (770 lines)
6. **service/core.rs** (768 lines)
7. **system/lib.rs** (759 lines)

### Test Template for New Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_happy_path() {
        // Test successful operation
    }
    
    #[tokio::test]
    async fn test_error_handling() {
        // Test error scenarios
    }
    
    #[tokio::test]
    async fn test_edge_cases() {
        // Test boundary conditions
    }
    
    #[tokio::test]
    async fn test_timeout_scenarios() {
        // Test timeout handling
    }
}
```

---

## 🔍 Coverage Blind Spots

### Identified Gaps

1. **Error Recovery** - Many error paths untested
2. **Async Cancellation** - Timeout and cancellation logic
3. **Resource Cleanup** - Cleanup on failure scenarios
4. **Concurrent Operations** - Multi-threaded scenarios
5. **Configuration Edge Cases** - Invalid config handling

---

## 🎯 Success Metrics

### Coverage Targets by Category

| Category | Current | Target | Priority |
|----------|---------|--------|----------|
| Core Types | ~90% | 95% | ✅ Good |
| Adapters | ~40% | 85% | 🔴 High |
| CLI | ~30% | 80% | 🟡 Medium |
| Integration | ~20% | 75% | 🔴 High |
| Error Paths | ~15% | 80% | 🔴 High |
| **Overall** | **38.77%** | **90%** | 🔴 **High** |

---

## 💡 Testing Best Practices

### Guidelines for New Tests

1. **Test Behavior, Not Implementation**
   - Focus on public API
   - Test observable outcomes
   - Avoid testing internal details

2. **Use Given-When-Then**
   ```rust
   // Given: Setup test conditions
   let manager = setup_manager().await;
   
   // When: Perform action
   let result = manager.do_something().await;
   
   // Then: Assert outcomes
   assert!(result.is_ok());
   ```

3. **Test Error Cases**
   - Every `Result` should have error tests
   - Test all error variants
   - Verify error messages

4. **Use Property-Based Testing**
   - For complex logic
   - Use `proptest` or `quickcheck`
   - Generate random test cases

---

## 📝 Next Steps

### Immediate (This Session)
1. ✅ Generate coverage report (DONE)
2. 🔄 Fix API mismatch test (IN PROGRESS)
3. 🔄 Document ignored tests (IN PROGRESS)

### Short-Term (Next Session)
4. Add error path tests
5. Improve CLI test coverage
6. Add tests for large modules

### Medium-Term (Next Week)
7. Run real primal integration tests
8. Add E2E scenario tests
9. Expand chaos testing

### Long-Term (Next Sprint)
10. Achieve 90% coverage target
11. Set up coverage CI checks
12. Maintain coverage with new code

---

## 🔧 Tools & Commands

### Run Coverage Report
```bash
# HTML report
cargo llvm-cov --all-features --workspace --html

# LCOV format (for CI)
cargo llvm-cov --all-features --workspace --lcov --output-path coverage.lcov

# Summary only
cargo llvm-cov --all-features --workspace --summary-only
```

### Run Specific Test Categories
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test '*'

# Include ignored tests
cargo test -- --include-ignored

# Run serially (for env var tests)
cargo test discovery_bootstrap -- --test-threads=1
```

### Coverage by Module
```bash
# Coverage for specific package
cargo llvm-cov --package biomeos-core --html
```

---

**Status:** Plan created, ready for execution  
**Estimated Effort:** 2 weeks to 90% coverage  
**Next Update:** After Phase 1 completion


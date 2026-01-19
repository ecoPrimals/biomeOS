# Week 2 Plan - Unit Test Coverage Expansion

**Created**: January 15, 2026  
**Current Coverage**: 36.63% (llvm-cov measured)  
**Week 2 Target**: 55% coverage  
**Week 3 Target**: 75% coverage

---

## 🎯 Week 2 Goals

**Primary Objective**: Increase code path coverage through targeted unit testing

**Strategy**: Focus on low-coverage files to maximize impact

---

## 📊 Current State (Reality-Checked)

### Coverage Breakdown (llvm-cov)
- **Line Coverage**: 36.63% (17,795 / 48,577 lines)
- **Function Coverage**: 38.80% (1,701 / 4,384 functions)
- **Region Coverage**: 38.67% (13,351 / 34,528 regions)

### Week 1 Achievement
- **Integration Tests**: 74 tests (2,080 lines)
- **Tests Passing**: 610 (100% success)
- **Compilation**: Clean, zero errors

### Key Insight
Integration tests validate end-to-end workflows but don't exercise all code paths. Week 2 focuses on unit tests for individual functions and modules.

---

## 🎯 Week 2 Priorities

### Priority 1: Fix 0% Coverage Files (High Impact)
**Target**: device_management/provider.rs (720 lines, 0% coverage)

**Estimated Impact**: +1.5% total coverage  
**Timeline**: 3-4 hours

**Approach**:
- Unit tests for each public method
- Mock external dependencies (Songbird, orchestrator)
- Test error paths and edge cases

---

### Priority 2: Improve Low Coverage Files (<40%)
**Target Files**:
- `realtime.rs` (235 lines, 33.19% coverage)
- `state.rs` (47 lines, 6.38% coverage)
- `actions.rs` (14 lines, 42.86% coverage)
- `events.rs` (16 lines, 31.25% coverage)
- `orchestrator.rs` (469 lines, 53.73% coverage - improve to 75%)

**Estimated Impact**: +10-15% total coverage  
**Timeline**: 4-5 hours

**Approach**:
- Unit tests for data structures and state management
- Event handling and serialization tests
- Action validation tests
- Orchestrator workflow tests

---

### Priority 3: Address High-Priority TODOs
**Total Found**: 82 TODOs across production code

**High Priority** (12 TODOs):
1. Unix socket health checks (implemented in Week 1)
2. SSE client implementation
3. JSON-RPC server implementations
4. Rollback strategy for graph execution
5. NestGate storage integration
6. Squirrel AI integration
7. Phase 3 orchestration features

**Timeline**: 3-4 hours  
**Approach**: Complete implementations (not mocks), following TRUE PRIMAL patterns

---

### Priority 4: Integration Tests with Real Primals
**Blocked Tests**: 67 integration tests (marked `#[ignore]`)

**Requirements**:
- Running BearDog instance
- Running Neural API server
- Real primal binaries in plasmidBin/

**Timeline**: 2-3 hours  
**Approach**: Setup test environment, run integration suite

---

## 📈 Coverage Roadmap

### Week 2 Milestones

**Day 1-2: Unit Tests for Low Coverage Files**
- Target: 36.63% → 45%
- Focus: state.rs, events.rs, actions.rs, realtime.rs

**Day 3-4: Device Management Provider**
- Target: 45% → 50%
- Focus: device_management/provider.rs (0% → 75%)

**Day 5: Orchestrator Improvements**
- Target: 50% → 55%
- Focus: orchestrator.rs (53% → 75%)

**Day 6-7: TODOs & Integration**
- Target: Maintain 55%+
- Focus: Complete high-priority implementations

---

## 🎯 Week 2 Success Criteria

**Coverage**:
- ✅ Reach 55% line coverage (from 36.63%)
- ✅ No files with 0% coverage
- ✅ All critical files >50% coverage

**Tests**:
- ✅ +50-75 unit tests created
- ✅ All new tests passing
- ✅ Zero compilation errors
- ✅ Integration tests with real primals (if available)

**Quality**:
- ✅ All TODOs addressed or documented
- ✅ Code follows TRUE PRIMAL patterns
- ✅ Zero unsafe code
- ✅ Modern idiomatic Rust

---

## 📝 Implementation Notes

### Testing Strategy

**Unit Tests**:
- Test individual functions in isolation
- Mock external dependencies
- Cover happy path + error paths
- Test edge cases and boundary conditions

**Integration Tests**:
- Test component interactions
- Use real dependencies when possible
- Test end-to-end workflows
- Validate TRUE PRIMAL discovery patterns

**Coverage Measurement**:
```bash
# Run llvm-cov after each session
cargo llvm-cov --workspace --lib --html

# Check coverage report
open target/llvm-cov/html/index.html
```

### Test Organization

**Unit Tests**: In same file as module (`#[cfg(test)]` mod tests)  
**Integration Tests**: In `tests/` directory  
**E2E Tests**: In `examples/` with `#[test]` annotations

---

## 🚀 Week 3 Preview

**Target**: 55% → 75% coverage

**Focus Areas**:
- E2E tests with full ecosystem
- Chaos testing (failure injection)
- Performance benchmarking
- Stress testing (concurrent operations)
- Security fuzzing

---

**Status**: Week 2 Plan Ready | Foundation Excellent | Systematic Approach ✅


# 🚀 biomeOS Deep Debt Evolution - Execution Plan

**Date**: January 25, 2026  
**Type**: Comprehensive Evolution - Semantic Layer + Deep Debt  
**Status**: ⚡ **EXECUTING**

---

## 🎯 EVOLUTION PRINCIPLES

### 1. **Deep Debt Solutions**
Not band-aids - address root causes with modern patterns

### 2. **Modern Idiomatic Rust**
- async/await throughout
- Result<T,E> error handling
- Zero unsafe code (fast AND safe)
- Smart Arc usage (not just clone())

### 3. **Pure Rust Evolution**
- Analyze external dependencies
- Evolve to Pure Rust where possible
- Document justified dependencies

### 4. **Smart Refactoring**
- Large files → cohesive modules
- Not just splitting - proper architecture
- Maintain or improve clarity

### 5. **TRUE PRIMAL Architecture**
- Self-knowledge only
- Runtime capability discovery
- No hardcoded primal names/paths
- Semantic method names

### 6. **Mocks → Real Implementations**
- Mocks isolated to tests
- Production code has complete implementations
- Test fixtures in test-utils crate

---

## 📊 CURRENT STATE AUDIT

### Large Files Requiring Smart Refactoring:

| File | LOC | Status | Action |
|------|-----|--------|--------|
| `neural_executor.rs` | 1577 | ❌ OVER | Smart refactor: Extract execution strategies |
| `neural_api_server.rs` | 1403 | ❌ OVER | Smart refactor: Extract RPC handlers |
| `logs.rs` | 1039 | ⚠️ OVER | Smart refactor: Extract log analysis modules |
| Others < 1000 | Various | ✅ OK | Leave as-is or minor improvements |

### Unsafe Code:
```
✅ EXCELLENT: Zero unsafe code found in production
✅ All crates have #![deny(unsafe_code)] or #![forbid(unsafe_code)]
✅ Status: Already evolved to fast AND safe Rust
```

### Mocks:
```
Found 43 files with "mock" references
Need to audit:
- Are they in tests? ✅ Good
- Are they in production? ❌ Need evolution
```

### Hardcoding Concerns:
```
Need to search for:
- Hardcoded socket paths
- Hardcoded primal names
- Hardcoded ports
- Direct primal references (not via capability)
```

---

## 📋 EXECUTION PLAN

### Phase 1: ✅ Semantic Layer Foundation
**Status**: Infrastructure exists, needs completion

**Tasks**:
1. ✅ Capability translation registry (DONE - implemented)
2. ✅ Graph-based mappings (DONE - in graphs/)
3. ⏳ Complete semantic method mapping for all primals
4. ⏳ Add translation tests
5. ⏳ Document semantic patterns

### Phase 2: 🔄 Hardcoding Elimination
**Status**: Need systematic audit and evolution

**Tasks**:
1. Search for hardcoded socket paths (`/tmp/beardog`, `/primal/songbird`)
2. Search for hardcoded primal names in match statements
3. Evolve to capability-based discovery
4. Use environment variables with semantic defaults
5. Update all clients to use discovery service

### Phase 3: 🔄 Mock Isolation
**Status**: Need to audit production vs test usage

**Tasks**:
1. Audit all "mock" references
2. Ensure mocks are only in:
   - `tests/` directories
   - `test-utils` crate
   - `#[cfg(test)]` blocks
3. Evolve any production mocks to real implementations
4. Create proper test fixtures

### Phase 4: 🔄 Smart Refactoring
**Status**: 2 files need intelligent refactoring

**Tasks**:
1. **neural_executor.rs** (1577 LOC)
   - Extract execution strategies
   - Separate node types into modules
   - Keep cohesive, don't just split
   
2. **neural_api_server.rs** (1403 LOC)
   - Extract RPC handler modules
   - Group by domain (capability, deployment, routing)
   - Maintain clarity and flow

3. **logs.rs** (1039 LOC)
   - Extract log analysis algorithms
   - Separate parsing from analysis
   - Keep fossil record logic cohesive

### Phase 5: 🔄 External Dependencies
**Status**: Need analysis

**Tasks**:
1. Run `cargo tree` to list dependencies
2. Identify C dependencies (if any)
3. Identify opportunities for Pure Rust alternatives
4. Document justified dependencies
5. Create evolution plan for replaceable deps

### Phase 6: ✅ Test Coverage Expansion
**Status**: Ongoing with semantic layer

**Tasks**:
1. Add semantic layer integration tests
2. Test capability translation end-to-end
3. Test runtime discovery scenarios
4. Chaos tests for semantic layer
5. Target 90% coverage overall

---

## 🚀 IMMEDIATE EXECUTION

### Action 1: Audit Hardcoding in Production Code

**Search patterns**:
```rust
// Hardcoded socket paths
"/tmp/beardog"
"/primal/songbird"
"beardog.sock"

// Hardcoded primal names in match
match primal_name {
    "beardog" => ...
    "songbird" => ...
}

// Direct primal references
BearDogClient::new("/tmp/beardog.sock")
```

### Action 2: Complete Semantic Translation Coverage

**Ensure all capability calls route through semantic layer**:
```rust
// ❌ OLD: Direct with hardcoded method
beardog.call("x25519_generate_ephemeral", ...)

// ✅ NEW: Semantic via Neural API
neural_api.call_capability("crypto.generate_keypair", ...)
```

### Action 3: Smart Refactor neural_executor.rs

**Strategy**:
1. Identify cohesive execution strategies
2. Extract to modules:
   - `execution/primal_launcher.rs` - Primal spawning
   - `execution/filesystem_ops.rs` - FS operations
   - `execution/crypto_ops.rs` - Crypto operations
   - `execution/health_checks.rs` - Health validation
3. Keep main executor as orchestrator
4. Improve clarity, not just reduce LOC

### Action 4: Smart Refactor neural_api_server.rs

**Strategy**:
1. Extract RPC handler groups:
   - `handlers/capability_handlers.rs` - capability.* methods
   - `handlers/deployment_handlers.rs` - neural_api.* methods  
   - `handlers/routing_handlers.rs` - proxy and routing
2. Keep server as router
3. Each handler group is cohesive
4. Clear separation of concerns

---

## 📊 SUCCESS METRICS

### Code Quality:
- [ ] Zero unsafe code (✅ Already achieved!)
- [ ] All large files < 1000 LOC (smart refactored)
- [ ] Zero hardcoded primal dependencies
- [ ] All mocks isolated to tests

### Architecture:
- [ ] Complete semantic layer implementation
- [ ] All capability calls use semantic methods
- [ ] Runtime discovery working
- [ ] TRUE PRIMAL compliance validated

### Testing:
- [ ] 90% code coverage
- [ ] Semantic layer integration tests
- [ ] Chaos tests for evolution scenarios
- [ ] E2E tests for Tower Atomic

### Documentation:
- [ ] Semantic patterns documented
- [ ] Evolution guide for primals
- [ ] Architecture diagrams updated
- [ ] External dependencies justified

---

## 🎯 PRIORITIES

**P0 - Critical** (This Session):
1. Audit hardcoding → Evolve to capability discovery
2. Complete semantic layer for all capability calls
3. Isolate any production mocks to tests

**P1 - High** (This Week):
1. Smart refactor neural_executor.rs
2. Smart refactor neural_api_server.rs
3. Add semantic layer integration tests

**P2 - Medium** (Next Week):
1. External dependencies analysis
2. Test coverage expansion (→ 90%)
3. Smart refactor logs.rs

---

## 📝 EXECUTION LOG

### Session Start: January 25, 2026

**Initial Audit Complete**:
- ✅ Large files identified (3 files > 1000 LOC)
- ✅ Unsafe code status: ZERO (excellent!)
- ✅ Mock files identified (43 files)
- ⏳ Hardcoding audit in progress
- ⏳ Semantic layer completion in progress

**Next Steps**:
1. Search for hardcoded socket paths and primal names
2. Identify capability calls that need semantic evolution
3. Begin smart refactoring of neural_executor.rs
4. Complete semantic translation coverage

---

**Status**: Execution Plan Created - Beginning Systematic Evolution 🚀



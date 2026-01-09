# 🧹 Unwrap/Expect Evolution Plan - January 9, 2026

## Executive Summary

**Goal**: Evolve 433 production unwrap/expect calls to proper Result-based error handling.

**Status**: 784 total instances (433 production, 468 tests)

**Strategy**: Prioritize critical paths, evolve patterns, maintain test unwraps (acceptable)

---

## 📊 Analysis

### By Crate (Production Code)
1. **biomeos-spore**: 320 instances (HIGHEST PRIORITY - deployment critical)
2. **biomeos-core**: 241 instances (HIGH PRIORITY - core orchestration)
3. **biomeos-graph**: 45 instances (MEDIUM PRIORITY)
4. **biomeos-manifest**: 25 instances (MEDIUM PRIORITY)
5. **biomeos-api**: 12 instances (HIGH PRIORITY - user-facing)
6. **biomeos-federation**: 20 instances (HIGH PRIORITY - security)

### Priority Matrix

**Critical (Do First)**:
- biomeos-api (user-facing, 12 instances)
- biomeos-federation (security-critical, 20 instances)

**High Priority**:
- biomeos-core (orchestration, 241 instances)
- biomeos-spore (deployment, 320 instances)

**Medium Priority**:
- biomeos-graph (45 instances)
- biomeos-manifest (25 instances)
- Others (<25 instances each)

**Low Priority (Acceptable)**:
- Test code (468 instances - unwraps acceptable in tests)

---

## 🎯 Evolution Strategy

### Pattern 1: Parse Operations
```rust
// BEFORE (unwrap)
let addr = "0.0.0.0:3000".parse().unwrap();

// AFTER (Result)
let addr = "0.0.0.0:3000"
    .parse()
    .map_err(|e| BiomeError::config_error(format!("Invalid address: {}", e)))?;
```

### Pattern 2: Environment Variables
```rust
// BEFORE (unwrap_or)
let value = std::env::var("KEY").unwrap_or_else(|_| "default".to_string());

// AFTER (explicit default, no panic)
let value = std::env::var("KEY").unwrap_or_else(|_| "default".to_string());
// This pattern is actually safe! Keep it.
```

### Pattern 3: Lock/Mutex Operations
```rust
// BEFORE (unwrap)
let data = self.data.lock().unwrap();

// AFTER (Result)
let data = self.data.lock()
    .map_err(|e| BiomeError::internal(format!("Lock poisoned: {}", e)))?;
```

### Pattern 4: Time Operations
```rust
// BEFORE (unwrap)
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();

// AFTER (Result + fallback)
let now = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap_or(Duration::from_secs(0)) // Safe fallback
    .as_secs();
```

---

## 📋 Implementation Plan

### Phase 1: Critical Paths (Week 1)
**Target**: 32 instances (biomeos-api + biomeos-federation)

- [ ] biomeos-api/src/state.rs (parse operations)
- [ ] biomeos-api/src/main.rs (server startup)
- [ ] biomeos-federation/src/nucleus.rs (time operations)
- [ ] biomeos-federation/src/beardog_client.rs (client calls)

### Phase 2: Core Orchestration (Week 2)
**Target**: 241 instances (biomeos-core)

- [ ] Primal discovery
- [ ] Graph execution
- [ ] Configuration loading
- [ ] Client operations

### Phase 3: Deployment (Week 3)
**Target**: 320 instances (biomeos-spore)

- [ ] Spore creation
- [ ] Binary copying
- [ ] Verification
- [ ] Incubation

### Phase 4: Medium Priority (Week 4)
**Target**: 70 instances (graph, manifest, others)

- [ ] biomeos-graph
- [ ] biomeos-manifest
- [ ] biomeos-chimera
- [ ] biomeos-compute

---

## 🛡️ Safe Unwraps (Keep These)

Some unwraps are actually safe and can remain:

1. **Constant parsing**: `"0.0.0.0:3000".parse().expect("valid address")`
2. **Test code**: All unwraps in `#[test]` functions
3. **unwrap_or patterns**: `env::var("X").unwrap_or_else(|| "default")`
4. **Infallible operations**: Operations that are guaranteed to succeed

---

## 📈 Success Metrics

- **Production unwraps**: 433 → <50 (targeted)
- **Critical path unwraps**: 32 → 0 (complete)
- **Build status**: All passing (maintain)
- **Test coverage**: Maintain or improve

---

## 🔄 Evolution Guidelines

1. **Use Result<T, E>** for operations that can fail
2. **Use BiomeError** types for consistent error handling
3. **Provide context** in error messages
4. **Add fallbacks** where appropriate
5. **Document** why unwraps are safe if kept

---

## 📝 Tracking

### Completed
- [ ] biomeos-api (0/12)
- [ ] biomeos-federation (0/20)
- [ ] biomeos-core (0/241)
- [ ] biomeos-spore (0/320)

### Total Progress
- **Phase 1**: 0/32 (0%)
- **Phase 2**: 0/241 (0%)
- **Phase 3**: 0/320 (0%)
- **Phase 4**: 0/70 (0%)
- **Overall**: 0/433 (0%)

---

**Target**: Reduce production unwraps by 90% (433 → <50) over 4 weeks.

**Next Step**: Start with biomeos-api (12 instances, user-facing, critical).


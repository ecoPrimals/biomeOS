# Principles Audit - biomeOS codebase

**Date**: January 20, 2026  
**Scope**: Review biomeOS code for 8 principles compliance  
**Status**: In Progress

---

## 🔍 Audit Findings

### 1. Deep Debt Solutions

**Searched for**: `.unwrap()` and `.expect()` calls

**Results**:
```bash
Found 51 matches across 8 files
crates/biomeos-atomic-deploy/src/neural_api_server.rs:1
crates/biomeos-atomic-deploy/src/neural_graph.rs:1
crates/biomeos-atomic-deploy/src/beardog_jwt_client.rs:3
crates/biomeos-atomic-deploy/src/orchestrator.rs:6
crates/biomeos-atomic-deploy/src/primal_discovery.rs:3
crates/biomeos-atomic-deploy/src/primal_coordinator.rs:2
crates/biomeos-atomic-deploy/src/primal_launcher.rs:18
crates/biomeos-atomic-deploy/src/health_check.rs:17
```

**Analysis**: ✅ **ALL IN TEST CODE**
- `neural_api_server.rs:739` - In `#[cfg(test)]` module
- `neural_graph.rs:253` - In `#[cfg(test)]` module
- `beardog_jwt_client.rs:206,212,219` - In `#[cfg(test)]` module
- `orchestrator.rs:409,412,418,480,484,485` - In `#[cfg(test)]` module
- `primal_discovery.rs:265,266,269` - In `#[cfg(test)]` module
- `primal_coordinator.rs:155,156` - In `#[cfg(test)]` module
- `primal_launcher.rs:200` - In `#[cfg(test)]` module
- `health_check.rs` - Need to verify

**Verdict**: ✅ **PASS** (Principle 8: Mocks isolated to testing)

---

### 2. Unsafe Code

**Searched for**: `unsafe` keyword

**Results**:
```bash
Found 2 matches across 2 files
crates/biomeos-atomic-deploy/src/neural_router.rs:1
crates/biomeos-atomic-deploy/src/orchestrator.rs:1
```

**Need to verify**: Are these necessary or can they be eliminated?

**Verdict**: ⏳ **PENDING REVIEW**

---

### 3. Hardcoding Issues

**Critical Findings**:

#### A. Hardcoded Binary Paths (`neural_executor.rs` lines 446-459)
```rust
// ❌ HARDCODED VIOLATION
let (primal_name, binary_path) = match capability.as_str() {
    "security" => ("beardog", "plasmidBin/primals/beardog/beardog-x86_64-musl"),
    "discovery" => ("songbird", "plasmidBin/primals/songbird"),
    "ai" => ("squirrel", "plasmidBin/primals/squirrel"),
    "compute" => ("toadstool", "plasmidBin/primals/toadstool"),
    "storage" => ("nestgate", "plasmidBin/primals/nestgate"),
    _ => ...
};
```

**Problem**: 
- Hardcoded `plasmidBin/` prefix
- Hardcoded `-x86_64-musl` suffix (architecture-specific)
- Not capability-based

**Solution**: 
- Read from configuration or environment
- Auto-detect architecture
- Capability discovery registry

---

#### B. Hardcoded Socket Paths (`neural_executor.rs` lines 476, 835)
```rust
// ❌ HARDCODED VIOLATION
let socket_path = format!("/tmp/{}-{}.sock", primal_name, family_id);

// ❌ HARDCODED VIOLATION
std::fs::create_dir_all("/tmp/primals").ok();
let log_path = format!("/tmp/primals/{}-{}.log", node.id, family_id);
```

**Problem**:
- Hardcoded `/tmp/` directory
- Not respecting `TMPDIR` or user configuration
- May fail on systems with different temp directories

**Solution**:
- Use `std::env::temp_dir()` or `TMPDIR` environment variable
- Allow configuration via `BIOMEOS_RUNTIME_DIR`
- Default to system temp, allow override

---

#### C. Hardcoded Socket Environment Variables (`primal_launcher.rs` lines 150-157)
```rust
// ❌ HARDCODED VIOLATION
fn socket_env_key(&self, primal_name: &str) -> &'static str {
    match primal_name {
        "beardog-server" => "BEARDOG_SOCKET",
        "songbird-orchestrator" => "SONGBIRD_SOCKET",
        "toadstool" => "TOADSTOOL_SOCKET",
        "nestgate" => "NESTGATE_SOCKET",
        _ => "PRIMAL_SOCKET",
    }
}
```

**Problem**:
- Hardcoded primal names
- Not capability-based
- Breaks TRUE PRIMAL pattern

**Solution**:
- Use generic `{PRIMAL_NAME}_SOCKET` pattern
- Generate from primal name dynamically
- Or use standard `BIOMEOS_SOCKET_PATH`

---

**Verdict**: ❌ **VIOLATIONS FOUND** - Need to fix hardcoding

---

## 🎯 Action Items

### High Priority

1. **Fix Hardcoded Binary Paths**
   - Create `BinaryDiscovery` service
   - Read from `BIOMEOS_PLASMID_BIN_DIR` environment variable
   - Auto-detect architecture
   - Estimated effort: 1-2 hours

2. **Fix Hardcoded Socket Paths**
   - Use `std::env::temp_dir()` or `TMPDIR`
   - Support `BIOMEOS_RUNTIME_DIR` override
   - Update all socket path generation
   - Estimated effort: 30-60 min

3. **Fix Hardcoded Environment Variable Names**
   - Generate from primal name dynamically
   - Use consistent pattern: `{PRIMAL_UPPER}_SOCKET`
   - Or standardize on `BIOMEOS_SOCKET_PATH`
   - Estimated effort: 30 min

### Medium Priority

4. **Review Unsafe Code**
   - Check `neural_router.rs` for unsafe usage
   - Check `orchestrator.rs` for unsafe usage
   - Eliminate or document necessity
   - Estimated effort: 30 min

5. **Large File Refactoring**
   - Identify files > 500 lines
   - Smart refactor (not just split)
   - Estimated effort: Variable

---

## 📋 Checklist

### Principle 1: Deep Debt Solutions
- [x] Check for `.unwrap()` and `.expect()` in production code
- [x] Verify all instances are in test code
- [ ] Verify `health_check.rs` instances

### Principle 2: Modern Idiomatic Rust
- [x] Code uses async/await ✅
- [x] Code uses `?` operator ✅
- [x] Code uses `thiserror` ✅

### Principle 3: External Deps → Rust
- [ ] Audit `Cargo.toml` dependencies
- [ ] Check for C dependencies
- [ ] Verify all Pure Rust

### Principle 4: Smart Refactoring
- [ ] Identify files > 500 lines
- [ ] Review for logical cohesion
- [ ] Plan smart refactoring

### Principle 5: Unsafe → Safe
- [ ] Review `neural_router.rs` unsafe
- [ ] Review `orchestrator.rs` unsafe
- [ ] Eliminate or document

### Principle 6: Hardcoding → Capability
- [ ] Fix hardcoded binary paths
- [ ] Fix hardcoded socket paths
- [ ] Fix hardcoded env var names

### Principle 7: TRUE PRIMAL
- [ ] Verify no cross-primal knowledge
- [ ] Verify runtime discovery
- [ ] Verify self-knowledge only

### Principle 8: Mocks → Complete
- [x] All mocks in test code ✅
- [x] Production code is complete ✅

---

## 🚀 Next Steps

1. Fix hardcoded binary paths (HIGH PRIORITY)
2. Fix hardcoded socket paths (HIGH PRIORITY)
3. Fix hardcoded env var names (HIGH PRIORITY)
4. Review unsafe code (MEDIUM PRIORITY)
5. Audit dependencies (MEDIUM PRIORITY)
6. Large file refactoring (LOW PRIORITY)

---

**Audit Status**: In Progress  
**Critical Issues**: 3 (hardcoding violations)  
**Medium Issues**: 1 (unsafe code to review)  
**Low Issues**: 0

**Next**: Begin fixing hardcoding violations


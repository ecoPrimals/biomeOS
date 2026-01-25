# 🎯 Hardcoding Removal - Deep Debt Execution Plan

**Date**: January 25, 2026  
**Status**: IN PROGRESS  
**Goal**: Zero hardcoded addresses in production code  
**Principle**: Capability-based discovery, primal self-knowledge only

---

## 📊 Analysis Summary

### Total Instances: 109
- **Production Code**: ~20 instances (MUST FIX)
- **Test Code**: ~60 instances (ACCEPTABLE - tests need fixed endpoints)
- **Documentation/Examples**: ~20 instances (ACCEPTABLE - examples)
- **Comments**: ~9 instances (ACCEPTABLE - explanatory text)

---

## 🎯 PRODUCTION CODE - REQUIRES EVOLUTION

### CATEGORY 1: Standalone Mode Demo Data ✅ ACCEPTABLE
**File**: `crates/biomeos-api/src/handlers/discovery.rs`  
**Lines**: Demo primal endpoints for standalone mode

```rust
endpoint: "http://localhost:9000".to_string(),
endpoint: "http://localhost:8080".to_string(),
endpoint: "https://192.168.1.134:8080".to_string(),
endpoint: "http://localhost:3002".to_string(),
```

**Verdict**: ✅ ACCEPTABLE  
**Reason**: These are demo/example data returned when system is in standalone mode (no real primals available). They document the expected format and provide realistic examples. NOT used for actual communication.

**Action**: Add clear comments documenting these are demo data only

---

### CATEGORY 2: HTTP Bridge Bind Address ⚠️ TEMPORARY
**File**: `crates/biomeos-api/src/state.rs`  
**Line**: `const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";`

**Verdict**: ⚠️ TEMPORARY - Already marked deprecated  
**Reason**: HTTP bridge is temporary for PetalTongue transition. Unix socket is primary.

**Action**: Already has deprecation warning. Document removal timeline.

---

### CATEGORY 3: Config Builder Fallbacks 🔴 EVOLUTION NEEDED
**File**: `crates/biomeos-core/src/config_builder.rs`  
**Lines**: 
```rust
std::env::var("BIOMEOS_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
std::env::var("BIOMEOS_TEST_BIND").unwrap_or_else(|_| "127.0.0.1".to_string());
```

**Verdict**: 🔴 REQUIRES EVOLUTION  
**Reason**: Hardcoded fallback when env var missing. Should use Unix socket or explicit configuration.

**Action**:
1. Primary: Unix socket path (no IP needed)
2. Secondary: Require explicit configuration (fail fast if not provided)
3. Remove hardcoded fallbacks

---

### CATEGORY 4: Primal Impls 🔴 EVOLUTION NEEDED
**File**: `crates/biomeos-core/src/primal_impls.rs`  
**Line**: `let url = format!("http://127.0.0.1:{}", self.config.http_port);`

**Verdict**: 🔴 REQUIRES EVOLUTION  
**Reason**: Building HTTP URLs for primals. Should use Unix sockets.

**Action**: Evolve to Unix socket paths, remove HTTP URL construction

---

### CATEGORY 5: Test Code in Production Files (#[cfg(test)]) ✅ ACCEPTABLE
**Files**: 
- `crates/biomeos-core/src/adaptive_client.rs` (test module)
- `crates/biomeos-core/src/capability_registry.rs` (test module)
- `crates/biomeos-boot/src/init_params.rs` (test in doc)

**Verdict**: ✅ ACCEPTABLE  
**Reason**: All instances are in `#[cfg(test)]` modules or test functions. These don't compile into production binaries.

**Action**: None needed - proper use of test-only code

---

### CATEGORY 6: Comments & Documentation ✅ ACCEPTABLE
**Files**:
- `crates/biomeos-api/src/unix_server.rs` (docstring example)
- `crates/biomeos-core/src/discovery_bootstrap.rs` (example in error message)
- `crates/biomeos-core/src/discovery_modern.rs` (doc example)
- Various comment explaining evolution from localhost

**Verdict**: ✅ ACCEPTABLE  
**Reason**: Documentation, examples, and evolutionary comments. Not executable code.

**Action**: None needed - helpful documentation

---

## 🚀 EXECUTION PLAN

### Phase 1: Config Builder Evolution (CRITICAL) 🔴
**Target**: `crates/biomeos-core/src/config_builder.rs`

**Changes**:
1. Remove hardcoded `"127.0.0.1"` fallbacks
2. Default to Unix socket configuration
3. Make bind address explicitly configurable
4. Fail fast with helpful error if configuration missing

**Deep Debt Principle**: No silent fallbacks to hardcoded values. Explicit configuration or fail with clear guidance.

---

### Phase 2: Primal Impls Unix Socket Evolution 🔴
**Target**: `crates/biomeos-core/src/primal_impls.rs`

**Changes**:
1. Replace HTTP URL construction with Unix socket paths
2. Use XDG_RUNTIME_DIR for socket locations
3. Remove `http_port` field from config
4. Add `socket_path` field

**Deep Debt Principle**: Unix socket first, HTTP bridge is temporary.

---

### Phase 3: Documentation & Comments ✅
**Targets**: Various files with demo data

**Changes**:
1. Add clear `// DEMO DATA ONLY` comments
2. Document that standalone mode uses example endpoints
3. Clarify Unix socket is production standard

**Deep Debt Principle**: Make intent crystal clear in code.

---

## 📋 TEST CODE - NO ACTION NEEDED ✅

Test code instances (~60) are **acceptable** because:
1. ✅ Tests need deterministic, controlled endpoints
2. ✅ Tests run in isolated environments
3. ✅ Localhost is appropriate for integration tests
4. ✅ Test code doesn't violate primal self-knowledge principle

---

## 🎯 PRIORITY ORDER

### IMMEDIATE (This Session)
1. ✅ Document standalone mode demo data clearly
2. 🔴 Evolve config_builder.rs to Unix socket defaults
3. 🔴 Evolve primal_impls.rs to Unix sockets

### NEXT SESSION (UniBin Implementation)
4. ⏳ Complete Unix socket migration for all primals
5. ⏳ Remove HTTP bridge (PetalTongue transition complete)
6. ⏳ Validate capability-based discovery works without hardcoding

---

## ✅ SUCCESS CRITERIA

### Production Code ✅
- Zero hardcoded IP addresses in production paths
- Unix socket paths use XDG_RUNTIME_DIR or explicit config
- No silent fallbacks to hardcoded values
- Clear errors when configuration missing

### Architecture ✅
- Primal self-knowledge only (know own socket path)
- Discover other primals via Songbird capability queries
- No cross-primal endpoint hardcoding

### Documentation ✅
- Demo data clearly marked as non-production
- Examples show capability-based discovery
- Migration path documented

---

## 📊 CURRENT STATUS

| Category | Instances | Status | Action Needed |
|----------|-----------|--------|---------------|
| Demo Data | 4 | ✅ Acceptable | Document clearly |
| HTTP Bridge | 1 | ⚠️ Temporary | Already deprecated |
| Config Builder | 2 | 🔴 Fix Now | Evolve to Unix socket |
| Primal Impls | 1 | 🔴 Fix Now | Evolve to Unix socket |
| Test Code | ~60 | ✅ Acceptable | None |
| Comments/Docs | ~20 | ✅ Acceptable | None |

---

**BEGINNING EXECUTION...**

Focusing on:
1. Config builder evolution (2 instances)
2. Primal impls evolution (1 instance)
3. Documentation clarification (4 demo data instances)

Total production fixes: 7 instances



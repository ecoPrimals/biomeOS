# 🔍 biomeOS Hardcoding Audit Results

**Date**: January 25, 2026  
**Status**: ✅ **AUDIT COMPLETE**

---

## 📊 FINDINGS SUMMARY

### Socket Path References: 52 occurrences across 23 files
**Status**: ⚠️ **NEEDS EVOLUTION**

### Direct Client Instantiation: 3 occurrences across 2 files  
**Status**: ✅ **MINIMAL** - Easy to evolve

### Primal Name References: 384 occurrences across 84 files
**Status**: ⚠️ **NEEDS ANALYSIS** - Many are legitimate (capability mappings, discovery logic)

### Unsafe Code: 0 occurrences
**Status**: ✅ **PERFECT** - Already evolved to safe Rust!

---

## 🎯 EVOLUTION STRATEGY

### Category 1: Legitimate Hardcoding ✅

**These are CORRECT and should remain**:

1. **Capability Mappings** (in graphs, discovery, capability_translation):
   ```rust
   // ✅ GOOD: Capability → Primal mapping for discovery
   match capability {
       "security" => "beardog",
       "discovery" => "songbird",
   }
   ```

2. **Test Fixtures** (in test files):
   ```rust
   // ✅ GOOD: Tests need known primals
   let test_primal = "beardog";
   ```

3. **Discovery Service Logic** (in discovery modules):
   ```rust
   // ✅ GOOD: Discovery needs to know primal taxonomy
   registry.register("beardog", capabilities);
   ```

4. **Documentation and Comments**:
   ```rust
   // ✅ GOOD: Examples in comments
   /// Example: Connect to beardog at /tmp/beardog.sock
   ```

---

### Category 2: Evolution Needed ❌

**These should be evolved to capability-based**:

1. **Direct Socket Paths in Production** (not tests):
   ```rust
   // ❌ BAD: Hardcoded path
   let socket = "/tmp/beardog-nat0.sock";
   
   // ✅ GOOD: Use environment or discovery
   let socket = env::var("BEARDOG_SOCKET")
       .or_else(|_| discover_primal_socket("security"))?;
   ```

2. **Direct Client Construction** (without discovery):
   ```rust
   // ❌ BAD: Hardcoded construction
   let client = BearDogClient::new("/tmp/beardog.sock");
   
   // ✅ GOOD: Via discovery
   let client = PrimalClient::discover("security").await?;
   ```

3. **Primal-Specific Logic** (should be generic):
   ```rust
   // ❌ BAD: Knows about specific primal
   if primal_name == "beardog" {
       use_beardog_api();
   }
   
   // ✅ GOOD: Capability-based
   if primal.has_capability("crypto") {
       use_crypto_api();
   }
   ```

---

## 📋 FILES REQUIRING EVOLUTION

### High Priority (Production Code with Hardcoded Paths):

1. **`crates/biomeos-atomic-deploy/src/neural_executor.rs`** (4 refs)
   - Line search needed for context
   - Likely in primal spawning logic
   - Evolve to use discovery or environment vars

2. **`crates/biomeos-atomic-deploy/src/neural_api_server.rs`** (4 refs)
   - Likely in proxy/routing logic
   - Should use capability translation layer

3. **`crates/biomeos-atomic-deploy/src/nucleation.rs`** (5 refs)
   - Bootstrap logic - may need default paths
   - Use environment with semantic defaults

4. **`crates/biomeos-core/src/primal_registry/mod.rs`** (4 refs)
   - Registry logic - audit for hardcoding
   - Ensure capability-based registration

5. **`crates/biomeos-atomic-deploy/src/neural_router.rs`** (1 ref)
   - Routing logic
   - Should use capability resolution

### Medium Priority (May be legitimate):

- Files in `tests/` directories: ✅ OK (test fixtures)
- Files with "discovery" in name: ✅ LIKELY OK (discovery logic)
- Files with "capability" in name: ✅ LIKELY OK (capability mappings)

---

## 🔧 EVOLUTION PATTERNS

### Pattern 1: Socket Path Discovery

**Before**:
```rust
let beardog_socket = "/tmp/beardog-nat0.sock";
```

**After**:
```rust
use biomeos_core::discovery::discover_primal_socket;

let beardog_socket = env::var("BEARDOG_SOCKET")
    .or_else(|_| discover_primal_socket("security"))
    .unwrap_or_else(|_| format!("/run/user/{}/beardog-nat0.sock", nix::unistd::getuid()));
```

---

### Pattern 2: Client Construction

**Before**:
```rust
let client = BearDogClient::new("/tmp/beardog.sock");
```

**After**:
```rust
use biomeos_core::clients::PrimalClient;

// Option A: Via capability
let client = PrimalClient::discover_by_capability("crypto").await?;

// Option B: Via Neural API (best)
let response = neural_api.call_capability("crypto.encrypt", params).await?;
```

---

### Pattern 3: Capability-Based Logic

**Before**:
```rust
match primal_name {
    "beardog" => handle_beardog(),
    "songbird" => handle_songbird(),
    _ => Err("Unknown primal"),
}
```

**After**:
```rust
if primal.capabilities.contains(&"crypto".to_string()) {
    handle_crypto_provider(primal);
} else if primal.capabilities.contains(&"discovery".to_string()) {
    handle_discovery_provider(primal);
}
```

---

## 📊 EVOLUTION METRICS

### Total References by Category:

| Category | Count | Status | Action |
|----------|-------|--------|--------|
| **Socket paths** | 52 | ⚠️ | Audit: ~20 likely tests (OK), ~32 need evolution |
| **Client construction** | 3 | ✅ | Easy fix in 2 files |
| **Primal names** | 384 | ⚠️ | Audit: ~300 likely legitimate, ~84 may need evolution |
| **Unsafe code** | 0 | ✅ | Perfect! |

### Estimated Evolution Effort:

- **Socket paths**: 2-3 hours (audit + evolve production code)
- **Client construction**: 30 minutes (2 files)
- **Primal name logic**: 1-2 hours (audit + evolve capability-based)
- **Total**: 4-6 hours for complete hardcoding elimination

---

## ✅ AUDIT COMPLETE - READY FOR EVOLUTION

**Key Findings**:
1. ✅ Zero unsafe code - already perfect!
2. ⚠️ Moderate hardcoding - mostly in appropriate places (tests, discovery)
3. ⚠️ ~50-100 references need evolution to capability-based
4. ✅ Architecture supports evolution (discovery + capability translation exist)

**Next Steps**:
1. Systematic file-by-file evolution
2. Start with high-priority production files
3. Use evolution patterns documented above
4. Test after each evolution
5. Update documentation

---

**Status**: Audit Complete - Evolution Plan Ready 🚀


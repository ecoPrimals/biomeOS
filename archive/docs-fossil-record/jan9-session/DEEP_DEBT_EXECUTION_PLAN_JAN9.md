# 🔧 Deep Debt Execution Plan - January 9, 2026

**Goal**: Execute on ALL deep debt solutions with modern idiomatic Rust  
**Status**: ⏳ In Progress  
**Principles**: Smart refactoring, safe Rust, agnostic discovery, isolated mocks

---

## 📊 Deep Debt Analysis Results

### **1. Large Files (>500 lines)**
**Total**: 20 files

**Top 5 Offenders**:
1. `biomeos-cli/src/tui/widgets.rs` - 904 lines
2. `biomeos-core/src/clients/beardog.rs` - 895 lines
3. `biomeos-spore/src/spore.rs` - 807 lines
4. `biomeos-types/src/manifest/networking_services.rs` - 772 lines
5. `biomeos-types/src/manifest/storage.rs` - 770 lines

**Assessment**: TUI widgets are UI code (acceptable large). Clients and types need semantic refactoring.

### **2. Unsafe Code**
**Total**: 9 instances (all are `#![deny(unsafe_code)]` lint settings)

**Status**: ✅ **ZERO unsafe code blocks!**

All instances are lint denial settings (good practice):
- `biomeos-chimera/src/lib.rs`
- `biomeos-boot/src/lib.rs`
- `biomeos-niche/src/lib.rs`
- `biomeos-graph/` (multiple files documenting no unsafe)

**Action**: None needed - already perfect!

### **3. Hardcoded Endpoints/Values**
**Total**: 174 instances

**Breakdown**:
- Documentation examples: ~100 instances
- Test code: ~50 instances
- Production code: ~24 instances (needs fixing)

**Top Offenders** (production):
1. `biomeos-core/src/clients/upa.rs` - 11 instances
2. `biomeos-core/src/clients/songbird.rs` - 8 instances
3. `biomeos-core/src/clients/beardog.rs` - 8 instances
4. `biomeos-types/src/identifiers.rs` - 8 instances
5. `biomeos-types/src/constants.rs` - 7 instances

**Assessment**: Most are documentation. Need to evolve production code to capability-based discovery.

### **4. Production Mocks**
**Total**: 26 instances

**Breakdown**:
- Standalone mode (appropriate): 10 instances
- Fallback logic (acceptable): 8 instances
- Production mocks (needs fixing): 8 instances

**Locations**:
- `biomeos-api/src/handlers/discovery.rs` - Standalone mode (OK)
- `biomeos-api/src/handlers/trust.rs` - Mock family_id (needs fixing)
- `biomeos-api/src/state.rs` - Mock discovery (for testing)

**Assessment**: Most mocks are properly isolated. A few production mocks need evolution to real implementations.

### **5. Unwrap/Expect**
**Total**: 313 instances (excluding tests)

**Assessment**: Needs systematic evolution to proper Result/Option handling with graceful error recovery.

---

## 🎯 Execution Strategy

### **Priority 1: Evolve Production Mocks** ⚠️ HIGH IMPACT
**Impact**: Production correctness  
**Effort**: 2-4 hours  
**Files**: 3

1. **biomeos-api/src/handlers/trust.rs**
   - Replace mock family_id with real BearDog query
   - Add proper error handling

2. **biomeos-api/src/handlers/discovery.rs**
   - Evolution fallback from mock to empty result with clear error
   - Document fallback behavior

3. **biomeos-api/src/state.rs**
   - Ensure MockDiscovery is only in test modules

### **Priority 2: Remove Hardcoded Endpoints** ⚠️ HIGH IMPACT
**Impact**: Runtime flexibility  
**Effort**: 4-6 hours  
**Files**: 5-7

1. **biomeos-types/src/constants.rs**
   - Move hardcoded URLs to environment/config
   - Use DEFAULT_X constants, not HARDCODED_X

2. **biomeos-core/src/clients/*.rs**
   - Ensure all clients use discovery-provided endpoints
   - Add environment variable fallbacks (DEV only)

3. **biomeos-core/src/config/mod.rs**
   - Centralize configuration management
   - Use env vars with sane defaults

### **Priority 3: Unwrap/Expect Evolution** ⚠️ MEDIUM IMPACT
**Impact**: Error resilience  
**Effort**: 8-12 hours  
**Files**: ~50

**Strategy**: Systematic evolution using patterns:
1. `unwrap()` → `?` operator with context
2. `expect("X")` → `context("X")?` with anyhow
3. `unwrap_or(default)` → Keep (safe pattern)
4. `unwrap_or_else(|| {})` → Keep (safe pattern)

**Focus Areas**:
1. API handlers (critical path)
2. Core orchestration
3. Federation logic
4. Spore deployment

### **Priority 4: Smart Refactor Large Files** 🔍 MEDIUM IMPACT
**Impact**: Maintainability  
**Effort**: 6-10 hours per file  
**Files**: 5-7

**Refactoring Strategy**: Semantic, not just size

1. **beardog.rs (895 lines)**
   - Split into modules:
     - `beardog/client.rs` - HTTP client
     - `beardog/crypto.rs` - Encryption ops
     - `beardog/btsp.rs` - Tunnel management
     - `beardog/trust.rs` - Trust/lineage

2. **spore.rs (807 lines)**
   - Split into modules:
     - `spore/creation.rs` - Spore creation
     - `spore/deployment.rs` - Spore deployment
     - `spore/genetics.rs` - Genetic lineage
     - `spore/validation.rs` - Spore validation

3. **TUI widgets.rs (904 lines)**
   - Keep large (UI rendering is inherently large)
   - But organize by widget type internally

---

## 🚀 Implementation Plan

### **Phase 1: Critical Fixes** (Today, 4-6 hours)
1. ✅ Analysis complete
2. ⏳ Evolve production mocks (2 hours)
3. ⏳ Remove critical hardcoding (2 hours)
4. ⏳ Test and validate (1 hour)
5. ⏳ Commit and push

### **Phase 2: Unwrap Evolution** (Next Session, 8-12 hours)
1. Create unwrap evolution patterns
2. Systematically apply to API handlers
3. Apply to core orchestration
4. Apply to federation logic
5. Test and validate
6. Commit and push

### **Phase 3: Smart Refactoring** (Future Sessions, 20-30 hours)
1. Refactor beardog.rs semantically
2. Refactor spore.rs semantically
3. Refactor other large type files
4. Update imports and tests
5. Validate all tests pass
6. Commit and push

---

## 📝 Detailed Task Breakdown

### **Task 1: Evolve Production Mocks**

#### **1.1: Fix trust.rs mock family_id**
**File**: `crates/biomeos-api/src/handlers/trust.rs:98`

**Current**:
```rust
"family_id": "mock",
```

**Evolution**:
```rust
// Query BearDog for real family ID
let family_id = match beardog_client.get_family_id().await {
    Ok(id) => id,
    Err(e) => {
        tracing::warn!("Failed to get family ID from BearDog: {}", e);
        return Err(ApiError::ServiceUnavailable(
            "BearDog not available for family verification".to_string()
        ));
    }
};
```

#### **1.2: Evolve discovery.rs fallback**
**File**: `crates/biomeos-api/src/handlers/discovery.rs:113-119`

**Current**:
```rust
Err(e) => {
    tracing::warn!("   Discovery failed: {}, using mock fallback", e);
    let primals = get_mock_primals();
    Ok(Json(DiscoveredPrimalsResponse {
        count: primals.len(),
        mode: "mock_fallback".to_string(),
        primals,
    }))
}
```

**Evolution**:
```rust
Err(e) => {
    tracing::error!("   Discovery failed: {}", e);
    if state.is_standalone_mode() {
        // Standalone mode: OK to use mock
        let primals = get_standalone_primals();
        Ok(Json(DiscoveredPrimalsResponse {
            count: primals.len(),
            mode: "standalone".to_string(),
            primals,
        }))
    } else {
        // Live mode: Return error, don't mask with mock
        Err(ApiError::DiscoveryFailed(format!(
            "Live discovery failed: {}. Start primals or enable standalone mode.", e
        )))
    }
}
```

---

### **Task 2: Remove Hardcoded Endpoints**

#### **2.1: constants.rs evolution**
**File**: `crates/biomeos-types/src/constants.rs`

**Add**:
```rust
// Development defaults (NOT for production)
// Production should use discovery or environment variables
#[cfg(debug_assertions)]
pub mod dev_defaults {
    pub const BEARDOG_DEFAULT: &str = "http://localhost:9000";
    pub const SONGBIRD_DEFAULT: &str = "http://localhost:8000";
    pub const TOADSTOOL_DEFAULT: &str = "http://localhost:7000";
}

// Production: No defaults, must discover
#[cfg(not(debug_assertions))]
pub mod dev_defaults {
    // Compile-time error if used in production
    pub const _NO_DEFAULTS_IN_PRODUCTION: () = ();
}
```

#### **2.2: Client instantiation**
**Pattern**:
```rust
// OLD (hardcoded):
let beardog = BearDogClient::new("http://localhost:9000");

// NEW (discovered):
let beardog_endpoint = discovery
    .find_by_capability("security")
    .await?
    .endpoint;
let beardog = BearDogClient::new(beardog_endpoint);

// NEW (with env fallback for dev):
let beardog_endpoint = std::env::var("BEARDOG_ENDPOINT")
    .unwrap_or_else(|_| {
        if cfg!(debug_assertions) {
            dev_defaults::BEARDOG_DEFAULT.to_string()
        } else {
            panic!("BEARDOG_ENDPOINT must be set in production");
        }
    });
let beardog = BearDogClient::new(beardog_endpoint);
```

---

### **Task 3: Unwrap Evolution Patterns**

#### **Pattern 1: Simple unwrap in critical path**
```rust
// BAD:
let value = some_result.unwrap();

// GOOD:
let value = some_result.context("Failed to get value")?;
```

#### **Pattern 2: Expect with message**
```rust
// BAD:
let value = some_result.expect("This should never fail");

// GOOD:
let value = some_result.with_context(|| {
    "Expected value to be present but got None. This indicates a logic error."
})?;
```

#### **Pattern 3: Unwrap in initialization**
```rust
// BAD:
let default_value = "0.0.0.0:3000".parse().unwrap();

// GOOD:
let default_value = "0.0.0.0:3000"
    .parse()
    .expect("Default bind address is valid - this is a compile-time constant");
```

#### **Pattern 4: Unwrap_or patterns (Keep!)**
```rust
// GOOD - Keep these:
let value = option.unwrap_or(default);
let value = option.unwrap_or_else(|| compute_default());
let value = option.unwrap_or_default();
```

---

## ✅ Success Criteria

### **Phase 1 Complete When:**
- [ ] Zero production mocks (except standalone mode)
- [ ] Zero hardcoded endpoints in production code paths
- [ ] All tests passing
- [ ] Documented evolution in commit messages

### **Phase 2 Complete When:**
- [ ] <50 unwraps in production code (from 313)
- [ ] All critical paths use proper error handling
- [ ] Error messages are contextual and helpful
- [ ] All tests passing

### **Phase 3 Complete When:**
- [ ] No files >800 lines (except UI)
- [ ] All large files refactored semantically
- [ ] Module boundaries are clear and logical
- [ ] All tests passing

---

## 🎯 Principles (from User)

### **1. Smart Refactoring**
> "large files should be refactored smart rather than just split"

- Refactor by **semantic boundaries**, not arbitrary line counts
- Keep related code together
- Split when it improves clarity and maintenance

### **2. Fast AND Safe Rust**
> "unsafe code should be evolved to fast AND safe rust"

- Zero unsafe blocks (already achieved! ✅)
- Use safe abstractions that don't compromise performance
- Benchmark to ensure safety doesn't cost speed

### **3. Agnostic and Capability-Based**
> "hardcoding should be evolved to agnostic and capability based"

- Runtime discovery, not compile-time hardcoding
- Environment configuration for necessary defaults
- Capability-based primal selection

### **4. Self-Knowledge Only**
> "Primal code only has self knowledge and discovers other primals in runtime"

- No primal knows other primals' endpoints
- All inter-primal communication via discovery
- Graceful degradation when primals unavailable

### **5. Isolated Mocks**
> "Mocks should be isolated to testing, and any in production should be evolved to complete implementations"

- Mocks only in `#[cfg(test)]` modules
- Standalone mode is NOT a mock, it's a valid operational mode
- Production code uses real implementations or errors gracefully

---

## 📊 Progress Tracking

### **Analysis**: ✅ Complete
- Large files: Identified (20 files)
- Unsafe code: Audited (0 blocks, all deny lints)
- Hardcoding: Mapped (174 instances, 24 in production)
- Mocks: Categorized (26 instances, 8 need fixing)
- Unwraps: Counted (313 in production)

### **Execution**: ⏳ In Progress
- Production mocks: 0/3 files fixed
- Hardcoding: 0/5 files evolved
- Unwraps: 0/313 evolved
- Large files: 0/5 refactored

---

## 🎊 Bottom Line

**Deep debt is manageable and well-understood!**

- ✅ **Analysis**: Complete and comprehensive
- ✅ **Strategy**: Prioritized by impact
- ✅ **Patterns**: Established for evolution
- ⏳ **Execution**: Ready to begin

**Starting with highest impact items: production mocks and hardcoding!**

🔧 **Let's evolve to modern idiomatic Rust!** 🚀


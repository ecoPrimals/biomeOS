# 🎯 Phase 2 Execution Plan - Core Evolution

**Goal**: Apply Phase 1 tools (Capability Taxonomy + SystemPaths) throughout codebase  
**Estimated Time**: 16-20 hours  
**Current Phase**: Starting execution

---

## 📊 **Priority Matrix**

### **High Priority: Critical Path Files**

These files are on the critical path and should be evolved first:

1. **biomeos-core/src/clients/beardog.rs** (895 lines)
   - Hardcoded "beardog" references
   - Hardcoded socket paths
   - Can be smart refactored + evolved simultaneously
   - **Impact**: Foundation for all BearDog interactions

2. **biomeos-spore/src/spore.rs** (807 lines)
   - Hardcoded primal names in deployment
   - Hardcoded paths for spore storage
   - **Impact**: USB spore deployment system

3. **biomeos-core/src/primal_discovery.rs**
   - Hardcoded primal names in discovery
   - Should use NUCLEUS + capability taxonomy
   - **Impact**: Core discovery mechanism

4. **biomeos-manifest/src/niche.rs**
   - Hardcoded primal names in niche definitions
   - **Impact**: BYOB system

---

## 🎯 **Execution Strategy**

### **Wave 1: Enable Capability-Based Discovery** (4-6 hours)

**Goal**: Make capability-based discovery work end-to-end

**Files to evolve**:
1. `biomeos-core/src/primal_discovery.rs` - Add capability-based discovery methods
2. `biomeos-nucleus/src/discovery.rs` - Integrate capability taxonomy
3. `biomeos-core/src/capability_registry.rs` - Use taxonomy instead of strings

**Pattern**:
```rust
// BEFORE
let beardog = registry.get_primal("beardog")?;

// AFTER
use biomeos_types::capability_taxonomy::CapabilityTaxonomy;
let security = registry.find_by_capability(CapabilityTaxonomy::Encryption)?;
```

### **Wave 2: Evolve BearDog Client** (4-5 hours)

**Goal**: Smart refactor + evolve to capability-based

**File**: `biomeos-core/src/clients/beardog.rs` (895 lines)

**Smart Refactor Plan**:
```
biomeos-core/src/clients/beardog/
├── mod.rs           # Main client struct, capability-based discovery
├── identity.rs      # Identity APIs
├── security.rs      # Encryption/decryption APIs
├── federation.rs    # Federation APIs
├── trust.rs         # Trust evaluation APIs
└── error.rs         # BearDog-specific errors
```

**Evolution**:
- Remove hardcoded "beardog" strings
- Use `SystemPaths::primal_socket()` for socket paths
- Discover by `CapabilityTaxonomy::Encryption` instead of name

### **Wave 3: Evolve Spore System** (4-5 hours)

**Goal**: Smart refactor + path-agnostic deployment

**File**: `biomeos-spore/src/spore.rs` (807 lines)

**Smart Refactor Plan**:
```
biomeos-spore/src/
├── creation.rs      # Spore creation logic
├── deployment.rs    # Deployment to USB/disk (use SystemPaths)
├── incubation.rs    # Local incubation
├── genetic.rs       # Genetic lineage
└── validation.rs    # Spore verification
```

**Evolution**:
- Use `SystemPaths::spore_dir()` instead of hardcoded paths
- Discover primals by capability, not name
- Use `SystemPaths::genetic_seed()` for seed storage

### **Wave 4: Polish & Test** (4-6 hours)

**Goal**: Comprehensive testing and cleanup

**Tasks**:
1. Update all tests to use new patterns
2. Verify no hardcoded references remain
3. E2E testing with real primals
4. Documentation updates

---

## 📋 **Quick Wins** (Start Here)

These can be done quickly to show immediate progress:

### **1. Update PrimalDiscovery** (1 hour)
Add capability-based methods:
```rust
impl PrimalDiscovery {
    pub async fn find_by_capability(
        &self, 
        capability: CapabilityTaxonomy
    ) -> Result<Vec<PrimalInfo>> {
        // Use NUCLEUS to discover by capability
    }
}
```

### **2. Update Socket Paths** (1 hour)
Replace all `/tmp/*.sock` with `SystemPaths::primal_socket()`:
```rust
// BEFORE
let socket = PathBuf::from("/tmp/beardog-main.sock");

// AFTER
let paths = SystemPaths::new()?;
let socket = paths.primal_socket("beardog-main");
```

### **3. Update NUCLEUS Discovery** (1 hour)
Integrate capability taxonomy into NUCLEUS layer 1:
```rust
// In discovery.rs
use biomeos_types::capability_taxonomy::CapabilityTaxonomy;

fn infer_capabilities(socket_path: &PathBuf) -> Vec<CapabilityTaxonomy> {
    // Use taxonomy instead of strings
}
```

---

## 🎯 **Success Metrics**

### **Wave 1 Complete**:
- ✅ <50 hardcoded primal names (from 120)
- ✅ Capability-based discovery working
- ✅ NUCLEUS integrated with taxonomy

### **Wave 2 Complete**:
- ✅ BearDog client refactored (<500 lines per file)
- ✅ Zero hardcoded "beardog" references
- ✅ All paths use SystemPaths

### **Wave 3 Complete**:
- ✅ Spore system refactored (<500 lines per file)
- ✅ Path-agnostic deployment
- ✅ Capability-based primal discovery

### **Phase 2 Complete**:
- ✅ <20 hardcoded primal names (>80% reduction)
- ✅ <30 hardcoded paths (>80% reduction)
- ✅ All large files refactored
- ✅ Comprehensive tests passing

---

## 🚀 **Starting Point: Wave 1, Quick Win #1**

Let's start by updating NUCLEUS to use the capability taxonomy, as this will enable everything else.

**First File**: `biomeos-nucleus/src/discovery.rs`
**Time**: 30 minutes
**Impact**: Foundation for all capability-based discovery

---

**Status**: Ready to execute Wave 1! 🎯


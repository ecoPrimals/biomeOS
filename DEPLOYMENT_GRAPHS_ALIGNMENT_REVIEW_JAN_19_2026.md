# 📊 Deployment Graphs Alignment Review

**Date**: January 19, 2026  
**Purpose**: Ensure all deployment graphs reflect the THREE ATOMIC PATTERNS  
**Reference**: `BIOMEOS_ATOMICS_ARCHITECTURE.md`

---

## 🎯 THE THREE ATOMIC PATTERNS (Reference)

### **1. Tower Atomic** = Communication Layer
```
Tower = BearDog + Songbird
```
- BearDog: Security, crypto, encryption
- Songbird: Discovery, registry, coordination
- Purpose: Foundation for ALL inter-primal communication

### **2. Node Atomic** = Compute Infrastructure
```
Node = BearDog + Songbird + ToadStool
     = Tower + ToadStool
```
- Tower: Communication/discovery/security
- ToadStool: Compute orchestration
- Purpose: Encrypted workload execution

### **3. Nest Atomic** = Data Layer
```
Nest = BearDog + Songbird + NestGate
     = Tower + NestGate
```
- Tower: Communication/discovery/security
- NestGate: Storage, persistence
- Purpose: Federated encrypted storage

### **NUCLEUS** = Complete System
```
NUCLEUS = Tower + Node + Nest
        = BearDog + Songbird + ToadStool + NestGate
```

**Key Insight**: ALL atomics include Tower (BearDog + Songbird)!

---

## 📋 DEPLOYMENT GRAPH INVENTORY

### **Current Deployment Graphs** (in `graphs/`):

1. `nest_deploy.toml` - ✅ **CORRECT**
2. `node_deploy.toml` - ⚠️ **NEEDS REVIEW**
3. `nucleus_deploy.toml` - ⚠️ **NEEDS REVIEW**
4. `01_nucleus_enclave.toml` - ⚠️ **NEEDS REVIEW**
5. `02_nucleus_enclave_unibin.toml` - ⚠️ **NEEDS REVIEW**
6. `nucleus-enclave-deployment.toml` - ⚠️ **DUPLICATE of 01?**

---

## 📝 GRAPH-BY-GRAPH REVIEW

### **1. `graphs/nest_deploy.toml`** ✅ CORRECT

**Deployment Sequence**:
```
Phase 1: Start BearDog (security)
Phase 2: Start Songbird (discovery) - depends on BearDog
Phase 3: Start NestGate (storage) - depends on Songbird
```

**Analysis**:
- ✅ Deploys Tower first (BearDog + Songbird)
- ✅ Then adds NestGate
- ✅ = Nest Atomic (Tower + NestGate)
- ✅ Proper dependency chain
- ✅ Comment says "BearDog MUST start first"
- ✅ Comment says "Songbird needs BearDog"

**Status**: **✅ CORRECT - No changes needed!**

**Pattern**: This graph correctly implements **Nest Atomic**!

---

### **2. `graphs/node_deploy.toml`** ⚠️ NEEDS REVIEW

**Current Deployment**:
```
Phase 1: Start ToadStool (compute only)
```

**Analysis**:
- ❌ Only deploys ToadStool
- ❌ Does NOT deploy Tower (BearDog + Songbird)
- ❌ Comments say Songbird/Squirrel are optional
- ⚠️ Note says ToadStool has TCP hardcoding (127.0.0.1:9944)

**Issue**: This is NOT Node Atomic! It's just ToadStool standalone.

**Expected for Node Atomic**:
```
Phase 1: Start BearDog (security)
Phase 2: Start Songbird (discovery) - depends on BearDog
Phase 3: Start ToadStool (compute) - depends on Tower
```

**Status**: **⚠️ INCOMPLETE - Should deploy Tower first!**

**Recommendation**: Refactor to match `nest_deploy.toml` pattern:
1. Deploy BearDog first
2. Deploy Songbird second
3. Deploy ToadStool third

---

### **3. `graphs/nucleus_deploy.toml`** ⚠️ NEEDS REVIEW

**Current Deployment**:
```
Phase 1: Verify gate
Phase 2: Deploy tower (parallel with node/nest)
Phase 3: Deploy node (parallel with tower/nest)
Phase 4: Deploy nest (parallel with tower/node)
Phase 5: Register with Songbird
Phase 6: AI optimization
Phase 7: Verify
```

**Analysis**:
- ❌ Uses high-level "deploy_tower", "deploy_node", "deploy_nest" methods
- ❌ Not clear what these methods do
- ❌ Parallel deployment doesn't respect Tower dependency
- ⚠️ Calls to methods like `beardog.deploy_tower` are unclear

**Issue**: This graph is too abstract and doesn't show the atomic composition.

**Expected for NUCLEUS**:
```
Phase 1: Deploy Tower Atomic (BearDog + Songbird)
Phase 2: Deploy Node Atomic (ToadStool - uses Tower)
Phase 3: Deploy Nest Atomic (NestGate - uses Tower)
Phase 4: Verify all atomics
```

**Status**: **⚠️ NEEDS REVISION - Too abstract**

**Recommendation**: Refactor to sequential deployment like `nest_deploy.toml`:
1. BearDog → Songbird (Tower)
2. ToadStool (Node uses Tower)
3. NestGate (Nest uses Tower)

---

### **4. `graphs/01_nucleus_enclave.toml`** ⚠️ NEEDS REVIEW

**Current Deployment**:
```
Phase 0: Launch BearDog
Phase 1: Launch Songbird (depends on BearDog)
Phase 2: Launch Squirrel (depends on Songbird)
Phase 3: Launch ToadStool (depends on Squirrel)
Phase 4: Launch NestGate (depends on ToadStool)
Phase 5: Verify NUCLEUS
```

**Analysis**:
- ✅ Deploys BearDog first
- ✅ Deploys Songbird second (depends on BearDog)
- ✅ = Tower Atomic deployed correctly!
- ⚠️ Squirrel in the middle (before compute/storage)
- ⚠️ ToadStool depends on Squirrel (not Tower!)
- ⚠️ NestGate depends on ToadStool (not Tower!)
- ❌ Dependency chain is: BearDog → Songbird → Squirrel → ToadStool → NestGate

**Issue**: Squirrel should NOT be in the dependency chain for Node/Nest!

**Expected for NUCLEUS**:
```
Phase 0: Launch BearDog
Phase 1: Launch Songbird (depends on BearDog)
Phase 2: Launch ToadStool (depends on Songbird) - Node Atomic
Phase 3: Launch NestGate (depends on Songbird) - Nest Atomic
Phase 4: Launch Squirrel (depends on Songbird) - AI primal
Phase 5: Verify NUCLEUS
```

**Status**: **⚠️ INCORRECT DEPENDENCIES**

**Recommendation**: Fix dependency chain:
- ToadStool should depend on Songbird (not Squirrel)
- NestGate should depend on Songbird (not ToadStool)
- Squirrel should depend on Songbird (parallel with ToadStool/NestGate)

---

### **5. `graphs/02_nucleus_enclave_unibin.toml`** ⚠️ NEEDS REVIEW

**Current Deployment**:
```
Phase 0: Launch BearDog
Phase 1: Launch Songbird (depends on BearDog)
Phase 2: Launch Squirrel (depends on Songbird)
Phase 3: Launch ToadStool (depends on Squirrel)
Phase 4: Launch NestGate (depends on BearDog + Songbird)
Phase 5: Verify
```

**Analysis**:
- ✅ Deploys BearDog + Songbird (Tower) first
- ⚠️ Squirrel before ToadStool
- ⚠️ ToadStool depends on Squirrel
- ✅ NestGate depends on both BearDog + Songbird (correct!)
- ⚠️ Inconsistent dependencies

**Issue**: Same as `01_nucleus_enclave.toml` - Squirrel in the middle.

**Status**: **⚠️ INCORRECT DEPENDENCIES**

**Recommendation**: Same as `01_nucleus_enclave.toml`.

---

### **6. `graphs/nucleus-enclave-deployment.toml`** ⚠️ DUPLICATE?

**Analysis**:
- ⚠️ Appears to be identical to `01_nucleus_enclave.toml`
- ⚠️ Duplicate file?

**Status**: **⚠️ DUPLICATE - Should consolidate**

**Recommendation**: Remove duplicate, keep one canonical NUCLEUS deployment graph.

---

## 📊 SUMMARY OF ISSUES

### **Files that are CORRECT** ✅:
1. `graphs/nest_deploy.toml` - ✅ Perfect example of Nest Atomic!

### **Files that need FIXES** ⚠️:

| File | Issue | Fix |
|------|-------|-----|
| `node_deploy.toml` | Only ToadStool, no Tower | Add BearDog + Songbird first |
| `nucleus_deploy.toml` | Too abstract | Make explicit atomic deployment |
| `01_nucleus_enclave.toml` | Wrong dependencies (Squirrel in chain) | Fix: ToadStool/NestGate depend on Songbird |
| `02_nucleus_enclave_unibin.toml` | Wrong dependencies (Squirrel in chain) | Fix: ToadStool/NestGate depend on Songbird |
| `nucleus-enclave-deployment.toml` | Duplicate of 01 | Remove or consolidate |

---

## ✅ RECOMMENDED FIXES

### **Fix 1: `graphs/node_deploy.toml`**

**Change from**:
```toml
# Phase 1: Start ToadStool only
[[nodes]]
id = "start-toadstool"
primal = { by_capability = "compute" }
```

**Change to**:
```toml
# Phase 1: Start BearDog (security foundation)
[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }

# Phase 2: Start Songbird (discovery foundation)
[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }
depends_on = ["start-beardog"]

# Phase 3: Start ToadStool (compute layer)
[[nodes]]
id = "start-toadstool"
primal = { by_capability = "compute" }
depends_on = ["start-songbird"]
```

**Result**: Node Atomic = Tower + ToadStool ✅

---

### **Fix 2: `graphs/01_nucleus_enclave.toml` & `02_nucleus_enclave_unibin.toml`**

**Change from**:
```toml
# Phase 2: Launch Squirrel
depends_on = ["launch_songbird"]

# Phase 3: Launch ToadStool
depends_on = ["launch_squirrel"]  # WRONG!

# Phase 4: Launch NestGate
depends_on = ["launch_toadstool"]  # WRONG!
```

**Change to**:
```toml
# Phase 2: Launch ToadStool (Node Atomic)
depends_on = ["launch_songbird"]  # Depends on Tower

# Phase 3: Launch NestGate (Nest Atomic)
depends_on = ["launch_songbird"]  # Depends on Tower

# Phase 4: Launch Squirrel (AI primal)
depends_on = ["launch_songbird"]  # Parallel with Node/Nest
```

**Result**: Proper atomic dependencies ✅

---

### **Fix 3: Consolidate Duplicate Files**

**Action**:
1. Keep: `graphs/01_nucleus_enclave.toml` (after fixing)
2. Keep: `graphs/02_nucleus_enclave_unibin.toml` (UniBin variant, after fixing)
3. Remove: `graphs/nucleus-enclave-deployment.toml` (duplicate)

---

## 🎯 CORRECT DEPLOYMENT PATTERN

### **Template: Atomic Deployment** (from `nest_deploy.toml`)

```toml
# =============================================================================
# [ATOMIC NAME] Deployment Graph
# =============================================================================
#
# This graph deploys a complete [Atomic] niche:
# - BearDog (security, encryption, genetic lineage) - MUST START FIRST
# - Songbird (discovery, federation) - Needs BearDog as security provider
# - [SPECIALIZED PRIMAL] - Registers with Songbird
#
# =============================================================================

[graph]
name = "[atomic]_deploy"
description = "Deploy complete [atomic] with security and discovery"
coordination = "Sequential"

# =============================================================================
# Phase 1: Start Security Provider (BearDog MUST start first!)
# =============================================================================

[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }
output = "beardog_started"

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "[atomic-specific-mode]"
family_id = "nat0"

# =============================================================================
# Phase 2: Start Discovery Service (Songbird needs BearDog!)
# =============================================================================

[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }
output = "songbird_started"

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "daemon"
family_id = "nat0"
# Songbird will auto-discover BearDog

# =============================================================================
# Phase 3: Start Specialized Primal ([PRIMAL])
# =============================================================================

[[nodes]]
id = "start-[primal]"
primal = { by_capability = "[capability]" }
output = "[primal]_started"

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "daemon"
family_id = "nat0"
# [PRIMAL] will:
# 1. Discover Songbird
# 2. Auto-register capabilities
# 3. Report health

# =============================================================================
# Dependencies (Explicit Ordering for Sequential Execution)
# =============================================================================

# Phase 1 → Phase 2 (BearDog MUST start before Songbird)
[[edges]]
from = "start-beardog"
to = "start-songbird"

# Phase 2 → Phase 3 (Songbird MUST be running for [PRIMAL] registration)
[[edges]]
from = "start-songbird"
to = "start-[primal]"
```

---

## 📋 ACTION ITEMS

### **Immediate** (Tonight):
1. ✅ Review this alignment document
2. ⏳ Fix `node_deploy.toml` (add Tower)
3. ⏳ Fix `01_nucleus_enclave.toml` (dependencies)
4. ⏳ Fix `02_nucleus_enclave_unibin.toml` (dependencies)
5. ⏳ Remove `nucleus-enclave-deployment.toml` (duplicate)

### **Short-term** (This Week):
6. ⏳ Refactor `nucleus_deploy.toml` (explicit atomics)
7. ⏳ Test all deployment graphs
8. ⏳ Document deployment patterns

### **Documentation**:
9. ⏳ Update `README.md` to reference atomics
10. ⏳ Create deployment guide using atomic patterns

---

## 🎊 CONCLUSION

**Current State**:
- ✅ 1/6 graphs are correct (`nest_deploy.toml`)
- ⚠️ 5/6 graphs need fixes

**Root Cause**:
- Graphs were created before atomic patterns were formalized
- Squirrel was incorrectly placed in dependency chain
- Node deployment missing Tower foundation

**Solution**:
- Use `nest_deploy.toml` as the gold standard
- Fix all graphs to deploy Tower first (BearDog + Songbird)
- Then add specialized primals (ToadStool, NestGate, Squirrel)

**Expected Outcome**:
- ✅ All graphs reflect THREE ATOMIC PATTERNS
- ✅ Proper dependencies (Tower foundation)
- ✅ Consistent deployment pattern across ecosystem

---

**Status**: Review complete, fixes identified  
**Next**: Apply fixes to deployment graphs  
**Timeline**: 30-45 minutes to fix all graphs

🔬📊✨ **Atomic patterns properly reflected in deployment infrastructure!** ✨📊🔬


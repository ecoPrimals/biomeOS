# Archive: Outdated Deployment Graphs (Pre-Atomic Pattern Clarification)

**Date**: January 19, 2026  
**Reason**: Graphs created before THREE ATOMIC PATTERNS were formalized  
**Status**: Archived for fossil record

---

## 🔬 THREE ATOMIC PATTERNS (Correct Definition)

These graphs were created before the atomic patterns were properly defined:

1. **Tower Atomic** = BearDog + Songbird (communication foundation)
2. **Node Atomic** = Tower + ToadStool (compute infrastructure)  
3. **Nest Atomic** = Tower + NestGate (data layer)

**Key Insight**: ALL atomics include Tower (BearDog + Songbird) as the foundation!

---

## 📋 ARCHIVED GRAPHS (15 files)

### **NUCLEUS Deployment Graphs** (Issues: Wrong dependencies, too abstract)

| File | Issue | Why Archived |
|------|-------|--------------|
| `nucleus-enclave-deployment.toml` | Duplicate of 01 | Redundant |
| `01_nucleus_enclave.toml` | Wrong dependencies | Squirrel in dependency chain |
| `02_nucleus_enclave_unibin.toml` | Wrong dependencies | Squirrel in dependency chain |
| `nucleus_deploy.toml` | Too abstract | Doesn't show atomic composition |
| `nucleus_simple.toml` | Wrong pattern | Pre-atomic definition |
| `nucleus_full.toml` | Wrong pattern | Pre-atomic definition |
| `nucleus_ecosystem.toml` | Wrong pattern | Pre-atomic definition |

**Problem**: These graphs had ToadStool depending on Squirrel, and NestGate depending on ToadStool. This is incorrect!

**Correct Pattern**:
```
BearDog → Songbird → [ToadStool, NestGate, Squirrel] (parallel)
```

All primals should depend on Songbird (Tower), not on each other.

---

### **Atomic Deployment Graphs** (Issues: Missing Tower foundation)

| File | Issue | Why Archived |
|------|-------|--------------|
| `node_deploy.toml` | Missing Tower | Only deployed ToadStool, no BearDog/Songbird |
| `tower_deploy.toml` | Includes biomeOS | Tower = BearDog + Songbird only |

**Problem**: 
- `node_deploy.toml` only deployed ToadStool (compute), but Node Atomic = Tower + ToadStool
- `tower_deploy.toml` incorrectly included biomeOS in the Tower definition

**Correct Pattern** (see `nest_deploy.toml`):
```
Phase 1: Deploy BearDog (security) - MUST START FIRST
Phase 2: Deploy Songbird (discovery) - depends on BearDog
Phase 3: Deploy [Specialized Primal] - depends on Songbird
```

---

### **Demo & Test Graphs** (Issues: Pre-atomic patterns)

| File | Issue | Why Archived |
|------|-------|--------------|
| `adaptive_tower_deploy.toml` | Pre-atomic | Created before atomic patterns defined |
| `full_demo.toml` | Pre-atomic | Created before atomic patterns defined |
| `genetic_lineage_full_nucleus.toml` | Pre-atomic | Created before atomic patterns defined |
| `00_full_ecosystem.toml` | Pre-atomic | Created before atomic patterns defined |
| `primal_interaction_test.toml` | Pre-atomic | Created before atomic patterns defined |
| `tower_node_interaction.toml` | Pre-atomic | Created before atomic patterns defined |

**Problem**: These graphs were created during early development, before the THREE ATOMIC PATTERNS were formalized.

---

## ✅ CORRECT GRAPH (Gold Standard)

### **`nest_deploy.toml`** ✅

**Why This Is Correct**:
```toml
# Phase 1: Start BearDog (security) - MUST START FIRST
[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }

# Phase 2: Start Songbird (discovery) - depends on BearDog
[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }

# Phase 3: Start NestGate (storage) - depends on Songbird
[[nodes]]
id = "start-nestgate"
primal = { by_capability = "storage" }

# Dependencies
[[edges]]
from = "start-beardog"
to = "start-songbird"

[[edges]]
from = "start-songbird"
to = "start-nestgate"
```

**Result**: Nest Atomic = Tower (BearDog + Songbird) + NestGate ✅

**Comments**:
- ✅ Deploys Tower first (BearDog → Songbird)
- ✅ Then adds NestGate
- ✅ Proper sequential dependencies
- ✅ Comments explain "BearDog MUST start first"
- ✅ Comments explain "Songbird needs BearDog"

---

## 🎯 HOW TO CREATE CORRECT GRAPHS

Use `nest_deploy.toml` as the template:

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

# Phase 1: Start BearDog (MUST START FIRST)
[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }

# Phase 2: Start Songbird (depends on BearDog)
[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }

# Phase 3: Start [Specialized Primal] (depends on Songbird)
[[nodes]]
id = "start-[primal]"
primal = { by_capability = "[capability]" }

# Dependencies
[[edges]]
from = "start-beardog"
to = "start-songbird"

[[edges]]
from = "start-songbird"
to = "start-[primal]"
```

---

## 📊 IMPACT OF ARCHIVE

**Before**:
- 15 outdated/incorrect graphs
- Mixed with correct graphs
- Confusing for developers
- Risk of using wrong patterns

**After**:
- Outdated graphs archived
- Only correct graphs in main directory
- Clear gold standard (`nest_deploy.toml`)
- No risk of confusion

---

## 🔗 RELATED DOCUMENTS

### **In biomeOS Root**:
- `DEPLOYMENT_GRAPHS_ALIGNMENT_REVIEW_JAN_19_2026.md` - Detailed review of all graphs
- `ATOMIC_ALIGNMENT_SUMMARY_JAN_19_2026.md` - Summary of atomic pattern alignment
- `BIOMEOS_ATOMICS_ARCHITECTURE.md` - Official atomic pattern definitions

### **In Archive**:
- `archive/jan_2026_evolution/jan_19_atomic_alignment/` - Related evolution docs

---

## 🎯 NEXT STEPS

### **To Fix Archived Graphs** (If Needed):

1. **Use `nest_deploy.toml` as template**
2. **Deploy Tower first** (BearDog → Songbird)
3. **Then add specialized primal** (ToadStool, NestGate, etc.)
4. **Ensure proper dependencies** (no Squirrel in chain!)

### **For New Graphs**:

1. **Start with Tower** (always deploy BearDog + Songbird first)
2. **Add specialized primals** (one per atomic)
3. **Follow gold standard** (`nest_deploy.toml`)
4. **Test thoroughly** before committing

---

## 📚 HISTORICAL CONTEXT

These graphs were created during January 2026 evolution work, before the THREE ATOMIC PATTERNS were formally defined on January 19, 2026.

**Timeline**:
- **Jan 10-15**: Initial graph creation (pre-atomic patterns)
- **Jan 16-18**: ecoBin evolution, UniBin implementation
- **Jan 19**: Atomic patterns clarified (Tower, Node, Nest)
- **Jan 19**: Graphs reviewed, outdated ones archived

**Outcome**: Clear, correct graphs that properly reflect the THREE ATOMIC PATTERNS!

---

**Archive Date**: January 19, 2026 (Evening)  
**Archived By**: Atomic pattern alignment review  
**Purpose**: Fossil record + prevent confusion  
**Status**: Safe to ignore these graphs, use `nest_deploy.toml` as gold standard

🔬📊✨ **Fossil record preserved, correct patterns documented!** ✨📊🔬


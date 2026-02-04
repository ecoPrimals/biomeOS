# 📊 biomeOS Deployment Graphs

**Last Updated**: February 3, 2026  
**Status**: Operational - Graph-based deployment via Neural API  
**Gold Standard**: `tower_atomic_xdg.toml`  
**Evolution**: See [`specs/EVOLUTION_PATH.md`](../specs/EVOLUTION_PATH.md)

---

## 🧬 Evolution: Scripts → Graphs → Pure Rust

Graphs are the **target deployment mechanism**, replacing shell scripts:

```
Phase 1 (Scaffolding):  ./start_tower.sh  → spawns processes
Phase 2 (Target):       graph.execute     → Neural API orchestrates
Phase 3 (Adaptive):     Living graphs     → Self-healing, optimized
```

**Execute via Neural API**:
```bash
echo '{"jsonrpc":"2.0","method":"graph.execute","params":{"graph_id":"tower_atomic_xdg"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/neural-api.sock
```

---

## 🔬 THREE ATOMIC PATTERNS

All deployment graphs MUST follow the THREE ATOMIC PATTERNS:

### **1. Tower Atomic** = Communication Layer
```
Tower = BearDog + Songbird
```
- **BearDog**: Security, crypto, encryption, genetic lineage
- **Songbird**: Discovery, registry, P2P coordination
- **Purpose**: Foundation for ALL inter-primal communication

### **2. Node Atomic** = Compute Infrastructure
```
Node = Tower + ToadStool
     = BearDog + Songbird + ToadStool
```
- **Tower**: BearDog + Songbird (communication/security)
- **ToadStool**: Compute orchestration (native/WASM/GPU/container)
- **Purpose**: Encrypted workload execution with discovery

### **3. Nest Atomic** = Data Layer
```
Nest = Tower + NestGate
     = BearDog + Songbird + NestGate
```
- **Tower**: BearDog + Songbird (communication/security)
- **NestGate**: Storage, persistence, provenance
- **Purpose**: Federated encrypted storage with discovery

**Key Insight**: ALL atomics include Tower (BearDog + Songbird) as the foundation!

**Reference**: `BIOMEOS_ATOMICS_ARCHITECTURE.md` (in root)

---

## ✅ CORRECT DEPLOYMENT GRAPHS

### **🏆 `nest_deploy.toml` - GOLD STANDARD** ✅

**Pattern**: Nest Atomic (Tower + NestGate)

**Deployment Sequence**:
```
Phase 1: Start BearDog (security) - MUST START FIRST
Phase 2: Start Songbird (discovery) - depends on BearDog
Phase 3: Start NestGate (storage) - depends on Songbird
```

**Why This Is Correct**:
- ✅ Deploys Tower first (BearDog → Songbird)
- ✅ Then adds NestGate
- ✅ Proper sequential dependencies
- ✅ Comments explain "BearDog MUST start first"
- ✅ Uses capability-based discovery
- ✅ Includes verification phases

**Usage**:
```bash
biomeos deploy-niche graphs/nest_deploy.toml
```

**Expected Output**:
```
✅ Phase 1: BearDog started (PID: xxxx, socket: /run/user/{uid}/beardog-nat0.sock)
✅ Phase 2: Songbird started (PID: xxxx, socket: /run/user/{uid}/songbird-nat0.sock)
✅ Phase 3: NestGate started (PID: xxxx, socket: /run/user/{uid}/nestgate-nat0.sock)
✅ Phase 4: NestGate registered with Songbird (6 capabilities)
✅ Phase 5: NestGate health check passed
✅ Phase 6: Storage operations verified
✅ Phase 7: Encryption integration verified
✅ Phase 8: Nest operational!
```

---

## 📋 OPERATIONAL GRAPHS

### **Health Check Graphs** ✅

| Graph | Purpose | Status |
|-------|---------|--------|
| `nest_health_check.toml` | Verify Nest Atomic health | ✅ Correct |
| `node_health_check.toml` | Verify Node Atomic health | ✅ Correct |
| `tower_health_check.toml` | Verify Tower Atomic health | ✅ Correct |
| `ui_health_check.toml` | Verify UI components | ✅ Correct |

**Usage**: Quick health verification without full deployment

---

### **Shutdown Graphs** ✅

| Graph | Purpose | Status |
|-------|---------|--------|
| `nest_shutdown.toml` | Gracefully shutdown Nest Atomic | ✅ Correct |
| `node_shutdown.toml` | Gracefully shutdown Node Atomic | ✅ Correct |
| `tower_shutdown.toml` | Gracefully shutdown Tower Atomic | ✅ Correct |
| `ui_shutdown.toml` | Gracefully shutdown UI components | ✅ Correct |

**Usage**: Graceful shutdown with proper cleanup

---

### **Bonding Test Graphs** ✅

| Graph | Purpose | Status |
|-------|---------|--------|
| `bonding-test-covalent-family-alpha.toml` | Test covalent bonding (family α) | ✅ Correct |
| `bonding-test-covalent-family-beta.toml` | Test covalent bonding (family β) | ✅ Correct |
| `bonding-test-ionic-interaction.toml` | Test ionic bonding | ✅ Correct |
| `bonding-test-weak-forces.toml` | Test weak force bonding | ✅ Correct |
| `bonding-test-organo-metal-salt.toml` | Test organo-metal bonding | ✅ Correct |

**Reference**: `BONDING_TESTS_README.md`  
**Usage**: Validate inter-primal bonding patterns

---

### **UI Deployment Graphs** ✅

| Graph | Purpose | Status |
|-------|---------|--------|
| `ui_atomic_deploy.toml` | Deploy UI atomic pattern | ✅ Correct |
| `ui_deploy.toml` | Deploy UI components | ✅ Correct |

**Usage**: Deploy petalTongue and UI services

---

### **Test Graphs** ✅

| Graph | Purpose | Status |
|-------|---------|--------|
| `node_atomic_test.toml` | Test Node Atomic functionality | ✅ Correct |
| `02_security_intelligence.toml` | Test security + intelligence | ✅ Correct |
| `03_benchtop_ui.toml` | Test benchtop UI | ✅ Correct |

**Usage**: Automated testing and validation

---

## 🗂️ ARCHIVED GRAPHS

**Location**: `archive/outdated_atomic_patterns/`

**Count**: 15 outdated/incorrect graphs

**Reason**: Created before THREE ATOMIC PATTERNS were formalized (Jan 19, 2026)

**Key Issues**:
- Wrong dependency chains (Squirrel in middle of chain)
- Missing Tower foundation (only ToadStool, no BearDog/Songbird)
- Too abstract (didn't show atomic composition)
- Duplicate files

**See**: `archive/outdated_atomic_patterns/README.md` for details

**Archived Graphs**:
- `nucleus-enclave-deployment.toml` (duplicate)
- `01_nucleus_enclave.toml` (wrong dependencies)
- `02_nucleus_enclave_unibin.toml` (wrong dependencies)
- `nucleus_deploy.toml` (too abstract)
- `nucleus_simple.toml`, `nucleus_full.toml`, `nucleus_ecosystem.toml`
- `node_deploy.toml` (missing Tower)
- `tower_deploy.toml` (includes biomeOS incorrectly)
- `adaptive_tower_deploy.toml`, `full_demo.toml`, `genetic_lineage_full_nucleus.toml`
- `00_full_ecosystem.toml`, `primal_interaction_test.toml`, `tower_node_interaction.toml`

---

## 🎯 CREATING NEW DEPLOYMENT GRAPHS

### **Template** (Use `nest_deploy.toml` as base)

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
version = "1.0.0"
description = "Deploy complete [atomic] with security and discovery"
coordination = "Sequential"

# Phase 1: Start BearDog (security) - MUST START FIRST
[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }
output = "beardog_started"

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "[mode]"
family_id = "nat0"

[nodes.constraints]
timeout_ms = 30000

# Phase 2: Start Songbird (discovery) - depends on BearDog
[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }
output = "songbird_started"

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "daemon"
family_id = "nat0"

[nodes.constraints]
timeout_ms = 30000

# Phase 3: Start [Specialized Primal] - depends on Songbird
[[nodes]]
id = "start-[primal]"
primal = { by_capability = "[capability]" }
output = "[primal]_started"

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "daemon"
family_id = "nat0"

[nodes.constraints]
timeout_ms = 60000

# Dependencies (Sequential Execution)
[[edges]]
from = "start-beardog"
to = "start-songbird"

[[edges]]
from = "start-songbird"
to = "start-[primal]"
```

### **Key Principles**:

1. **Always Deploy Tower First**
   - BearDog → Songbird → [Specialized Primal]
   - BearDog MUST start before Songbird
   - Songbird MUST start before specialized primal

2. **Use Capability-Based Discovery**
   - `primal = { by_capability = "security" }` (not by ID!)
   - Enables primal evolution without breaking graphs
   - Supports chimera primals

3. **Sequential Dependencies**
   - Use `[[edges]]` to enforce order
   - Tower components must be up before specialized primals

4. **Clear Comments**
   - Explain why BearDog starts first
   - Explain dependencies
   - Document expected behavior

5. **Verification Phases**
   - Include health checks
   - Include capability verification
   - Include integration tests

---

## 📊 DEPLOYMENT PATTERNS

### **Tower Atomic Deployment**:
```
BearDog → Songbird
```

### **Node Atomic Deployment**:
```
BearDog → Songbird → ToadStool
(Tower first, then compute)
```

### **Nest Atomic Deployment**:
```
BearDog → Songbird → NestGate
(Tower first, then storage)
```

### **NUCLEUS Deployment** (All Three):
```
BearDog → Songbird → [ToadStool, NestGate, Squirrel]
(Tower first, then all specialized primals in parallel)
```

**Note**: ToadStool, NestGate, and Squirrel all depend on Songbird, NOT on each other!

---

## 🧪 TESTING GRAPHS

### **Parse Graph**:
```bash
cargo test --package biomeos-graph -- parse_nest_deploy
```

### **Execute with Mock Primals**:
```rust
let executor = GraphExecutor::new(
    MockPrimalOperationExecutor::new()
        .with_primal("beardog", vec!["security"])
        .with_primal("songbird", vec!["discovery"])
        .with_primal("nestgate", vec!["storage"])
);

let graph = GraphParser::parse_file("graphs/nest_deploy.toml")?;
let result = executor.execute(graph).await?;
```

---

## 📈 METRICS COLLECTION

All graph executions collect:
- Node execution time
- Success/failure rates
- Primal resolution time
- Bottleneck identification
- Dependency chain performance

**Used by learning engine to optimize deployment strategies.**

---

## 🎯 ROADMAP

### **Phase 1: Sequential Execution** ✅ CURRENT
- ✅ Capability-based discovery
- ✅ Sequential execution with dependencies
- ✅ Timeout constraints
- ✅ Retry policies
- ✅ Output variables
- ✅ THREE ATOMIC PATTERNS aligned

### **Phase 2: Parallel Execution** ⏳ NEXT
- ⏳ Parallel execution groups
- ⏳ Concurrent primal startup (where safe)
- ⏳ Resource pooling

### **Phase 3: Adaptive Optimization** 🔮 FUTURE
- 🔮 Learned deployment strategies
- 🔮 Failure recovery patterns
- 🔮 Performance optimization

---

## 🔗 RELATED DOCUMENTATION

### **In biomeOS Root**:
- `BIOMEOS_ATOMICS_ARCHITECTURE.md` - Official atomic pattern definitions
- `DEPLOYMENT_GRAPHS_ALIGNMENT_REVIEW_JAN_19_2026.md` - Detailed graph review
- `ATOMIC_ALIGNMENT_SUMMARY_JAN_19_2026.md` - Summary of alignment work
- `ATOMIC_VALIDATION_DEPLOYMENT_JAN_19_2026.md` - Validation plan

### **In This Directory**:
- `BONDING_TESTS_README.md` - Bonding test documentation
- `archive/outdated_atomic_patterns/README.md` - Archived graph documentation

---

## 🎊 STATUS

**Current State**: ✅ Clean and aligned with THREE ATOMIC PATTERNS

**Correct Graphs**: 19 operational graphs (health checks, shutdowns, bonding tests, UI, tests)

**Gold Standard**: `nest_deploy.toml` (use as template for all new graphs)

**Archived**: 15 outdated/incorrect graphs (safely archived with documentation)

**Ready For**: 
- NUCLEUS validation deployment
- New atomic deployments following correct patterns
- Production use with confidence

---

**Last Cleanup**: January 19, 2026 (Evening)  
**Status**: ✅ Aligned with THREE ATOMIC PATTERNS  
**Gold Standard**: `nest_deploy.toml`  
**Next**: Deploy NUCLEUS using correct atomic patterns!

🔬📊✨ **Clean graphs, correct patterns, ready for deployment!** ✨📊🔬

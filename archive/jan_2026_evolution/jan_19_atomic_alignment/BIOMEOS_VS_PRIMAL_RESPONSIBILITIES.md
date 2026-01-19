# 🧬 biomeOS vs Primal Responsibilities

**Date**: January 12, 2026  
**Status**: Comprehensive Analysis  
**Goal**: Clarify what belongs to biomeOS vs external primals

---

## 🎯 **TL;DR - The Split**

### **biomeOS OWNS** (Orchestration)
- ✅ neuralAPI (Graph orchestration engine)
- ⏳ NUCLEUS (Secure discovery & federation)
- ⏳ liveSpore (Portable deployment engine)
- ✅ Atomic deployment coordination
- ✅ Graph execution engine
- ⏳ Federation protocol
- ⏳ JSON-RPC service orchestration

### **Primals IMPLEMENT** (Capabilities)
- BearDog → Encryption & crypto
- Songbird → Discovery & topology
- ToadStool → Compute & GPU
- NestGate → Storage & persistence
- petalTongue → UI & visualization
- Squirrel → AI & MCP integration

---

## 🧠 **neuralAPI** (100% biomeOS)

**Status**: ✅ Core engine implemented, ⏳ JSON-RPC server needed

**What It Is**: Graph-based orchestration for coordinating primal deployments

**biomeOS Implements**:
- ✅ Graph parsing (TOML)
- ✅ Topological sorting
- ✅ Parallel execution
- ✅ Node executors
- ✅ Dependency resolution
- ⏳ **JSON-RPC server** (for petalTongue view 6)

**Files**:
- ✅ `crates/biomeos-atomic-deploy/src/neural_executor.rs`
- ✅ `crates/biomeos-atomic-deploy/src/neural_graph.rs`
- ✅ `graphs/genetic_lineage_full_nucleus.toml`
- ⏳ Need: `crates/biomeos-neural-api/` (JSON-RPC server)

**Integration**: petalTongue view 6 needs these endpoints:
```json
neural_api.list_graphs
neural_api.get_execution_status
```

---

## 🔐 **NUCLEUS** (100% biomeOS Core)

**Status**: ⏳ Not yet implemented (specs complete)

**What It Is**: Secure 5-layer discovery + Full atomic deployment

**biomeOS Implements**:
- ⏳ 5-layer discovery (Local, USB, Network, Federation, External)
- ⏳ Trust matrix & verification
- ⏳ Genetic lineage tracking
- ⏳ Atomic coordination (Tower + Node + Nest)
- ⏳ Federation protocol
- ⏳ **JSON-RPC server** (for petalTongue view 7)

**Long-Term Goal** 🌟:
> **NUCLEUS can deploy another NUCLEUS**
> - Self-replicating deployment
> - LiveSpore USB bootability (PopOS-like)
> - Run from USB, install to disk, OR run on top of OS
> - Each NUCLEUS has genetic lineage

**Files**:
- ⏳ Need: `crates/biomeos-nucleus/` (all of it)
- ✅ `specs/NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md` (spec ready)

**Integration**: petalTongue view 7 needs:
```json
nucleus.get_discovery_layers
nucleus.get_trust_matrix
```

---

## 🌱 **liveSpore** (100% biomeOS Orchestrator)

**Status**: ✅ Partial (seed management), ⏳ Main orchestrator needed

**What It Is**: Portable deployment engine for USB/install/on-top-of-OS modes

**biomeOS Implements**:
- ✅ DeploymentMode detection (`biomeos-core`)
- ✅ Seed derivation (`biomeos-spore`)
- ⏳ Adaptive socket paths
- ⏳ OS detection & isolation
- ⏳ Capability delegation to primals
- ⏳ **JSON-RPC server** (for petalTongue view 8)

**Capability Delegation**:
```
biomeOS orchestrates
  ↓
BearDog: crypto.* (seed, encrypt, verify)
  ↓
Songbird: discovery.* (local, usb, network)
  ↓
ToadStool: hardware.* (detect, validate, gpu)
  ↓
NestGate: storage.* (partition, bootloader, install)
  ↓
petalTongue: installer.ui (progress, user input)
```

**Files**:
- ✅ `crates/biomeos-spore/` (seed management)
- ✅ `crates/biomeos-core/src/deployment_mode.rs`
- ⏳ Need: `crates/biomeos-livespore/` (main orchestrator)
- ✅ `specs/LIVESPORE_ARCHITECTURE_SPEC.md` (990 lines)

**Integration**: petalTongue view 8 needs:
```json
livespore.list_deployments
livespore.get_node_status
```

---

## 🌸 **petalTongue** (Primal → biomeOS Integration)

**Status**: ✅ Binaries harvested, ⏳ Waiting for biomeOS endpoints

**Primal Implements** (petalTongue owns):
- ✅ All 8 view rendering
- ✅ Keyboard navigation
- ✅ ASCII art visualization
- ✅ Socket discovery
- ✅ JSON-RPC client

**biomeOS Must Implement** (Integration endpoints):
- ⏳ neuralAPI JSON-RPC server (view 6)
- ⏳ NUCLEUS JSON-RPC server (view 7)
- ⏳ liveSpore JSON-RPC server (view 8)

**Priority**: HIGH (enables full TUI functionality)

---

## 🐿️ **Squirrel** (Primal → biomeOS Integration)

**Status**: ✅ Implemented in biomeos-graph, ⏳ Binary needs harvesting

**Primal Implements** (Squirrel owns):
- ✅ MCP server (stdio/SSE)
- ✅ Tool definitions
- ✅ Context management

**biomeOS Implements** (Integration):
- ✅ AI Graph Advisor
- ✅ Graph modification
- ✅ Template system
- ⏳ Live deployment AI decisions

**Priority**: MEDIUM (enhances but not required)

---

## 📊 **biomeOS Remaining Work**

### **Immediate (This Week) - 10-14 hours**

1. **neuralAPI JSON-RPC Server** (4-6h)
   - Create `crates/biomeos-neural-api/`
   - Expose graph listing & execution status
   - Socket: `/run/user/<uid>/biomeos-neural-api.sock`
   - Enable petalTongue view 6

2. **Deploy Nest Atomic** (2-4h)
   - Test NestGate compliance
   - Complete 3/3 atomics

3. **Test Node Atomic** (2-3h)
   - Deploy Tower + ToadStool
   - Verify compute operations

4. **Harvest Squirrel Binary** (1-2h)
   - Build from ecoPrimals/phase1/squirrel
   - Copy to plasmidBin/primals/

### **Short-Term (Week 1-2) - 30-40 hours**

5. **NUCLEUS Core Implementation** (12-16h)
   - Create `crates/biomeos-nucleus/`
   - 5-layer discovery protocol
   - Trust matrix
   - JSON-RPC server (petalTongue view 7)

6. **liveSpore Core Implementation** (16-20h)
   - Create `crates/biomeos-livespore/`
   - DeploymentMode orchestration
   - Capability delegation
   - JSON-RPC server (petalTongue view 8)

7. **Full NUCLEUS Deployment** (4-6h)
   - Deploy Tower + Node + Nest
   - Verify genetic lineage
   - Test federation

### **Medium-Term (Week 3-4) - 50-60 hours**

8. **LiveSpore Phase 1: Runtime Adaptation** (20-24h)
   - DeploymentMode detection refinement
   - Adaptive socket paths
   - OS-specific isolation

9. **LiveSpore Phase 2: USB Boot** (24-32h)
   - Cold Spore deployment
   - Seed derivation from USB
   - Live environment setup

10. **NUCLEUS Self-Deployment** (8-12h)
    - NUCLEUS deploys NUCLEUS
    - Genetic lineage propagation

### **Long-Term (Month 2+) - 120+ hours**

11. **LiveSpore Phase 3: Installation** (40-48h)
    - Disk partitioning (NestGate)
    - Bootloader (NestGate)
    - Full OS install

12. **LiveSpore Phase 4: Sibling Spore** (32-40h)
    - Run on existing OS
    - Namespace isolation

13. **Full AI Integration** (20-24h)
    - AI-driven deployments
    - Predictive optimizations

---

## 🧹 **Specs Directory Cleanup**

### **Mark as Implemented** ✅
- `GRAPH_BASED_ORCHESTRATION_SPEC.md`
- `COLLABORATIVE_INTELLIGENCE_SPEC.md`
- `INTERACTIVE_UI_SPEC.md` (backend)

### **Ready for Implementation** 🟢
- `NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md`
- `LIVESPORE_ARCHITECTURE_SPEC.md`
- `ATOMIC_DEPLOYMENT_SYSTEM_SPEC.md` (partial)

### **Archive Candidates** 📦
- `UNIVERSAL_CONNECTOR_SPEC.md` (superseded by JSON-RPC)
- `UNIVERSAL_PARSER_ADAPTER_SPEC.md` (likely obsolete)
- `BOOTSTRAP_ORCHESTRATION_SEQUENCE.md` (superseded)
- `boot-observability.md` (review needed)

### **Primal-Owned** 🌸
- `PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md`
- `TOADSTOOL_BIOMEOS_UNIFICATION_SPEC.md`

---

## 🎯 **Next Actions (Priority Order)**

1. **neuralAPI JSON-RPC Server** → Enable petalTongue view 6
2. **Deploy Nest Atomic** → Complete atomic trio
3. **NUCLEUS Core** → Enable federation & discovery
4. **liveSpore Core** → Enable portable deployment
5. **NUCLEUS Self-Deploy** → Achieve replication goal

---

**Different orders of the same architecture.** 🍄🐸


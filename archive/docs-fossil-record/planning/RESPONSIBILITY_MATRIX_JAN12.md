# 🧬 biomeOS vs Primal Responsibility Matrix

**Date**: January 12, 2026  
**Status**: Comprehensive Analysis  
**Goal**: Clarify what belongs to biomeOS vs external primals

---

## 🎯 **TL;DR - Who Does What**

### **biomeOS OWNS** (Orchestration)
- neuralAPI (Graph orchestration)
- NUCLEUS (Secure discovery & federation)
- liveSpore (Portable deployment engine)
- Atomic deployment coordination
- Graph execution engine
- Federation protocol
- JSON-RPC service orchestration

### **Primals IMPLEMENT** (Capabilities)
- BearDog → Encryption & crypto operations
- Songbird → Discovery & topology
- ToadStool → Compute & GPU
- NestGate → Storage & persistence
- petalTongue → UI & visualization
- Squirrel → AI & MCP integration

---

## 📊 **Complete Responsibility Breakdown**

### **🧠 neuralAPI** (100% biomeOS)

**What It Is**: Graph-based orchestration engine for coordinating primal deployments and workflows

**biomeOS Implements**:
- ✅ Graph definition & parsing (TOML)
- ✅ Topological sorting (Kahn's algorithm)
- ✅ Parallel execution engine
- ✅ Node executors (filesystem, crypto, primal, health, lineage)
- ✅ Dependency resolution
- ✅ Environment variable management
- ✅ Output propagation
- ✅ Status tracking & reporting
- ✅ Rollback hooks
- ⏳ JSON-RPC server (for petalTongue view 6)

**Primals Used By neuralAPI**:
- Songbird → Primal discovery
- BearDog → Cryptographic operations
- ToadStool → Compute-heavy graph nodes
- NestGate → Graph state persistence
- petalTongue → Graph visualization & management UI
- Squirrel → AI-driven graph optimization

**Files**:
- `crates/biomeos-atomic-deploy/src/neural_executor.rs` (✅ Implemented)
- `crates/biomeos-atomic-deploy/src/neural_graph.rs` (✅ Implemented)
- `graphs/genetic_lineage_full_nucleus.toml` (✅ Example graph)
- ⏳ `crates/biomeos-neural-api/` (to create JSON-RPC server)

---

### **🔐 NUCLEUS** (100% biomeOS Core + Primal Integration)

**What It Is**: Secure 5-layer discovery protocol + Full atomic deployment

**biomeOS Implements**:
- ⏳ 5-layer discovery protocol (Local, USB, Network, Federation, External)
- ⏳ Trust matrix & verification
- ⏳ Genetic lineage tracking
- ⏳ Atomic coordination (Tower + Node + Nest)
- ⏳ Federation protocol
- ⏳ JSON-RPC server (for petalTongue view 7)

**Primals Used By NUCLEUS**:
- **Songbird** → Local discovery (layer 1)
- **BearDog** → Encryption & lineage verification
- **ToadStool** → Network scanning (layer 3)
- **NestGate** → Trust matrix persistence
- **petalTongue** → Discovery visualization

**Long-Term Goal** 🌟:
> **NUCLEUS can deploy another NUCLEUS**
> - Self-replicating deployment capability
> - LiveSpore with PopOS-like USB bootability
> - Can run from USB, install to disk, or run on top of existing OS
> - Each deployed NUCLEUS is a full biomeOS instance with genetic lineage

**Files**:
- ⏳ `crates/biomeos-nucleus/` (to create)
- ✅ `specs/NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md` (spec complete)
- ✅ `specs/COMPLETE_ECOSYSTEM_NUCLEUS_INTEGRATION.md` (spec complete)

---

### **🌱 liveSpore** (100% biomeOS Orchestrator + Primal Delegation)

**What It Is**: Portable deployment engine for running biomeOS from USB, installing, or running on top of other OSes

**biomeOS Implements**:
- ⏳ DeploymentMode detection (Cold/Live/Sibling)
- ✅ Seed derivation & genetic lineage (biomeos-spore)
- ⏳ Adaptive socket path configuration
- ⏳ OS detection & isolation level management
- ⏳ Atomic deployment orchestration
- ⏳ Capability delegation to primals
- ⏳ JSON-RPC server (for petalTongue view 8)

**Primals Used By liveSpore**:
- **BearDog** → Seed derivation, encryption, secure boot verification
- **Songbird** → Runtime discovery of available primals
- **ToadStool** → Hardware detection, compute resource management
- **NestGate** → Bootloader, partition management, persistence
- **petalTongue** → Installation UI, live monitoring
- **Squirrel** → AI-driven deployment decisions

**Capability Delegation** (From LIVESPORE_PRIMAL_RESPONSIBILITIES.md):
```
biomeOS: Orchestrate deployment workflow
  ↓
BearDog: crypto.* operations (seed, encrypt, verify)
  ↓
Songbird: discovery.* operations (local, usb, network)
  ↓
ToadStool: hardware.* operations (detect, validate, gpu)
  ↓
NestGate: storage.* operations (partition, bootloader, install)
  ↓
petalTongue: installer.ui (progress, user input)
```

**Files**:
- ✅ `crates/biomeos-spore/` (seed management - implemented)
- ✅ `crates/biomeos-core/src/deployment_mode.rs` (mode detection - implemented)
- ⏳ `crates/biomeos-livespore/` (main orchestrator - to create)
- ✅ `specs/LIVESPORE_ARCHITECTURE_SPEC.md` (990-line spec complete)
- ✅ `specs/LIVESPORE_PRIMAL_RESPONSIBILITIES.md` (delegation matrix complete)

---

### **🌸 petalTongue Integration** (Primal → biomeOS Integration)

**What It Is**: Rich TUI for managing biomeOS, neuralAPI, NUCLEUS, and liveSpore

**Status**: ✅ Binaries harvested, 5/8 views working, 3/8 views need biomeOS endpoints

**Primal Implements** (petalTongue owns):
- ✅ All 8 view rendering
- ✅ Keyboard navigation
- ✅ ASCII art visualization
- ✅ Color-coded logs
- ✅ Graceful degradation
- ✅ Socket discovery

**biomeOS Implements** (Integration endpoints):
- ⏳ neuralAPI JSON-RPC server (view 6)
  - `neural_api.list_graphs`
  - `neural_api.get_execution_status`
  
- ⏳ NUCLEUS JSON-RPC server (view 7)
  - `nucleus.get_discovery_layers`
  - `nucleus.get_trust_matrix`
  
- ⏳ liveSpore JSON-RPC server (view 8)
  - `livespore.list_deployments`
  - `livespore.get_node_status`

**Files**:
- ✅ `plasmidBin/primals/petaltongue` (binary harvested)
- ⏳ `crates/biomeos-neural-api/src/jsonrpc_server.rs` (to create)
- ⏳ `crates/biomeos-nucleus/src/jsonrpc_server.rs` (to create)
- ⏳ `crates/biomeos-livespore/src/jsonrpc_server.rs` (to create)

---

### **🐿️ Squirrel (AI/MCP) Integration** (Primal → biomeOS Integration)

**What It Is**: AI-powered Model Context Protocol (MCP) server for adaptive intelligence

**Primal Implements** (Squirrel owns):
- ✅ MCP server (stdio/SSE protocols)
- ✅ Tool definitions & execution
- ✅ Prompt templates
- ✅ Context management
- ✅ Model interactions

**biomeOS Implements** (Integration points):
- ✅ AI Graph Advisor (biomeos-graph)
- ✅ Graph modification suggestions
- ✅ Template system (NestGate + Squirrel)
- ⏳ Live deployment AI decisions
- ⏳ Adaptive graph optimization
- ⏳ Predictive failure detection

**Files**:
- ✅ `crates/biomeos-graph/src/advisor.rs` (AI advisor implemented)
- ✅ `crates/biomeos-graph/src/modification.rs` (graph modification)
- ✅ `examples/neural_api_ai_advisor.rs` (working example)
- `plasmidBin/primals/squirrel` (binary - needs harvesting)

---

## 🗂️ **Specs Directory Cleanup Analysis**

### **✅ Fully Implemented Specs** (Keep, mark complete)
1. `GRAPH_BASED_ORCHESTRATION_SPEC.md` - ✅ Implemented
2. `COLLABORATIVE_INTELLIGENCE_SPEC.md` - ✅ Implemented
3. `INTERACTIVE_UI_SPEC.md` - ✅ Backend done (waiting petalTongue)

### **🟢 Ready for Implementation** (biomeOS owns)
1. `NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md` - 🟢 Spec ready
2. `LIVESPORE_ARCHITECTURE_SPEC.md` - 🟢 Spec ready (990 lines)
3. `LIVESPORE_PRIMAL_RESPONSIBILITIES.md` - 🟢 Delegation matrix ready
4. `ATOMIC_DEPLOYMENT_SYSTEM_SPEC.md` - 🟢 Partially implemented (Tower ✅, Node ✅, Nest ⏳)

### **📚 Reference/Planning** (Keep for long-term)
1. `ARCHITECTURE_OVERVIEW.md` - Core reference
2. `CORE_NICHE_SPEC.md` - Core concepts
3. `ENCRYPTION_STRATEGY_SPEC.md` - Security planning
4. `UNIVERSAL_FEDERATION_SPEC.md` - Federation planning
5. `BYOB_*.md` - Future build system
6. `CRYPTO_LOCK_*.md` - Future crypto lock system
7. `PRIMAL_INTEGRITY_MONITOR.md` - Future monitoring

### **🔄 Needs Primal Implementation** (Delegate)
1. `PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md` - petalTongue owns
2. `TOADSTOOL_BIOMEOS_UNIFICATION_SPEC.md` - ToadStool integration (needs review)

### **🧹 Potentially Outdated/Redundant** (Review & Archive)
1. `UNIVERSAL_CONNECTOR_SPEC.md` - May be superseded by JSON-RPC
2. `UNIVERSAL_PARSER_ADAPTER_SPEC.md` - May be obsolete
3. `BOOTSTRAP_ORCHESTRATION_SEQUENCE.md` - May be superseded by neuralAPI
4. `boot-observability.md` - May be obsolete
5. `MANIFEST_SPEC_V1.md` - Check if still relevant
6. `SOURCE_MANAGEMENT_SYSTEM.md` - Check if still relevant

---

## 🎯 **biomeOS Remaining Work (Priority Order)**

### **Immediate (This Week)**
1. ⏳ **Implement neuralAPI JSON-RPC Server**
   - Create `crates/biomeos-neural-api/`
   - Expose graph listing and execution status
   - Socket: `/run/user/<uid>/biomeos-neural-api.sock`
   - Enable petalTongue view 6
   - **Estimate**: 4-6 hours

2. ⏳ **Deploy Nest Atomic (Tower + NestGate)**
   - Test NestGate Unix socket compliance
   - Verify JSON-RPC API
   - Complete 3/3 atomic deployments
   - **Estimate**: 2-4 hours

3. ⏳ **Test Node Atomic in Production**
   - Deploy Tower + ToadStool
   - Verify compute operations
   - Document findings
   - **Estimate**: 2-3 hours

### **Short-Term (Week 1-2)**
4. ⏳ **Implement NUCLEUS Core**
   - Create `crates/biomeos-nucleus/`
   - 5-layer discovery protocol
   - Trust matrix & verification
   - JSON-RPC server (for petalTongue view 7)
   - **Estimate**: 12-16 hours

5. ⏳ **Implement liveSpore Core**
   - Create `crates/biomeos-livespore/`
   - DeploymentMode orchestration
   - Capability delegation
   - JSON-RPC server (for petalTongue view 8)
   - **Estimate**: 16-20 hours

6. ⏳ **Full NUCLEUS Deployment Test**
   - Deploy Tower + Node + Nest together
   - Verify genetic lineage
   - Test federation
   - **Estimate**: 4-6 hours

### **Medium-Term (Week 3-4)**
7. ⏳ **LiveSpore Phase 1: Runtime Adaptation**
   - DeploymentMode detection refinement
   - Adaptive socket paths
   - OS-specific isolation
   - **Estimate**: 20-24 hours

8. ⏳ **LiveSpore Phase 2: USB Boot**
   - Cold Spore deployment
   - Seed derivation from USB
   - Live environment setup
   - **Estimate**: 24-32 hours

9. ⏳ **NUCLEUS Self-Deployment**
   - NUCLEUS deploys another NUCLEUS
   - Genetic lineage propagation
   - Verification & testing
   - **Estimate**: 8-12 hours

### **Long-Term (Month 2+)**
10. ⏳ **LiveSpore Phase 3: Installation**
    - Disk partitioning (NestGate)
    - Bootloader installation (NestGate)
    - Full OS install
    - **Estimate**: 40-48 hours

11. ⏳ **LiveSpore Phase 4: Sibling Spore**
    - Run on top of existing OS
    - Namespace isolation
    - Resource sharing
    - **Estimate**: 32-40 hours

12. ⏳ **Full AI Integration**
    - Squirrel harvesting
    - AI-driven deployments
    - Predictive optimizations
    - **Estimate**: 20-24 hours

---

## 🧹 **Specs Cleanup Recommendations**

### **Action 1: Mark Implemented**
Update these specs with "✅ IMPLEMENTED" banner:
- `GRAPH_BASED_ORCHESTRATION_SPEC.md`
- `COLLABORATIVE_INTELLIGENCE_SPEC.md`
- `INTERACTIVE_UI_SPEC.md` (backend)

### **Action 2: Archive Outdated**
Move to `specs/archive/`:
- `UNIVERSAL_CONNECTOR_SPEC.md` (superseded by JSON-RPC)
- `UNIVERSAL_PARSER_ADAPTER_SPEC.md` (likely obsolete)
- `BOOTSTRAP_ORCHESTRATION_SEQUENCE.md` (superseded by neuralAPI)
- `boot-observability.md` (needs review)

### **Action 3: Create Missing Specs**
New specs needed:
- `NUCLEUS_IMPLEMENTATION_GUIDE.md` - Implementation roadmap
- `LIVESPORE_IMPLEMENTATION_PHASES.md` - Detailed phase breakdown
- `JSONRPC_SERVER_SPEC.md` - JSON-RPC server implementation guide

### **Action 4: Update README**
Update `specs/README.md`:
- Mark neuralAPI, Collaborative Intelligence as ✅ Complete
- Add petalTongue integration status
- Update LiveSpore timeline
- Add NUCLEUS self-deployment goal

---

## 🌟 **NUCLEUS Long-Term Vision**

### **The Dream: Self-Replicating NUCLEUS** 🌱

**Goal**: NUCLEUS can deploy another complete NUCLEUS instance

**Capabilities**:
1. **USB Bootability** (PopOS-style)
   - Boot from USB stick
   - Full biomeOS environment
   - Network federation with other NUCLEUS instances

2. **Installation Mode**
   - Install to new disk
   - Partition management (NestGate)
   - Bootloader setup (NestGate)
   - Full OS deployment

3. **Sibling Spore Mode**
   - Run on top of existing OS (Linux, macOS, Windows)
   - Namespace isolation
   - Resource sharing
   - No installation required

4. **Genetic Lineage**
   - Each NUCLEUS has unique genetic seed
   - Parent-child lineage tracking
   - Cryptographic family verification
   - Trust propagation

5. **Federation**
   - Multiple NUCLEUS instances discover each other
   - Share primals & capabilities
   - Distribute workloads
   - Resilient network

**Implementation Path**:
```
Phase 1: Complete NUCLEUS deployment (Tower + Node + Nest)
  ↓
Phase 2: Implement liveSpore runtime adaptation
  ↓
Phase 3: Add USB boot capability
  ↓
Phase 4: Add installation capability
  ↓
Phase 5: Add sibling spore capability
  ↓
Phase 6: NUCLEUS deploys NUCLEUS! 🎊
```

**Timeline**: ~12 weeks from atomic deployment completion

---

## 📊 **Summary Statistics**

### **Specs Analysis**
- **Total Specs**: 36
- **Implemented**: 3 (neuralAPI, Collaborative Intelligence, UI Backend)
- **Ready to Implement**: 4 (NUCLEUS, LiveSpore, neuralAPI Server, Nest Atomic)
- **Reference/Planning**: 22
- **Primal-Owned**: 2 (petalTongue UI, ToadStool unification)
- **Archive Candidates**: 5

### **Work Distribution**
- **biomeOS Owns**: ~200 hours remaining
  - neuralAPI server: 4-6h
  - NUCLEUS core: 12-16h
  - liveSpore core: 16-20h
  - LiveSpore phases: 120-150h
  - AI integration: 20-24h

- **Primal Integration**: ~40 hours
  - petalTongue JSON-RPC: ✅ Ready (waiting biomeOS)
  - Squirrel harvesting: 2-4h
  - ToadStool integration: 8-12h
  - NestGate testing: 4-6h

---

**Different orders of the same architecture.** 🍄🐸

**Next Priority**: Implement neuralAPI JSON-RPC server (4-6h) → Enable petalTongue view 6


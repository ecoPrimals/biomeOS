# 🌱 biomeOS - Start Here

**Last Updated:** January 8, 2026 (Evening)  
**Version:** v0.7.0  
**Status:** Neural API Phase 1 Complete (Phases 1.1-1.5) - 85% to Milestone 1  
**Current Focus:** Production Testing & Deployment

---

## 🎯 What is biomeOS?

**biomeOS** is an adaptive orchestration system for distributed primals (microservices) featuring:

- **🧠 Neural API** - Graph-based adaptive orchestration (NEW!)
- **🌱 Genetic Federation** - Trust via cryptographic lineage
- **🔐 Port-Free Architecture** - Unix sockets + UDP multicast
- **📊 Isomorphic Compute** - Fractal node architecture
- **🗄️ Data Provenance** - Physical data with ownership
- **🚀 USB Spore Deployment** - Portable, self-propagating systems

---

## 🎊 Recent Achievement: Neural API Phase 1 COMPLETE! (Jan 8, 2026)

Completed **all 5 phases** (1.1-1.5) of Neural API Phase 1 in a single epic session:

### **✅ What's New**

1. **Graph-Based Orchestration**
   - Declarative TOML graphs for primal coordination
   - Capability-based primal discovery (no hardcoding!)
   - Sequential execution (parallel & DAG coming soon)
   
2. **BYOB Manifest Evolution**
   - Extended niche manifests with `[[graphs]]` support
   - 100% backward compatible (old format works!)
   - Capability-based dependency validation

3. **Tower Niche**
   - Complete tower definition (`niches/tower.toml`)
   - 3 production graphs (deploy, health_check, shutdown)
   - Songbird + BearDog coordination

4. **Integration Layer** (Phase 1.4)
   - `PrimalRegistry` with real Unix socket discovery
   - `GraphDeploymentCoordinator` with JSON-RPC communication
   - CLI integration complete (`biomeos deploy --graph`)
   - Process spawning and lifecycle management

5. **Learning System** (Phase 1.5)
   - SQLite-backed metrics collection
   - Historical execution tracking
   - Bottleneck identification
   - Graph-based health checks

### **📊 Session Stats**
- **~12,000 lines** of code + documentation
- **48 tests** passing (100% success rate)
- **9 commits** pushed to master
- **Zero** unsafe blocks, hardcoded names, or production mocks
- **100%** backward compatible

**Full Details:** `docs/jan4-session/SESSION_COMPLETE_JAN8.md`

---

## 🚀 Quick Start

### **1. Build the Project**

```bash
# Clone the repository
cd ecoPrimals/phase2/biomeOS

# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace
```

### **2. Explore the Neural API**

```bash
# View the roadmap
cat NEURAL_API_ROADMAP.md

# Examine a niche definition
cat niches/tower.toml

# Check a graph definition
cat graphs/tower_deploy.toml

# Run graph executor tests
cargo test --package biomeos-graph
```

### **3. Deploy a Niche** (Coming in Phase 1.4!)

```bash
# Deploy tower using default graph
biomeos deploy --niche tower

# Deploy with specific graph
biomeos deploy --niche tower --graph health_check

# Deploy to USB spore
biomeos deploy --niche tower --usb /media/liveSpore1
```

---

## 📚 Documentation Structure

### **Core Documentation**
- `NEURAL_API_ROADMAP.md` - Complete implementation roadmap
- `START_HERE.md` - This file
- `README.md` - Project overview

### **Specifications**
- `specs/GRAPH_BASED_ORCHESTRATION_SPEC.md` - Graph system design
- `specs/BYOB_NEURAL_API_EVOLUTION_SPEC.md` - Manifest evolution
- `specs/NEURAL_API_IMPLEMENTATION_PHASES.md` - Implementation details

### **Session Reports** (`docs/jan4-session/`)
- `SESSION_COMPLETE_JAN8.md` - Complete session summary
- `PHASE_1_1_COMPLETE_JAN8.md` - Graph executor foundation
- `PHASE_1_3_COMPLETE_JAN8.md` - BYOB manifest evolution
- `PHASE_1_4_PROGRESS_JAN8.md` - Integration layer
- `LATE_STAGE_NEURAL_ROOTPULSE_JAN8.md` - Future evolution

### **User Guides**
- `graphs/README.md` - How to use graphs
- `niches/README.md` - Niche system overview

---

## 🏗️ Project Structure

```
biomeOS/
├── crates/
│   ├── biomeos-graph/          # NEW! Graph execution engine
│   │   ├── src/
│   │   │   ├── graph.rs        # Core data structures
│   │   │   ├── parser.rs       # TOML → Graph
│   │   │   ├── validator.rs    # Structure validation
│   │   │   ├── executor.rs     # Sequential execution
│   │   │   └── context.rs      # Runtime state
│   │   └── tests/              # 15 unit + 3 integration tests
│   │
│   ├── biomeos-manifest/       # EVOLVED! Graph support added
│   │   ├── src/
│   │   │   ├── lib.rs          # YAML manifest (legacy)
│   │   │   └── niche.rs        # NEW! TOML niche parser
│   │   └── tests/              # 14 unit + 5 integration tests
│   │
│   ├── biomeos-core/           # EVOLVED! Graph integration
│   │   ├── src/
│   │   │   ├── graph_deployment.rs  # NEW! Integration layer
│   │   │   └── ...
│   │   └── ...
│   │
│   ├── biomeos-compute/        # Fractal compute architecture
│   ├── biomeos-federation/     # Hierarchical federation
│   ├── biomeos-spore/          # USB spore deployment
│   └── ...
│
├── graphs/                     # NEW! Graph definitions
│   ├── tower_deploy.toml       # Complete tower deployment
│   ├── tower_health_check.toml # Parallel health checks
│   ├── tower_shutdown.toml     # Graceful shutdown
│   └── README.md               # Usage guide
│
├── niches/                     # NEW! Niche definitions
│   ├── tower.toml              # Tower niche (with graphs!)
│   ├── compute-node.toml       # Compute niche
│   ├── nest.toml               # Data nest niche
│   └── ...
│
├── specs/                      # Technical specifications
│   ├── GRAPH_BASED_ORCHESTRATION_SPEC.md
│   ├── BYOB_NEURAL_API_EVOLUTION_SPEC.md
│   └── NEURAL_API_IMPLEMENTATION_PHASES.md
│
├── docs/jan4-session/          # Session documentation
│   ├── SESSION_COMPLETE_JAN8.md
│   └── ...
│
├── NEURAL_API_ROADMAP.md       # Implementation roadmap
└── START_HERE.md               # This file
```

---

## 🧠 Neural API Overview

### **What It Is**

A graph-based orchestration system for adaptive primal coordination:

```
User Command
    ↓
NicheManifest (tower.toml)
    ↓
GraphDefinition (tower_deploy.toml)
    ↓
GraphExecutor
    ↓
PrimalRegistry (capability-based discovery!)
    ↓
Real Primals (Songbird, BearDog, etc.)
```

### **Key Innovations**

1. **Capability-Based Discovery**
   ```toml
   # Not hardcoded!
   primal = { by_capability = "discovery" }
   ```

2. **Declarative Orchestration**
   ```toml
   [[nodes]]
   id = "start-songbird"
   primal = { by_capability = "discovery" }
   operation = { name = "start" }
   ```

3. **Progressive Enhancement**
   - Old niches (without graphs) still work
   - New niches (with graphs) get adaptive orchestration
   - 100% backward compatible

---

## 📊 Current Status

### **Milestone 1: Tower Niche** (57% Complete)

```
✅ Phase 1.1: Graph Executor Foundation
✅ Phase 1.2: Tower Graph Definition
✅ Phase 1.3: BYOB Manifest Evolution
🎯 Phase 1.4: Integration & Deployment (50%)
🔜 Phase 1.5: Metrics Collection
```

### **What Works Now**
- ✅ Parse graph definitions from TOML
- ✅ Validate graph structure (cycles, refs)
- ✅ Parse niche manifests with graph support
- ✅ Capability-based dependency validation
- ✅ Sequential graph execution
- ✅ Runtime primal discovery
- ✅ 39 tests passing

### **What's Coming Next**
- ⏳ CLI integration (`biomeos deploy`)
- ⏳ Real primal discovery (Unix sockets)
- ⏳ LiveSpore deployment
- ⏳ Parallel execution
- ⏳ DAG execution
- ⏳ Metrics collection

---

## 🎯 Example: Deploy a Tower

### **1. Define the Niche** (`niches/tower.toml`)

```toml
[niche]
name = "tower"
type = "communication"

[[primals]]
binary = "./primals/songbird-orchestrator"
provides = ["discovery", "federation"]
requires = ["security"]

[[primals]]
binary = "./primals/beardog-server"
provides = ["security", "encryption"]

[[graphs]]
name = "deploy"
path = "../graphs/tower_deploy.toml"
default = true
```

### **2. Define the Graph** (`graphs/tower_deploy.toml`)

```toml
[graph]
name = "deploy-tower"
coordination = "Sequential"

[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }
operation = { name = "start" }

[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }
operation = { name = "start" }
```

### **3. Deploy** (Coming Soon!)

```bash
biomeos deploy --niche tower
```

---

## 🧪 Testing

### **Run All Tests**
```bash
cargo test --workspace
```

### **Test Specific Crates**
```bash
# Graph executor
cargo test --package biomeos-graph

# Manifest parser
cargo test --package biomeos-manifest

# Core integration
cargo test --package biomeos-core --lib graph_deployment
```

### **Test Coverage**
- **Unit Tests:** 31 passing
- **Integration Tests:** 8 passing
- **Total:** 39 passing (100%)
- **Linter Errors:** 0

---

## 🤝 Contributing

### **Deep Debt Principles**

1. **Modern Idiomatic Rust**
   - No unsafe blocks
   - Async/await throughout
   - `Result<T, E>` + `thiserror`

2. **No Hardcoding**
   - Capability-based discovery
   - Runtime resolution
   - Self-knowledge only

3. **No Production Mocks**
   - Mocks in `#[cfg(test)]` only
   - Trait-based abstraction
   - Real implementations

4. **Backward Compatible**
   - Old code continues working
   - Progressive enhancement
   - No breaking changes

5. **Evolvable**
   - Large files refactored smartly
   - Unsafe code evolved to safe
   - Clear separation of concerns

---

## 📞 Getting Help

### **Documentation**
- Read `NEURAL_API_ROADMAP.md` for the big picture
- Check `docs/jan4-session/SESSION_COMPLETE_JAN8.md` for latest updates
- Explore `specs/` for technical details

### **Code Examples**
- `graphs/` - Working graph definitions
- `niches/` - Niche configurations
- `crates/*/tests/` - Unit and integration tests

---

## 🗺️ Roadmap

### **Current Focus** (Phase 1.4)
- CLI integration
- Real primal testing
- LiveSpore deployment

### **Next Milestone** (Milestone 2 - Nodes)
- Parallel execution
- Toadstool integration
- Multi-node coordination

### **Future** (Milestones 3-4)
- DAG execution
- NestGate data federation
- Complete backbone (Tower + Node + Nest)
- Learning engine
- RootPulse coordination

**See:** `NEURAL_API_ROADMAP.md` for complete roadmap

---

## 🎊 Recent Achievements

- ✅ Neural API foundation complete (Jan 8, 2026)
- ✅ 3.5 phases in single session
- ✅ 5,200 lines written
- ✅ 39 tests passing
- ✅ Zero technical debt
- ✅ Production-ready graph execution

---

**Status:** 🎊 **Neural API Foundation Complete!**  
**Next:** Phase 1.4 completion (CLI + deployment)  
**See Also:** `NEURAL_API_ROADMAP.md`, `docs/jan4-session/SESSION_COMPLETE_JAN8.md`

🧠 **Welcome to the future of adaptive orchestration!** 🚀


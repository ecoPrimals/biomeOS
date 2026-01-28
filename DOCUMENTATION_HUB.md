# 📚 biomeOS Documentation Hub

**Last Updated**: January 28, 2026 (Final)  
**Status**: Production Ready - 93% TLS Validation  
**Version**: Tower Atomic + NUCLEUS Lifecycle + Protocol Escalation Phase 1  
**Tests**: 400+ passing (153 in atomic-deploy) | **Crates**: 21 | **Lines**: ~108k | **Unsafe**: 0 | **TODOs**: 3

Welcome to the biomeOS documentation hub! This is your central navigation point.

---

## 🚀 **START HERE**

### **New to biomeOS?**
1. **[START_HERE.md](./START_HERE.md)** ⭐ - Quick introduction & status
2. **[README.md](./README.md)** - Project overview
3. **[QUICK_START.md](./QUICK_START.md)** - Get up and running

### **Ready to Deploy?**
4. **[deploy_tower_atomic.sh](./deploy_tower_atomic.sh)** - One-command deployment
5. **[DEPLOYMENT.md](./DEPLOYMENT.md)** - Deployment guide

---

## 🏆 **CURRENT STATUS** (Jan 28, 2026)

| Metric | Value |
|--------|-------|
| **TLS 1.3 Validation** | 93% (81/87 sites) |
| **Web Compatibility** | 96% |
| **Cipher Suites** | 100% (all 3 mandatory) |
| **Pure Rust** | 100% |
| **Test Suites** | 106 |
| **Tests Passing** | 400+ |

### Key Commits
| Component | Commit | Feature |
|-----------|--------|---------|
| biomeOS | `75b88ee` | Protocol Escalation + Living Graph ⭐ |
| BearDog | `964babd25` | SHA-384 complete |
| Songbird | `f6cb661b4` | v8.14.0 - HTTP headers, dual-mode |
| Squirrel | `28e59176` | biomeOS integration fixes |

### New in This Release
- **Protocol Escalation** - Living Graph + JSON-RPC → tarpc ⭐
- **10 New JSON-RPC Methods** - `protocol.*` and `graph.protocol_map`
- **Automated Bootstrap** - `scripts/bootstrap_tower_atomic.sh`
- **NUCLEUS Lifecycle** - Germination through Apoptosis
- **Socket Discovery** - Capability-based resolution

---

## 🏗️ **ARCHITECTURE & DESIGN**

### **Core Architecture**
- **[BIOMEOS_ATOMICS_ARCHITECTURE.md](./BIOMEOS_ATOMICS_ARCHITECTURE.md)** - Atomic deployment
- **[BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md](./BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md)** - Neural API
- **[TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](./TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)** - Zero coupling

### **Integration Standards**
- **[BIOMEOS_PRIMAL_INTEGRATION_SPEC.md](./BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)** - Primal integration
- **[WATERINGHOLE_INTEGRATION.md](./WATERINGHOLE_INTEGRATION.md)** - Standards
- **[SEMANTIC_EVOLUTION_STRATEGY.md](./SEMANTIC_EVOLUTION_STRATEGY.md)** - Naming conventions

### **Evolution Roadmap**
- **[PROTOCOL_ESCALATION_ROADMAP.md](./PROTOCOL_ESCALATION_ROADMAP.md)** ⭐ - JSON-RPC → tarpc Living Graph
- **[RUST_EVOLUTION_ROADMAP.md](./RUST_EVOLUTION_ROADMAP.md)** - Scripts → Pure Rust
- **[docs/handoffs/PRIMAL_TARPC_EVOLUTION_HANDOFF.md](./docs/handoffs/PRIMAL_TARPC_EVOLUTION_HANDOFF.md)** - tarpc guide for all primals
- **[docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF.md](./docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF.md)** - HTTP headers complete
- **[INFRASTRUCTURE_EVOLUTION.md](./INFRASTRUCTURE_EVOLUTION.md)** - Terraria, Apoptosis

---

## 📖 **SPECIFICATIONS**

### **Core Specs** (`specs/`)
- **[specs/README.md](./specs/README.md)** ⭐ - All specifications index
- **[specs/LIVESPORE_IMPRINTING_SPEC.md](./specs/LIVESPORE_IMPRINTING_SPEC.md)** - 64-byte seeds
- **[specs/BIRDSONG_DARK_FOREST_TRUST_MODEL.md](./specs/BIRDSONG_DARK_FOREST_TRUST_MODEL.md)** - Encrypted discovery
- **[specs/NUCLEUS_DEPLOYMENT_SPEC.md](./specs/NUCLEUS_DEPLOYMENT_SPEC.md)** - Tower/Node/Nest
- Neural API Routing
- Security & Federation

### **Lifecycle Specs** (`specs/lifecycle/`)
- Bootstrap Mode
- Neural API Nucleation
- Primal Lifecycle (Germination → Apoptosis)

### **LiveSpore System**
- **[graphs/livespore_create.toml](./graphs/livespore_create.toml)** - Neural API imprinting
- **[graphs/livespore_validate.toml](./graphs/livespore_validate.toml)** - System validation
- **[scripts/validate_spore.sh](./scripts/validate_spore.sh)** - Portable validator

---

## 📄 **NEW DOCUMENTATION**

### **Lifecycle Management**
- **[docs/LIFECYCLE_MANAGEMENT.md](./docs/LIFECYCLE_MANAGEMENT.md)** ⭐ - NUCLEUS lifecycle API
  - Primal states: Germinating, Incubating, Active, Degraded, Apoptosis
  - Health monitoring & auto-resurrection
  - Dependency-aware shutdown

### **Socket Discovery**
- **[docs/SOCKET_DISCOVERY.md](./docs/SOCKET_DISCOVERY.md)** ⭐ - Capability-based resolution
  - No hardcoded `/tmp` paths
  - Multi-strategy discovery (env, XDG, family-scoped, Neural API)
  - Caching with TTL

### **Team Handoffs**
- **[docs/handoffs/PRIMAL_TARPC_EVOLUTION_HANDOFF.md](./docs/handoffs/PRIMAL_TARPC_EVOLUTION_HANDOFF.md)** ⭐ - tarpc guide for all primals
- **[docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF.md](./docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF.md)** - HTTP headers complete
- **[docs/handoffs/SONGBIRD_LAN_DISCOVERY_HANDOFF.md](./docs/handoffs/SONGBIRD_LAN_DISCOVERY_HANDOFF.md)** - LAN discovery
- **[docs/handoffs/SQUIRREL_EVOLUTION_HANDOFF.md](./docs/handoffs/SQUIRREL_EVOLUTION_HANDOFF.md)** - AI primal

---

## 🗂️ **ARCHIVE & HISTORY**

### **Recent Sessions** (`archive/`)
| Archive | Focus |
|---------|-------|
| `session_jan_28_2026_lifecycle_tests/` ⭐ | NUCLEUS lifecycle, concurrent tests |
| `session_jan_27_2026_deep_debt_final/` | Deep debt complete (85→3 TODOs) |
| `session_jan_26_2026_tls_analysis/` | TLS validation (87 sites) |
| `session_jan_26_2026_tower_atomic/` | Tower Atomic integration |
| `session_jan_25_2026_complete/` | capability.call |

### **Historical Archives**
- `docs-fossil-record/` - 198 files
- `specs-fossil-record/` - 11 files
- `sessions/` - 124 files

**Note**: All archives preserved as "fossil record".

---

## 🎯 **NAVIGATION BY TASK**

### **I want to...**

| Task | Document |
|------|----------|
| **Deploy Tower Atomic** | `./deploy_tower_atomic.sh` |
| **Deploy LiveSpore (USB)** | `scripts/validate_spore.sh --update` |
| **Understand Architecture** | `BIOMEOS_ATOMICS_ARCHITECTURE.md` |
| **Manage Primal Lifecycle** | `docs/LIFECYCLE_MANAGEMENT.md` |
| **Discover Sockets** | `docs/SOCKET_DISCOVERY.md` |
| **Create Genetic Lineage** | `specs/LIVESPORE_IMPRINTING_SPEC.md` |
| **Enable Dark Forest** | `specs/BIRDSONG_DARK_FOREST_TRUST_MODEL.md` |
| **Evolve Scripts to Rust** | `RUST_EVOLUTION_ROADMAP.md` |
| **Review Standards** | `../wateringHole/` |

---

## 📊 **KEY CONCEPTS**

| Concept | Description |
|---------|-------------|
| **TRUE PRIMAL** | Zero coupling via capability.call |
| **Tower Atomic** | BearDog + Songbird Pure Rust TLS |
| **Neural API** | Universal router & orchestrator |
| **Living Graph** ⭐ | Runtime protocol state tracking |
| **Protocol Escalation** ⭐ | JSON-RPC → tarpc based on metrics |
| **NUCLEUS** | Lifecycle management (resurrection, apoptosis) |
| **ecoBin** | UniBin + Pure Rust (no C deps) |
| **LiveSpore** | Portable USB deployment with genetic lineage |
| **Dark Forest** | Encrypted discovery (only family sees beacons) |
| **64-byte Seed** | `[genesis:32] + [node_key:32]` structure |
| **Vault** | Spore-specific personal data (preserved across updates) |
| **Socket Discovery** | Capability-based socket resolution |

---

## 🔍 **SEARCH TIPS**

- **Architecture**: Root `.md` files, `docs/`
- **Specifications**: `specs/` directory
- **Standards**: `../wateringHole/`
- **History**: `archive/` subdirectories
- **Deployment**: `graphs/`, `deployments/`
- **Handoffs**: `docs/handoffs/`

---

**Status**: ✅ Production Ready | **TLS**: 93% | **Pure Rust**: 100% | **Tests**: 400+

*For quick start, see [START_HERE.md](./START_HERE.md)*

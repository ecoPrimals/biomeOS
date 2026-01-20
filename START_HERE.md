# biomeOS - Start Here

**Version**: v0.28.0  
**Date**: January 20, 2026  
**Status**: 🎯 Production Ready - Deep Debt Evolution Mode

---

## 🚀 Quick Start

### Deploy Tower Atomic + Squirrel (AI Routing Stack)

```bash
# Navigate to biomeOS
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Set Anthropic API key
export ANTHROPIC_API_KEY="sk-ant-api03-..."

# Deploy full stack (BearDog + Songbird + Squirrel)
./scripts/deploy_tower_squirrel_manual.sh nat0

# Verify deployment
ls -lh /tmp/*-nat0.sock
```

**That's it!** You now have a secure AI routing mesh running.

---

## 📚 Essential Documentation

### Start Here
- **This File** - Main entry point
- **[README.md](README.md)** - Project overview and goals
- **[QUICK_START.md](QUICK_START.md)** - Deployment quickstart

### Core Architecture (MUST READ)
- **[BONDING_MODEL_CORRECTION_JAN_20_2026.md](BONDING_MODEL_CORRECTION_JAN_20_2026.md)** - ⭐ Critical architectural model
  - Ecological interactions (within system)
  - Chemical bonding (between systems)
  - Routing priorities and trust levels

- **[BIOMEOS_ATOMICS_ARCHITECTURE.md](BIOMEOS_ATOMICS_ARCHITECTURE.md)** - Atomic patterns
  - Tower Atomic (BearDog + Songbird)
  - Node Atomic (Tower + ToadStool)
  - Nest Atomic (Tower + NestGate)

- **[TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)** - Port-free design

### Deployment
- **[TOWER_ATOMIC_READY_JAN_20_2026.md](TOWER_ATOMIC_READY_JAN_20_2026.md)** - Tower deployment guide
- **[DEPLOYMENT_SYSTEM_EVOLUTION_PLAN_JAN_20_2026.md](DEPLOYMENT_SYSTEM_EVOLUTION_PLAN_JAN_20_2026.md)** - Evolution roadmap
- **[QUICK_START_TOWER_DEPLOYMENT.md](QUICK_START_TOWER_DEPLOYMENT.md)** - Quick deployment

### Session Summary
- **[SESSION_SUMMARY_TOWER_ATOMIC_JAN_20_2026.md](SESSION_SUMMARY_TOWER_ATOMIC_JAN_20_2026.md)** - Comprehensive session summary

### Standards
- **[GENOMEBIN_ARCHITECTURE_STANDARD.md](GENOMEBIN_ARCHITECTURE_STANDARD.md)** - genomeBin specification
- **[wateringHole/](../../wateringHole/)** - All ecoPrimals standards

---

## 🎯 What Is biomeOS?

**biomeOS** is the deployment and orchestration layer for **ecoPrimals** - Pure Rust microservices with genetic bonding.

### Key Concepts

#### 1. **Primals** (Ecological Layer)
Individual Pure Rust binaries (UniBin/ecoBin):
- **BearDog** - Security (crypto, keys, birdsong)
- **Songbird** - Discovery (service mesh, HTTP gateway)
- **Squirrel** - AI Orchestration (multi-provider routing)
- **ToadStool** - Compute Orchestration
- **NestGate** - Storage Orchestration

**Within a system**, primals have **ecological interactions**:
- Symbiotic (mutual benefit)
- Mutualistic (cooperation)
- Competitive (resource sharing)

#### 2. **Systems** (Molecular Layer)
Collections of primals that interact ecologically:
```
System: Basement HPC
├── BearDog ──┐
├── Songbird ─┤ Ecological interactions
├── ToadStool ─┤ (within same environment)
├── NestGate ──┤
└── Squirrel ──┘

Presents as: Covalent (to other systems)
```

#### 3. **Chemical Bonding** (System-to-System)
How systems interact with OTHER systems:

**Covalent** (High Trust, Shared Resources):
- Your HPC ↔ Friend's HPC
- Free sharing of compute/storage
- Direct peer-to-peer
- High bandwidth, low latency

**Ionic** (Contract-Based, Metered):
- Your HPC ↔ Cloud provider
- Pay per use
- Contractual SLA
- Lower trust (validate responses)

**Metallic** (Specialized Pools):
- Cloud's internal GPU farm
- Storage cluster optimization
- Electron sea (dynamic work sharing)

**Weak** (Discovery Only):
- Public service discovery
- No trust assumptions
- Transient connections

---

## 🧬 Routing Priority

**Ecological First** (same system, free, instant):
```
Squirrel needs compute → Check local ToadStool first
```

**Covalent Second** (trusted system, free, fast):
```
Local ToadStool busy → Check friend's HPC ToadStool
```

**Ionic Third** (contract system, metered, costs):
```
All covalent busy → Use cloud ToadStool ($$)
```

**This is automatic** - primals don't need to know!

---

## 📁 Project Structure

```
biomeOS/
├── crates/
│   ├── biomeos-atomic-deploy/    # Neural API, graph execution
│   ├── biomeos-core/              # Core abstractions
│   ├── neural-api-client/         # Client library for primals
│   └── ...
├── scripts/
│   ├── deploy_tower_atomic_manual.sh        # Deploy Tower
│   └── deploy_tower_squirrel_manual.sh      # Deploy Tower + Squirrel
├── graphs/                        # Deployment graphs
├── plasmidBin/                    # Production ecoBins
│   └── primals/
│       ├── beardog/
│       ├── songbird/
│       └── squirrel/
├── specs/                         # Specifications
├── docs/                          # Additional documentation
└── archive/                       # Archived documentation
```

---

## 🔧 Development Workflow

### 1. Build biomeOS
```bash
cargo build --release
```

### 2. Deploy Primals
```bash
# Tower Atomic only
./scripts/deploy_tower_atomic_manual.sh nat0

# Tower + Squirrel (AI)
export ANTHROPIC_API_KEY="sk-ant-..."
./scripts/deploy_tower_squirrel_manual.sh nat0
```

### 3. Verify Deployment
```bash
# Check sockets
ls -lh /tmp/*-nat0.sock

# Check processes
ps aux | grep -E "(beardog|songbird|squirrel)" | grep nat0

# Check logs
tail -f /tmp/beardog-nat0.log
tail -f /tmp/songbird-nat0.log
tail -f /tmp/squirrel-nat0.log
```

### 4. Test AI Routing
```bash
# Make AI call via Squirrel
echo '{"jsonrpc":"2.0","method":"ai.chat",
  "params":{"messages":[{"role":"user","content":"Hello!"}]},
  "id":1}' | nc -U /tmp/squirrel-nat0.sock
```

---

## 🎯 Current Status

### ✅ Production Ready
- **Tower Atomic**: BearDog + Songbird deployment ✅
- **AI Routing**: Squirrel + Tower integration ✅
- **Genetic Bonding**: System-level bonding model ✅
- **Pure Rust**: 100% Rust (zero C dependencies!) ✅
- **Code Quality**: Grade A (93%) - Deep debt audit complete ✅
- **ecoBins**: All harvested and ready ✅

### 🔧 Evolution Mode (Deep Debt)
- **Deployment System**: Evolving DAG execution engine
- **Bonding Primitives**: Implementing first-class bonding types
- **Graph Composition**: Subgraph support for atomic patterns
- **Multi-Environment**: System-to-system orchestration
- **Smart Refactoring**: `neural_executor.rs` plan ready (~8 hours)

**Timeline**: 6 weeks for production-ready orchestration  
**Audit**: [DEEP_DEBT_AUDIT_JAN_20_2026.md](DEEP_DEBT_AUDIT_JAN_20_2026.md) - Grade A (93%)

---

## 📊 Key Metrics

- **Primals**: 5 core (BearDog, Songbird, Squirrel, ToadStool, NestGate)
- **ecoBins**: 3 harvested (BearDog 5.1M, Songbird 16M, Squirrel 4.2M)
- **Lines of Code**: ~50K Pure Rust
- **Dependencies**: 100% Pure Rust (zero C dependencies in core)
- **Deployment Scripts**: 2 manual (pinned while DAG evolves)
- **Documentation**: ~20 files (cleaned Jan 20, 2026)

---

## 🚀 Next Steps

### For Users
1. Read [BONDING_MODEL_CORRECTION_JAN_20_2026.md](BONDING_MODEL_CORRECTION_JAN_20_2026.md)
2. Deploy Tower Atomic: `./scripts/deploy_tower_atomic_manual.sh nat0`
3. Test AI routing with Squirrel
4. Explore multi-system bonding

### For Developers
1. Review [DEPLOYMENT_SYSTEM_EVOLUTION_PLAN_JAN_20_2026.md](DEPLOYMENT_SYSTEM_EVOLUTION_PLAN_JAN_20_2026.md)
2. Implement bonding primitives (Milestone 3)
3. Build DAG Engine v2 (Milestone 4)
4. Create system-level abstractions

### For Primal Teams
- **BearDog**: No changes needed, working perfectly ✅
- **Songbird**: Fresh ecoBin harvested, ready ✅
- **Squirrel**: Neural API integration complete ✅
- **ToadStool**: Review for UniBin/ecoBin compliance
- **NestGate**: Review for UniBin/ecoBin compliance

---

## 💡 Key Insights

### 1. **Primals Have Ecological Interactions**
Within a system, primals cooperate ecologically (symbiotic, mutualistic).

### 2. **Systems Have Chemical Bonds**
Between systems, bonding determines trust, routing, metering (covalent, ionic, metallic).

### 3. **UniBins Are Universal**
Same binary works everywhere - adapts based on environment, no knowledge of upper layers.

### 4. **Routing Is Intelligent**
Ecological → Covalent → Ionic priority ensures cost-effective, secure routing.

---

## 📞 Getting Help

- **Documentation**: Start with this file, then explore links
- **Issues**: Check `archive/` for historical context
- **Standards**: See `../../wateringHole/` for ecoPrimals standards
- **Session Summary**: Read [SESSION_SUMMARY_TOWER_ATOMIC_JAN_20_2026.md](SESSION_SUMMARY_TOWER_ATOMIC_JAN_20_2026.md)

---

## 🎉 Quick Wins

### Deploy in 30 seconds
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
./scripts/deploy_tower_squirrel_manual.sh nat0
```

### Test AI in 5 seconds
```bash
echo '{"jsonrpc":"2.0","method":"ai.chat",
  "params":{"messages":[{"role":"user","content":"Hi!"}]},"id":1}' \
  | nc -U /tmp/squirrel-nat0.sock
```

### See the magic
```bash
# Watch logs in real-time
tail -f /tmp/*.log
```

---

**Welcome to biomeOS!** 🧬🚀

**The deployment system for genetically bonded Pure Rust microservices.**

---

**Last Updated**: January 20, 2026  
**Version**: v0.28.0  
**Status**: Production Ready + Evolution Mode

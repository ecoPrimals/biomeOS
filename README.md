# 🌱 biomeOS - Universal Operating System

**Production-Ready** | **Modern Rust** | **Zero Unsafe Code** | **Port-Free Architecture**

biomeOS is a capability-based orchestration layer for managing primals and ecosystems. It provides secure, adaptive coordination through the NUCLEUS discovery protocol and Neural API.

---

## 🎊 Current Status: Production-Ready! (January 9, 2026)

✅ **Core Infrastructure** - BYOB, graphs, discovery, federation  
✅ **NUCLEUS** - 5-layer secure discovery (14 tests passing)  
✅ **Topology API** - Ready for petalTongue UI integration  
✅ **Zero Unsafe Code** - 100% safe Rust throughout  
✅ **Deep Debt Evolution Complete** - Phases 1 & 2 done (24 commits, 43 fixes)  
✅ **LAN Federation** - Working and verified  
✅ **Production-Ready** - Zero mocks, zero hardcoding, graceful errors  

🚧 **In Progress** - petalTongue UI integration  
⏳ **Next** - Internet deployment with full encryption  

---

## 🏗️ Architecture

### **Primals** (Sovereign Services)
- **biomeOS**: Orchestrator (this project)
- **Songbird**: P2P communication, discovery, BTSP
- **BearDog**: Security, encryption, identity, trust
- **Toadstool**: Compute, workload management
- **NestGate**: Storage, provenance, compression
- **petalTongue**: Universal UI (visual, audio, text)

### **Niches** (Deployment Patterns)
- **Tower**: Communication stack (biomeOS + Songbird + BearDog)
- **Node**: Compute (Toadstool + optional BearDog + conditional Songbird)
- **Nest**: Data federation (NestGate + BearDog + Songbird)
- **UI**: Interface (petalTongue + biomeOS)

### **Communication**
- **Primary**: Unix sockets (JSON-RPC)
- **Discovery**: UDP multicast (Songbird/BirdSong P2P)
- **Secure Tunnels**: BTSP (BirdSong Tunnel Protocol)
- **Fallback**: HTTP (legacy, being deprecated)

---

## 🧬 NUCLEUS (Secure Discovery Protocol)

5-layer verification for primal discovery:

1. **Physical Discovery** (Songbird) - UDP multicast, socket scanning
2. **Identity Verification** (BearDog) - Ed25519 challenge-response
3. **Capability Verification** (biomeOS) - Query primal directly
4. **Trust Evaluation** (BearDog) - Genetic lineage, family membership
5. **Registration** (biomeOS) - Add to verified registry

### Trust Levels
- **0 - Unknown**: Unverified
- **1 - Basic**: Discovered + identity verified
- **2 - Elevated**: Capabilities verified
- **3 - High**: Same family
- **4 - Highest**: Sibling node

---

## 🚀 Quick Start

### **Prerequisites**
- Rust 1.75+ (nightly recommended)
- USB drives (optional, for spore deployment)
- BearDog, Songbird binaries (in `plasmidBin/primals/`)

### **Build & Test**
```bash
# Build all crates
cargo build --workspace

# Run tests
cargo test --workspace

# Run NUCLEUS tests
cargo test --package biomeos-federation nucleus_tests
```

### **Deploy a Tower (Communication Stack)**
```bash
# Create a tower spore
cargo run --bin biomeos-spore -- create --niche tower --output /path/to/usb

# Or run locally
cargo run --bin biome
```

### **Start API Server**
```bash
# Live mode (discovers real primals)
cargo run --bin biomeos-api

# Standalone mode (graceful degradation for demos)
BIOMEOS_STANDALONE_MODE=true cargo run --bin biomeos-api
```

---

## 📚 Documentation

### **Key Documents**
- [`ROADMAP.md`](ROADMAP.md) - Phased implementation plan
- [`STATUS.md`](STATUS.md) - Current status and statistics
- [`docs/DEEP_DEBT_EVOLUTION_JAN9.md`](docs/DEEP_DEBT_EVOLUTION_JAN9.md) - Deep debt work summary
- [`plasmidBin/MANIFEST.md`](plasmidBin/MANIFEST.md) - Binary deployment guide

### **Specifications** (`specs/`)
- 30+ active specifications
- NUCLEUS, Neural API, BYOB, Federation, Deployment
- All production-ready and up-to-date

### **Guides** (`docs/guides/`)
- Federation setup
- Spore deployment
- Niche configuration
- Testing strategies

---

## 🧪 Testing

We maintain comprehensive test coverage:

- **Unit Tests**: Core functionality (200+)
- **Integration Tests**: Primal coordination
- **E2E Tests**: Full deployment scenarios
- **Chaos Tests**: Fault injection and recovery
- **NUCLEUS Tests**: 14 tests covering secure discovery

```bash
# Run all tests
cargo test --workspace

# Run specific test suites
cargo test --package biomeos-federation nucleus_tests
cargo test --package biomeos-spore
cargo test --package biomeos-graph
```

---

## 🌱 Key Features

### **BYOB (Build Your Own Biome)**
User-driven manifest system for defining custom biome configurations. Define niches, primals, graphs, and dependencies in TOML.

### **Neural API**
Adaptive, learning-based orchestration through multi-layer graph execution. Evolves from static workflows to intelligent coordination.

### **Genetic Lineage**
Cryptographic verification of family membership using BearDog's HKDF-SHA256 derivation. Enables hierarchical federation with trust levels.

### **USB Spore Deployment**
Portable deployment via USB "spores" with:
- Unique genetic seeds (siblings, not clones)
- Self-propagation capability
- Encrypted with BearDog
- Fossil record for tracking

### **Port-Free Architecture**
Unix sockets for local IPC, UDP multicast for discovery. HTTP only as legacy fallback.

---

## 🔒 Security

- **Zero Unsafe Code**: 100% safe Rust throughout
- **BearDog Integration**: All crypto delegated to BearDog
- **NUCLEUS**: 5-layer verification for primal discovery
- **BTSP**: Encrypted P2P tunnels via BearDog + Songbird
- **Genetic Lineage**: Cryptographic family verification

---

## 📊 Statistics

- **~95,000** lines of Rust
- **15+** crates
- **200+** tests (unit, integration, E2E, chaos)
- **0** unsafe blocks
- **5+** crates with `#![deny(unsafe_code)]`
- **30+** active specifications

---

## 🤝 Contributing

biomeOS is part of the ecoPrimals ecosystem. Each primal is sovereign and evolves independently:

- **biomeOS**: This repository (orchestration)
- **Songbird**: `ecoPrimals/phase1/songbird/` (P2P, BTSP)
- **BearDog**: `ecoPrimals/phase1/beardog/` (security, crypto)
- **Toadstool**: `ecoPrimals/phase1/toadstool/` (compute)
- **NestGate**: `ecoPrimals/phase1/nestgate/` (storage)
- **petalTongue**: `ecoPrimals/phase2/petalTongue/` (UI)

Contributions welcome! Follow deep debt principles:
- Modern idiomatic Rust
- No unsafe code
- Capability-based, not hardcoded
- Delegates to primals, doesn't reimplement
- Comprehensive tests

---

## 📝 License

[Add your license here]

---

## 🌟 Acknowledgments

Built with Rust 🦀, inspired by nature 🌱, powered by the ecoPrimals ecosystem.

**The ecosystem is alive and evolving!** ✨

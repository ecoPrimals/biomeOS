# 🌱 biomeOS

**Primal Operating System for Autonomous Compute**

[![ecoBin Certified](https://img.shields.io/badge/ecoBin-Certified-green)](./GENOMEBIN_ARCHITECTURE_STANDARD.md)
[![Pure Rust](https://img.shields.io/badge/Rust-100%25-orange)](https://www.rust-lang.org)
[![TLS 1.3](https://img.shields.io/badge/TLS-1.3_Pure_Rust-blue)](./START_HERE.md)
[![License](https://img.shields.io/badge/License-AGPL--3.0--or--later-blue)](LICENSE)

---

## 🎉 Production Ready

**Tower Atomic validated** with 93% TLS 1.3 success across 87 sites.  
**NUCLEUS lifecycle management** complete with resurrection & apoptosis.  
**Multi-AI coordination** validated (9/9 tests: Anthropic, OpenAI, HuggingFace, Toadstool).  
**Protocol escalation** roadmap defined (JSON-RPC → tarpc).  
**Dual USB LiveSpore** deployment ready for distributed AI workloads.

| Metric | Value |
|--------|-------|
| **Multi-AI Tests** | 9/9 passed |
| **Sites Tested** | 87 (11 categories) |
| **TLS 1.3 Success** | 93% |
| **Web Compatibility** | 96% |
| **Pure Rust** | 100% |
| **Tests Passing** | 277+ |
| **AI E2E Latency** | 560ms avg (Anthropic) |
| **HTTPS Latency** | 366ms avg |
| **Songbird** | v8.14.0 (HTTP headers, dual-mode) |
| **Squirrel** | AI coordination (Anthropic, OpenAI, HuggingFace) |
| **Toadstool** | Local compute coordinator |
| **NestGate** | JSON-RPC storage (persistence pending) |

---

## 🎯 Overview

biomeOS is a **Pure Rust operating system layer** that orchestrates autonomous compute primals through capability-based discovery and semantic routing.

### Key Features

✅ **Pure Rust** - Zero C dependencies (ecoBin compliant)  
✅ **Tower Atomic** - Pure Rust TLS 1.3 (BearDog + Songbird)  
✅ **capability.call** - Semantic translation layer  
✅ **Graph Deployment** - Declarative primal orchestration  
✅ **TRUE PRIMAL** - Zero coupling between primals  
✅ **Capability-based Discovery** - Runtime primal detection via XDG sockets  
✅ **NUCLEUS Lifecycle** - Resurrection, apoptosis, health monitoring  
✅ **Socket Discovery** - Capability-based socket resolution (no hardcoding)  
✅ **Concurrent Testing** - All tests use proper async patterns (no sleeps)

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ (stable)
- Linux (kernel 5.4+)
- Unix socket support

### Build & Deploy

```bash
# Build
cargo build --release --workspace

# Deploy Tower Atomic (Unix socket only, TRUE PRIMAL)
./deploy_tower_atomic.sh

# Or use LiveSpore on USB
cd /media/user/USB/biomeOS && ./deploy.sh

# Test JSON-RPC (no HTTP!)
echo '{"jsonrpc":"2.0","method":"health.check","id":1}' | nc -U /tmp/beardog-nat0.sock
```

### LiveSpore USB Deployment

Create genetically-linked LiveSpores for federation:

```bash
# Create sibling spore from existing parent
./scripts/create_sibling_spore.sh /media/parent/biomeOS /media/newusb node-beta

# Verify genetic lineage between siblings
./scripts/verify_sibling_lineage.sh /media/usb1/biomeOS /media/usb2/biomeOS

# Test federation (starts both spores)
./scripts/test_federation.sh
```

---

## 📚 Architecture

### Tower Atomic

Pure Rust HTTPS via BearDog (crypto) + Songbird (HTTP/TLS):

```
Consumer → Neural API → Songbird → BearDog → External HTTPS
              ↓
    capability.call("secure_http", "http.request", {...})
              ↓
    Graph-based semantic translation
```

### TRUE PRIMAL Pattern

Primals don't know each other. Communication via semantic capabilities:

```rust
// Consumer uses semantic name
neural_api.call_capability("crypto", "sha256", data).await?;

// Neural API translates via graph:
// "sha256" → "crypto.sha256" (BearDog method)

// Provider can change methods without breaking consumers
```

### NUCLEUS Lifecycle

```
Germinating → Incubating → Active ↔ Degraded → Apoptosis → Dead
                             ↓
                       Resurrection
                    (from deployment graph)
```

---

## 🏆 Validation Results

### TLS 1.3 Categories (93% Success)

| Category | Sites | Success |
|----------|-------|---------|
| AI/ML | 10 | 100% ✅ |
| Cloud | 10 | 90% ✅ |
| Containers | 6 | 100% ✅ |
| Databases | 7 | 100% ✅ |
| Serverless | 7 | 100% ✅ |
| Security | 6 | 100% ✅ |

### Validated Services

- **AI/ML**: OpenAI, Anthropic, HuggingFace, Cohere, Groq
- **Cloud**: AWS, GCP, Azure, DigitalOcean, Vercel
- **Code**: GitHub, GitLab, Bitbucket
- **Research**: NCBI, PubMed, arXiv

---

## 📖 Documentation

### Start Here
- **[START_HERE.md](./START_HERE.md)** ⭐ - Quick orientation
- **[README.md](./README.md)** - This file
- **[DOCUMENTATION_HUB.md](./DOCUMENTATION_HUB.md)** - Navigation

### Architecture
- **[specs/README.md](./specs/README.md)** - All specifications
- **[BIOMEOS_ATOMICS_ARCHITECTURE.md](./BIOMEOS_ATOMICS_ARCHITECTURE.md)**
- **[TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md](./TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)**

### Lifecycle & Evolution
- **[docs/LIFECYCLE_MANAGEMENT.md](./docs/LIFECYCLE_MANAGEMENT.md)** - NUCLEUS lifecycle
- **[docs/SOCKET_DISCOVERY.md](./docs/SOCKET_DISCOVERY.md)** - Capability-based discovery
- **[INFRASTRUCTURE_EVOLUTION.md](./INFRASTRUCTURE_EVOLUTION.md)** - Terraria, Apoptosis
- **[PROTOCOL_ESCALATION_ROADMAP.md](./PROTOCOL_ESCALATION_ROADMAP.md)** - JSON-RPC → tarpc

### Handoffs
- **[docs/handoffs/](./docs/handoffs/)** - Team handoff documents
- **[docs/handoffs/NESTGATE_PERSISTENCE_HANDOFF.md](./docs/handoffs/NESTGATE_PERSISTENCE_HANDOFF.md)** ⭐ - Storage persistence
- **[docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF.md](./docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF.md)** - HTTP, TLS, discovery
- **[docs/handoffs/TOADSTOOL_JSONRPC_HANDOFF.md](./docs/handoffs/TOADSTOOL_JSONRPC_HANDOFF.md)** ✅ - JSON-RPC fixed

---

## 🧪 Testing

```bash
# Run all tests (400+ passing, 106 suites)
cargo test --workspace

# Run specific integration tests
cargo test --package biomeos-atomic-deploy

# Check coverage
cargo llvm-cov --workspace
```

### Concurrency-First Testing

All tests follow modern concurrent Rust patterns:
- ✅ No `sleep()` for readiness - use `oneshot` channels
- ✅ Timeouts on all network calls
- ✅ RAII guards for global state cleanup
- ✅ Proper async/await throughout

---

## 🤝 Contributing

Key areas:
1. **Protocol Escalation** - JSON-RPC → tarpc runtime evolution
2. **TLS 1.2** - Fallback for older servers (7% remaining)
3. **LAN Federation** - Cross-node discovery and trust
4. **Documentation** - Examples and guides

### Code Quality
- ✅ `cargo fmt` for formatting
- ✅ `cargo clippy` for linting
- ✅ Pure Rust only (no C dependencies)
- ✅ `#![deny(unsafe_code)]` in all crates

---

## 📜 License

AGPL-3.0-or-later WITH Sovran-Exemption-1.0

See [LICENSE](LICENSE) for details.

---

## 🔗 Related Projects

- **[BearDog](https://github.com/ecoPrimals/beardog)** - Pure Rust crypto
- **[Songbird](https://github.com/ecoPrimals/songbird)** - Pure Rust HTTP/TLS
- **[wateringHole](https://github.com/ecoPrimals/wateringHole)** - Standards

---

## 📊 Metrics

| Metric | Value |
|--------|-------|
| **Crates** | 21 |
| **Tests Passing** | 277+ |
| **Rust Files** | 369 |
| **Lines of Code** | ~111k |
| **Largest File** | <1000 lines (smart refactored) |
| **Formatting** | ✅ Clean (`cargo fmt`) |
| **Clippy** | ✅ 0 errors (all auto-fixes applied) |
| **Unsafe Code** | 0 blocks (`#![deny(unsafe_code)]`) |
| **TODOs/FIXMEs** | 0 |
| **Mocks in Production** | 0 (all in `#[cfg(test)]`) |
| **Hardcoded Paths** | 0 (XDG-compliant) |

---

## 🧬 LiveSpore System

**Genetic lineage** enables automatic federation trust with Dark Forest privacy:

| Feature | Description |
|---------|-------------|
| **64-Byte Seed** | `[genesis:32] + [node_key:32]` structure |
| **Genesis Seed** | Shared family root (bytes 0-31) |
| **Node Key** | `Blake3(genesis, "node-identity-v1:" + node_id)` (bytes 32-63) |
| **Dark Forest** | Encrypted beacons - only family can decrypt |
| **Validation** | `validate_spore.sh --update /path/to/spore` |
| **Personal Vault** | `vault/experience.json` tracks lived experience |

### LiveSpore Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    USB LiveSpore                            │
├────────────────────────────┬────────────────────────────────┤
│   SYSTEM (Validated)       │   VAULT (Spore-Specific)       │
│   • primals/beardog    ✓   │   • vault/experience.json      │
│   • primals/songbird   ✓   │   • vault/deployments/         │
│   • deploy.sh          ✓   │   • vault/logs/                │
│   • .family.seed       ✓   │   • vault/workdata/            │
│   ↳ MD5 checksum validated │   ↳ Preserved across updates   │
└────────────────────────────┴────────────────────────────────┘
```

See `specs/LIVESPORE_IMPRINTING_SPEC.md` and `specs/BIRDSONG_DARK_FOREST_TRUST_MODEL.md`.

---

## 🦀 Pure Rust Evolution - Complete

**Deep Debt Evolution Complete** (January 28-29, 2026):

| Metric | Before | After |
|--------|--------|-------|
| **TODOs/FIXMEs** | 85 | 0 (-100%) |
| **Unsafe Code** | 0 | 0 (enforced) |
| **Mocks in Prod** | several | 0 (all in `#[cfg(test)]`) |
| **Hardcoded Paths** | many | 0 (XDG-compliant) |
| **Hanging Tests** | several | 0 (concurrent) |
| **Clippy Warnings** | several | 0 (auto-fixed) |

### Key Evolutions Completed

✅ **NUCLEUS Lifecycle** - Germination, Incubation, Active, Degraded, Apoptosis  
✅ **Socket Discovery** - Capability-based resolution (no hardcoded /tmp)  
✅ **Concurrent Tests** - All tests use proper async patterns  
✅ **Rollback Strategy** - Full graph execution recovery  
✅ **Songbird Mesh** - UDP multicast peer discovery  
✅ **BearDog Integration** - Lineage verification & key derivation  
✅ **XDG Compliance** - SystemPaths throughout  
✅ **Smart Refactoring** - Large files → modular crates  
✅ **Capability Taxonomy** - Unified enum for all capabilities  

**Architecture Principles**:
- Rust: Borrow checker, type system, zero-cost abstractions
- `#![deny(unsafe_code)]` enforced in all crates
- JSON-RPC over Unix sockets (no HTTP in production)
- Concurrency-first: no sleeps, proper async waits

See `RUST_EVOLUTION_ROADMAP.md` for full details.

---

**Status**: ✅ Production Ready | **TLS**: 93% | **Pure Rust**: 100% | **Tests**: 277+ | **Deep Debt**: ✅

🎉 **biomeOS: Autonomous Compute Through Semantic Evolution**

*Updated: January 29, 2026*

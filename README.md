# 🌱 biomeOS

**Primal Operating System for Autonomous Compute**

[![ecoBin Certified](https://img.shields.io/badge/ecoBin-Certified-green)](./GENOMEBIN_ARCHITECTURE_STANDARD.md)
[![Pure Rust](https://img.shields.io/badge/Rust-100%25-orange)](https://www.rust-lang.org)
[![TLS 1.3](https://img.shields.io/badge/TLS-1.3_Pure_Rust-blue)](./START_HERE.md)
[![License](https://img.shields.io/badge/License-AGPL--3.0--or--later-blue)](LICENSE)

---

## 🎉 Production Ready

**Tower Atomic validated** with 93% TLS 1.3 success across 87 sites.

| Metric | Value |
|--------|-------|
| **Sites Tested** | 87 (11 categories) |
| **TLS 1.3 Success** | 93% |
| **Web Compatibility** | 96% |
| **Pure Rust** | 100% |

---

## 🎯 Overview

biomeOS is a **Pure Rust operating system layer** that orchestrates autonomous compute primals through capability-based discovery and semantic routing.

### Key Features

✅ **Pure Rust** - Zero C dependencies (ecoBin compliant)  
✅ **Tower Atomic** - Pure Rust TLS 1.3 (BearDog + Songbird)  
✅ **capability.call** - Semantic translation layer  
✅ **Graph Deployment** - Declarative primal orchestration  
✅ **TRUE PRIMAL** - Zero coupling between primals  

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

### Evolution
- **[SONGBIRD_EVOLUTION_HANDOFF.md](./SONGBIRD_EVOLUTION_HANDOFF.md)** - TLS roadmap
- **[INFRASTRUCTURE_EVOLUTION.md](./INFRASTRUCTURE_EVOLUTION.md)** - Terraria, Apoptosis

---

## 🧪 Testing

```bash
# Run all tests
cargo test --workspace

# Run specific integration tests
cargo test --package biomeos-atomic-deploy

# Check coverage
cargo llvm-cov --workspace
```

---

## 🤝 Contributing

Key areas:
1. **TLS 1.2** - Fallback for older servers (7% remaining)
2. **Documentation** - Examples and guides
3. **Primal Integration** - New primal development

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
| **Tests** | 321 passing |
| **Rust Files** | 360 |
| **Lines of Code** | ~101k |
| **Largest File** | 933 lines |
| **Formatting** | ✅ Clean (`cargo fmt`) |
| **Unsafe Code** | 0 blocks |

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

## 🦀 Pure Rust Evolution

Scripts are **bootstrap solutions**. We evolve to Pure Rust for:

| Script → Rust | Why |
|---------------|-----|
| `deploy.sh` → `biomeos-deploy` | Type safety, cross-platform |
| `validate_spore.sh` → `biomeos-validate` | Compile-time guarantees |
| Shell orchestration → Neural API | Full portability, no bash dependency |

**Physics with gravity** (Rust) vs **jelly strings** (bash):
- Rust: Borrow checker, type system, zero-cost abstractions
- C: "Physics in vacuum" - some constraints, no safety net
- Bash: Flexible but fragile, no compile-time checks

See `RUST_EVOLUTION_ROADMAP.md` for migration plan.

---

**Status**: ✅ Production Ready | **TLS**: 93% | **Pure Rust**: 100% (core)

🎉 **biomeOS: Autonomous Compute Through Semantic Evolution**

*Updated: January 27, 2026*

# 🌟 biomeOS - Universal Ecosystem Orchestrator

**Version**: 0.1.0  
**Status**: ✅ TRUE ecoBin #5 (Certified Jan 24, 2026)  
**Architecture**: UniBin + ecoBin + genomeBin (in progress)

---

## 🎯 What is biomeOS?

biomeOS is a **Pure Rust ecosystem orchestrator** that deploys and manages distributed systems using:

- **Neural API**: Graph-based declarative deployment (TOML)
- **Atomic Patterns**: Tower Atomic for pure Rust TLS/HTTPS
- **Primal Architecture**: Self-contained, capability-based services
- **JSON-RPC IPC**: Unix socket communication between primals
- **Universal Portability**: Static binary, zero C dependencies

### Core Philosophy

> **Deploy and assume ecosystems, not isolated services.**

biomeOS orchestrates complete, living systems where primals discover each other at runtime through capability-based discovery via Songbird.

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (or use sourDough for installation)
- Linux (x86_64 or ARM64)

### Build

```bash
cargo build --release
```

### Deploy a Simple Ecosystem

```bash
# Deploy NUCLEUS (BearDog + Songbird + NestGate + Toadstool)
./target/release/biomeos deploy \
  --family nat0 \
  --graph graphs/nucleus_simple.toml

# Check status
./target/release/biomeos status

# Verify health
./target/release/biomeos doctor
```

---

## 📚 Key Documentation

### Getting Started
- **[Quick Start](QUICK_START.md)**: 5-minute deployment guide
- **[Deployment Guide](DEPLOYMENT.md)**: Comprehensive deployment documentation
- **[Tower Deployment](QUICK_START_TOWER_DEPLOYMENT.md)**: Tower-specific quick start

### Architecture
- **[Primal Integration Spec](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)**: How primals interact
- **[Neural API Plan](BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md)**: Graph-based deployment
- **[Atomics Architecture](BIOMEOS_ATOMICS_ARCHITECTURE.md)**: Tower Atomic patterns
- **[Port-Free Architecture](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)**: Unix socket IPC
- **[genomeBin Standard](GENOMEBIN_ARCHITECTURE_STANDARD.md)**: One-command deployment vision

### Certification & Quality
- **[ecoBin Certification](BIOMEOS_ECOBIN_CERTIFICATION_JAN_24_2026.md)**: TRUE ecoBin #5 validation
- **[Test Coverage Milestone](TEST_COVERAGE_MILESTONE_JAN_24_2026.md)**: 41.78% coverage achieved
- **[Tower Atomic Validation](TOWER_ATOMIC_VALIDATION_AND_EVOLUTION.md)**: Pure Rust TLS validation

### Project Organization
- **[Documentation Index](DOCS_INDEX.md)**: Complete documentation map
- **[Root Docs Index](ROOT_DOCS_INDEX.md)**: This directory structure
- **[Start Here](START_HERE.md)**: Orientation guide

---

## 🏗️ Architecture Overview

### Primal Ecosystem

```
┌─────────────┐
│   biomeOS   │  ← Orchestrator (THIS)
└──────┬──────┘
       │ Deploys & manages via Neural API
       │
       ├── BearDog (Security & Crypto)
       ├── Songbird (Discovery & P2P)
       ├── NestGate (Storage & Persistence)
       ├── Toadstool (Compute & GPU)
       ├── Squirrel (Data/ML)
       └── Tower (AI/Neural via Atomic)
```

### Communication Flow

```
┌──────────┐                  ┌──────────┐
│  Primal  │ ←── JSON-RPC ──→ │ Songbird │
│  (any)   │   (Unix Socket)  │ (discover)
└──────────┘                  └──────────┘
     │                              │
     └──── Capability Query ────────┘
     
No hardcoded IPs, ports, or endpoints!
Everything discovered at runtime.
```

### Neural API Graphs

```toml
# graphs/nucleus_simple.toml
[[nodes]]
id = "beardog"
primal = "beardog-server"
mode = "server"
capabilities = ["crypto", "encryption", "identity"]

[[nodes]]
id = "songbird"
primal = "songbird"
mode = "orchestrator"
capabilities = ["discovery", "p2p", "registry"]
depends_on = ["beardog"]

# ... more nodes
```

---

## 🎯 Key Features

### ✅ Production Ready
- **TRUE ecoBin**: 100% Pure Rust, zero C dependencies
- **Static Binary**: 6.8MB, fully self-contained
- **Universal**: Runs on any Linux (x86_64, ARM64)
- **Zero Config**: Discovers everything at runtime
- **Test Coverage**: 41.78% and growing

### ✅ Modern Architecture
- **Graph-Based Deployment**: Declarative TOML graphs
- **DAG Resolution**: Automatic dependency ordering
- **Parallel Execution**: Optimal performance
- **Rollback on Failure**: Automatic cleanup
- **Metrics Collection**: Full observability

### ✅ Developer Experience
- **Single Binary**: One `biomeos` command for everything
- **Multiple Modes**: `deploy`, `status`, `doctor`, `verify`
- **Rich CLI**: Beautiful output with progress indicators
- **Comprehensive Logs**: `.spore.logs` for forensics
- **Live Spores**: Deploy to USB for portable ecosystems

---

## 📦 What's Included

### Core Crates

| Crate | Purpose | Status |
|-------|---------|--------|
| `biomeos` | Main orchestrator binary | ✅ |
| `biomeos-api` | REST/HTTP API | ✅ |
| `biomeos-core` | Core utilities | ✅ |
| `biomeos-types` | Shared types | ✅ |
| `biomeos-graph` | Neural API graph parsing | ✅ |
| `biomeos-atomic-deploy` | Deployment engine | ✅ |
| `biomeos-spore` | LiveSpore creation | ✅ |
| `biomeos-federation` | Multi-node federation | ✅ |
| `biomeos-nucleus` | Nucleus client | ✅ |
| `biomeos-primal-sdk` | Primal integration SDK | ✅ |
| `biomeos-ui` | Web UI | ✅ |

### Key Directories

```
biomeOS/
├── crates/          # Rust workspace crates
├── graphs/          # Neural API deployment graphs
├── specs/           # Technical specifications
├── docs/            # Comprehensive documentation
├── examples/        # Example deployments
├── templates/       # Primal templates
├── scripts/         # Utility scripts
├── tests/           # Integration tests
└── archive/         # Historical documentation
```

---

## 🧬 Primal IPC Protocol

biomeOS uses the **Primal IPC Protocol** for all inter-primal communication:

### Communication Standard
- **Transport**: Unix domain sockets
- **Format**: JSON-RPC 2.0
- **Discovery**: Via Songbird registry
- **Security**: BearDog provides identity/encryption

### Example: Discovering a Primal

```rust
// 1. Ask Songbird for a capability
let response = songbird_client.request(
    "registry.find_by_capability",
    json!({ "capability": "crypto" })
).await?;

// 2. Get socket path
let socket_path = response["socket_path"].as_str()?;

// 3. Connect and call
let result = unix_socket_rpc(socket_path, "crypto.encrypt", params).await?;
```

No hardcoding! Everything is discovered at runtime.

---

## 🔬 Development

### Running Tests

```bash
# All tests
cargo test --workspace

# With coverage
cargo llvm-cov --workspace

# Specific package
cargo test -p biomeos-graph
```

### Building for Different Targets

```bash
# Linux musl (universal static)
cargo build --release --target x86_64-unknown-linux-musl

# ARM64
cargo build --release --target aarch64-unknown-linux-gnu

# Check ecoBin compliance
cargo tree | grep -E "(openssl|curl|reqwest)"
```

### Code Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy --all-targets --all-features

# Check
cargo check --workspace
```

---

## 🌐 Deployment Scenarios

### 1. Single Computer (Development)
```bash
biomeos deploy --family dev --graph graphs/nucleus_simple.toml
```

### 2. Multiple Computers (Production)
```bash
# On each computer:
biomeos deploy --family prod --graph graphs/nucleus_ecosystem.toml --node-id node-1
```

### 3. USB LiveSpore (Portable)
```bash
# Create spore
biomeos spore create --output /media/usb0

# Deploy from spore (on any computer)
cd /media/usb0/biomeOS
./primals/nucleus deploy --family nat0 --graph graphs/nucleus_simple.toml
```

### 4. Cloud/HPC (Distributed)
See `deployments/basement-hpc/` for multi-node examples.

---

## 🎓 Learning Path

### Beginners
1. Read [Quick Start](QUICK_START.md)
2. Try deploying `nucleus_simple.toml`
3. Explore [Primal Integration Spec](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md)

### Intermediate
1. Study [Neural API Plan](BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md)
2. Create custom deployment graphs
3. Understand [Port-Free Architecture](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md)

### Advanced
1. Read [Atomics Architecture](BIOMEOS_ATOMICS_ARCHITECTURE.md)
2. Contribute to primal SDKs
3. Explore [genomeBin](GENOMEBIN_ARCHITECTURE_STANDARD.md) evolution

---

## 🏆 Recent Achievements

### TRUE ecoBin #5 Certification (Jan 24, 2026)
- ✅ 100% Pure Rust (zero C dependencies)
- ✅ 6.8MB static binary
- ✅ Universal portability validated
- ✅ First workspace-based ecoBin!

### Test Coverage Milestone
- ✅ 37.43% → 41.78% coverage
- ✅ 60 comprehensive tests added
- ✅ 100% test pass rate

### Code Quality
- ✅ Zero unsafe code
- ✅ Idiomatic Rust throughout
- ✅ Full formatting compliance
- ✅ All linting issues resolved

---

## 🤝 Contributing

biomeOS is part of the **ecoPrimals** ecosystem. Contributions welcome!

### Areas of Focus
- [ ] Increase test coverage (target: 90%)
- [ ] Add more deployment graph examples
- [ ] Improve error messages
- [ ] Add ARM architecture CI
- [ ] Enhance web UI
- [ ] Write more documentation

### Standards
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Maintain zero unsafe code
- Add tests for new features
- Update documentation
- Keep ecoBin compliance

---

## 📊 Project Status

| Metric | Status |
|--------|--------|
| Build | ✅ Passing |
| Tests | ✅ 79/79 (100%) |
| Coverage | 🟡 41.78% (→ 90%) |
| ecoBin | ✅ TRUE #5 |
| Documentation | ✅ Comprehensive |
| Production Ready | ✅ Yes |

---

## 🔗 Related Projects

- **BearDog**: Security & crypto primal (TRUE ecoBin #1)
- **Songbird**: Discovery & P2P primal (TRUE ecoBin #4)
- **NestGate**: Storage primal (TRUE ecoBin #2)
- **sourDough**: Scaffolding tool (TRUE ecoBin #3)
- **Tower**: AI/Neural primal (Pure Rust TLS via Atomic!)
- **Toadstool**: Compute primal
- **Squirrel**: Data/ML primal

---

## 📝 License

Part of the ecoPrimals ecosystem. See individual primal repositories for license information.

---

## 🌟 Vision

biomeOS represents a **new paradigm in distributed systems**:

- **No hardcoding**: Everything discovered at runtime
- **Pure Rust**: Universal portability without compromise
- **Declarative**: Graph-based deployment
- **Autonomous**: Primals self-organize
- **Portable**: Deploy anywhere, from USB to HPC

> "Deploy and assume ecosystems, not isolated services." ✨

---

**Documentation**: See `docs/` and `specs/` for comprehensive guides  
**Examples**: See `examples/` and `graphs/` for deployment templates  
**Support**: See individual primal documentation for specific features

**Built with 🦀 Pure Rust • ecoBin Certified ✅ • UniBin Compliant 🎯**

# 🧠 biomeOS Documentation Hub

**Version**: 0.1.0  
**Status**: Production Ready  
**Compliance**: ecoBin ✅ | UniBin ✅

---

## 🚀 Quick Start

**New to biomeOS?** Start here:
1. [`START_HERE.md`](START_HERE.md) - Project overview and getting started
2. [`QUICK_START.md`](QUICK_START.md) - Quick deployment guide
3. [`README.md`](README.md) - Technical overview

---

## 📚 Core Documentation

### Essential Reading

| Document | Description |
|----------|-------------|
| [`DOCUMENTATION_INDEX.md`](DOCUMENTATION_INDEX.md) | Master index of all documentation |
| [`ROOT_DOCS_INDEX.md`](ROOT_DOCS_INDEX.md) | Root-level documentation guide |

### Architecture & Design

| Document | Description |
|----------|-------------|
| [`BIOMEOS_ATOMICS_ARCHITECTURE.md`](BIOMEOS_ATOMICS_ARCHITECTURE.md) | Atomic deployment architecture |
| [`BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md`](BIOMEOS_NEURAL_API_TOWER_ATOMIC_DEPLOYMENT_PLAN.md) | Neural API & Tower Atomic deployment |
| [`BIOMEOS_PRIMAL_INTEGRATION_SPEC.md`](BIOMEOS_PRIMAL_INTEGRATION_SPEC.md) | Primal integration specifications |
| [`GENOMEBIN_ARCHITECTURE_STANDARD.md`](GENOMEBIN_ARCHITECTURE_STANDARD.md) | GenomeBin architecture standard |
| [`ISOMORPHIC_EVOLUTION.md`](ISOMORPHIC_EVOLUTION.md) | Isomorphic evolution patterns |
| [`SEMANTIC_EVOLUTION_STRATEGY.md`](SEMANTIC_EVOLUTION_STRATEGY.md) | Semantic method evolution strategy |
| [`TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md`](TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md) | Port-free, capability-based architecture |
| [`WATERINGHOLE_INTEGRATION.md`](WATERINGHOLE_INTEGRATION.md) | WateringHole ecosystem integration |

### Deployment

| Document | Description |
|----------|-------------|
| [`DEPLOYMENT.md`](DEPLOYMENT.md) | Deployment guide and procedures |
| [`QUICK_START_TOWER_DEPLOYMENT.md`](QUICK_START_TOWER_DEPLOYMENT.md) | Tower Atomic quick deployment |
| [`TOWER_ATOMIC_STATUS_JAN_25_2026.md`](TOWER_ATOMIC_STATUS_JAN_25_2026.md) | Current Tower Atomic status |

---

## 📂 Detailed Documentation

### `/docs` Directory

Comprehensive documentation organized by topic:

- **Architecture**: System design and patterns
- **API**: API specifications and examples
- **Guides**: How-to guides and tutorials
- **Primal Integrations**: Primal-specific documentation
- **Deep Debt**: Technical debt resolution reports
- **Sessions**: Development session logs

See [`docs/INDEX.md`](docs/INDEX.md) for complete directory navigation.

### `/specs` Directory

Technical specifications for all major components:

- Neural API specifications
- BYOB (Build Your Own Biome) specifications
- Tower Atomic specifications
- Integration specifications
- Graph execution specifications

See [`specs/`](specs/) for all specifications.

---

## 🏆 Compliance Status

### ecoBin Compliance ✅

**Status**: ACHIEVED (Jan 25, 2026)

- ✅ Pure Rust crypto (BearDog)
- ✅ Pure Rust TLS (Songbird)
- ✅ Zero reqwest in production
- ✅ Minimal C dependencies (only `libc`)
- ✅ Full cross-compilation capability

**Details**: See [archive/session_jan_25_2026_deep_debt/](archive/session_jan_25_2026_deep_debt/)

### UniBin Compliance ✅

**Status**: FULLY COMPLIANT (Jan 25, 2026)

- ✅ Single binary: `biomeos`
- ✅ 7 operational modes
- ✅ Professional CLI with clap
- ✅ Comprehensive `--help`
- ✅ Beautiful `--version`

**Details**: See [archive/session_jan_25_2026_deep_debt/UNIBIN_IMPLEMENTATION_COMPLETE.md](archive/session_jan_25_2026_deep_debt/UNIBIN_IMPLEMENTATION_COMPLETE.md)

### WateringHole Standards ✅

- ✅ Capability-based discovery
- ✅ Unix socket IPC priority
- ✅ Primal self-knowledge only
- ✅ JSON-RPC over Unix sockets

---

## 🔧 Development

### Code Quality

- ✅ Zero unsafe blocks (enforced with `#![deny(unsafe_code)]`)
- ✅ Zero production mocks
- ✅ Modern error handling (Result<T,E>)
- ✅ Comprehensive logging (tracing)

### Testing

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test --package biomeos-core

# Build all
cargo build --workspace
```

### UniBin Modes

```bash
# Show all modes
biomeos --help

# Start Neural API server
biomeos neural-api --graphs-dir ./graphs

# Run health diagnostics
biomeos doctor

# Deploy a graph
biomeos deploy graphs/production.toml

# Show version
biomeos version --detailed
```

---

## 🗂️ Archive

Historical documentation and session reports are preserved in [`archive/`](archive/):

- **Session Reports**: Deep Debt execution (Jan 25, 2026)
- **Old Documentation**: Pre-cleanup docs (for reference)
- **Legacy Code**: Archived implementations

All archives are maintained as "fossil record" for historical reference.

---

## 📊 Project Statistics

| Metric | Value |
|--------|-------|
| Crates | 20+ |
| Lines of Code | ~50,000 |
| Test Coverage | Expanding (target: 90%) |
| Documentation | Comprehensive |
| External Dependencies | ~22 (minimal) |
| C Dependencies | 1 (`libc` only) |
| UniBin Modes | 7 |

---

## 🤝 Contributing

See [`docs/guides/`](docs/guides/) for development guides.

### Key Principles

1. **Deep Debt Solutions** - Not just fixes, but evolutionary improvements
2. **Modern Idiomatic Rust** - async/await, Result<T,E>, zero unsafe
3. **Capability-Based** - Runtime discovery, no hardcoding
4. **Pure Rust** - ecoBin compliance, minimize C dependencies
5. **UniBin Architecture** - Professional CLI, single binary

---

## 📞 Support

- **Issues**: GitHub Issues
- **Discussions**: WateringHole (inter-primal)
- **Documentation**: This hub + `/docs` directory

---

## 🎯 Quick Links

### For Users
- [Getting Started](START_HERE.md)
- [Quick Start Guide](QUICK_START.md)
- [Deployment Guide](DEPLOYMENT.md)

### For Developers
- [Architecture Overview](docs/architecture/)
- [API Documentation](docs/api/)
- [Contributing Guide](docs/guides/)

### For Operators
- [Tower Atomic Deployment](QUICK_START_TOWER_DEPLOYMENT.md)
- [Health Diagnostics](docs/guides/) (`biomeos doctor`)
- [Deployment Topologies](topologies/)

---

## 📜 License

MIT

---

## 🏅 Achievements

- 🏆 **2nd primal** to achieve full UniBin compliance
- ✅ **ecoBin ready** - Pure Rust stack
- ✅ **Production ready** - All standards met
- ✅ **Professional UX** - Beautiful CLI
- ✅ **Zero technical debt** - Deep Debt execution complete

---

**🦀✨ Pure Rust. Fast AND Safe. Capability-Based. Production Ready! ✨🦀**

---

**Last Updated**: January 25, 2026  
**Status**: Active Development  
**Version**: 0.1.0


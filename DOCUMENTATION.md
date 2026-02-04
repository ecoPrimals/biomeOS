# biomeOS Documentation Index

**Updated**: February 4, 2026

---

## Quick Navigation

| I want to... | Go to... |
|--------------|----------|
| Get started quickly | [QUICK_START.md](QUICK_START.md) |
| Understand the architecture | [START_HERE.md](START_HERE.md) |
| Check current status | [CURRENT_STATUS.md](CURRENT_STATUS.md) |
| Deploy to USB/Pixel | [livespore-usb/README.md](livespore-usb/README.md) |
| Understand deployment standard | [specs/PRIMAL_DEPLOYMENT_STANDARD.md](specs/PRIMAL_DEPLOYMENT_STANDARD.md) |

---

## Root Documents

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | Project overview and architecture |
| [START_HERE.md](START_HERE.md) | Getting started guide |
| [CURRENT_STATUS.md](CURRENT_STATUS.md) | Latest status and metrics |
| [QUICK_START.md](QUICK_START.md) | 5-minute deployment guide |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

---

## Specifications

Located in `specs/`:

| Document | Purpose |
|----------|---------|
| [PRIMAL_DEPLOYMENT_STANDARD.md](specs/PRIMAL_DEPLOYMENT_STANDARD.md) | Deployment standard v1.0 |
| [EVOLUTION_PATH.md](specs/EVOLUTION_PATH.md) | Scripts to graphs migration |
| [VALIDATION_GOALS.md](specs/VALIDATION_GOALS.md) | Testing and validation |
| [README.md](specs/README.md) | Specs index |

---

## Handoffs & Reports

Located in `docs/handoffs/`:

| Document | Purpose |
|----------|---------|
| [PRIMAL_EVOLUTION_STATUS_FEB03_2026.md](docs/handoffs/PRIMAL_EVOLUTION_STATUS_FEB03_2026.md) | Ecosystem evolution status |
| [DEEP_DEBT_EVOLUTION_FEB03_2026.md](docs/handoffs/DEEP_DEBT_EVOLUTION_FEB03_2026.md) | Deep debt evolution report |
| [BIOMEOS_DEEP_AUDIT_FEB03_2026.md](docs/handoffs/BIOMEOS_DEEP_AUDIT_FEB03_2026.md) | Codebase audit |
| [UNSAFE_CODE_AUDIT_FEB03_2026.md](docs/handoffs/UNSAFE_CODE_AUDIT_FEB03_2026.md) | Unsafe code audit |
| [HARDCODING_REPLACEMENT_REPORT_FEB03_2026.md](docs/handoffs/HARDCODING_REPLACEMENT_REPORT_FEB03_2026.md) | Hardcoding removal |

---

## Deployment

### USB LiveSpore

| Location | Content |
|----------|---------|
| `livespore-usb/x86_64/` | Intel/AMD deployment |
| `livespore-usb/aarch64/` | ARM64 deployment |
| `livespore-usb/x86_64/scripts/` | Deployment scripts |
| `livespore-usb/x86_64/graphs/` | TOML deployment graphs |
| `livespore-usb/x86_64/primals/` | Primal binaries |

### Pixel 8a

| Location | Content |
|----------|---------|
| `pixel8a-deploy/` | Android deployment package |
| `pixel8a-deploy/primals/` | aarch64 binaries |
| `pixel8a-deploy/graphs/` | Deployment graphs |
| `pixel8a-deploy/start_nucleus_mobile.sh` | Mobile startup script |

---

## Graph Definitions

Located in `graphs/` and `livespore-usb/*/graphs/`:

| Graph | Purpose |
|-------|---------|
| `tower_atomic_bootstrap.toml` | Tower Atomic deployment |
| `tower_atomic_xdg.toml` | XDG-compliant Tower |
| `node_atomic_compute.toml` | Node Atomic (+ Toadstool) |
| `nest_deploy.toml` | Nest Atomic (+ NestGate) |
| `nucleus_complete.toml` | Full NUCLEUS deployment |

---

## Crate Documentation

### Core Crates

| Crate | Purpose |
|-------|---------|
| `biomeos-core` | Core orchestration logic |
| `biomeos-types` | Shared types and constants |
| `biomeos-graph` | Graph execution engine |
| `biomeos-spore` | Deployment packaging |

### API Crates

| Crate | Purpose |
|-------|---------|
| `biomeos-api` | HTTP/WebSocket API server |
| `biomeos-cli` | Command-line interface |
| `neural-api-client` | Neural API client library |

### Specialized Crates

| Crate | Purpose |
|-------|---------|
| `biomeos-atomic-deploy` | Atomic deployment |
| `biomeos-genomebin-v3` | genomeBin format |
| `biomeos-primal-sdk` | Primal development SDK |

---

## External Standards

Located in `../wateringHole/`:

| Document | Purpose |
|----------|---------|
| `ECOBIN_ARCHITECTURE_STANDARD.md` | ecoBin v2.0 standard |
| `UNIBIN_ARCHITECTURE_STANDARD.md` | UniBin standard |
| `PRIMAL_IPC_PROTOCOL.md` | IPC protocol |
| `SEMANTIC_METHOD_NAMING_STANDARD.md` | Method naming |
| `README.md` | Standards index |

---

## Session Archives

Historical documentation in `docs/sessions/`:

| Directory | Content |
|-----------|---------|
| `feb02-2026/` | TRUE Dark Forest implementation |
| `feb02-2026/archive/` | Archived session docs |

---

## Build & Test

```bash
# Generate crate documentation
cargo doc --workspace --no-deps --open

# Run tests
cargo test --workspace

# Check coverage
cargo llvm-cov --workspace
```

---

## Contributing

1. Follow ecoBin v2.0 standard (Pure Rust)
2. Use capability-based discovery
3. Follow PRIMAL_DEPLOYMENT_STANDARD
4. Update relevant documentation

---

**Status**: Documentation current as of January 29, 2026

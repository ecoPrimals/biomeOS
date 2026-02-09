# biomeOS Documentation Index

**Updated**: February 9, 2026

---

## Quick Navigation

| I want to... | Go to... |
|--------------|----------|
| Get started quickly | [START_HERE.md](START_HERE.md) |
| Understand the architecture | [README.md](README.md) |
| Deploy in 5 minutes | [QUICK_START.md](QUICK_START.md) |
| Check current status | [CURRENT_STATUS.md](CURRENT_STATUS.md) |
| See what changed | [CHANGELOG.md](CHANGELOG.md) |
| Deploy to USB/Pixel | [livespore-usb/README.md](livespore-usb/README.md) |
| Understand deployment standard | [specs/PRIMAL_DEPLOYMENT_STANDARD.md](specs/PRIMAL_DEPLOYMENT_STANDARD.md) |
| NAT traversal / mesh | [specs/MESH_IPC_METHODS_SPEC.md](specs/MESH_IPC_METHODS_SPEC.md) |
| Sovereign onion service | [docs/handoffs/SOVEREIGN_BEACON_MESH_HANDOFF_FEB06_2026.md](docs/handoffs/SOVEREIGN_BEACON_MESH_HANDOFF_FEB06_2026.md) |
| Over-NUCLEUS collective | [specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md](specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md) |
| Model cache | [docs/handoffs/NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md](docs/handoffs/NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md) |

---

## Root Documents

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | Project overview, architecture, quick start |
| [START_HERE.md](START_HERE.md) | Getting started guide with deep debt principles |
| [CURRENT_STATUS.md](CURRENT_STATUS.md) | Latest status, metrics, and remaining work |
| [QUICK_START.md](QUICK_START.md) | 5-minute deployment guide |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

---

## Specifications

Located in `specs/`:

| Document | Purpose |
|----------|---------|
| [PRIMAL_DEPLOYMENT_STANDARD.md](specs/PRIMAL_DEPLOYMENT_STANDARD.md) | Deployment standard v1.0 |
| [MESH_IPC_METHODS_SPEC.md](specs/MESH_IPC_METHODS_SPEC.md) | Mesh JSON-RPC method definitions |
| [SOVEREIGN_NAT_TRAVERSAL_EVOLUTION.md](specs/SOVEREIGN_NAT_TRAVERSAL_EVOLUTION.md) | NAT traversal design |
| [GENETIC_LINEAGE_EVOLUTION_SPEC.md](specs/GENETIC_LINEAGE_EVOLUTION_SPEC.md) | Genetic model spec |
| [DARK_FOREST_BEACON_GENETICS_SPEC.md](specs/DARK_FOREST_BEACON_GENETICS_SPEC.md) | Dark Forest beacon spec |
| [SOVEREIGN_BIRDSONG_MESH_SPEC.md](specs/SOVEREIGN_BIRDSONG_MESH_SPEC.md) | Birdsong mesh architecture |
| [PLASMODIUM_OVER_NUCLEUS_SPEC.md](specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md) | Over-NUCLEUS collective coordination |
| [NUCLEUS_BONDING_MODEL.md](specs/NUCLEUS_BONDING_MODEL.md) | Chemical bonding model for inter-NUCLEUS interaction |

---

## Handoffs & Reports

Located in `docs/handoffs/`:

### Latest (Feb 2026)

| Document | Content |
|----------|---------|
| [PLASMODIUM_OVER_NUCLEUS_HANDOFF_FEB09_2026.md](docs/handoffs/PLASMODIUM_OVER_NUCLEUS_HANDOFF_FEB09_2026.md) | Over-NUCLEUS collective coordination |
| [NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md](docs/handoffs/NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md) | Model cache + NestGate bugs |
| [TOADSTOOL_DISTRIBUTED_GPU_HANDOFF_FEB09_2026.md](docs/handoffs/TOADSTOOL_DISTRIBUTED_GPU_HANDOFF_FEB09_2026.md) | Distributed GPU inference |
| [SONGBIRD_ROUTER_EVOLUTION_HANDOFF_FEB08_2026.md](docs/handoffs/SONGBIRD_ROUTER_EVOLUTION_HANDOFF_FEB08_2026.md) | Songbird multi-path routing |
| [SOVEREIGN_MULTI_PATH_PROTOCOL_FEB08_2026.md](docs/handoffs/SOVEREIGN_MULTI_PATH_PROTOCOL_FEB08_2026.md) | Sovereign multi-path protocol |
| [TOR_INTEGRATION_HANDOFF_FEB07_2026.md](docs/handoffs/TOR_INTEGRATION_HANDOFF_FEB07_2026.md) | Tor gateway integration |

---

## Deployment

### USB LiveSpore

| Location | Content |
|----------|---------|
| `livespore-usb/x86_64/` | Intel/AMD deployment |
| `livespore-usb/aarch64/` | ARM64 deployment |
| `livespore-usb/x86_64/scripts/` | Deployment scripts |
| `livespore-usb/x86_64/primals/` | Primal binaries |

### Pixel 8a

| Location | Content |
|----------|---------|
| `pixel8a-deploy/` | Android deployment package |
| `pixel8a-deploy/primals/` | aarch64 binaries |

---

## Graph Definitions

Located in `graphs/`:

| Graph | Purpose |
|-------|---------|
| `tower_atomic_bootstrap.toml` | Tower Atomic deployment |
| `sovereign_onion_genome.toml` | Sovereign onion mesh deployment |

---

## Crate Documentation

### Core Crates (25 total)

| Crate | Purpose |
|-------|---------|
| `biomeos-core` | Core orchestration, discovery, P2P coordination, plasmodium, model cache |
| `biomeos-types` | Shared types, paths, capabilities, defaults |
| `biomeos-graph` | Graph execution engine |
| `biomeos-spore` | Deployment packaging, Dark Forest beacons |
| `biomeos-boot` | ISO/initramfs builder |

### API & UI Crates

| Crate | Purpose |
|-------|---------|
| `biomeos-api` | HTTP/WebSocket API server, capability handlers |
| `biomeos-ui` | Interactive UI orchestration, dynamic primal connections |
| `biomeos-cli` | Command-line interface |

### Deployment Crates

| Crate | Purpose |
|-------|---------|
| `biomeos-atomic-deploy` | Atomic deployment, Neural API, capability routing |
| `biomeos-genomebin-v3` | genomeBin format |
| `biomeos-primal-sdk` | Primal development SDK |
| `genome-deploy` | genomeBin deployment (1 justified unsafe: mmap) |
| `biomeos-deploy` | VM deployment and verification |

### Specialized Crates

| Crate | Purpose |
|-------|---------|
| `biomeos-federation` | Multi-tower federation |

---

## Environment Variables Reference

| Variable | Purpose | Default |
|----------|---------|---------|
| `FAMILY_ID` | Genetic lineage identifier | (required) |
| `BIOMEOS_SECURITY_PROVIDER` | Security/crypto primal | `beardog` |
| `BIOMEOS_NETWORK_PROVIDER` | Network orchestration primal | `songbird` |
| `BIOMEOS_REGISTRY_PROVIDER` | Service registry primal | `songbird` |
| `BIOMEOS_STORAGE_PROVIDER` | Persistence primal | `nestgate` |
| `BIOMEOS_STRICT_DISCOVERY` | Disable bootstrap name fallbacks | (unset) |
| `BIOMEOS_SOVEREIGN` | Enable sovereign mode | (unset) |
| `BIOMEOS_BIND_ADDRESS` | Network bind address | `::1` |
| `XDG_RUNTIME_DIR` | XDG runtime directory | `/run/user/$UID` |
| `BIOMEOS_SOCKET_DIR` | Override socket directory | (auto-resolved) |

---

## Build & Test

```bash
# Generate crate documentation
cargo doc --workspace --no-deps --open

# Run all tests (1,747 tests)
cargo test --workspace

# Clippy (0 warnings outside biomeos-boot)
cargo clippy --workspace

# Check coverage
cargo llvm-cov --workspace
```

---

## Contributing

1. Follow ecoBin v2.0 standard (Pure Rust, zero C deps)
2. Use capability-based discovery (no hardcoded primal names)
3. Follow PRIMAL_DEPLOYMENT_STANDARD
4. Keep files under 1000 lines
5. Ensure zero clippy warnings
6. Update relevant documentation

---

**Status**: Documentation current as of February 9, 2026  
**Tests**: 1,747 passing | **Clippy**: PASS

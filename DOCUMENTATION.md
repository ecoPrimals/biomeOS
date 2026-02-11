# biomeOS Documentation Index

**Updated**: February 11, 2026

---

## Quick Navigation

| I want to... | Go to... |
|--------------|----------|
| Get started quickly | [START_HERE.md](START_HERE.md) |
| Understand the architecture | [README.md](README.md) |
| Deploy in 5 minutes | [QUICK_START.md](QUICK_START.md) |
| Check current status | [CURRENT_STATUS.md](CURRENT_STATUS.md) |
| See what changed | [CHANGELOG.md](CHANGELOG.md) |
| See evolution roadmap | [specs/EVOLUTION_ROADMAP.md](specs/EVOLUTION_ROADMAP.md) |
| Deploy to USB/Pixel | [livespore-usb/README.md](livespore-usb/README.md) |
| Start a NUCLEUS | `biomeos nucleus start --mode full --node-id tower1` |

---

## Root Documents

| Document | Purpose |
|----------|---------|
| [README.md](README.md) | Project overview, architecture, quick start |
| [START_HERE.md](START_HERE.md) | Getting started with validated systems |
| [CURRENT_STATUS.md](CURRENT_STATUS.md) | Validated systems, bypasses, evolution needs |
| [QUICK_START.md](QUICK_START.md) | 5-minute deployment guide |
| [CHANGELOG.md](CHANGELOG.md) | Version history |

---

## Specifications (19 active)

See [specs/README.md](specs/README.md) for full index. Key specs:

| Spec | Purpose |
|------|---------|
| [EVOLUTION_ROADMAP.md](specs/EVOLUTION_ROADMAP.md) | Bypasses, multi-family, Plasmodium agents, waves |
| [PLASMODIUM_OVER_NUCLEUS_SPEC.md](specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md) | Over-NUCLEUS collective coordination |
| [NUCLEUS_BONDING_MODEL.md](specs/NUCLEUS_BONDING_MODEL.md) | Chemical bonding model |
| [NUCLEUS_ATOMIC_COMPOSITION.md](specs/NUCLEUS_ATOMIC_COMPOSITION.md) | Tower/Node/Nest/Full patterns |
| [NEURAL_API_ROUTING_SPECIFICATION.md](specs/NEURAL_API_ROUTING_SPECIFICATION.md) | Capability translation v2.0 |
| [PRIMAL_DEPLOYMENT_STANDARD.md](specs/PRIMAL_DEPLOYMENT_STANDARD.md) | Deterministic deployment v1.0 |

47 specs archived to `ecoPrimals/archive/biomeos-feb09-2026/specs/`.

---

## Handoffs & Evolution Reports (15 active)

### Relay-Assisted Coordinated Punch (Feb 11, 2026)

| Document | Team | Priority |
|----------|------|----------|
| [RELAY_ASSISTED_COORDINATED_PUNCH_HANDOFF_FEB11_2026.md](docs/handoffs/RELAY_ASSISTED_COORDINATED_PUNCH_HANDOFF_FEB11_2026.md) | Songbird + BearDog | HIGH |

### Covalent Bond Investigation (Feb 10, 2026)

| Document | Team | Priority |
|----------|------|----------|
| [COVALENT_BOND_EVOLUTION_HANDOFF_FEB10_2026.md](docs/handoffs/COVALENT_BOND_EVOLUTION_HANDOFF_FEB10_2026.md) | Songbird | **CRITICAL** |

### Primal Evolution Handoffs (Feb 9, 2026)

| Document | Team | Priority |
|----------|------|----------|
| [SONGBIRD_EVOLUTION_HANDOFF_FEB09_2026.md](docs/handoffs/SONGBIRD_EVOLUTION_HANDOFF_FEB09_2026.md) | Songbird | HIGH |
| [NESTGATE_EVOLUTION_HANDOFF_FEB09_2026.md](docs/handoffs/NESTGATE_EVOLUTION_HANDOFF_FEB09_2026.md) | NestGate | HIGH |
| [SQUIRREL_EVOLUTION_HANDOFF_FEB09_2026.md](docs/handoffs/SQUIRREL_EVOLUTION_HANDOFF_FEB09_2026.md) | Squirrel | MEDIUM |
| [TOADSTOOL_EVOLUTION_HANDOFF_FEB09_2026.md](docs/handoffs/TOADSTOOL_EVOLUTION_HANDOFF_FEB09_2026.md) | Toadstool | MEDIUM |
| [BEARDOG_EVOLUTION_HANDOFF_FEB09_2026.md](docs/handoffs/BEARDOG_EVOLUTION_HANDOFF_FEB09_2026.md) | BearDog | LOW |
| [BIOMEOS_EVOLUTION_HANDOFF_FEB09_2026.md](docs/handoffs/BIOMEOS_EVOLUTION_HANDOFF_FEB09_2026.md) | biomeOS | COMPLETE |

### System Handoffs (Feb 7-9, 2026)

| Document | Content |
|----------|---------|
| [SQUIRREL_NEURAL_API_BRIDGE_HANDOFF_FEB09_2026.md](docs/handoffs/SQUIRREL_NEURAL_API_BRIDGE_HANDOFF_FEB09_2026.md) | AI bridge validation |
| [PLASMODIUM_OVER_NUCLEUS_HANDOFF_FEB09_2026.md](docs/handoffs/PLASMODIUM_OVER_NUCLEUS_HANDOFF_FEB09_2026.md) | Over-NUCLEUS collective |
| [NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md](docs/handoffs/NESTGATE_MODEL_CACHE_HANDOFF_FEB09_2026.md) | Model cache + NestGate bugs |
| [TOADSTOOL_DISTRIBUTED_GPU_HANDOFF_FEB09_2026.md](docs/handoffs/TOADSTOOL_DISTRIBUTED_GPU_HANDOFF_FEB09_2026.md) | Distributed GPU inference |
| [SONGBIRD_ROUTER_EVOLUTION_HANDOFF_FEB08_2026.md](docs/handoffs/SONGBIRD_ROUTER_EVOLUTION_HANDOFF_FEB08_2026.md) | Songbird multi-path routing |
| [SOVEREIGN_MULTI_PATH_PROTOCOL_FEB08_2026.md](docs/handoffs/SOVEREIGN_MULTI_PATH_PROTOCOL_FEB08_2026.md) | Sovereign multi-path protocol |
| [TOR_INTEGRATION_HANDOFF_FEB07_2026.md](docs/handoffs/TOR_INTEGRATION_HANDOFF_FEB07_2026.md) | Tor gateway integration |

14 earlier handoffs archived to `ecoPrimals/archive/biomeos-feb09-2026/handoffs/`.

---

## Environment Variables Reference

### Core

| Variable | Purpose | Default |
|----------|---------|---------|
| `NODE_ID` | Node identifier | (required for Neural API) |
| `FAMILY_ID` | Genetic lineage identifier | (from .family.seed) |
| `BIOMEOS_SECURITY_PROVIDER` | Security/crypto primal | `beardog` |
| `BIOMEOS_NETWORK_PROVIDER` | Network orchestration primal | `songbird` |
| `BIOMEOS_REGISTRY_PROVIDER` | Service registry primal | `songbird` |
| `BIOMEOS_STORAGE_PROVIDER` | Persistence primal | `nestgate` |
| `BIOMEOS_STRICT_DISCOVERY` | Disable bootstrap name fallbacks | (unset) |
| `BIOMEOS_SOVEREIGN` | Enable sovereign mode | (unset) |
| `BIOMEOS_BIND_ADDRESS` | Network bind address | `::1` |
| `XDG_RUNTIME_DIR` | XDG runtime directory | `/run/user/$UID` |
| `BIOMEOS_SOCKET_DIR` | Override socket directory | (auto-resolved via SystemPaths) |
| `SONGBIRD_MESH_PORT` | Plasmodium remote gate port | `8080` |
| `SONGBIRD_HTTP_PORT` | Songbird HTTP listen port (beacon payload) | `8080` |

### AI Bridge

| Variable | Purpose | Default |
|----------|---------|---------|
| `HTTP_REQUEST_PROVIDER_SOCKET` | Squirrel HTTP capability socket | (auto-discover) |
| `AI_HTTP_PROVIDERS` | Enabled cloud AI providers | (none) |
| `ANTHROPIC_API_KEY` | Anthropic Claude API key | (none) |
| `OPENAI_API_KEY` | OpenAI GPT API key | (none) |

### Socket Paths

| Variable | Purpose | Default |
|----------|---------|---------|
| `BEARDOG_SOCKET` | BearDog socket path | (auto-discover via SystemPaths) |
| `SONGBIRD_SOCKET` | Songbird socket path | (auto-discover via SystemPaths) |
| `SONGBIRD_SECURITY_PROVIDER` | BearDog socket for Songbird TLS | (auto-discover) |
| `NEURAL_API_SOCKET` | Neural API socket path | (auto-discover via SystemPaths) |
| `NESTGATE_JWT_SECRET` | NestGate authentication secret | (auto-generate) |

---

## Build & Test

```bash
# Build
cargo build --workspace

# Test (2,539 tests)
cargo test --workspace

# Coverage (~57% region)
cargo llvm-cov --workspace

# Clippy (0 warnings, entire workspace)
cargo clippy --workspace

# Format
cargo fmt --check

# Start NUCLEUS (Pure Rust)
biomeos nucleus start --mode full --node-id tower1
```

---

## Fossil Archive

Previous docs archived to `ecoPrimals/archive/biomeos-feb09-2026/`.
See `ARCHIVE_INDEX.md` in archive for details.

---

**Documentation current as of February 11, 2026**
**Active: 19 specs, 15 handoffs, 10 scripts**
**Tests: 2,539 passing (56.75% region coverage) | Clippy: PASS | Docs: Full coverage**

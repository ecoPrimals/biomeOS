# biomeOS Documentation Index

**Updated**: April 13, 2026

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
| Deployment graphs (42 incl. provenance trio) | [graphs/README.md](graphs/README.md) |
| See evolution handoffs (v2.43–v3.08) | [wateringHole/handoffs/](../../infra/wateringHole/handoffs/) |
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

## Specifications (24 active)

See [specs/README.md](specs/README.md) for full index. Key specs:

| Spec | Purpose |
|------|---------|
| [EVOLUTION_ROADMAP.md](specs/EVOLUTION_ROADMAP.md) | Bypasses, multi-family, Plasmodium agents, waves |
| [PLASMODIUM_OVER_NUCLEUS_SPEC.md](specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md) | Over-NUCLEUS collective coordination |
| [NUCLEUS_BONDING_MODEL.md](specs/NUCLEUS_BONDING_MODEL.md) | Chemical bonding model |
| [NUCLEUS_ATOMIC_COMPOSITION.md](specs/NUCLEUS_ATOMIC_COMPOSITION.md) | Tower/Node/Nest/Full patterns |
| [NEURAL_API_ROUTING_SPECIFICATION.md](specs/NEURAL_API_ROUTING_SPECIFICATION.md) | Capability translation v2.0 |
| [PRIMAL_DEPLOYMENT_STANDARD.md](specs/PRIMAL_DEPLOYMENT_STANDARD.md) | Deterministic deployment v1.0 |

47 specs archived to `ecoPrimals/archive/` (fossil record).

---

## Handoffs & Evolution Reports

Handoffs live in the central **wateringHole** at `ecoPrimals/wateringHole/handoffs/`.

### Recent (April 2026)

| Document | Focus | Date |
|----------|-------|------|
| BIOMEOS_V304_COMPOSITION_ELEVATION_DEEP_DEBT_CLEANUP | Composition e2e proven (15 integration tests), `composition.health` standard, `lifecycle.composition` enriched dashboard, hardcoding→`primal_names::` constants, blake3 pure-only, deep debt cleanup VII, 7,784 tests | Apr 12, 2026 |
| BIOMEOS_V303_DEEP_DEBT_CLEANUP_VI | `Box<dyn Error>`→anyhow, `#[allow]`→`#[expect]` bulk migration, hot-path clone reduction, 7,749 tests | Apr 11, 2026 |
| BIOMEOS_V302_PORTABILITY_DEBT_INFERENCE_WIRE | `capability.resolve` metrics, `inference.register_provider`, expanded `inference.*` routes, 7,749 tests | Apr 11, 2026 |
| BIOMEOS_V301_PRIMALSPRING_GAP_RESOLUTION_DEEP_DEBT_V | 7 primalSpring gaps resolved, capability.resolve, consumed_capabilities, lifecycle.composition, deep debt overstep V, 7,726 tests | Apr 11, 2026 |
| BIOMEOS_V300_DEEP_DEBT_CLEANUP_IV_DOC_EVOLUTION | Dep cleanup (itertools, async-trait×3), native async trait migration, hardcoding→constants, orphan deletion, license harmonization, root doc cleanup, 7,724 tests | Apr 9, 2026 |
| BIOMEOS_V299_DEEP_DEBT_OVERSTEP_CLEANUP_III | 3 large files smart-refactored, #[allow]→#[expect], comprehensive zero-debt audit, 7,695 tests | Apr 8, 2026 |
| BIOMEOS_V298_GAP_MATRIX_11_BTSP_INSECURE_GUARD | BTSP validate_insecure_guard() wired into all 3 server paths, security posture logging | Apr 8, 2026 |
| BIOMEOS_V297_DEEP_DEBT_OVERSTEP_CLEANUP_II | #![forbid(unsafe_code)] on all binaries, niche hardcoding→constants, 3 large files refactored | Apr 8, 2026 |

### Earlier (March 2026)

| Document | Focus | Date |
|----------|-------|------|
| BIOMEOS_V273_CROSS_GATE_DEPLOYMENT_EVOLUTION | `route.register` batch API, cross-gate executor forwarding, 7,186 tests | Mar 28, 2026 |
| BIOMEOS_V268_DEEP_AUDIT_HARDCODING_EVOLUTION | Blocking-in-async fix, `/tmp` + IP centralization, license reconciliation | Mar 27, 2026 |
| BIOMEOS_V257_COVERAGE_PUSH_FILE_SPLITS | 6,998 tests, 90%+ coverage, 0 files >1000 LOC | Mar 20, 2026 |
| BIOMEOS_V254_CONCURRENCY_EVOLUTION_COVERAGE_PUSH | Sleep elimination, concurrency evolution, 6,169 tests | Mar 19, 2026 |
| BIOMEOS_V252_CAPABILITY_FIRST_DISCOVERY | Capability-named sockets, MCP aggregation, Provenance metadata type | Mar 18, 2026 |
| BIOMEOS_V251_ECOSYSTEM_ABSORPTION | IPC resilience, proptest, MCP tools, capability cost/deps, 8 springs absorbed | Mar 18, 2026 |

### Archived (February 2026)

Earlier handoffs (Feb 7–11, 2026) archived to `ecoPrimals/archive/biomeos-mar14-2026/handoffs/`.

Topics include: relay-assisted coordinated punch, covalent bond evolution, primal evolution (Songbird, NestGate, Squirrel, Toadstool, BearDog), Squirrel-Neural API bridge, Plasmodium over-NUCLEUS, NestGate model cache, Toadstool distributed GPU, Songbird router evolution, sovereign multi-path protocol, Tor integration.

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

# Test (7,784 tests — 0 ignored — fully concurrent)
cargo test --workspace

# Coverage (90%+ line / function / region, llvm-cov verified)
cargo llvm-cov --workspace

# Clippy (0 warnings, pedantic+nursery, -D warnings)
cargo clippy --workspace

# Format
cargo fmt --check

# Start NUCLEUS (Pure Rust)
biomeos nucleus start --mode full --node-id tower1
```

---

## Fossil Archive

Previous docs archived to `ecoPrimals/archive/biomeos-mar14-2026/`.
See `ARCHIVE_INDEX.md` in archive for details.

---

**Documentation current as of April 13, 2026 (v3.08)**
**Active: 24 specs, handoffs in wateringHole (v2.43–v3.08), 4 scripts**
**Deploy graphs: 40 (incl. provenance trio: loamspine, rhizocrypt, sweetgrass, provenance_trio)**
**Architecture: capability-based discovery compliant (`CAPABILITY_BASED_DISCOVERY_STANDARD` v1.2.0); identity-based discovery APIs removed**
**Tests: 7,784 passing (0 failures, 0 ignored, fully concurrent) | 90%+ line / function / region (llvm-cov) | Clippy: PASS (0 warnings, pedantic+nursery, `-D warnings`) | Docs: Full coverage | C deps: 0 | Unsafe: 0 | Deprecated: 0 | TODO/FIXME: 0 | Blocking debt: 0 | Hardcoded primal names: 0**

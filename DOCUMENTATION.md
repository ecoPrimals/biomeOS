# biomeOS Documentation Index

**Updated**: May 27, 2026

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
| Deployment graphs (43 incl. provenance trio + membrane) | [graphs/README.md](graphs/README.md) |
| See evolution handoffs (v2.43–v3.78) | [wateringHole/handoffs/](../../infra/wateringHole/handoffs/) |
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

## Specifications (26 active)

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

Handoffs live in the central **wateringHole** at `ecoPrimals/infra/wateringHole/handoffs/`.

### Recent (May 2026)

| Document | Focus | Date |
|----------|-------|------|
| BIOMEOS_V378_DEEP_DEBT_CLEANUP_MAY27_2026 | Hardcoded primal names → constants, large file refactor, live_discovery REST wired | May 27, 2026 |
| BIOMEOS_V377_NUCLEUS_SPORE_GATEWAY_MAY27_2026 | NUCLEUS spore ingest/emit subcommands, nest_ingest_spore signal, Neural API wiring | May 27, 2026 |
| BIOMEOS_V375_CROSS_GATE_MESH_DISPATCH_MAY24_2026 | Songbird mesh cross-gate dispatch, shadow deploy membrane gate | May 24, 2026 |
| BIOMEOS_V373_DEEP_DEBT_CAPABILITY_COMPOSITION_MAY24_2026 | Capability-domain composition, weights/ refactor, port helper rename, `DOMAIN_PRIMAL_BOOTSTRAP` | May 24, 2026 |
| BIOMEOS_V367_WAVE43_ROUTING_WEIGHTS_MAY23_2026 | Persistent routing weights, utilization tracking | May 23, 2026 |
| BIOMEOS_V366_WAVE38_CROSS_GATE_RELAY_MAY22_2026 | Cross-gate dispatch, songbird relay fallback | May 22, 2026 |
| BIOMEOS_V365_WAVE31_SCHEMA_ALIGNMENT_MAY20_2026 | `primal.list` schema alignment | May 20, 2026 |
| BIOMEOS_V364_WS2_CROSS_SPRING_SYNC_MAY19_2026 | `nest.sync` cross-spring provenance exchange | May 19, 2026 |
| BIOMEOS_V363_UPSTREAM_GAPS_RESOLVED_MAY19_2026 | All 17 signals as first-class routes | May 19, 2026 |
| BIOMEOS_V360_STADIAL_GATE_READINESS_MAY17_2026 | Stadial gate readiness, braid signal tier | May 17, 2026 |
| BIOMEOS_V359_DEEP_DEBT_REFACTORING_MAY15_2026 | Signal dispatch, capability interception | May 15, 2026 |

### April 2026

| Document | Focus | Date |
|----------|-------|------|
| BIOMEOS_V322_DEEP_DEBT_EVOLUTION | UDS dual-protocol auto-detect (HTTP + raw JSON-RPC), Box\<dyn Error\>→anyhow in chimera codegen, stale demos removed (songbird_universal_ui_demo + reqwest), tools primal lists synced with registry, 4 tools compile fixes | Apr 20, 2026 |
| BIOMEOS_V321_DEEP_DEBT_EVOLUTION | Cross-arch armv7 fix (cast.rs u64), hardcoded IPs completed (dns_sd.rs, init.rs), runtime paths centralized (path_builder.rs, defaults.rs), ANDROID_RUNTIME_BASE + RFC5737_ROUTE_PROBE constants | Apr 19, 2026 |
| BIOMEOS_V320_DEEP_DEBT_EVOLUTION | nucleus.rs smart refactor (820→780L), hardcoded IPs→constants, #[allow]→#[expect], tensor translations (33 methods), nucleus_complete.toml expanded (NestGate streaming + barraCuda/coralReef nodes) | Apr 20, 2026 |
| BIOMEOS_V319_DATA_DRIVEN_LAUNCH_PROFILES | Data-driven launch profiles (nucleus + spawner TOML), port constants, translation_loader agnostic counter, walkdir pruned from 3 crates | Apr 20, 2026 |
| BIOMEOS_V318_SPRING_AUDIT_FIXES | .gitignore secret module fix, TCP port conflict avoidance (bind-probe), post-spawn Neural API auto-registration, discovery_init pub(crate) | Apr 20, 2026 |
| BIOMEOS_V313_DEEP_DEBT_EVOLUTION | Deep debt cleanup: hardcoding→capability-based (spore/CLI/verify), deprecated API removal, learn_from_event wired, topology live probes, capability.rs 804→744L, tools/ ring-free | Apr 12, 2026 |
| BIOMEOS_V312_COMPOSITION_FORWARDING_GRAPH_LIST_FIX | Composition forwarding via Tower Atomic relay (BTSP, socket path, method prefix), graph.list recursive scan, primalSpring benchScale validation | Apr 14, 2026 |
| BIOMEOS_V311_TCP_ONLY_GRAPH_BOOTSTRAP_FIX | TCP-only graph bootstrap: 4 root causes (graph.load loop, graphs_dir resolution, TCP self-registration, silent parse failures), auto-scan all graphs on startup | Apr 12, 2026 |
| BIOMEOS_V311_DEEP_DEBT_SELF_KNOWLEDGE_EVOLUTION | Deep debt audit + self-knowledge evolution: capability-based naming, resolved provider logs, configurable TCP host, dependency analysis | Apr 12, 2026 |
| BIOMEOS_V310_DEEP_DEBT_HANDOFF | Smart refactoring (5 files >800L), hardcoding to capability-based resolution, dependency evolution, idiomatic Rust, 7,784 tests | Apr 12, 2026 |
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

# Test (8,036 tests — fully concurrent)
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

**Documentation current as of May 27, 2026 (v3.78)**
**Active: 26 specs, handoffs in wateringHole (v2.43–v3.78), 4 scripts**
**Deploy graphs: 43 (incl. provenance trio + membrane_deploy)**
**Architecture: capability-based discovery compliant (`CAPABILITY_BASED_DISCOVERY_STANDARD` v1.2.0); identity-based discovery APIs removed; data-driven launch profiles; cross-arch armv7 safe; UDS dual-protocol auto-detect; BTSP Phase 3 LIVE; membrane + nucleated composition models; signal-tier announce protocol; method_gate modularized; adaptive routing weights (redb-persistent); attestation verification; capability-domain composition health; Songbird mesh cross-gate dispatch; NUCLEUS spore gateway (ingest/emit)**
**Tests: 8,036 workspace-wide (0 failures, fully concurrent) | 90%+ line / function / region (llvm-cov) | Clippy: PASS (0 warnings, pedantic+nursery, `-D warnings`) | Docs: Full coverage | C deps: 0 | Unsafe: 0 | Deprecated: 0 | TODO/FIXME: 0 | Blocking debt: 0 | Hardcoded primal names: 0 | Production files >800L: 0**

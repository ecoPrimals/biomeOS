# biomeOS Documentation Index

**Updated**: March 28, 2026

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
| Deployment graphs (40 incl. provenance trio) | [graphs/README.md](graphs/README.md) |
| See evolution handoffs (v2.43–v2.74) | [wateringHole/handoffs/](../../infra/wateringHole/handoffs/) |
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

### Recent (March 2026)

| Document | Focus | Date |
|----------|-------|------|
| BIOMEOS_V273_CROSS_GATE_DEPLOYMENT_EVOLUTION | `route.register` batch API, `gate` field on GraphNode, GateRegistry, cross-gate executor forwarding, 7,186 tests | Mar 28, 2026 |
| BIOMEOS_V272_ARM64_CROSS_COMPILATION | ARM64 cross-compilation (`aarch64-unknown-linux-musl`), 9.6 MB static binary, all 6 genomeBins ARM64-ready | Mar 28, 2026 |
| BIOMEOS_V271_DEEP_DEBT_MULTI_TRANSPORT_EVOLUTION | Multi-transport IPC, deep debt resolution, zero-copy optimizations, BearDog client cleanup, 7,167 tests | Mar 28, 2026 |
| BIOMEOS_V270_MULTI_TRANSPORT_IPC_EVOLUTION | Neural router Unix→universal transport, TransportEndpoint Serialize/Deserialize, P0/P1 gaps resolved | Mar 28, 2026 |
| BIOMEOS_V269_ZEROCOPY_TOKIO_COVERAGE_EVOLUTION | Zero-copy Value::take(), tokio workspace unification, base64 upgrade, deny.toml cleanup | Mar 28, 2026 |
| BIOMEOS_V268_DEEP_AUDIT_HARDCODING_EVOLUTION | Blocking-in-async fix, `/tmp` + IP centralization, license reconciliation, phase2 path cleanup | Mar 27, 2026 |
| BIOMEOS_V267_REMAINING_DEBT_CLEANUP | Caller-agnostic lineage, roster evolution, comprehensive debt scan (0 TODO/FIXME/HACK) | Mar 22, 2026 |
| BIOMEOS_V266_PRIMALSPRING_ALIGNED_CAPABILITY_DISCOVERY | 5-tier centralized discovery, GeneticLineage→BearDog, niche self-knowledge, Neural API early-bind | Mar 22, 2026 |
| BIOMEOS_V265_DEEP_DEBT_EXECUTION_ZERO_COPY | Tower refactored, zero-copy, hardcode evolution, flaky test fixes | Mar 22, 2026 |
| BIOMEOS_V264_FLAKY_TEST_HARDENING_COVERAGE_SERDE_YML | serde_yml migration, large file refactoring, flaky test hardening, 7,135 tests | Mar 22, 2026 |
| BIOMEOS_V257_COVERAGE_PUSH_FILE_SPLITS | 6,998 tests, 90%+ coverage, 0 files >1000 LOC, CWD evolution | Mar 20, 2026 |
| BIOMEOS_V255_COVERAGE_HARDENING | Deep debt audit, zero-copy JsonRpcVersion, large file refactoring | Mar 20, 2026 |
| BIOMEOS_V254_CONCURRENCY_EVOLUTION_COVERAGE_PUSH | Sleep elimination, concurrency evolution, 6,169 tests | Mar 19, 2026 |
| BIOMEOS_V253_COORDINATION_EXTRICATION | Coordination pattern audit for primalSpring | Mar 18, 2026 |
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

# Test (7,192 tests — ~135 ignored hardware-dependent — use --ignored --test-threads=1 for those)
cargo test --workspace

# Coverage (90%+ all metrics, llvm-cov verified)
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

Previous docs archived to `ecoPrimals/archive/biomeos-mar14-2026/`.
See `ARCHIVE_INDEX.md` in archive for details.

---

**Documentation current as of March 28, 2026**
**Active: 24 specs, handoffs in wateringHole (v2.43–v2.74), 4 scripts**
**Deploy graphs: 40 (incl. provenance trio: loamspine, rhizocrypt, sweetgrass, provenance_trio)**
**Tests: 7,192 passing (0 failures), ~135 ignored | 90%+ coverage (llvm-cov) | Clippy: PASS (0 warnings, pedantic+nursery) | Docs: Full coverage | C deps: 0 | Unsafe: 0 production**

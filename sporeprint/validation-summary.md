+++
title = "biomeOS Validation Summary"
description = "Orchestration kernel — guideStone startup contract + HEALTH-01 compliant + NUCLEUS supervisor, 27 capability domains, 320+ translations, 43 deploy graphs, zero blocking debt"
date = 2026-06-11

[taxonomies]
primals = ["biomeos", "beardog", "songbird", "skunkbat", "toadstool", "coralreef", "barracuda", "nestgate", "rhizocrypt", "loamspine", "sweetgrass", "squirrel", "petaltongue"]
+++

## Status

- **7,983+ tests** workspace-wide, 0 failures, fully concurrent
- **90%+ coverage** region / function / line (llvm-cov workspace-wide)
- **v4.26** — Wave 111: riboCipher transport signal detection in API + neural-api sockets (Stream 7 convergent evolution, WARN phase)
- **v4.25** — Wave 111: Deep debt — security fail-closed, real metrics, agnostic naming, router refactor, all 26 crates #![forbid(unsafe_code)]
- **v4.24** — Wave 111: Divergence pressure — stale registration pruning + partition-aware routing
- **v4.23** — Wave 110: Deep debt cleanup — Duration constant consolidation + magic number elimination
- **v4.22** — Wave 109: guideStone startup contract (`--bind-mode`) + HEALTH-01 (`{status,primal,version,uptime_s}`)
- **v4.19** — Wave 107: NUCLEUS auto-registration with songBird discovery service
- **v4.18** — Wave 106: Graceful TCP fallback for SELinux/Android substrates
- **v4.17** — Wave 106: NUCLEUS supervision — automatic primal restart on crash
- **v4.16** — Wave 106: Deep debt — error chains, hardcoding elimination, smart extraction
- **v4.05** — Wave 75c: Deep debt evolution (5 test extractions ~1989L, hardcode→constants, &Vec→&[T], idiomatic Rust sweep)
- **v4.04** — Wave 75b: Consolidation sprint (Result<_,String> sweep complete, map_err(format!) eliminated, L5 perceptron remote infer wired)
- **v4.03** — Wave 75: SONGBIRD_FEDERATION_ENABLED alignment, AtomicType dedup, typed errors
- **v4.02** — Wave 74c: String error evolution (thiserror/anyhow), API visibility tightening, SSOT hardening
- **v4.01** — Wave 74b: Zero hardcoded primal names, mock→neutral_default, deprecated cleanup, test extraction wave 4
- **v4.00** — Wave 74: composition.patterns.reload, perceptron wire contract verified
- **v3.99** — Wave 73b: L5 perceptron consumer (36-dim features, shadow mode), test extraction wave 3
- **v3.98** — Wave 73: gate.register + gate.list JSON-RPC, GeneticsTier/EscalationManager→thiserror
- **v3.97** — Wave 72+: map_err sweep (27/28), test extraction wave 2 (7 files), HTTP transport removed, env safety
- **v3.96** — Wave 72: env SSOT +14 constants, 56 map_err→context, test extraction wave 1 (5 files)
- **v3.95** — Wave 71+: shadow analysis, PathwayLearner, perceptron prep
- **v3.94** — Wave 71: L4 weighted routing LIVE, topology affinity, --tcp-only deprecated
- **v3.86** — Wave 60b: DH-1 complete (zero /tmp + zero env::temp_dir()), inline test extraction
- **v3.85** — Wave 60: manifest.gate_profile Neural API, DH-1 /tmp hardcoding eliminated
- **v3.84** — Deep Debt W58b (wired 22 more env var constants, test module extraction, zero production files >800L)
- **v3.83** — Env var centralization W58 (env_config::vars SSOT, ~90% of env::var call sites wired)
- **v3.82** — Deep Debt W57 (nucleus_ingest module split, bearDog fix, LogConfig XDG, flate2 pure Rust)
- **v3.81** — NC-1.4 canonical pseudoSpore validation + NC-1.emit full materialization
- **v3.80** — Deep Debt W56 (routing.rs 920→551L, nucleus.rs 883→605L, rustix 1.x, capability-based config)
- **28 capability domains**, **320+ translations** across 13 primals
- **19 atomic signal graphs** across 5 tiers (tower, node, nest, meta, braid)
- **43 deploy graphs** (incl. membrane_deploy, provenance trio)
- **20 niche templates** (+ RootPulse, soil-microbiome, ecology)
- **26 workspace crates**
- **Zero blocking debt** — 0 unsafe, 0 C deps, 0 TODO/FIXME, 0 clippy warnings
- **Edition 2024** all crates, ecoBin v3.0 compliant
- **Cross-arch** — x86_64 + aarch64 + armv7 (USB + Pixel + Raspberry Pi)
- **Security A++** — 100/100, Dark Forest Gate, BTSP Phase 3 encrypted framing
- **scyBorg triple-copyleft** — AGPL-3.0-or-later + ORC + CC-BY-SA 4.0

## Architecture

biomeOS is the orchestration kernel — it composes all other primals into
functioning ecosystems. It does not perform compute, storage, or security
itself; it coordinates the primals that do.

- **NUCLEUS** — process supervision, startup ordering, auto-resurrection
- **Neural API** — JSON-RPC routing, capability translation, signal dispatch
- **Plasmodium** — multi-machine meld/split/mix, cross-device federation
- **Dark Forest** — zero metadata leakage, encrypted beacons, genetic model
- **RootPulse** — emergent provenance pattern (rhizoCrypt + loamSpine + sweetGrass)

## Signal Dispatch (5 tiers, 19 graphs)

| Tier | Signals | Purpose |
|------|---------|---------|
| tower | publish, authenticate, discover, health, bootstrap | Security + mesh orchestration |
| node | compute | Compute-level dispatch |
| nest | store, commit, retrieve, sync | Storage + content + cross-spring exchange |
| braid | partial_update, complete | Provenance braid lifecycle |
| meta | observe, intent, render, health, deploy | Observability + composition |

## NUCLEUS Modes

| Mode | Primals | Use Case |
|------|---------|----------|
| Tower | 3 (BearDog, Songbird, SkunkBat) | Security-only |
| Node | 6 | Compute node |
| Nest | 8 | Storage node |
| Core | 5 | Legacy minimal |
| Full | 12 | Full ecosystem |

## Key Capabilities

- **Neural API routing** — semantic fallback, signal-tier interception, cross-gate forwarding
- **Capability-based discovery** — 5-tier protocol, taxonomy-driven, zero identity coupling
- **BTSP** — negotiate, escalate, status; cleartext→enforced one-way transition
- **Deploy graph execution** — atomic types (Tower/Node/Nest/Nucleus), graph signing (BLAKE3+Ed25519)
- **Composition health** — pipeline readiness (content + compute), adaptive daemon surface
- **Stale socket cleanup** — startup scan + PID files + shutdown hygiene
- **Cross-spring sync** — `nest.sync` signal for provenance exchange via trio pipeline

## Evolution Timeline (recent)

| Version | Date | Highlight |
|---------|------|-----------|
| v3.84 | May 28 | Deep Debt W58b — wired 22 more env var constants, test module extraction, zero production files >800L |
| v3.83 | May 28 | Env var centralization W58 — env_config::vars SSOT, ~90% of env::var call sites wired |
| v3.82 | May 27 | Deep Debt W57 — nucleus_ingest module split, bearDog casing fix, LogConfig XDG, flate2 pure Rust |
| v3.81 | May 27 | NC-1.4 canonical pseudoSpore validation + NC-1.emit full materialization |
| v3.80 | May 27 | Deep debt W56 — smart refactoring (routing 920→551L, nucleus 883→605L), rustix 1.x, capability-based config |
| v3.79 | May 27 | Wave 55 Gateway Completion — signal graph synced, emit pipeline, receipt shape aligned |
| v3.78 | May 27 | Deep debt cleanup — hardcoded primal names → constants, large file refactor, live_discovery REST |
| v3.77 | May 27 | NUCLEUS spore gateway (ingest/emit), nest_ingest_spore signal, Neural API wiring |
| v3.75 | May 24 | Songbird mesh cross-gate dispatch, shadow deploy membrane gate |
| v3.73 | May 24 | Capability-domain composition, weights/ refactor, port helper rename |
| v3.72 | May 24 | health.check normalized to "alive" |
| v3.71 | May 23 | Membrane composition model live execution |
| v3.70 | May 23 | Weight health introspection, attestation verification, persistent startup |
| v3.69 | May 22 | Persistent routing weights, utilization tracking |
| v3.66 | May 22 | Cross-gate dispatch, songbird relay fallback |
| v3.65 | May 20 | `primal.list` Wave 31 schema alignment |
| v3.64 | May 19 | WS-2: `nest.sync` cross-spring provenance exchange |

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog-status-science-and-evolution/) on primals.eco
- [Orchestration Architecture](https://primals.eco/architecture/) — NUCLEUS, Neural API, federation

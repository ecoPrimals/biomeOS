+++
title = "biomeOS Validation Summary"
description = "Orchestration kernel — 8,053 tests, 27 capability domains, 320+ translations, 19 signal graphs, 43 deploy graphs, zero blocking debt"
date = 2026-05-28

[taxonomies]
primals = ["biomeos", "beardog", "songbird", "skunkbat", "toadstool", "coralreef", "barracuda", "nestgate", "rhizocrypt", "loamspine", "sweetgrass", "squirrel", "petaltongue"]
+++

## Status

- **8,053 tests** workspace-wide (1,315 `biomeos-atomic-deploy`), 0 failures, fully concurrent
- **90%+ coverage** region / function / line (llvm-cov workspace-wide)
- **v3.84** — Deep Debt W58b (wired 22 more env var constants, test module extraction, zero production files >800L)
- **v3.83** — Env var centralization W58 (env_config::vars SSOT, ~90% of env::var call sites wired)
- **v3.82** — Deep Debt W57 (nucleus_ingest module split, bearDog fix, LogConfig XDG, flate2 pure Rust)
- **v3.81** — NC-1.4 canonical pseudoSpore validation + NC-1.emit full materialization
- **v3.80** — Deep Debt W56 (routing.rs 920→551L, nucleus.rs 883→605L, rustix 1.x, capability-based config)
- **27 capability domains**, **320+ translations** across 13 primals
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

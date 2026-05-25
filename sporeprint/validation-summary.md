+++
title = "biomeOS Validation Summary"
description = "Orchestration kernel — 8,026 tests, 27 capability domains, 320+ translations, 17 signal graphs, 43 deploy graphs, zero blocking debt"
date = 2026-05-24

[taxonomies]
primals = ["biomeos", "beardog", "songbird", "skunkbat", "toadstool", "coralreef", "barracuda", "nestgate", "rhizocrypt", "loamspine", "sweetgrass", "squirrel", "petaltongue"]
+++

## Status

- **8,026 tests** workspace-wide (1,315 `biomeos-atomic-deploy`), 0 failures, fully concurrent
- **90%+ coverage** region / function / line (llvm-cov workspace-wide)
- **v3.75** — production ready, deep debt CLEAN, Songbird mesh cross-gate dispatch
- **27 capability domains**, **320+ translations** across 13 primals
- **17 atomic signal graphs** across 5 tiers (tower, node, nest, meta, braid)
- **43 deploy graphs** (incl. membrane_deploy, provenance trio)
- **20 niche templates** (+ RootPulse, soil-microbiome, ecology)
- **25 workspace crates**
- **Zero blocking debt** — 0 unsafe, 0 C deps, 1 tracked TODO, 0 clippy warnings
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

## Signal Dispatch (5 tiers, 17 graphs)

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

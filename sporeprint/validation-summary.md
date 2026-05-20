+++
title = "biomeOS Validation Summary"
description = "Orchestration kernel — 7,924+ tests, 27 capability domains, 320+ translations, 17 signal graphs, 40 deploy graphs, zero blocking debt"
date = 2026-05-20

[taxonomies]
primals = ["biomeos", "beardog", "songbird", "skunkbat", "toadstool", "coralreef", "barracuda", "nestgate", "rhizocrypt", "loamspine", "sweetgrass", "squirrel", "petaltongue"]
+++

## Status

- **7,924+ tests** passing (lib + bin + doc + proptest, 0 failures, fully concurrent)
- **90%+ coverage** region / function / line (llvm-cov workspace-wide)
- **v3.64** — production ready, deep debt CLEAN
- **27 capability domains**, **320+ translations** across 13 primals
- **17 atomic signal graphs** across 5 tiers (tower, node, nest, meta, braid)
- **40 deploy graphs** + 2 pipeline coordination graphs
- **20 niche templates** (+ RootPulse, soil-microbiome, ecology)
- **28 workspace crates**, 838 files, 208,327 LOC
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
| v3.64 | May 19 | WS-2: `nest.sync` cross-spring provenance exchange |
| v3.63 | May 19 | R5: all 17 signals as first-class routes; R7 deferred |
| v3.62 | May 18 | R9: stale socket cleanup, PID files, shutdown hygiene |
| v3.61 | May 18 | Clippy zero, resource enforcement, pipeline readiness |
| v3.60 | May 17 | Stadial gate: braid signal tier, wire standard compliance |
| v3.59 | May 16 | Signal dispatch 16 graphs, capability interception |

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog-status-science-and-evolution/) on primals.eco
- [Orchestration Architecture](https://primals.eco/architecture/) — NUCLEUS, Neural API, federation

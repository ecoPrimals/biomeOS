+++
title = "biomeOS Validation Summary"
description = "Orchestration kernel — composition broker, BTSP session propagation, riboCipher transport, NUCLEUS supervisor, 27 capability domains, 320+ translations, 43 deploy graphs, 27 signal graphs, zero blocking debt"
date = 2026-07-29

[taxonomies]
primals = ["biomeos", "beardog", "songbird", "skunkbat", "toadstool", "coralreef", "barracuda", "nestgate", "rhizocrypt", "loamspine", "sweetgrass", "squirrel", "petaltongue"]
+++

## Status

- **8,564+ tests** workspace-wide, 0 failures, fully concurrent
- **88%+ coverage** line / region / function (llvm-cov workspace-wide)
- **v4.47** — Wave 155i: Deep debt cleanup — dead deps, test extraction, capability-based BTSP
- **v4.46** — Wave 155i: NUCLEUS Orchestrator — riboCipher executor, socket unification, persistence
- **v4.45** — Wave 155i: Composition Broker E2E + deep debt audit clean
- **v4.44** — Wave 155i: riboCipher framing + BTSP session propagation in signal graph executor
- **v4.43** — Wave 155d: Live signal graph validation — tower.health + tower.mesh_status
- **v4.42** — Wave 155b: Plasmodium G8 multi-gate bonding + workload dispatch
- **v4.41** — Wave 151c: Deep debt cleanup + 100% clippy --tests
- **v4.40** — Wave 151b: SDK BTSP handshake evolution, strict mode ready
- **v4.39** — Wave 150y: Neural Router pool integration + chimera schema Phase 0
- **v4.38** — Wave 150x: Connection pooling + service crash-loop guard
- **v4.37** — Wave 150t: Deep debt clippy zero + transport assessment
- **v4.36** — Wave 149b: Gap resolution + executor introspection
- **v4.35** — Wave 144b: Phase 2 transport + deep debt + manifest discovery
- **27 capability domains**, **320+ translations** across 13 primals
- **27 atomic signal graphs** across 5 tiers (tower, node, nest, meta, braid)
- **43 deploy graphs** (incl. membrane_deploy, provenance trio, 2 pipeline coordination)
- **20 niche templates** (+ RootPulse, soil-microbiome, ecology)
- **26 workspace crates**
- **Zero blocking debt** — 0 unsafe, 0 C deps, 0 TODO/FIXME, 0 clippy warnings
- **Edition 2024** all crates, ecoBin v3.0 compliant
- **Cross-arch** — x86_64 + aarch64 + armv7 + x86_64-pc-windows-gnu
- **Security A++** — 100/100, Dark Forest Gate, BTSP Phase 3 encrypted framing
- **Composition broker** — Neural API as central trust broker for inter-primal BTSP propagation
- **scyBorg triple-copyleft** — AGPL-3.0-or-later + ORC + CC-BY-SA 4.0

## Architecture

biomeOS is the orchestration kernel — it composes all other primals into
functioning ecosystems. It does not perform compute, storage, or security
itself; it coordinates the primals that do.

- **NUCLEUS** — process supervision, startup ordering, auto-resurrection
- **Neural API** — JSON-RPC routing, capability translation, signal dispatch, composition broker
- **Plasmodium** — multi-machine meld/split/mix, cross-device federation, G8 multi-gate bonding
- **Dark Forest** — zero metadata leakage, encrypted beacons, genetic model
- **RootPulse** — emergent provenance pattern (rhizoCrypt + loamSpine + sweetGrass)

## Signal Dispatch (5 tiers, 27 graphs)

| Tier | Signals | Purpose |
|------|---------|---------|
| tower | publish, authenticate, discover, health, bootstrap, enroll, key_rotate, mesh_status | Security + mesh orchestration |
| node | compute, discover_hardware, dispatch | Compute-level dispatch |
| nest | store, commit, retrieve, sync, verify, federate, ingest_spore, ingest_dataset | Storage + content + cross-spring exchange |
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

- **Composition broker** — Neural API holds BTSP sessions, propagates trust through graph chain
- **riboCipher transport** — `[0xEC, 0x01]` prefix on all new connections, enforced by Neural API
- **Neural API routing** — semantic fallback, signal-tier interception, cross-gate forwarding
- **Capability-based discovery** — 5-tier protocol, taxonomy-driven, zero identity coupling
- **BTSP** — negotiate, escalate, status; cleartext→enforced one-way transition; Phase 3 encrypted framing
- **Deploy graph execution** — atomic types (Tower/Node/Nest/Nucleus), graph signing (BLAKE3+Ed25519)
- **Plasmodium G8** — remote compute discovery, workload dispatch, graph auto-dispatch across gates
- **Connection pool** — UDS connection reuse with buffered IO (BufReader)
- **Composition health** — pipeline readiness (content + compute), adaptive daemon surface
- **Stale socket cleanup** — startup scan + PID files + shutdown hygiene
- **Cross-spring sync** — `nest.sync` signal for provenance exchange via trio pipeline

## Evolution Timeline (recent)

| Version | Date | Highlight |
|---------|------|-----------|
| v4.45 | Jul 29 | Composition Broker E2E + connection pool buffered IO + deep debt audit clean |
| v4.44 | Jul 29 | riboCipher framing + BTSP session propagation in signal graph executor |
| v4.43 | Jul 28 | Live signal graph validation — tower.health + tower.mesh_status (19→27 graphs) |
| v4.42 | Jul 27 | Plasmodium G8 multi-gate bonding + workload dispatch |
| v4.41 | Jul 26 | Deep debt cleanup + 100% clippy --tests, dead dep purge |
| v4.40 | Jul 26 | SDK BTSP handshake evolution, strict mode ready |
| v4.39 | Jul 24 | Neural Router pool integration + chimera schema Phase 0 |
| v4.38 | Jul 24 | Connection pooling + service crash-loop guard |
| v4.37 | Jul 21 | Deep debt clippy zero + transport assessment |
| v4.36 | Jul 18 | Gap resolution + executor introspection + socket naming |
| v4.35 | Jul 16 | Phase 2 transport + deep debt + manifest discovery |

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog-status-science-and-evolution/) on primals.eco
- [Orchestration Architecture](https://primals.eco/architecture/) — NUCLEUS, Neural API, federation

+++
title = "biomeOS Validation Summary"
description = "Orchestration kernel v4.57 — G22 complete, cell attach CLI, 30 signal graphs, 27 domains, 320+ translations, zero blocking debt, ironGate Phase 1 OPERATIONALLY READY"
date = 2026-08-04

[taxonomies]
primals = ["biomeos", "beardog", "songbird", "skunkbat", "toadstool", "coralreef", "barracuda", "nestgate", "rhizocrypt", "loamspine", "sweetgrass", "squirrel", "petaltongue"]
+++

## Status

- **8,578+ tests** workspace-wide, 0 failures, fully concurrent
- **88%+ coverage** line / region / function (llvm-cov workspace-wide)
- **v4.57** — Wave 156d: Cell attach CLI (`biomeos nucleus attach`), ironGate ops gap CLOSED
- **v4.56** — Wave 155n/156b: G22 COMPLETE, cell deploy graphs, data federation signals
- **v4.55** — Wave 155n: Coevolution contract (composition.test_swap), mode gap fix
- **v4.54** — Wave 155m: P1 fixes (respawn storm, socket deletion), P3 sweep
- **v4.49** — Wave 155k: P2 divergence fixes — capability wipe cycle, test extraction, dep narrowing
- **v4.48** — Wave 155j: Composition lifecycle — cellMembrane boot_order integration
- **v4.47** — Wave 155i: Deep debt cleanup — dead deps, test extraction, capability-based BTSP
- **v4.46** — Wave 155i: NUCLEUS Orchestrator — riboCipher executor, socket unification, persistence
- **v4.45** — Wave 155i: Composition Broker E2E + deep debt audit clean
- **v4.44** — Wave 155i: riboCipher framing + BTSP session propagation in signal graph executor
- **27 capability domains**, **320+ translations** across 13 primals
- **30 atomic signal graphs** across 5 tiers (tower, node, nest, meta, braid)
- **45 deploy graphs** (incl. membrane_deploy, provenance trio, 2 cell attachment)
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

## Signal Dispatch (5 tiers, 30 graphs)

| Tier | Signals | Purpose |
|------|---------|---------|
| tower | publish, authenticate, discover, health, bootstrap, enroll, key_rotate, mesh_status | Security + mesh orchestration |
| node | compute, discover_hardware, dispatch | Compute-level dispatch |
| nest | store, commit, retrieve, sync, verify, federate, ingest_spore, ingest_dataset, declare_dataset, acquire_file, complete_dataset | Storage + content + data federation |
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
| v4.57 | Aug 4 | Cell attach CLI (`biomeos nucleus attach`), ironGate Phase 1 operationally ready |
| v4.56 | Aug 3 | G22 COMPLETE, cell deploy graphs, data federation signals, 47 dead deps removed |
| v4.55 | Jul 31 | Coevolution contract (composition.test_swap), btsp_optional mode gap |
| v4.54 | Jul 31 | P1 fixes: respawn storm, socket deletion, zombie reaping |
| v4.49 | Jul 30 | P2 fixes: capability wipe cycle 3-strike, 8-file test extraction, dep narrowing |
| v4.48 | Jul 30 | Composition lifecycle: cellMembrane boot_order, shutdown ordering |
| v4.47 | Jul 29 | Deep debt: dead deps, test extraction, capability-based BTSP |
| v4.46 | Jul 29 | NUCLEUS Orchestrator: riboCipher executor, socket unification, persistence |
| v4.45 | Jul 29 | Composition Broker E2E + connection pool buffered IO |
| v4.44 | Jul 29 | riboCipher framing + BTSP session propagation in signal graph executor |
| v4.43 | Jul 28 | Live signal graph validation — tower.health + tower.mesh_status |
| v4.42 | Jul 27 | Plasmodium G8 multi-gate bonding + workload dispatch |

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog-status-science-and-evolution/) on primals.eco
- [Orchestration Architecture](https://primals.eco/architecture/) — NUCLEUS, Neural API, federation

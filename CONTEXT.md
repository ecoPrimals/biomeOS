# Context — biomeOS

## What This Is

biomeOS is a pure Rust orchestration layer that deploys, discovers, and routes
between autonomous binaries ("primals") in the ecoPrimals sovereign computing
ecosystem. It provides capability-based routing via a Neural API, graph-driven
deployment, and multi-transport IPC — all with zero compile-time coupling
between components.

## Role in the Ecosystem

biomeOS is the deployment substrate and routing fabric. It starts primals,
discovers their capabilities at runtime via JSON-RPC probing, and forwards
requests to the right primal based on capability (not name). Other primals
never import biomeOS code — they communicate exclusively over IPC. biomeOS
manages NUCLEUS compositions (Tower, Node, Nest, Full) and federation across
multiple gates (devices).

## Technical Facts

- **Language:** 100% Rust, zero C dependencies
- **Architecture:** Single binary (UniBin) with multiple operational modes (bootstrap, nucleus, deploy, doctor, continuous, rootpulse)
- **Communication:** JSON-RPC 2.0 over Unix sockets, abstract sockets, TCP, and HTTP — with tarpc binary protocol escalation for hot paths
- **License:** AGPL-3.0-only (scyBorg triple-copyleft: AGPL-3.0 + ORC + CC-BY-SA 4.0)
- **Tests:** 7,204 passing, 0 failures
- **Coverage:** 90%+ line coverage (llvm-cov verified)
- **Blocking debt:** 0 (graph rollback, DNS discovery, remote acquisition, federation manifest — all resolved)
- **Edition:** Rust 2024 across all workspace crates
- **Crate count:** 26 workspace crates
- **Clippy:** 0 warnings (pedantic + nursery lints)
- **Unsafe:** 0 in production code

## Key Capabilities (JSON-RPC methods)

| Domain | Methods |
|--------|---------|
| **Capability routing** | `capability.call`, `capability.register`, `capability.list`, `capability.route` |
| **Discovery** | `discovery.discover`, `discovery.discover_all`, `discovery.protocols` |
| **Graph deployment** | `graph.deploy`, `graph.status`, `graph.pipeline`, `graph.continuous` |
| **Health** | `health.check`, `health.metrics`, `health.version` |
| **Topology** | `topology.get`, `topology.proprioception` |
| **Lifecycle** | `lifecycle.start`, `lifecycle.stop`, `lifecycle.status` |
| **Nucleus** | `nucleus start --mode tower|node|nest|full` |

## What This Does NOT Do

- Does not provide cryptography (that is BearDog)
- Does not perform network discovery or TLS (that is Songbird)
- Does not manage storage (that is NestGate / Squirrel)
- Does not run GPU compute (that is ToadStool / coralReef)
- Does not serve a web UI (that is petalTongue)
- Does not contain any primal-specific business logic — it is the substrate

## Related Repositories

- **ecoPrimals/wateringHole** — Inter-primal standards, handoffs, and guidance
- **ecoPrimals/primals/** — Individual primal repositories (BearDog, Songbird, etc.)
- **ecoPrimals/infra/** — Infrastructure, deployment scripts, gate configurations

## Architecture Overview

```
User / AI ──► Neural API (JSON-RPC) ──► Capability Router ──► Primal (via TransportEndpoint)
                                              │
                                    ┌─────────┼─────────┐
                                    ▼         ▼         ▼
                               Unix sock  Abstract   TCP/HTTP
                              (Tier 1)    (Tier 1)   (Tier 2)
```

Discovery is 5-tier: centralized registry → taxonomy bootstrap → environment
hints → socket scanning → fallback mapping. No primal name is ever hardcoded
in routing logic.

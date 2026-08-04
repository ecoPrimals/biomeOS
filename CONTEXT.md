# Context — biomeOS

**Version**: v4.57 | **Updated**: August 4, 2026

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
- **Communication:** JSON-RPC 2.0 over `biomeos-core::ipc` transport (`TransportStream`, `TransportListener`, `connect_transport`) — Unix sockets primary, abstract sockets, TCP fallback (Windows via `.port` file); tarpc binary protocol escalation for hot paths (HTTP transport endpoint removed v3.97; inter-gate HTTP via Songbird gateway)
- **License:** AGPL-3.0-or-later (scyBorg triple-copyleft: AGPL-3.0-or-later + ORC + CC-BY-SA 4.0)
- **Tests:** 8,578+ workspace-wide (0 failures)
- **Coverage:** 88%+ line coverage (llvm-cov verified)
- **Blocking debt:** 0 (primalSpring Phase 43 gaps resolved, all composition gaps resolved)
- **Edition:** Rust 2024 across all workspace crates
- **Crate count:** 26 workspace crates
- **File size:** 0 production files >800 LOC; 0 test files >450 LOC
- **Cross-arch:** x86_64 + aarch64 + armv7 + x86_64-pc-windows-gnu
- **Phase 2 transport:** complete (12/14 primals on unified `biomeos-core::ipc` trait dispatch)
- **Clippy:** pedantic + nursery enabled via workspace lint inheritance, zero warnings (`-D warnings`)
- **Production unwraps:** 0 (workspace `unwrap_used = "deny"`)
- **Unsafe:** 0 in production code (`#[forbid(unsafe_code)]` on all crate roots + all 20+ binary entry points)
- **TODO/FIXME/HACK:** 0 (all resolved)

## Key Capabilities (JSON-RPC methods)

| Domain | Methods |
|--------|---------|
| **Capability routing** | `capability.call`, `capability.register`, `capability.list`, `capability.route` |
| **Discovery** | `discovery.discover`, `discovery.discover_all`, `discovery.protocols` |
| **Graph deployment** | `graph.deploy`, `graph.status`, `graph.pipeline`, `graph.continuous` |
| **Health** | `health.check`, `health.liveness`, `health.readiness` |
| **Topology** | `topology.get`, `topology.proprioception`, `topology.rescan` |
| **Lifecycle** | `lifecycle.start`, `lifecycle.stop`, `lifecycle.status` |
| **Manifest** | `manifest.gate_profile` |
| **Nucleus** | `nucleus start --mode tower|node|nest|full|core` — launch sets via `NucleusMode::resolve_launch_set()` from `ecosystem_manifest.toml` |

## What This Does NOT Do

- Does not provide cryptography (that is BearDog)
- Does not perform network discovery or TLS (that is Songbird)
- Does not manage storage (that is NestGate)
- Does not run GPU compute (that is ToadStool / coralReef)
- Does not serve a web UI (that is petalTongue)
- Does not contain any primal-specific business logic — it is the substrate

## Related Repositories

- **ecoPrimals/infra/wateringHole** — Inter-primal standards, handoffs, and guidance
- **ecoPrimals/primals/** — Individual primal repositories (BearDog, Songbird, etc.)
- **ecoPrimals/infra/** — Infrastructure, deployment scripts, gate configurations

## Architecture Overview

```
User / AI ──► Neural API (JSON-RPC) ──► Capability Router ──► Primal (via TransportEndpoint)
                                              │
                              biomeos-core::ipc (TransportStream / TransportListener)
                                              │
                                    ┌─────────┼─────────┐
                                    ▼         ▼         ▼
                               Unix sock  Abstract     TCP
                              (primary)   (Tier 1)   (fallback / Windows)
```

Discovery is capability-first: runtime registry (live `capability.register`) →
bootstrap hints → taxonomy → environment hints → socket scanning. No primal
name is ever hardcoded in routing logic. `NucleusMode` launch sets resolve
from `ecosystem_manifest.toml` composition profiles via `resolve_launch_set()`.

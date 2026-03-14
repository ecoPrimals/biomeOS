# biomeOS V2.35 — Current Handoff — March 14, 2026

**From:** biomeOS v2.35
**To:** All springs, petalTongue, barraCuda, toadStool, provenance trio
**License:** AGPL-3.0-only
**Supersedes:** V2.28, V2.30, V2.32, V2.33, V2.34

---

## Executive Summary

biomeOS v2.35 completes the deep debt evolution started in v2.34:

- **4,275 tests** (0 failures, 167 ignored)
- **75.21% region coverage** (llvm-cov)
- **205+ capability translations** across 16 domains
- **30 deploy graphs**, 15 niche templates
- **0 clippy warnings**, 0 unsafe code, 0 files >1000 LOC
- **0 hardcoded primal strings** — all via `primal_names` constants
- **Zero-copy binary payloads** — `bytes::Bytes` with base64 serde
- **tarpc transport helpers** — Unix socket naming, protocol escalation ready

---

## What Changed Since V2.34

1. **Zero-copy**: `SignatureResult.signature` → `Option<bytes::Bytes>`, base64 serde helpers
2. **Primal constants**: `biomeos-types::primal_names` module eliminates all hardcoded primal strings
3. **tarpc wiring**: `unix` feature enabled, `tarpc_transport.rs` helpers in primal-sdk
4. **+183 tests**: capability taxonomy, subfederation, beacon, service types
5. **Test extraction**: 6 files split to stay under 1000 LOC

---

## Canonical Handoff

Full handoff at: `ecoPrimals/wateringHole/handoffs/BIOMEOS_V235_ZEROCOPY_CONSTANTS_TARPC_COVERAGE_HANDOFF_MAR14_2026.md`

---

## For Primal Teams

### tarpc Protocol Escalation

biomeOS now provides Unix socket helpers for tarpc:
- `tarpc_socket_name()`: converts `primal.sock` → `primal.tarpc.sock`
- `prepare_socket()`: cleans stale sockets, creates parent directories
- `tarpc_socket_path()`: full path conversion

Primals that implement tarpc will be automatically discovered when biomeOS finds a `.tarpc.sock` file alongside the standard `.sock` file.

### Primal Name Constants

All primals are now referenced by constants from `biomeos-types::primal_names`. If your primal references biomeOS capabilities, you should use these constants rather than string literals.

### Continuous Coordination

The `CONTINUOUS_COORDINATION_REQUIREMENTS_MAR_2026.md` handoff requirements are met:
- `ContinuousExecutor` with 60Hz tick loop
- `GraphEventBroadcaster` for push events
- `SensorEventBus` for input routing
- Feedback edges for closed-loop graphs

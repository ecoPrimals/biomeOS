# biomeOS V2.36 — Current Handoff — March 14, 2026

**From:** biomeOS v2.36
**To:** All springs, petalTongue, barraCuda, toadStool, provenance trio
**License:** AGPL-3.0-only
**Supersedes:** V2.28, V2.30, V2.32, V2.33, V2.34, V2.35

---

## Executive Summary

biomeOS v2.36 completes the deep debt audit evolution pass:

- **4,383 tests** (0 failures, 204 ignored)
- **76.06% region coverage** (llvm-cov)
- **205+ capability translations** across 16 domains
- **30 deploy graphs**, 15 niche templates
- **0 clippy warnings**, 0 unsafe code, 0 files >1000 LOC
- **0 hardcoded primal strings** — all via `primal_names` constants
- **Zero-copy binary payloads** — `bytes::Bytes` with base64 serde
- **JSON-RPC builders** — `JsonRpcRequest::new()`, `JsonRpcResponse::success()`
- **SystemPaths** — no hardcoded `/tmp/` socket paths
- **cargo-deny 0.19** — config valid

---

## What Changed Since V2.35

1. **JSON-RPC**: `JsonRpcRequest::new()` / `::notification()`, `JsonRpcResponse::success()` / `::error()` — replaces 30+ manual `json!({...})` constructions
2. **Zero-copy**: Extended to `SecurityRpc`, p2p types, `Workload.code`, `CompressedBinary.data`, `fetch_binary()`
3. **Safe casts**: 15 `as` truncation casts evolved to `try_from()`, `u64::from()`, arithmetic
4. **SystemPaths**: `rootpulse`, `neural_api`, `continuous`, `enroll` — all use `SystemPaths::new_lazy()`
5. **File compliance**: `node_handlers.rs` 1015→461 lines + `node_handlers_tests.rs`
6. **deny.toml**: Evolved for cargo-deny 0.19 (removed deprecated keys)
7. **Test safety**: 4 env-var race tests marked `#[ignore]`
8. **Dead code**: 8 `#[allow(dead_code)]` sites resolved

---

## Canonical Handoff

Full handoff at: `ecoPrimals/wateringHole/handoffs/BIOMEOS_V236_DEEP_DEBT_AUDIT_EVOLUTION_HANDOFF_MAR14_2026.md`

---

## For Primal Teams

### JSON-RPC Protocol

Use `JsonRpcRequest::new(method, params)` instead of manual `json!({...})` construction. For fire-and-forget: `JsonRpcRequest::notification(method, params)`.

### Primal Name Constants

All primals are referenced by constants from `biomeos-types::primal_names`. Use these constants rather than string literals.

### Socket Paths

Use `SystemPaths::new_lazy()` — never hardcode `/tmp/` or `/run/` paths.

### tarpc Protocol Escalation

biomeOS provides Unix socket helpers for tarpc:
- `tarpc_socket_name()`: converts `primal.sock` → `primal.tarpc.sock`
- `prepare_socket()`: cleans stale sockets, creates parent directories
- `tarpc_socket_path()`: full path conversion

Scaffolding exists; JSON-RPC remains primary until server/client implementations land.

### Continuous Coordination

The `CONTINUOUS_COORDINATION_REQUIREMENTS_MAR_2026.md` handoff requirements are met:
- `ContinuousExecutor` with 60Hz tick loop
- `GraphEventBroadcaster` for push events
- `SensorEventBus` for input routing
- Feedback edges for closed-loop graphs

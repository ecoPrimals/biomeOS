# biomeOS Session 155i — NUCLEUS Orchestrator

**Date**: July 29, 2026
**Gate**: eastGate
**Version**: v4.46
**Wave**: 155i (second pass — NUCLEUS orchestration evolution)

## Summary

Evolved biomeOS from "composition broker" (capability routing) to "NUCLEUS orchestrator"
(startup ordering, health gating, composition transitions, unified socket paths).

## Changes

### 1. Graph Executor RiboCipher Fix (P1)

The graph executor's `send_jsonrpc_async` fallback path was using `send_jsonrpc_request()`
(no riboCipher prefix). Fixed to `send_ribocipher_jsonrpc_request()`. This unblocks
orchestrated graph execution across compositions on westGate where the Neural API
enforces riboCipher detection.

`MockJsonRpcServer` in `biomeos-test-utils` evolved to consume the 2-byte `[0xEC, 0x01]`
prefix on new connections, keeping all 35 rollback tests and all downstream mock users
working.

### 2. Socket Path Unification (P2)

**Root cause**: Socket discovery (`discover_via_xdg`, `try_unix_socket_xdg`,
`discover_capability_socket`, `discover_via_socket_registry`) used
`primal_names::BIOMEOS` ("biomeos") for directory lookups, while `SystemPaths` and
`capability_discovery` used `runtime_paths::MEMBRANE_SUBDIR` ("membrane").

**Fix**: All production socket resolution paths now use `MEMBRANE_SUBDIR`. Legacy
`biomeos/` directory is still scanned by `topology.rs` for backwards compat on existing
gates with old socket layouts.

Affected:
- `crates/biomeos-core/src/socket_discovery/engine_probes.rs` (4 paths)
- `crates/neural-api-client-sync/src/lib.rs` (tier 3 fallback)
- `crates/biomeos-nucleus/src/client/family_seed.rs` (UID fallback)
- All 22 graph `.toml` files in `graphs/`
- 15+ doc comment references across 12 crates

### 3. Socket Evaporation Fix (P2)

**Root cause**: Capability registry was purely in-memory. Neural API restart wiped all
dynamically registered capabilities. Background sweep (30s) eventually recovered them,
but the cold-start window caused routing failures.

**Fix**: JSON persistence layer for capabilities:
- `persist_capability_registry()` writes snapshot to `$SOCKET_DIR/capability-registry.json`
- `load_persisted_capability_registry()` loads warm cache on startup (step 4c, before probing)
- Background discovery sweep persists after each cycle

### 4. Composition Lifecycle Management (P1)

New `composition.start` RPC endpoint:
- Takes `{"composition": "tower" | "nest" | "node" | "nucleus"}`
- Checks health prerequisites (e.g., tower must be "ok" before nest can start)
- Returns graph name + readiness status (or `blocked_by` list)
- Maps: tower → `tower_atomic_bootstrap`, nest → `nest_deploy`, node → `node_atomic_compute`, nucleus → `nucleus_complete`

This enables signal-graph-driven orchestration (replacing `nucleus_launcher.sh`).

### 5. Primal Bind Flag Standardization

Proposal at `specs/PRIMAL_BIND_FLAGS_STANDARD.md`:
- Standard flags: `--bind-mode`, `--port`, `--family-id`
- Standard env vars: `BIND_ADDRESS`, `PRIMAL_BIND_MODE`, `FAMILY_ID`
- Socket convention: `$XDG_RUNTIME_DIR/membrane/{primal}-{family_id}.sock`

## Metrics

| Metric | Value |
|--------|-------|
| Tests | 8,564 passed, 0 failed |
| Clippy | 0 warnings (workspace + tests, `-D warnings`) |
| Files changed | ~40 |
| Lines added | ~200 |
| Lines removed | ~60 |

## Next-Wave Candidates (biomeOS)

1. Evolve `nucleus local.rs` to use graph executor instead of manual startup loop
2. `composition.stop` — graceful reverse-order shutdown via graph rollback
3. Composition transition state machine (persisted state across restarts)
4. Health-gated auto-scaling (start Nest only when Tower reports sufficient capacity)

## Gaps Found for Upstream Teams

| Team | Gap |
|------|-----|
| All primals | Adopt `PRIMAL_BIND_FLAGS_STANDARD.md` — biomeOS cannot uniformly start primals without consistent flags |
| cellMembrane | `RuntimeDirectory=membrane` already correct in systemd units — verify no `biomeos/` symlinks remain on deployed gates |
| westGate ops | Remove `/run/user/1000/biomeos` symlink if present (legacy compat scanner handles it) |

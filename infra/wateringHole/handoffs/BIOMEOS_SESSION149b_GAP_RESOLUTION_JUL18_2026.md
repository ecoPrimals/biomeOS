# biomeOS Session 149b — Wave 149b Gap Resolution

**Date**: July 18, 2026  
**Version**: v4.36  
**Wave**: 149b  
**Scope**: Ecosystem demand signal gap resolution (GAP-017, 018, 036, 038)

---

## Summary

Resolved all 4 biomeOS gaps from the Wave 149b dimensional review demand signal.

---

## Gap Resolution

### GAP-017: Neural-API Resurrection (P2) — VERIFIED COMPLETE

The resurrection chain was already fully implemented:
- `LifecycleManager::start_monitoring()` → interval health checks
- `check_primal_health()` → failure threshold → `LifecycleState::Degraded`
- Auto-triggers `attempt_resurrection()` with exponential backoff
- Kill old process (SIGTERM → SIGKILL) → remove stale socket → respawn
- `lifecycle.resurrect` JSON-RPC for manual trigger also wired

**No code changes needed** — chain was complete but status was "Open" because esotericWebb hadn't tested against it.

### GAP-018: Executors Not Exposed (P2) — SHIPPED

New handler: `crates/biomeos-atomic-deploy/src/handlers/executor.rs`

| Method | Returns |
|--------|---------|
| `executor.list` | All executor types with name, type, active_sessions, status |
| `executor.status` | Detailed metrics per executor (active/pending/completed/failed/uptime_s) |

Executor types: `continuous` (graph sessions), `pipeline` (multi-step), `single-shot` (one-off graph execution).

### GAP-036: Socket Naming Convention (P2) — SHIPPED

New module: `crates/biomeos-types/src/constants/socket_naming.rs`

```rust
pub fn primal_socket_path(socket_dir: &Path, primal_name: &str, family_id: &str) -> PathBuf
pub fn parse_socket_filename(filename: &str) -> Option<(&str, &str)>
```

Convention: `{socket_dir}/{primal_name}-{family_id}.sock`  
Handles hyphenated primal names (splits on last `-`).

### GAP-038: Stale UDS Socket Cleanup (P2) — SHIPPED

New handler: `crates/biomeos-atomic-deploy/src/handlers/cleanup.rs`

| Method | Behavior |
|--------|----------|
| `cleanup.sockets` | Scan socket dirs → probe each `.sock` file with `connect_transport_timed` → remove stale → return `{removed: [...], active: [...]}` |

Optional `socket_dir` param; defaults to discovered socket directories.  
Startup cleanup in `nucleus_procs.rs` unchanged (still runs at boot).

---

## Metrics

| Metric | Value |
|--------|-------|
| Tests passing | **8,492** (0 failures) |
| New JSON-RPC methods | 3 (`executor.list`, `executor.status`, `cleanup.sockets`) |
| New modules | 3 (executor handler, cleanup handler, socket_naming) |
| Production files >800L | 0 |
| Test files >450L | 0 |

---

## Build Status

```
cargo check                                    ✅
cargo check --target x86_64-pc-windows-gnu     ✅
cargo test --workspace                         ✅ (8,492 passed, 0 failed)
```

---

## Remaining biomeOS Demand (from Wave 149b)

All biomeOS-specific gaps are now resolved. Remaining ecosystem items that depend on OTHER primals:

| Gap | Depends on | Status |
|-----|-----------|--------|
| squirrel: accept `null` params on health | squirrel team | Open |
| nestGate: `PROJECTS_PATH` CAS wiring | nestGate team | Open |
| petalTongue: `WS_PATH` agent bridge | petalTongue team | Open |
| bearDog: crypto JSON-RPC sigs | bearDog team | Open |
| sweetGrass: `braid.create/query` | sweetGrass team | Open |

# biomeOS Session 155m-e — P3 Fixes: Permission Reset + Sandbox Self-Test

**Date**: July 31, 2026
**Version**: v4.53
**Focus**: Close 2 biomeOS-owned P3s from sporeGate AAR

---

## P3 #1: /run/membrane Permission Reset — CLOSED

### Problem
`apply_dir_group_ownership()` in `nucleation.rs` was called unconditionally on every
socket assignment (both `assign_socket` and `xdg_runtime_path_with`). When the
`/run/membrane/` directory already existed with correct ownership, biomeOS would
reset it to `0770 :membrane` — but the `chown` operation would change the group
from whatever the system/CI user had set it to. On sporeGate, the sporegate CI
user lost access after each biomeOS restart.

### Fix
Guard `apply_dir_group_ownership()` behind `!dir.exists()` — only apply ownership
when the directory is freshly created. Pre-existing directories retain their perms.

```rust
let freshly_created = !parent.exists();
if let Err(e) = std::fs::create_dir_all(parent) { ... }
#[cfg(unix)]
if freshly_created {
    apply_dir_group_ownership(parent);
}
```

---

## P3 #2: Sandbox False Positive — CLOSED

### Problem
cellMembrane's sandbox validates primals by calling a JSON-RPC method. biomeOS (as
orchestrator) requires a full composition to be running to serve most capabilities.
Sandbox could not distinguish "orchestrator needs composition" from "binary broken,"
resulting in false validation failures.

### Fix
Added `composition.self_test` RPC method — a lightweight endpoint that proves the
Neural API is functional without needing any primals running:

```json
{
  "ok": true,
  "role": "orchestrator",
  "version": "0.1.0",
  "routes_loaded": true,
  "capability_registry": "available",
  "primals_registered": 0,
  "ipc": "json-rpc"
}
```

cellMembrane sandbox calls `composition.self_test` → gets `ok: true` → validation passes.

---

## P3 Triage (Not biomeOS Code)

| P3 | Owner | Reason |
|----|-------|--------|
| `GATE_NAME` vs `MEMBRANE_GATE_NAME` | cellMembrane | biomeOS doesn't reference either env var |
| GNU depot incomplete (4/16) | sporeGate | Builder config, not biomeOS code |
| cellMembrane not in sources.toml | cellMembrane | Self-rebuild config issue |

---

## Verification

| Check | Result |
|-------|--------|
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --tests -- -D warnings` | 0 warnings |
| `cargo test --workspace` | 8,570 pass, 0 failures |
| `cargo fmt --check` | PASS |
| `cargo deny check` | clean |
| Unsafe blocks | 0 |
| TODOs in prod | 0 |

---

## Posture

**STANDBY-READY**. Zero biomeOS-owned P0/P1/P2/P3. All deep debt criteria verified clean.
Next: G18 (neuralAPI agent orchestration) when squirrel wires in.

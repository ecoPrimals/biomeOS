# biomeOS Session 155k — P2 Divergence Fixes

**Date**: July 30, 2026
**Version**: v4.49
**Gate**: eastGate
**Wave**: 155k

---

## Summary

Resolved all 3 biomeOS-owned P2 divergences cataloged by overwatch during
westGate/blueGate NUCLEUS deployment.

---

## P2-1: Capability Wipe Cycle (654→0→187→654 over ~60s)

**Root Cause**: `prune_stale_registrations()` (60s interval) and
`discover_and_register_primals()` (30s interval) race when primals are busy
serving both probes simultaneously. Socket contention causes health check
timeouts, which `prune_stale_registrations` interprets as dead — unregistering
ALL capabilities for every primal whose single probe fails.

**Fix**: 3-strike consecutive failure threshold before pruning.

- Added `prune_strikes: Arc<RwLock<HashMap<Arc<str>, u8>>>` to `NeuralRouter`
- `prune_stale_registrations()` now:
  - Tracks healthy vs failed primals per sweep
  - Resets strike counter to 0 on successful health check
  - Increments strike counter on failure
  - Only calls `unregister_primal()` when counter >= 3
  - Clears strike entry after successful prune
- Single transient failure no longer cascades to full registry wipe
- Worst case: primal truly dead takes 3×60s = 180s to prune (acceptable for production stability)

**File**: `crates/biomeos-atomic-deploy/src/neural_router/registry.rs` (prune logic)
**File**: `crates/biomeos-atomic-deploy/src/neural_router/mod.rs` (struct field + constructor)

---

## P2-2: Neural API Socket "Hardcoded to membrane/" (Stale Documentation)

**Root Cause**: Doc comments in `neural-api-client-sync/src/lib.rs` still referenced
legacy `biomeos/` path in discovery tier descriptions. Code itself was correct since
v4.46 (uses `MEMBRANE_SUBDIR` constant everywhere).

**Fix**: Updated doc comments from `/run/user/{uid}/biomeos/` and `{temp_dir}/biomeos/`
to `/run/user/{uid}/membrane/` and `{temp_dir}/membrane/`.

**File**: `crates/neural-api-client-sync/src/lib.rs` (doc comments only)

---

## P2-3: API 403 on Non-/health Endpoints (Dark Forest Gate)

**Root Cause**: This is **intentional behavior**, not a bug. The HTTP REST API surface
(`biomeos-api` crate) runs the Dark Forest gate in sovereign mode. All non-health
HTTP requests require an `X-Dark-Forest-Token` header.

**Clarification**:
- HTTP REST API = public-facing, locked down by sovereign mode
- Neural API UDS = inter-primal communication, `capabilities.list` is Public
- blueGate (Windows/TCP) clients should use the JSON-RPC TCP path with riboCipher+BTSP

**Resolution**: Documentation-only. No code change needed — this is the security posture.

**Escape hatches** (dev only):
- `BIOMEOS_SOVEREIGN=false` — disables Dark Forest gate entirely
- `X-Dark-Forest-Token: <token>` header — authenticates HTTP requests

---

## Verification

| Metric | Value |
|--------|-------|
| Tests | 8,570+ pass, 0 failures |
| Clippy | 0 warnings (pedantic+nursery, --tests) |
| Regressions | 0 |

---

## Upstream Signal

biomeOS P2 divergences from Wave 155k are resolved:
- ✅ Capability wipe cycle — strike threshold prevents false-positive mass prune
- ✅ Socket hardcoding — was stale docs, not code
- ✅ 403 on non-health — intentional Dark Forest sovereign behavior

biomeOS remains in **STANDBY** posture. No P0 or P1 blockers.

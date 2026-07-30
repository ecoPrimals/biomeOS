# biomeOS Session 155m — Socket Evaporation + Binary Path Retention

**Date**: Jul 30, 2026
**Version**: v4.50
**Wave**: 155m
**Gate**: eastGate

---

## P2 Divergences Resolved

### 1. Socket Evaporation — RPC Ping Tolerance

**Root cause**: `check_endpoint_health` (used by `prune_stale_registrations` every 60s)
parsed the JSON-RPC response body and required `"healthy": true`. Primals that responded
with alternative formats (`{"status":"alive"}`, `{"version":"1.0"}`, etc.) were falsely
marked dead. Combined with the 3-strike threshold (v4.49), this meant capabilities
evaporated after ~180s (3× 60s sweeps).

**Fix**: Any successful `call_btsp()` response = alive. `call_btsp` already rejects:
- Connection failures → `IpcError::ConnectionFailed`
- Timeouts → `IpcError::Timeout`
- JSON-RPC error responses → `IpcError::JsonRpcError`

So `Ok(_)` from `call_btsp` is definitive proof of liveness.

**Files changed**:
- `crates/biomeos-atomic-deploy/src/neural_router/discovery_primal.rs` — `check_endpoint_health` + `quick_health_check`
- `crates/biomeos-atomic-deploy/src/capability_handlers/health.rs` — `call_primal_health`
- `crates/biomeos-atomic-deploy/src/executor/node_handlers.rs` — tolerant fallback (respects explicit `false`)

### 2. Binary Path Retention — Auto-Discovery Resurrection

**Root cause**: `discovery_init.rs` registered auto-discovered primals via
`lifecycle.register` with `binary_path: None`. When those primals crashed,
`attempt_resurrection` found neither `deployment_node` nor `binary_path` and
logged "cannot resurrect".

**Fix**: Auto-discovery now probes plasmidBin directories (`probe_binary()`) during
registration. If found, the binary path is passed to `lifecycle.register` which
uses `register_primal_binary()` — enabling `respawn_primal_binary()` on crash.

**Files changed**:
- `crates/biomeos-atomic-deploy/src/neural_api_server/discovery_init.rs` — probe binary during UDS + TCP discovery
- `crates/biomeos-atomic-deploy/src/handlers/lifecycle/registration.rs` — accept `binary_path` parameter

---

## Validation

- 1,577 tests pass (`biomeos-atomic-deploy`)
- 0 clippy warnings (workspace-wide, pedantic, -D warnings)
- Existing `test_health_check_reports_unhealthy` passes (explicit `"healthy": false` still respected)

---

## Upstream Impact

- strandGate: socket evaporation was causing capability churn on 60s intervals. v4.50 resolves.
- All gates: auto-discovered primals can now be resurrected after crash without NUCLEUS direct-launch.
- No API changes — purely internal behavior fix.

---

## Status

**biomeOS**: v4.50 STANDBY. ZERO P0/P1/P2 divergences. Ready for redeploy.

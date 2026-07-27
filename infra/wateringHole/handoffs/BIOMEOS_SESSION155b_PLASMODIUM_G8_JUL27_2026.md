# biomeOS Session Handoff — Wave 155b: Plasmodium G8 Multi-Gate Bonding

**Date**: July 27, 2026
**Version**: v4.42
**From**: eastGate overwatch
**Wave**: 155b

---

## Summary

Implemented the Plasmodium G8 multi-gate bonding infrastructure for biomeOS.
This enables compute-aware workload routing across the gate collective and
cross-platform graph execution via Plasmodium dispatch.

---

## Changes

### 1. Remote Compute Discovery (`system.compute` + `system.load` RPC)

- `crates/biomeos-core/src/plasmodium/remote.rs`: Remote gate queries now
  call `system.compute` and `system.load` on each peer, populating `GateInfo`
  with actual GPU/RAM/CPU/load data instead of defaults.
- `crates/biomeos-atomic-deploy/src/neural_api_server/routing.rs`: Added
  `handle_system_compute()` and `handle_system_load()` dispatch handlers.
- `crates/biomeos-atomic-deploy/src/neural_api_server/route_table.rs`: Added
  `SystemCompute` and `SystemLoad` route variants and method mappings.

### 2. Workload Dispatch Engine

- `crates/biomeos-core/src/plasmodium/dispatch.rs` (NEW): Score-based gate
  selection for compute workloads. `select_gates()` evaluates all reachable
  gates against `WorkloadRequirements` (VRAM, RAM, CPU cores, capability,
  max load) and returns ranked candidates.
- 8 unit tests covering VRAM preference, locality, overload exclusion, RAM
  filtering, unreachable exclusion, and load-based ranking.

### 3. Graph Executor Auto-Dispatch

- `crates/biomeos-atomic-deploy/src/neural_graph/types.rs`: Added
  `ComputeRequirements` struct and `compute_requirements` field to `GraphNode`.
- `crates/biomeos-atomic-deploy/src/neural_executor/dispatch.rs`: `gate = "auto"`
  triggers Plasmodium collective query → workload dispatch → remote forwarding.
  Falls back to local execution when no remote gate meets requirements.
- Integration test validates graceful fallback path.

### 4. Module Visibility

- `biomeos_core::plasmodium::system` promoted to `pub` (needed by neural API).
- `biomeos_core::plasmodium::dispatch` added as `pub` module.
- `ComputeRequirements` re-exported from `neural_graph` module.

---

### 5. Deep Debt: Test Monolith Elimination

- `provenance_trio_e2e.rs` (651L) → `provenance_trio_e2e/{main,graph_validation,live_workflow}.rs`
- `nucleus_composition_e2e.rs` (591L) → `nucleus_composition_e2e/{main,parsing,topology,execution,cross_gate}.rs`
- `discovery_integration.rs` (558L) → `discovery_integration/{main,primal_discovery,live_service,resilience}.rs`
- All 39 tests (9+15+15) continue passing from split modules.

### 6. Deep Debt: Dead Monolith + Dep Purge

- Deleted `sovereign_mesh_e2e.rs` (859L) — redundant monolith superseded by
  split phase files.
- Purged 10 unused deps from root crate `[dependencies]`.

---

## Test Results

- **Total**: 8,522 workspace-wide (all pass, 0 failures)
- **New tests**: 8 dispatch unit tests + 1 auto-gate integration test
- **Size compliance**: Zero test files > 450 LOC. Zero prod files > 800 LOC.
- **Clippy**: `cargo clippy --workspace --tests -- -D warnings` = ZERO errors

---

## Architecture Decision

The Plasmodium dispatch is **advisory** — it does not mandate remote execution.
When `gate = "auto"` is specified:
1. Query the collective for current state
2. Score all reachable gates against declared requirements
3. If a remote gate scores better than local, forward there
4. If no remote meets requirements, execute locally (graceful degradation)

This preserves the principle that any single gate can function independently
(no hard dependency on the collective being reachable).

---

## Next Work (G8 Continuation)

- Wire Plasmodium dispatch into continuous graph execution (tick-based re-evaluation)
- Add `system.models` RPC for model-aware inference routing
- Implement bonding protocol (gates declare willingness to accept workloads)
- Test with real multi-gate topology (ironGate + eastGate at minimum)

---

## Upstream Impact

- **songBird**: No changes needed (mesh.peers already provides addresses)
- **toadStool**: Should expose `system.compute` handler for GPU details
- **cellMembrane**: Graph definitions can now use `gate = "auto"` in TOML

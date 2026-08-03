# biomeOS Session 155n — Spring Dispatch Infrastructure

**Date**: August 3, 2026 08:00 EDT
**Version**: v4.56
**Commits**: `77866b4c` (executor), `a3a48919` (bootstrap hints)
**Gate**: eastGate

---

## Summary

Spring deploy graphs can now be **executed** through biomeOS signal graphs. Previously,
spring TOMLs used a simplified `action = "check_primal"` syntax that the executor silently
skipped. Now the executor normalizes these to canonical operations and resolves parameters
from all sources. This unblocks P2 #10 (multi-gate deploy), P1 #6 (G18 squirrel dispatch),
and enables `biomeos deploy graphs/hotspring_deploy.toml --validate-only`.

---

## What Was Done

### 1. `GraphNode.action` Field Normalization

Spring deploy graphs use `action = "check_primal"` instead of `[nodes.operation]`:

| action | normalizes to | handler |
|--------|---------------|---------|
| `check_primal` / `check_health` | `health_check` | health ping on primal socket |
| `start_primal` / `start_service` / `launch` | `start` | primal start capability |
| `wire_data` / `wire_content` / `register` | `register_capabilities` | cap registration |
| `invoke` / `call` / `dispatch` | `capability_call` | semantic dispatch |
| `rpc` / `send_rpc` | `rpc_call` | direct JSON-RPC |
| `verify` / `verify_lineage` | `lineage.verify_siblings` | lineage check |
| other | passthrough | existing handler match |

### 2. `GraphNode.params` + `effective_param()` Helper

Node-level `params = { primal = "beardog" }` now resolves alongside `operation.params`
and `config`. Priority: `operation.params` > `params` > `config`. The alias
`primal` → `primal_name` bridges the spring shorthand to handler expectations.

### 3. Shadow Deploy Gate Validation

`composition.deploy.shadow` now validates that every remote `gate` referenced in graph
nodes has a matching `[graph.env]` endpoint. Previously, unresolved gates were only
caught at runtime when the executor tried to forward. Now they fail at preflight:

```json
{
  "valid": false,
  "validation_errors": [
    "Unresolved gate(s): westGate. Add them to [graph.env] (e.g. gate_name = \"tcp://host:port\")"
  ]
}
```

### 4. Bootstrap Capability Hints

Added hotSpring (physics) and groundSpring (measurement) to compile-time bootstrap hints.
These were in the TOML registry but missing from cold-start fallbacks. Without them,
`capability.call("physics.md_run")` would fail on a fresh composition before runtime
registration completes.

### 5. Spring Deploy Graphs → v2.0.0

Updated 7 spring deploy graphs with:
- `[graph.metadata]` gate assignments (assigned_gate, data_gate)
- `wire-content-access` nodes for NestGate capability registration
- Corrected dependency chains
- Version bump to 2.0.0

---

## Verification of P1 #4 (Inter-gate content.get)

biomeOS routing infrastructure is **VERIFIED COMPLETE** for cross-gate content access:

| Mechanism | Status | File |
|-----------|--------|------|
| `capability.call` with `gate` param | ✅ Implemented | `handlers/capability/call/gate.rs` |
| Gate registry from `[graph.env]` | ✅ Implemented | `gate_registry.rs` |
| `route.register` for remote capabilities | ✅ Implemented | `handlers/capability/registration.rs` |
| Songbird mesh relay fallback | ✅ Implemented | `handlers/capability/call/mesh.rs` |
| `nest.sync` signal graph (DAG fetch) | ✅ Implemented | `graphs/signals/nest_sync.toml` |
| `content.get` translation → nestgate | ✅ Registered | `config/capability_registry.toml` |

**What's needed**: live operational validation (nestGate + songBird running on separate
gates, 1 GB roundtrip). This is not a biomeOS code gap — it's a deployment test.

---

## For Upstream

biomeOS spring dispatch is READY. Key messages:

1. **Spring deploy graphs are now executable**: `biomeos deploy graphs/hotspring_deploy.toml`
   will parse and execute the `action`-based format correctly.

2. **Shadow deploy validates gates**: `composition.deploy.shadow` catches missing gate
   endpoints at preflight, not runtime. Use this before live multi-gate deploys.

3. **Inter-gate content.get is WIRED**: biomeOS routes `content.get` to NestGate via
   translation registry. Cross-gate needs `gate` param or Songbird mesh. No code gaps.

4. **G18 squirrel dispatch**: biomeOS side is ready. Springs are registered in bootstrap
   hints and TOML registry. Squirrel needs to wire `signal.plan` → biomeOS `graph.execute`.

### Remaining operational items (not code):
- Live inter-gate content.get E2E (nestGate + songBird across gates)
- southGate NUCLEUS launch + bonding
- Depot rebuild of v4.56 with spring dispatch

# biomeOS Session 150y — Neural Router Pool Integration + Chimera Phase 0

**Date**: 2026-07-24
**Version**: v4.39
**Wave**: 150y (eastGate local)

---

## Summary

Wired the `ConnectionPool` (built in 150x) directly into the `NeuralRouter` hot dispatch path, completing the performance loop. Evolved the chimera definition schema to support Tower Atomic Phase 0. Cleaned dead workspace dependencies.

## Changes

### 1. Neural Router Pool Integration (`biomeos-atomic-deploy`)
- `NeuralRouter` struct now holds a `ConnectionPool` field
- `forward_request_inner` uses `pool.send_jsonrpc()` instead of creating a fresh `AtomicClient` per request
- `tokio::time::timeout` wraps the pool call for deadline enforcement
- Proper `IpcError::JsonRpcError` extraction from pooled JSON-RPC response
- Both `new()` and `with_persistent_weights()` constructors initialize the pool

### 2. TransportEndpoint Display Optimization (`biomeos-core`)
- `Display::fmt` now writes directly via match arms (was calling `display_string()` → intermediate String)
- Pool key path (`endpoint.to_string()`) benefits from zero intermediate allocation

### 3. Chimera Schema Evolution (`biomeos-chimera`)
- `DeploymentSpec`: `requires_network`, `can_federate`, `composition: Option<String>`, `replaces: Vec<String>`
- `Fusion::shared_state: Vec<SharedStateEntry>` — zero-IPC memory sharing contracts
- `SharedStateEntry` type: name, owner, readers, type_hint
- `PerformanceSpec`: HashMap-based targets + baseline for chimera perf validation
- New test: `test_tower_atomic_definition_loads` — validates full YAML round-trip

### 4. Dead Dependency Removal
- `glob = "0.3"` — removed from workspace (zero crate imports)
- `regex = "1.11"` — removed from workspace (zero crate imports)
- `pool.rs` `to_string()` → `to_owned()` micro-optimization

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 8,494 | 8,610 |
| Clippy | PASS | PASS |
| Workspace deps | 75 | 73 |
| Neural router forward path | AtomicClient per-request | ConnectionPool persistent |

## Architecture Notes

The pool integration means the dispatch chain is now:
```
capability.call → NeuralRouter::forward_request → forward_request_inner
  → pool.send_jsonrpc(endpoint, request)
    → take pooled stream OR connect fresh
    → write NDJSON, read response line
    → return stream to pool
```

This eliminates: socket(), connect(), close() on every request. The server side already loops on the connection reading NDJSON lines.

## Upstream Notes (for other primal teams)

- **bearDog**: Tower Atomic fusion bindings reference `btsp`, `enrollment`, `crypto`, `hsm` modules. Library extraction starts when these modules have clean `pub` API boundaries.
- **songBird**: Referenced modules: `mesh`, `relay`, `federation`, `drawbridge`. Same library extraction requirement.
- **skunkBat**: Referenced modules: `threat_detector`, `firewall`, `audit`.

## Next Steps

1. Library extraction: each primal team exposes named modules as crate features
2. `biomeos-chimera` builder: generate Rust code from YAML definitions
3. Shadow validation: run chimera alongside 3-process Tower
4. Coverage: push toward 90% (currently 88.37%)

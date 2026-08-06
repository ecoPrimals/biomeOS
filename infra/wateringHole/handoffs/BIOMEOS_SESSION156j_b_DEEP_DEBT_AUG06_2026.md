# biomeOS Session 156j-b — Deep Debt: tcp_only Deprecated + Arc<str> + Flaky Tests Fixed

**Date**: August 6, 2026
**Version**: v4.57
**Gate**: eastGate

---

## Summary

Three deep debt evolutions in a single session:

1. **tcp_only mode deprecated** — Transport strategy now resolved by atomic composition profile, not a global "skip UDS" flag. All CLI flags, env vars, and APIs emit deprecation warnings. Backward-compatible (nothing breaks).

2. **DashMap<String> → DashMap<Arc<str>>** on hot dispatch paths:
   - `RUNTIME_CAPABILITY_REGISTRY` (capability→provider, queried every RPC call)
   - `ChimeraRegistry::definitions` (chimera lookup)

3. **3 flaky tests fixed** — Root cause: `lazy_rescan_sockets()` scanning the real filesystem during parallel tests. Tests expecting "no capabilities" could find live primals' `.sock` files. Fix: wrap in `temp_env::async_with_vars` pointing socket dirs to empty tempdirs.

---

## Changes

| File | Change |
|------|--------|
| `crates/biomeos-types/src/env_config.rs` | `BindMode::TcpOnly` marked deprecated with full context |
| `crates/biomeos/src/main.rs` | CLI `--tcp-only` and `--bind-mode tcp_only` emit deprecation warnings |
| `crates/biomeos/src/modes/neural_api.rs` | Runtime warning when tcp_only mode active |
| `crates/biomeos-atomic-deploy/src/neural_api_server/mod.rs` | `with_tcp_only()` builder deprecated with warning |
| `crates/biomeos-atomic-deploy/src/capability_domains.rs` | `DashMap<String, String>` → `DashMap<Arc<str>, Arc<str>>` |
| `crates/biomeos-chimera/src/registry.rs` | `DashMap<String, Arc<ChimeraDefinition>>` → `DashMap<Arc<str>, ...>` |
| `crates/biomeos-atomic-deploy/src/neural_router/discovery_tests.rs` | 4 tests wrapped in env isolation (tempdir) |

---

## Test Flake Root Cause

```
NeuralRouter::discover_capability() includes lazy_rescan_sockets() at step 2.
This scans $XDG_RUNTIME_DIR/membrane/ for .sock files.
In parallel test execution, other tests (or live primals on dev host) create sockets.
Tests expecting "no capabilities" would intermittently succeed instead of failing.

Fix: temp_env::async_with_vars points BIOMEOS_SOCKET_DIR + XDG_RUNTIME_DIR
to empty tempdirs, isolating each test from the real filesystem.
```

---

## Verification

- `cargo check --workspace` — clean
- `cargo clippy --workspace --all-targets -- -D warnings` — 0 warnings
- `cargo fmt --check` — no diffs
- `cargo deny check` — advisories, bans, licenses, sources all OK
- `cargo test --workspace` — 8,578 passed, 0 failed (across 3 full runs)
- `cargo test -p biomeos-atomic-deploy --lib` — 1,577 passed, 0 failed (previously 3 flaky)

---

## Next Wave Candidates

- G64 Phase 3: Remove `tcp_only` entirely once grapheneGate confirms Dual mode works
- D1: `biomeos nucleus attach` for tideGlass on westGate (operational)
- E2: squirrel systemd on ironGate (agent panel)
- O7: Inter-gate `content.get` E2E validation

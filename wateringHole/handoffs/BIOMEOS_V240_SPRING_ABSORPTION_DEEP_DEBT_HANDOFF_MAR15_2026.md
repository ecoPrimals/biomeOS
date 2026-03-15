# biomeOS v2.40 — Spring Absorption Deep Debt Handoff

**Date**: March 15, 2026
**Version**: 2.40
**Previous**: v2.39 (Concurrency Evolution)
**Status**: Production Ready

---

## Summary

9-phase deep debt evolution absorbing spring capabilities, implementing BYOB graph deployment, JSON-RPC 2.0 batch support, runtime TOML capability registry, and eliminating 50 `#[ignore]` tests via dependency injection.

---

## Changes

### Phase 1: BYOB Graph Deployment
- **BYOB** redefined from "Bring Your Own Beardog" to **"Build Your Own Biome"** — niche deployment via Neural API graphs
- Deleted orphaned `byob/manager.rs` (incompatible with graph architecture)
- `NicheDeployment::start_organism()` now discovers binaries via `which::which(name)` and spawns processes
- `NicheDeployment::stop()` uses `rustix::process::kill_process` (pure Rust, replaces `libc::kill`)
- `ByobManager::validate_team_config()` implements real validation (team_id, isolation, resource limits)

### Phase 2: JSON-RPC 2.0 Batch
- `JsonRpcInput` enum: `Single(JsonRpcRequest)` | `Batch(Vec<JsonRpcRequest>)`
- `JsonRpcInput::parse()` distinguishes single objects from arrays
- Neural API `dispatch_line()` processes batch elements concurrently via `futures::future::join_all`

### Phase 3: Compute Dispatch Translations
- 6 translations: `compute.dispatch.submit/status/cancel`, `compute.hardware.observe/distill/apply`
- Maps barraCuda dispatch and toadstool hw-learn capabilities

### Phase 4: Runtime TOML Registry
- Neural API startup loads three layers: hardcoded defaults → `config/capability_registry.toml` → graph
- `load_translations_on_startup()` supports overlay loading with warn-on-failure

### Phase 5: Real Capability Querying
- `query_primal_capabilities()` in `topology.rs` connects to primal sockets
- Sends `capability.list` JSON-RPC, parses response with 500ms timeouts

### Phase 6: Dependency Injection (50 #[ignore] removed)
- `network_config.rs`: 18 `#[ignore]` removed — `from_env_with()`, `parse_port_with()`, `resolve_stun_servers_with()`
- `defaults.rs`: 11 `#[ignore]` removed + 35 `env::set_var` calls → `_with` DI variants
- `env_config.rs`: 9 `#[ignore]` removed — 7 private `_with` helpers
- `engine_tests.rs`: 11 `#[ignore]` removed — `build_socket_path_with()` + 4 more

### Phase 7: Primal Name Constants
- `primal_discovery.rs` `matches!()` → `primal_names::is_known_primal()` (case-insensitive)
- Added `BIOMEOS`, `BIOMEOS_DEVICE_MANAGEMENT` to `primal_names.rs`

### Phase 8: Dead Code Cleanup
- `#[allow(dead_code)]` resolved: `#[serde(rename)]` for wire fields, `#[cfg(test)]` for planned utilities

### Phase 9: Health Alignment
- `health.ping` and `health.status` → canonical `health.check`

---

## Dependencies Added
- `biomeos-niche`: `which = "6"`, `rustix = { version = "0.38", features = ["process"] }`
- `biomeos-atomic-deploy`: `futures = { workspace = true }`

---

## Metrics

| Metric | v2.39 | v2.40 |
|--------|-------|-------|
| Tests | 4,885 | 4,946 (+61) |
| Ignored | 181 | 131 (-50) |
| Clippy | PASS | PASS |
| Format | PASS | PASS |
| Unsafe | 0 | 0 |
| C deps | 0 | 0 |

---

## Files Modified

| File | Change |
|------|--------|
| `biomeos-core/src/byob.rs` | BYOB redefined, real validation |
| `biomeos-core/src/byob/manager.rs` | **Deleted** |
| `biomeos-niche/src/deployment.rs` | Process spawn/kill via which+rustix |
| `biomeos-niche/Cargo.toml` | +which, +rustix |
| `biomeos-types/src/jsonrpc.rs` | JsonRpcInput enum |
| `biomeos-types/src/primal_names.rs` | +BIOMEOS constants |
| `biomeos-atomic-deploy/src/neural_api_server/connection.rs` | Batch dispatch |
| `biomeos-atomic-deploy/src/capability_translation/defaults.rs` | Compute+health translations |
| `biomeos-atomic-deploy/src/neural_api_server/server_lifecycle.rs` | TOML overlay loading |
| `biomeos-atomic-deploy/src/handlers/topology.rs` | Real capability querying |
| `biomeos-atomic-deploy/src/primal_discovery.rs` | is_known_primal() |
| `biomeos-atomic-deploy/src/beardog_jwt_client.rs` | Dead code cleanup |
| `biomeos-types/src/network_config.rs` | DI for 18 tests |
| `biomeos-types/src/defaults.rs` | DI for 11 tests + 35 env calls |
| `biomeos-types/src/env_config.rs` | DI for 9 tests |
| `biomeos-core/src/socket_discovery/engine_tests.rs` | DI for 11 tests |
| `biomeos-api/src/handlers/livespores.rs` | #[cfg(test)] gating |
| `biomeos-cli/src/commands/spore.rs` | #[cfg(test)] gating |
| `biomeos-cli/src/commands/chimera.rs` | #[cfg(test)] gating |

---

## Next Evolution
- Test coverage toward 90% (currently 76.15%)
- ARM64 genomeBin for biomeOS
- biomeOS on gate2 for cross-gate Neural API
- Remaining 131 `#[ignore]` tests (mostly E2E/chaos/integration)

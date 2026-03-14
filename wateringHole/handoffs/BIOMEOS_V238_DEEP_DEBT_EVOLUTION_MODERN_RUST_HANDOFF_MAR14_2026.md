# biomeOS v2.38 — Deep Debt Evolution: Modern Idiomatic Rust

**Date**: March 14, 2026
**Version**: 2.38
**Scope**: Comprehensive deep debt evolution — zero-copy, capability-based discovery, async-first tests, smart module refactoring

---

## Summary

Full-stack evolution of biomeOS codebase toward modern idiomatic Rust patterns. This session addressed all major categories from a comprehensive audit against wateringHole standards.

---

## Changes

### 1. Zero-Copy: `Vec<u8>` → `bytes::Bytes` (22 sites, 13 files)

Migrated all binary payload fields/return types to `bytes::Bytes` per wateringHole zero-copy standard:

| Crate | Files Changed | Fields Migrated |
|-------|---------------|-----------------|
| `biomeos-types` | `manifest/storage.rs`, `manifest_extensions.rs` | `binary_data: HashMap<String, Bytes>`, `data: HashMap<String, Bytes>` |
| `biomeos-spore` | `incubation.rs`, `dark_forest/beacon.rs`, `derivation/utils.rs` | `random_nonce`, `derive_deployed_seed()`, `generate_pure_noise_beacon()`, `generate_device_entropy()` |
| `biomeos-federation` | `beardog_client.rs` | `decrypt_data() -> Result<Bytes>` |
| `biomeos-core` | `family_discovery.rs`, `adapters.rs`, `types.rs` | `genesis_seed`, `node_key`, `parse_broadcast_key()`, `key_data` |
| `biomeos-nucleus` | `client.rs` | `get_family_seed_from_storage()` |
| `chimeras/platypus` | `crypto.rs`, `mesh.rs`, `fusion.rs` | `public_key`, `payload`, `signature` |

### 2. Capability-Based Discovery: Hardcoded Primal Names → Constants (9 files)

All production code now uses `biomeos_types::primal_names::*` constants instead of string literals:

- `rootpulse.rs` — `PROVENANCE_PRIMALS.iter().copied().chain([BEARDOG, NESTGATE])`
- `federation/modules.rs` — service tuples use `TOADSTOOL`, `SONGBIRD`, `NESTGATE`, `BEARDOG`
- `beardog_client.rs`, `discovery.rs` — lookups use constants
- `node_handlers.rs`, `plasmodium/mod.rs`, `config/mod.rs` — provider fallbacks
- `genomebin-v3/composer.rs` — atomic composition validation

### 3. Async-First Tests: Sleep → Proper Synchronization (~70 sites, 7 files)

Replaced all sleep-based test synchronization with proper async primitives:

| File | Pattern Used |
|------|-------------|
| `tests/atomics/common/helpers.rs` | `wait_for_socket()` + `wait_for_health()` polling |
| `tests/chaos_testing.rs` | Recovery polling loops, `yield_now()` |
| `tests/atomics/tower_chaos.rs` | Event-driven detection |
| `biomeos-graph/continuous.rs` | `watch` channel state observation |
| `biomeos/modes/continuous.rs` | Oneshot readiness, `Notify` for first request |
| `neural-api-client/tests.rs` | Oneshot server readiness |
| `tests/helpers/sync.rs` | `yield_now()` replacing sleeps |

### 4. Smart Module Refactoring (3 files → 8 files)

Split at logical cohesion boundaries, not arbitrary lines:

| Original | Size | Refactored Into |
|----------|------|----------------|
| `capability_translation.rs` | 985 | `mod.rs` (302) + `defaults.rs` (191) + `socket.rs` (28) + tests (337) |
| `device_management/provider.rs` | 944 | `provider.rs` (407) + `discovery.rs` (494) |
| `concurrent_startup.rs` | 931 | `concurrent_startup.rs` (210) + tests (672) |

### 5. Path Hardcoding → XDG-Aware Resolution (2 production files)

- `neural-api-client/client.rs` — fallback → `biomeos_types::defaults::socket_path()`
- `biomeos-federation/discovery.rs` — socket dir → `SystemPaths::new_lazy().runtime_dir()`

### 6. Stale Reference Cleanup

- Fixed `interaction.poll_sensors` → `interaction.poll` in `game_engine_tick.toml` and `continuous.rs`
- Fixed `visualization.build_scene` → `visualization.render` in `game_engine_tick.toml`
- SPDX header added to `beacon_genetics/manager/mod.rs` (619/619 coverage)
- Doc collision fixed via `[lib] doc = false` on workspace root

### 7. Coverage Expansion (~25 new tests)

Tests added for lowest-coverage production files:
- `doctor/checks_config.rs` (27→34%)
- `doctor/checks_primal.rs` (39→46%)
- `model_cache.rs` (47→54%)
- `rootpulse.rs` (45→67%)
- `main.rs` (38→44%)
- `neural-api-client-sync` (36%→higher)

---

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS (0 diffs) |
| `cargo clippy -D warnings` | PASS (0 warnings, pedantic+nursery) |
| `cargo doc --no-deps` | PASS (0 warnings, 0 collisions) |
| `cargo test --workspace` | PASS (4,728 passed, 0 failed, 203 ignored) |
| Files >1000 LOC | PASS (max 925) |
| SPDX headers | 619/619 |
| `#![forbid(unsafe_code)]` | All lib.rs crates |
| `unsafe` blocks | 0 |
| License | AGPL-3.0-only all crates |
| `cargo llvm-cov` | 76.15% line coverage |

---

## Standards Compliance

| Standard | Status |
|----------|--------|
| ecoBin v3.0 | COMPLIANT (zero C deps, deny.toml enforced) |
| UniBin | COMPLIANT (single binary, multi-mode) |
| Universal IPC v3.0 | COMPLIANT (JSON-RPC primary, tarpc escalation) |
| Semantic Method Naming | COMPLIANT (capability.call routing, 210+ translations) |
| AGPL-3.0-only / scyBorg | COMPLIANT (all crates) |
| wateringHole zero-copy | COMPLIANT (bytes::Bytes for binary payloads) |
| wateringHole primal naming | COMPLIANT (primal_names constants, no hardcoded strings) |
| XDG Base Directory | COMPLIANT (SystemPaths, no hardcoded /tmp/ in production) |
| Sovereignty | COMPLIANT (forbid(unsafe_code), no C deps, no telemetry) |

---

## Remaining Evolution Needs

1. **Coverage to 90%**: Currently 76.15%. Binary entry points (0%), doctor checks behind `#[ignore]`, network_config NAT traversal (60%) are the biggest gaps.
2. **Shell scripts**: 18 `.sh` files remain; `scripts/` has 4 shell scripts that should evolve to pure Rust or be archived.
3. **Python**: `scripts/mini_stun_server.py` — temporary STUN server for testing.
4. **Orphan examples**: 11 example files not declared in `Cargo.toml` — need wiring or archiving.
5. **Stale model references**: `claude-3-haiku-20240307` in JSON-RPC examples could use version-agnostic notation.

---

**Predecessor**: [BIOMEOS_V237_COMPREHENSIVE_AUDIT_CAPABILITY_DISCOVERY_HANDOFF_MAR14_2026.md]
**Test Count**: 4,728 (was 4,383, +345)
**Coverage**: 76.15% line (was 75.38%)

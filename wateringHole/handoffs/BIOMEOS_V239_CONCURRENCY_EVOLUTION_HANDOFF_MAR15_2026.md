# biomeOS v2.39 — Concurrency Evolution: Fully Concurrent Test Suite

**Date**: March 15, 2026
**Version**: 2.39
**From**: biomeOS team
**To**: All spring teams, barraCuda, toadStool, coralReef
**License**: AGPL-3.0-or-later
**Scope**: Systematic elimination of global state races — dependency injection, fully concurrent tests
**Supersedes**: BIOMEOS_V238_DEEP_DEBT_EVOLUTION_MODERN_RUST_HANDOFF_MAR14_2026.md

---

## Executive Summary

Systematic evolution of the entire biomeOS codebase to achieve fully concurrent test
execution. All 4,885 tests now run in parallel with zero failures. The core pattern:
functions that previously read environment variables or the current working directory
now accept explicit configuration parameters via `_with` / `_in` variant functions.
Tests call these variants directly, completely eliminating global state mutation.

Race conditions in tests are production pitfalls. By leaning into concurrent testing
as DNA synthesis and repair, we've evolved deep debt solutions that make both the
tests and the production code more robust.

---

## Quality Gates

| Gate | Status |
|------|--------|
| `cargo fmt --check` | PASS (0 diffs) |
| `cargo clippy -D warnings` | PASS (0 warnings, pedantic+nursery) |
| `cargo doc --no-deps` | PASS (0 warnings) |
| `cargo test --workspace` | PASS (4,885 passed, 0 failed, 181 ignored) |
| Files >1000 LOC | PASS (max 925) |
| `unsafe` blocks | 0 |
| License | AGPL-3.0-only all crates |
| Concurrency | All non-chaos tests run fully parallel |

---

## Pattern: Dependency Injection for Testability

Every function that previously read global state (env vars, CWD) now has a
`_with` variant that accepts the state as parameters:

```rust
// Production entry point — reads env vars as before
fn resolve_primal_socket(primal: &str) -> PathBuf {
    resolve_primal_socket_with(primal, std::env::var("BIOMEOS_SOCKET_DIR").ok())
}

// Testable variant — accepts config explicitly
fn resolve_primal_socket_with(primal: &str, socket_dir: Option<String>) -> PathBuf {
    if let Some(dir) = socket_dir { /* ... */ }
    SystemPaths::new_lazy().primal_socket(primal)
}
```

Tests call the `_with` variant directly with isolated, per-test configuration.
No env var mutation. No CWD mutation. No `#[serial]`. No races.

---

## Changes by Module

### Environment Variable Races Eliminated

| Module | Function | Config Param |
|--------|----------|-------------|
| `continuous.rs` | `resolve_primal_socket_with` | `socket_dir: Option<String>` |
| `enroll.rs` | `discover_beardog_socket_in` | `socket_dir: Option<&Path>, family_id: Option<&str>` |
| `family_discovery.rs` | discovery functions | `FamilyDiscoveryConfig` struct with override fields |
| `genome_dist/discovery.rs` | `get_genome_bin_path_with` | `env_path: Option<&str>, search_paths: &[&Path]` |
| `biomeos-ui/discovery.rs` | `discover_*_with_config` | `DiscoveryConfig` struct |
| `capability_taxonomy` | `default_primal_with` | `strict: bool` |
| `nucleus.rs` | `resolve_socket_dir_with`, `discover_binaries_with`, `build_primal_command_with` | Explicit env values |
| `model_cache.rs` | `run_with` | `cache_dir: &Path, hf_hub_dir: &Path` |
| `doctor/checks_*.rs` | `check_plasmid_bin_at`, `check_configuration_with` | `base_dir: &Path`, `config_dir: &Path` |
| `paths.rs` | `SystemPaths::new_with_xdg_overrides` | `xdg_runtime_dir, xdg_data_home` |
| `identifiers.rs` | `FamilyId::get_or_create_with` | `env_value: Option<&str>` |
| `defaults.rs` | `RuntimeConfig::from_env_with` | `socket_dir_override, xdg_runtime_dir_override` |
| `discovery_bootstrap.rs` | `find_universal_adapter_with` | `discovery_endpoint, songbird_endpoint, skip_env` |
| `neural-api-client-sync` | `resolve_socket_with` | `neural_api_socket, family_id_override` |

### CWD Races Eliminated

| Module | Change |
|--------|--------|
| `biomeos-spore` | Added `SporeConfig.plasmid_bin_dir: Option<PathBuf>` — `copy_binaries()` uses it instead of CWD-relative `"plasmidBin"` |
| `test_support.rs` | Removed `std::env::set_current_dir()` — tests pass paths explicitly |
| All spore tests | Removed `DirGuard`, `std::env::set_current_dir()`, CWD save/restore |
| `biomeos-atomic-deploy` | `discover_primal_binary` prioritizes explicit `BIOMEOS_PLASMID_BIN_DIR` over CWD fallbacks |

### Annotations Removed

| Annotation | Count Removed | Where |
|------------|---------------|-------|
| `#[serial_test::serial]` | 13 | biomeos-core, biomeos-spore, biomeos-api, continuous, enroll |
| `#[ignore]` | 22 | nucleus_tests, model_cache, doctor checks, paths, identifiers, defaults, discovery_bootstrap, neural-api-client-sync, capability_taxonomy |

### Dependencies Cleaned

- `serial_test = "3.0"` removed from `biomeos-core/Cargo.toml` and `biomeos-spore/Cargo.toml`
- Only `tests/atomics/` (legitimate E2E/chaos tests that launch processes) retains `serial_test`

---

## Metrics

| Metric | v2.38 | v2.39 | Delta |
|--------|-------|-------|-------|
| Tests passing | 4,728 | 4,885 | +157 |
| Tests ignored | 203 | 181 | -22 |
| `#[serial]` (non-chaos) | 13 | 0 | -13 |
| `#[ignore]` (env-var) | 22 | 0 | -22 |
| Test execution | sequential (some) | fully concurrent | evolved |
| Line coverage | 76.15% | 76.15% | stable |
| Clippy | PASS | PASS | stable |

---

## Guidance for Springs

The dependency injection pattern used here should be adopted by all springs:

1. **Never mutate env vars in tests** — pass config directly
2. **Never mutate CWD in tests** — pass paths as parameters
3. **`#[serial]` is a code smell** — only use for true E2E/chaos that launches processes
4. **`#[ignore]` is hidden debt** — if a test can't run in CI, fix the architecture
5. **Race conditions in tests are production bugs** — concurrent testing is DNA repair

---

**Predecessor**: BIOMEOS_V238_DEEP_DEBT_EVOLUTION_MODERN_RUST_HANDOFF_MAR14_2026.md
**Test Count**: 4,885 (was 4,728, +157)
**Ignored**: 181 (was 203, -22)
**Concurrency**: Fully parallel

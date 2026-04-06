# Contributing to biomeOS

**License**: scyBorg triple-copyleft — AGPL-3.0-or-later (code), ORC (operational/mechanics), CC-BY-SA 4.0 (documentation/creative)

All contributions must comply with the ecoPrimals wateringHole standards.
See `ecoPrimals/wateringHole/STANDARDS_AND_EXPECTATIONS.md` (monorepo-relative) for the full standard.

---

## Pre-Push Checklist

Every commit must pass all three gates:

```bash
cargo test --all-features          # 0 failures
cargo clippy --all-features --all-targets -- -D warnings  # 0 warnings
cargo doc --all-features --no-deps # 0 warnings
cargo fmt --all -- --check         # 0 diffs
```

## Code Standards

| Rule | Detail |
|------|--------|
| **Edition** | Rust 2024 (`edition = "2024"`) |
| **MSRV** | Rust **1.87** (`rust-version = "1.87"` in workspace `Cargo.toml`) |
| **Linting** | `clippy::pedantic` + `clippy::nursery` — zero warnings, inherited via `[lints] workspace = true` |
| **Unsafe** | `#![forbid(unsafe_code)]` on all crate roots; `#![deny(unsafe_code)]` only for crates with justified test-only `#[allow]` |
| **Error handling** | `Result<T, E>` everywhere; no `.unwrap()` in library/production code; workspace denies `unwrap_used`/`expect_used` |
| **Documentation** | `#![warn(missing_docs)]` on all library crates; doc-tests count as tests |
| **File size** | No file over 1000 lines; refactor at logical boundaries, not arbitrary splits |
| **TODO/FIXME** | No TODO, FIXME, HACK, or XXX in committed code |
| **Commented code** | No commented-out code |
| **License headers** | `// SPDX-License-Identifier: AGPL-3.0-or-later` + `// Copyright 2025-2026 ecoPrimals Project` on every `.rs` file |

## IPC Standards

- **JSON-RPC 2.0** is required for all primal communication
- **tarpc** is optional for high-performance internal paths (protocol escalation)
- Primals discover each other at runtime via **capability-based discovery** per **CAPABILITY_BASED_DISCOVERY_STANDARD** v1.2.0 (audit narrative in `CURRENT_STATUS.md`) — do not add identity-based helpers (`discover_beardog_*`, primal-named socket fields in routing, etc.); use capability domains and `discover_capability_socket` / provider discovery APIs
- Use `biomeos_types::primal_names::*` constants only for bootstrap hints and packaging — not for runtime routing
- Use `biomeos_types::constants::*` for ports, endpoints, and other constants

## Zero-Copy Guidelines

- Binary payloads: `bytes::Bytes` (not `Vec<u8>`) on IPC boundaries
- Hot-path string IDs: `Arc<str>` (not `String`) for keys and identifiers that are cloned frequently
- Primal discovery results, graph node IDs, and connection maps should use `Arc<str>`

## ecoBin Compliance

- Zero external C dependencies — `deny.toml` enforces this
- Pure Rust: `rustix` for POSIX, `/proc` for metrics, `rtnetlink` for networking
- No `openssl`, `ring`, `aws-lc-sys`, `native-tls`, `sysinfo`, `libc`, `nix`
- YAML serde uses `serde_yml` via Cargo package rename: `serde_yaml = { package = "serde_yml", ... }` in workspace deps
- Cross-compilation must work: `cargo build --target x86_64-unknown-linux-musl`

## Test Requirements

- Every module has tests; `cargo test` passes
- New code should target 90%+ line coverage (measure with `cargo llvm-cov`)
- Property-based tests (`proptest`) for serialization roundtrips and invariants
- Doc-tests for all major public APIs
- Chaos/fault injection tests for resilience-critical paths
- Test modules use `#[allow(clippy::unwrap_used, clippy::expect_used)]`
- **Zero sleeps in tests**: Use `tokio::time::pause()` + `tokio::time::advance()` for time-dependent logic; use `ReadySender`/`ReadyReceiver` from `biomeos-test-utils` for server readiness signaling; only chaos tests may serialize
- **No `#[ignore]` for concurrency**: Evolve tests to be truly concurrent via dependency injection and `TestEnvGuard` instead of ignoring them

## Sovereignty and Human Dignity

- No vendor lock-in (no CUDA SDK, no proprietary APIs)
- `SovereigntyGuardian` evaluates all AI routing for dignity violations
- `HumanDignityPolicy` enforced: prevent discrimination, require human oversight, prevent manipulation, right to explanation
- Community-run STUN servers only (no Google, no Cloudflare for STUN)

## Commit Messages

Follow conventional commit style:

```
feat: add capability-based socket discovery for http_bridge
fix: replace unsafe env var mutation with TestEnvGuard RAII
refactor: extract registry queries from engine.rs (1023→871 lines)
test: add 19 tests for device_management_server (37%→88% coverage)
```

## Architecture

```
biomeOS = UniBin orchestrator (single binary, 14+ subcommands)
         ↓
  JSON-RPC 2.0 over Unix sockets (Tier 1)
  tarpc protocol escalation (optional)
  HTTP JSON-RPC for cross-machine (Tier 2)
         ↓
  Primals discover each other by capability (taxonomy + probes), not hardcoded identity
  Socket paths: $XDG_RUNTIME_DIR/biomeos/{primal}-{family_id}.sock (naming convention; resolution via capability discovery, not fixed primal imports)
```

See `specs/` for detailed specifications and `CURRENT_STATUS.md` for current state.

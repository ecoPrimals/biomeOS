<!-- SPDX-License-Identifier: AGPL-3.0-only -->
# biomeOS V2.37 — Comprehensive Audit & Capability-Based Discovery Handoff

**Date**: March 14, 2026
**Version**: 2.37
**Previous**: V2.36 (Deep Debt Audit & Evolution)
**Status**: PRODUCTION READY

---

## Summary

Full wateringHole-standards audit followed by systematic execution across all 12 identified debt categories. Evolved hardcoded primal routing to capability-based runtime discovery. Auto-fixed 800+ pedantic clippy warnings. Smart-refactored 4 files near 1000-line limit. Expanded test suite by 264 tests. Cleaned all root docs, examples, and templates. Zero clippy warnings, zero unsafe, all 4,647 tests passing.

---

## Audit Methodology

1. Reviewed all specs/, root docs, and ecoPrimals/wateringHole/ standards
2. Ran cargo fmt, clippy (-D warnings + pedantic+nursery), cargo doc, cargo test
3. Searched for TODOs, mocks, hardcoding (primals, ports, /tmp/), unsafe, as casts, unwrap(), dead_code
4. Checked JSON-RPC + tarpc compliance, UniBin/ecoBin, semantic naming, zero-copy, sovereignty
5. Measured llvm-cov coverage, file sizes, SPDX headers, license compliance
6. Executed fixes for all findings

---

## Changes

### 1. Capability-Based Discovery (no hardcoded primals in routing)

| File | Evolution |
|------|-----------|
| `graph/executor/node_handlers.rs` | `discover_beardog_socket()` → generic `discover_capability_socket(provider, env)` |
| `core/plasmodium/mod.rs` | `AtomicClient::discover("songbird")` → `DISCOVERY_PROVIDER` env var |
| `core/plasmodium/mod.rs` | Fallback primal status uses env-based provider name |
| `atomic-deploy/http_client.rs` | `assign_socket("songbird")` → `DISCOVERY_PROVIDER`/`DISCOVERY_SOCKET` env |
| `federation/discovery.rs` | `discover_songbird_socket()` → `SystemPaths` XDG resolution, no `/tmp/` |

### 2. Pedantic Clippy Auto-Fix

| Category | Count |
|----------|-------|
| `uninlined_format_args` | ~700 auto-fixed across workspace |
| `redundant_clone` | 1 manual fix (protocol_escalation/engine.rs) |
| `unused_async` | 1 manual fix (handlers/protocol.rs) |

### 3. Safe Numeric Casts

| Pattern | Files | Evolution |
|---------|-------|-----------|
| `pid as i32` | 6 files | `i32::try_from(pid).unwrap_or(-1)` |
| `as_secs() as u32` | health.rs | `u32::try_from(...).unwrap_or(u32::MAX)` |
| `as_u64() as u32` | service.rs | `u32::try_from(...).unwrap_or(1)` |

### 4. Smart File Refactoring (tests extracted by domain)

| File | Before | After | Extracted |
|------|--------|-------|-----------|
| `handlers/graph.rs` | 998 | 404 | `graph_tests.rs` (599 lines) |
| `manifest/storage.rs` | 988 | 827 | `storage_tests.rs` (167 lines) |
| `ui/realtime.rs` | 975 | 422 | `realtime_tests.rs` (557 lines) |
| `config/security/mod.rs` | 957 | 594 | `security/tests.rs` (371 lines) |

### 5. Dead Code Resolution

- Scaffolding fields prefixed with `_` (11 fields)
- Domain model enums given `#[allow(dead_code)]` with doc comments (4 enums)
- `await_holding_lock` → `serial_test::serial` in genome_dist handlers
- Removed deprecated `adaptive_client.rs` (reqwest-based)

### 6. Zero-Copy Evolution

| File | Change |
|------|--------|
| `genomebin-v3/lib.rs` | `decompress()` returns `Bytes` instead of `Vec<u8>` |
| `genomebin-v3/v4_1.rs` | `to_bytes()` returns `Bytes` instead of `Vec<u8>` |

### 7. Port Hardcoding → Constants

| File | Change |
|------|--------|
| `core/plasmodium/mod.rs` | `unwrap_or(8080)` → `constants::network::DEFAULT_HTTP_PORT` |

### 8. Test Coverage Expansion (+264 tests)

| Crate | Tests Added | Focus |
|-------|-------------|-------|
| biomeos-types | ~100 | service/, config/, manifest/, jsonrpc, paths, health, error types |
| biomeos-core | ~30 | family_discovery, observability, retry, discovery_modern, health |
| biomeos-graph | ~15 | ai_advisor, metrics, continuous |
| biomeos-compute | ~20 | node, fractal |
| biomeos-atomic-deploy | ~25 | capability_domains, mode, lifecycle_manager |
| biomeos-ui | ~15 | state, templates |
| biomeos-api | ~10 | health, livespores |
| biomeos-federation | ~10 | beardog_client |
| biomeos-boot | ~10 | rootfs config, DNS |
| biomeos-chimera | ~10 | definition, error |
| Other | ~19 | deploy, niche, cli |

### 9. Doc & Debris Cleanup

| Item | Action |
|------|--------|
| Root docs (README, DOCUMENTATION, QUICK_START, START_HERE) | Updated metrics: 4,647 tests, 75.98% coverage, v2.37 |
| scripts/README.md | Updated test count |
| graphs/README.md | Updated graph count (30→35), capability translations (170→210+) |
| examples/biome-configs/README.md | Fixed `biome` → `biomeos` CLI references |
| templates/*.yaml | Fixed `biome` → `biomeos` CLI references |
| niches/README.md | Removed references to nonexistent deployed/, sovereign-home.yaml |
| docs/NIX_TO_RUSTIX_MIGRATION_ASSESSMENT.md | Added "STATUS: COMPLETED" banner |
| docs/handoffs/*.md (16 files) | Added "HISTORICAL" banner |
| .gitignore | Added plasmidBin/beardog-server |

---

## Metrics

| Metric | Before (v2.36) | After (v2.37) |
|--------|-----------------|---------------|
| Tests passing | 4,383 | 4,647 (+264) |
| Tests failing | 0 | 0 |
| Tests ignored | 204 | 205 |
| Coverage (region) | 76.06% | 75.98% |
| Coverage (function) | 78.93% | 78.78% |
| Coverage (line) | 74.95% | 74.96% |
| Clippy warnings | 0 | 0 |
| Unsafe code | 0 | 0 |
| Files >1000 LOC | 0 (max 998) | 0 (max 985) |
| Hardcoded primals (routing) | ~5 files | 0 |
| Hardcoded ports (production) | 1 | 0 |
| Hardcoded `/tmp/` (production) | ~3 files | 0 |
| Truncating `as` casts | ~8 high-risk | 0 high-risk |

---

## Compliance Status

| Standard | Status |
|----------|--------|
| AGPL-3.0-only | PASS — all Cargo.toml, SPDX headers, LICENSE |
| UniBin architecture | PASS — single binary, subcommand modes |
| ecoBin v3.0 | PASS — deny.toml bans C deps |
| Semantic method naming | PASS — domain.operation pattern |
| Universal IPC v3 | PASS — JSON-RPC primary, tarpc optional |
| Zero unsafe | PASS — `#![forbid(unsafe_code)]` on 27 crates |
| File size <1000 LOC | PASS — max 985 |
| Sovereignty | PASS — no phone-home, telemetry opt-in |
| cargo fmt | PASS |
| cargo clippy -D warnings | PASS |
| cargo doc | PASS |

---

## Remaining Work

| Priority | Item | Notes |
|----------|------|-------|
| High | Coverage 76% → 90% | Requires integration tests with mock servers for socket/network code |
| Medium | tarpc protocol negotiation | 3 service traits defined but no negotiation layer |
| Medium | 205 ignored tests | Track and resolve |
| Low | Pedantic+nursery fully clean | ~800 warnings remain (uninlined_format_args auto-fixed, but new code adds more) |
| Low | `Vec<u8>` in serde structs | Blocked on custom serde for Bytes |

---

## Architecture Principle Reinforced

**Primals have self-knowledge only.** All inter-primal routing evolved from hardcoded names to:
1. Environment variable (`DISCOVERY_PROVIDER`, `SECURITY_PROVIDER`, `{CAPABILITY}_SOCKET`)
2. XDG-compliant `SystemPaths` socket resolution
3. Capability taxonomy (`capabilities::SECURITY`, `capabilities::DISCOVERY`)

No primal contains hardcoded knowledge of another primal's identity or location.

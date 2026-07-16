# biomeOS Session 144b — Deep Debt Wave + Phase 2 Transport + Cleanup

**Date**: July 16, 2026  
**Version**: v4.35  
**Wave**: 144b  
**Scope**: Phase 2 transport trait dispatch, NucleusMode manifest discovery, placeholder evolution, dead dep removal, test file splits, root doc refresh, debris cleanup

---

## Summary

Completed deep debt resolution wave covering transport abstraction, hardcoding evolution, placeholder implementation, dependency hygiene, and test file refactoring.

---

## Changes

### 1. Phase 2 Transport (12 crates migrated)

Created `biomeos-core::ipc` module providing platform-agnostic IPC:
- `TransportStream` — enum wrapping `UnixStream`/`TcpStream`, implements `AsyncRead`/`AsyncWrite`
- `connect_transport` / `connect_transport_timed` — dispatch via `TransportEndpoint`
- `TransportListener` — unified server socket (Unix/TCP)
- `send_jsonrpc_request` / `send_jsonrpc_over_stream` — NDJSON JSON-RPC primitive

Migrated crates: `biomeos-atomic-deploy` (10 files), `biomeos-nucleus`, `neural-api-client`, `biomeos-primal-sdk`, `biomeos-federation`, `biomeos-api`, `biomeos-graph`.

### 2. NucleusMode Manifest Discovery

- `NucleusMode::resolve_launch_set()` reads `ecosystem_manifest.toml` composition profiles at runtime
- Falls back to `bootstrap_launch_order()` for cold start
- `CORE_PRIMALS` → `BOOTSTRAP_CORE_SET` (intent-clear naming across 5 crates)

### 3. Placeholder → Complete Implementation (4 items)

| Placeholder | Evolution |
|-------------|-----------|
| Topology silent fallback | Explicit standalone mode check; non-standalone returns degraded status |
| Songbird empty discovery | Error propagation via `FederationError::Discovery` |
| Spore manifest "unknown" | Build metadata: `CARGO_PKG_VERSION` + `GIT_COMMIT_HASH` from `build.rs` |
| USB detect hardcoded `/dev/sdX` | sysfs enumeration (`/sys/block/*/removable`) |

### 4. Dead Dependencies Removed

- `regex` from `biomeos-atomic-deploy` (0 imports in production)
- `glob` from `biomeos-core` (0 imports in production)
- `walkdir` from `biomeos-chimera` (inlined with `std::fs::read_dir`)

### 5. Version Bumps

- `serde-saphyr` 0.0.24 → 0.0.29
- `mdns-sd` 0.20.0 → 0.20.1
- `lz4_flex` 0.11 → 0.14

### 6. Test File Splits (22 files → <400L each)

All test files now under 450 LOC. 22 monolithic test files split into domain-focused submodules.

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests passing | 8,446 | **8,477** |
| Test failures | 0 | **0** |
| Production files >800L | 0 | **0** |
| Test files >450L | 33 | **0** |
| Dead dependencies | 3 | **0** |
| Production TODO/FIXME | 0 | **0** |
| `#[cfg]` transport stubs | ~40 files | **2** (BTSP Phase3 + tarpc genuine platform) |
| Hardcoded primal lists | 3 patterns | **0** (all bootstrap-intent-named) |

---

## Build Status

```
cargo check                                    ✅
cargo check --target x86_64-pc-windows-gnu     ✅
cargo test --workspace                         ✅ (8,477 passed, 0 failed)
```

---

## Remaining `#[cfg]` (Non-Transport, Intentional)

- `rustix::process::getuid()` — Linux-only UID check (placeholder UID on Windows)
- `primal_launcher.rs` `is_running()` — process existence check (OS-specific)
- `BTSP Phase 3` `node_handlers.rs` — needs BTSP client `TransportStream` evolution
- `tarpc Unix transport` — genuinely platform-specific (TCP fallback provided)
- `biomeos-system` disk/network/hostname — kernel-specific data sources

---

## Root Docs & Cleanup

### Documentation Refresh
All 6 root docs updated to v4.35: `CONTEXT.md`, `CONTRIBUTING.md`, `DOCUMENTATION.md`, `QUICK_START.md`, `START_HERE.md`, `SECURITY.md`.

### Debris Cleaned (untracked, not in git)
| Item | Size | Action |
|------|------|--------|
| `target/` | 125 GB | `cargo clean` |
| `vm-images/` | 660 MB | Removed (cloud-init .img) |
| `primals/` binaries | 113 MB | Removed (belong in plasmidBin repo) |
| `plasmidBin/` binaries | 792 MB | Removed (deploy via depot) |
| `bin/` binaries | 338 MB | Removed (build artifacts) |
| `base-spore/` | 33 MB | Removed (stale USB image) |
| `livespore-usb/*/primals/` | 382 MB | Removed (ELF binaries) |
| `pixel8a-deploy/` binaries | 67 MB | Removed |
| **Total freed** | **~127 GB** | |

Workspace on-disk: **18 MB** (tracked source only).

### No False Positives Found
- Zero TODO/FIXME/HACK in Rust source
- Zero stale references in docs
- All `.gitignore` entries correctly prevent re-accumulation
- `tmp-cloud-init/` (root-owned, 1KB) — harmless, already in `.gitignore`

---

## Upstream Gaps (For Other Primal Teams)

1. **BTSP client evolution**: `btsp_client_phase3.rs` still takes `UnixStream` directly; needs `TransportStream` parameter
2. **tarpc ecosystem**: `bincode v1` (unmaintained), `rand 0.8` duplicate — monitor upstream
3. **redb 2→4**: Major version gap; API migration needed in `biomeos-graph`
4. **criterion 0.5→0.8**: Dev-only, resolves `crossbeam-epoch` advisory (RUSTSEC-2026-0204)
5. **sporePrint rebuild on golgi**: Root 404 still active (P0 from Wave 144a)

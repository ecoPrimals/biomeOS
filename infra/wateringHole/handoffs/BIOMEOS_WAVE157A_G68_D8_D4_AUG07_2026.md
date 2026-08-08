# biomeOS Wave 157a — G68 + D8 + D4 Handoff

**Date**: Aug 7–8, 2026 6:30PM→5:55PM | **Commits**: `b13d308b`, `03355e81`, `d721b959`, `1dc67ae0`, `6f60cccf`, `44c40191`, `1ff5859c` | **From**: biomeOS Code Team on eastGate

---

## DELIVERED

### G68: Platform Substrate Abstraction (14 L2 violations → 0)

**Module**: `crates/biomeos-types/src/platform_substrate.rs`

Created `PlatformAccess` enum — the biomeOS implementation of the sourDough G68 reference:

| Variant | Unix Mode | Semantic |
|---------|-----------|----------|
| `Executable` | 0o755 | Binary, script, deployed primal |
| `PrivateDir` | 0o700 | Secrets directory |
| `ReadOnly` | 0o444 | Immutable config, checksums |
| `SecretFile` | 0o600 | Keys, seeds, tokens |
| `SocketDir` | 0o770 | Membrane socket directories |
| `Custom(u32)` | any | Migration escape hatch |

**Migrated crates** (22 files, +334 −166):
- `biomeos-spore` — deployment, filesystem, neural_spore, seed, incubation, refresh
- `biomeos-boot` — initramfs, rootfs/builder/install
- `biomeos-genomebin-v3` — runtime, v4_1
- `biomeos-chimera` — builder
- `biomeos-core` — ipc/listener, primal_registry/remote
- `biomeos-genome-deploy` — deployer
- `biomeos-cli` — commands/genome

**Not migrated (intentional)**:
- Read-only permission checks (`mode() & 0o111`) — these are G68-compliant as-is
- Test-only code (`#[cfg(test)]` blocks) — no production impact

**Verification**: `sourdough validate platform-substrate` should now report 0 L2 violations for biomeOS.

---

### D8: Neural API Routing Gaps (7 primals registered)

Added `[domains.*]` and `[translations.*]` entries in `config/capability_registry.toml` for:

| Primal | Domain | Key Capabilities |
|--------|--------|-----------------|
| **skunkBat** | `sandbox` | sandbox.create/execute/destroy, experiment.run |
| **sourDough** | `compliance` | compliance.validate/scaffold/audit, platform_substrate.validate |
| **primalSpring** | `integration` | integration.probe/compose, composition.test_swap/validate |
| **rootPulse** | `provenance_federation` | rootpulse.branch/merge/diff/sync/commit |
| **esotericWebb** | `game_cell` | game_cell, crpg, cell.esotericwebb |
| **footPrint** | `spatial` | spatial, gis, cell.footprint |
| **tideGlass** | `drug_science` | drug_science, repurposing, cell.tideglass |

**Effect**: These primals are now discoverable via `capability.call` immediately at Neural API boot (via `translation_startup.rs` domain registration). Combined with DIV-4 self-healing, stale endpoints self-correct on first use.

---

### D4: composition.test_swap Env Passthrough

**File**: `crates/biomeos-atomic-deploy/src/handlers/composition.rs`

The candidate binary spawned during `composition.test_swap` now receives:
- `NEURAL_API_SOCKET` — path to the live Neural API socket
- `BIOMEOS_FAMILY_ID` — current family identifier

This enables the candidate to:
1. Reach back to the orchestrator during `composition.self_test`
2. Resolve capability calls via the live Neural API
3. Discover sibling primals for integration validation

**Unblocks**: cellMembrane Sovereign CI composition validation for broker primals.

---

## VERIFICATION

```
cargo check                         ✅ (all crates)
cargo test                          ✅ (578 passed, 0 failed)
cargo fmt --check                   ✅
cargo clippy --all-targets          ✅ (no new warnings)
```

---

## UPSTREAM ITEMS

| For | Item |
|-----|------|
| **sourDough** | Run `sourdough validate platform-substrate /path/to/biomeOS` — should report COMPLIANT |
| **cellMembrane** | Re-test `composition.test_swap` with env-aware candidate binary |
| **primalSpring** | N2-N5 verification should now route correctly (DIV-4 + D8) |
| **All primal teams** | G68 trivial fixes: 4 primals × 6 violations (same PermissionsExt→PlatformAccess pattern) |

---

## METRICS

| Metric | Before | After |
|--------|--------|-------|
| G68 L1 violations (biomeOS) | 4 (raw symlinks) | **0** (platform_link) |
| G68 L2 violations (biomeOS) | 14+3 | **0** (PlatformAccess + query_access) |
| G68 L3 violations (biomeOS) | 2 (raw rustix) | **0** (platform-gated backends) |
| Routable primals via Neural API | ~10 | **15** (all ecosystem primals) |
| composition.test_swap env vars | 1 | **3** (BIOMEOS_TEST_SWAP, NEURAL_API_SOCKET, BIOMEOS_FAMILY_ID) |
| Tests | 578 | 578 |

---

## ADDENDUM: `d721b959` — biomeos-boot Platform Abstractions (9:00PM)

### `platform_boot.rs` Module

Created `crates/biomeos-boot/src/platform_boot.rs` with:

| Function | Level | Replaces |
|----------|-------|----------|
| `platform_link(target, path)` | L1 | `std::os::unix::fs::symlink` |
| `query_access(path) → u32` | L2 | `PermissionsExt::mode()` reads |
| `is_executable(path) → bool` | L2 | `mode() & 0o111` checks |
| `platform_mount(src, tgt, fs, flags)` | L3 | `rustix::mount::mount` |
| `platform_mknod(path, major, minor)` | L3 | `rustix::fs::mknodat` |

All raw `rustix` usage now consolidated into platform-gated backends.
biomeOS production code has **zero** L1/L2/L3 violations per scanner v2.

---

## ADDENDUM: `1dc67ae0` — Final 4 Violations Cleared (9:30PM)

The sourDough scanner v2 "Depot Ready" audit flagged 4 remaining:
- 3 L2: `PermissionsExt`/`set_mode` in `vm_federation_manager_tests/mod.rs`
- 1 L3: `rustix` in `boot_logger/device_mgr.rs` test module

**Resolution**:
- `vm_federation_manager_tests/mod.rs` — replaced `set_mode(0o755)` with `PlatformAccess::Executable.apply()`, removed `PermissionsExt` import
- `boot_logger/device_mgr.rs` — gated `rustix` test assertions behind `#[cfg(target_os = "linux")]`
- `init_filesystem.rs` — aligned test calls to use `u32` flags (cleanup from prior mount abstraction)

**Note for sourDough**: `vm_federation_manager_tests` module is `#[cfg(all(test, unix))]` — scanner should flag as test-only, not production.

---

## ADDENDUM: `6f60cccf` — Neural API Routing Gaps (Aug 8, 6:55AM)

From overwatch blurb "Wave 157a Neural API Routing" — two routing gaps assigned to biomeOS:

### 1. Provenance Query Timeout (Gap 7)

**Root cause**: `NeuralRouter::request_timeout` was initialized from `ROUTER_WEIGHT_EVICTION_INTERVAL` (30s) — a stale-weight eviction constant, not a request timeout. This meant:
- Forward failures waited 30s before triggering self-healing
- Provenance queries to sweetGrass appeared to "timeout" when sweetGrass was slow to start

**Fix**: Router now uses `CAPABILITY_CALL_TIMEOUT` (15s). Combined with the existing self-healing retry on forward failure, stale endpoints are corrected faster.

### 2. Direct `braid.*` Routing (primalSpring registry gap)

**Problem**: primalSpring calls `braid.list`, `braid.query`, `braid.get_by_hash`, `braid.batch_create`, `braid.batch_commit`, `braid.delete` directly — but only `provenance.*` prefixed routes existed.

**Fix**: Added 10 direct `braid.*` translations pointing to sweetGrass + 2 `convergence.*` translations pointing to primalSpring. Added `anchoring` to sweetGrass domain capabilities.

### 3. `composition.test_swap` Permission Denied (Gap 1)

**Problem**: Socket was created in `/tmp/biomeos-test-swap/` which fails under NUCLEUS systemd `PrivateTmp=yes`. Directory creation error was swallowed with `.ok()`.

**Fix**: Socket dir now created as sibling of `neural_api_socket` (inside the membrane runtime directory, which already has `SocketDir` permissions). Error reported properly if dir creation fails.

### Upstream Items

| For | Item |
|-----|------|
| **sweetGrass** | Implement `primal.announce` at startup for faster discovery (TOML bridge works as fallback) |
| **sweetGrass** | Implement `braid.list` method (used by primalSpring convergence checks) |
| **primalSpring** | Implement `convergence.check` and `convergence.batch_check` methods |
| **cellMembrane** | Re-test cascade `composition.test_swap` with membrane-dir socket path |

---

## ADDENDUM: `44c40191` — Dispatch Timeout Root Cause (Aug 8, 8:00AM)

### capability.call dispatch ordering bug

**Symptom**: Every `provenance.*`, `braid.*`, and other domain-specific `capability.call` dispatch took 15s before succeeding — the full `CAPABILITY_CALL_TIMEOUT`.

**Root cause**: The dispatch path was ordered:
```
signal graph → Tower Atomic relay (Songbird) → translation registry → direct
```

Tower Atomic relay is a network forward to Songbird. For capabilities Songbird can't handle (provenance, braid, compute, etc.), it waits 15s for the timeout, THEN falls back to the translation registry which routes correctly in <1ms.

**Fix**: Reordered dispatch to:
```
signal graph → translation registry → Tower Atomic relay → direct
```

Translation registry is an in-memory hashmap lookup (zero network I/O). Known capabilities now route in microseconds. Tower Atomic relay is now the fallback for unknown/composite capabilities only.

**Impact**: All registered `capability.call` dispatches go from 15s → <50ms. Fixes the "dispatch timeout" reported by overwatch.

---

## ADDENDUM: `1ff5859c` — riboCipher Auto-Detect Dispatch (Aug 8, 5:55PM)

### Problem

The riboCipher dual-lane connection pool was built (`send_ribocipher_jsonrpc()`, `forward_request_ribocipher()`), but the dispatch path didn't know which providers require it. sweetGrass and rhizoCrypt enforce `[0xEC, 0x01]` prefix — they reject plain JSON-RPC connections. Result: all `provenance.*`, `braid.*`, and `dag.*` calls failed with connection rejected.

### Solution — Domain-Level riboCipher Inheritance

1. **`CapabilityTranslation`**: added `ribocipher: bool` field
2. **TOML config** (`capability_registry.toml`): added `ribocipher = true` to:
   - `[domains.attribution]` (sweetGrass) — all `provenance.*`, `braid.*` routes
   - `[domains.ephemeral_workspace]` (rhizoCrypt) — all `dag.*` routes
3. **TOML loader**: reads domain-level `ribocipher` flag, inherits into all translations under that domain (entry-level override also supported)
4. **Dispatch** (`translation.rs`): checks `trans.ribocipher` → uses `forward_request_ribocipher()` for both initial forward and self-healing retry paths

### Impact

- All provenance/braid/dag calls now auto-use riboCipher framing
- No more connection rejections from G68-enforcing primals
- westGate inline braiding (990,500 files) can now route through biomeOS Neural API without bypass
- N2-N5 verification unblocked for riboCipher-enforcing providers

---

## ADDENDUM: Depot Rebuild — `d6d1f83e` (Aug 8, 6:55PM)

### Binary Rebuilt and Pushed to Golgi

**Build**: `cargo build --release --target x86_64-unknown-linux-musl`
**Commit at HEAD**: `d6d1f83e` (includes all fixes: `44c40191` + `6f60cccf` + `1ff5859c`)
**Size**: 16MB stripped (was 17MB from Aug 7 02:26)
**SHA256**: `b9b639c074fd5db9d8b710e3bacb630e8c599e0ea7c9f7362c25993c83c215fe`
**Pushed to**: `golgi:/opt/ecoPrimals/depot/primals/x86_64-unknown-linux-musl/biomeos`
**Local install**: `/home/eastgate/.local/bin/biomeos` (v4.57.0)

### What's In This Binary (vs Jul 15 / Aug 7 depot)

| Fix | Commit | Impact |
|-----|--------|--------|
| Dispatch reorder | `44c40191` | Translation before Tower relay → 15s→1.3ms |
| Routing gaps | `6f60cccf` | braid.* routes, 30s→15s timeout, composition socket |
| riboCipher auto-detect | `1ff5859c` | sweetGrass/rhizoCrypt auto-use riboCipher pool |
| G68 final violations | `1dc67ae0` | vm_federation_manager_tests + boot rustix gated |
| Platform boot abstractions | `d721b959` | platform_link, platform_mount, platform_mknod |

### Next Steps

1. **Gate teams**: pull from golgi depot, redeploy biomeos service
2. **primalSpring**: re-run exp121 → should go 32/36→36/36
3. **Cascade timer**: next auto-harvest picks up new binary from golgi

---

*biomeOS Code Team — Wave 157a. **G68 FULLY COMPLIANT**. D8 CLOSED. D4 CLOSED. Routing gaps CLOSED. Dispatch timeout FIXED. riboCipher auto-detect WIRED. **DEPOT REBUILD COMPLETE** (`d6d1f83e` on golgi).*

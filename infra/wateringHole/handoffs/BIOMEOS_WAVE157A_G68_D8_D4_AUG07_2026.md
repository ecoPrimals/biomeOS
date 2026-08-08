# biomeOS Wave 157a — G68 + D8 + D4 Handoff

**Date**: Aug 7, 2026 6:30PM→9:30PM | **Commits**: `b13d308b`, `03355e81`, `d721b959`, `1dc67ae0` | **From**: biomeOS Code Team on eastGate

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

*biomeOS Code Team — Wave 157a. **G68 FULLY COMPLIANT** (0 prod violations, 0 scanner hits). D8 CLOSED. D4 CLOSED.*

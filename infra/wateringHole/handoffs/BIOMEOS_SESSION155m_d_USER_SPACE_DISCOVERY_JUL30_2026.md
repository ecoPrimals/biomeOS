# biomeOS Session 155m-d — P2 Final Fix: User-Space Binary Discovery

**Date**: July 30, 2026
**Version**: v4.52
**Focus**: Close reopened P2 — socket evaporation on user-space deploys (westGate, strandGate)

---

## Problem

The P2 socket evaporation fix (v4.50) worked on sporeGate because it uses standard
`plasmidBin/` depot paths. However, westGate and strandGate deploy primals via:
- Source builds (`cargo install` → `~/.cargo/bin/`)
- User-space installs (`~/.local/bin/`)
- Manual PATH additions

When a primal crashed on these gates, `binary_search_dirs()` could not find its binary
for resurrection → socket disappeared → capability pruned → evaporation cascade.

---

## Root Cause

`binary_search_dirs()` in `handlers/spring_status.rs` only searched:
1. `$ECOPRIMALS_PLASMID_BIN`
2. `$BIOMEOS_PLASMID_BIN_DIR`
3. `./plasmidBin`, `../plasmidBin`, `../../plasmidBin`

User-space paths were completely absent. Additionally:
- `capability_handlers::discovery::discover_primal_binary` maintained its own hardcoded list
- `executor::primal_spawner::discover_primal_binary_impl` had yet another copy

Three independent search implementations, all with the same blind spot.

---

## Fix

Expanded `binary_search_dirs()` to be the single source of truth:

```
Search order (priority):
1. $ECOPRIMALS_PLASMID_BIN (explicit env override)
2. $BIOMEOS_PLASMID_BIN_DIR (explicit env override)
3. ./plasmidBin, ../plasmidBin, ../../plasmidBin (depot-relative)
4. $HOME/.local/bin/ (XDG user-space standard)
5. $HOME/.cargo/bin/ (source builds via cargo install)
6. $PATH entries (system-wide, arbitrary install locations)
```

Unified all three call sites to delegate to this single function:
- `handlers::spring_status::binary_search_dirs()` — authoritative implementation
- `executor::primal_spawner::discover_primal_binary_impl()` — now delegates
- `capability_handlers::discovery::discover_primal_binary()` — now delegates

### Bonus Fix
- Removed duplicate `#![warn(missing_docs)]` in root `src/lib.rs` (clippy error).

---

## Verification

| Check | Result |
|-------|--------|
| `cargo check --workspace` | PASS |
| `cargo clippy --workspace --tests -- -D warnings` | 0 warnings |
| `cargo test --workspace` | 8,570 pass, 0 failures |
| `cargo fmt --check` | PASS |

---

## Impact

| Gate | Deploy Style | Before v4.52 | After v4.52 |
|------|-------------|--------------|-------------|
| sporeGate | plasmidBin depot | Working | Working |
| westGate | source build + ~/.cargo/bin | **BROKEN** (evaporation) | **FIXED** |
| strandGate | source build + ~/.cargo/bin | **BROKEN** (evaporation) | **FIXED** |
| steamGate | ~/.local/bin user-space | **BROKEN** (evaporation) | **FIXED** |
| Any custom PATH | arbitrary | **BROKEN** (evaporation) | **FIXED** |

---

## Posture

**P2 CLOSED**. Socket evaporation fix now covers all deployment topologies:
- Depot-style (plasmidBin/)
- User-space (~/.local/bin/)
- Source builds (~/.cargo/bin/)
- System-wide ($PATH)

biomeOS is **STANDBY-READY** with zero open P0/P1/P2.

# biomeOS Session 155m-c — Deep Debt: Dependency Cleanup + Recovery

**Date**: July 30, 2026
**Version**: v4.51
**Focus**: Dead dependency removal, allocation optimization, workspace recovery

---

## Deliverables

| Delivery | Commit |
|----------|--------|
| Workspace recovery (30+ files restored from HEAD) | `0e45262f` |
| 14 dead dependencies removed (8 crates + 1 workspace) | `c6f83a73` |
| Registry alloc optimization (Arc<str> reuse, eliminate .to_string()) | `744b2d17` |
| Unfulfilled lint expectation removed | `744b2d17` |

---

## Dependencies Removed

| Crate | Removed |
|-------|---------|
| biomeos-nucleus | `ed25519-dalek`, `tokio-serde`, `tokio-util` |
| biomeos-ui | `indexmap`, `thiserror`, `tokio-stream` |
| biomeos-deploy | `biomeos-core` |
| biomeos-spore | `serde_bytes` |
| biomeos-manifest | `anyhow`, `uuid` |
| biomeos-chimera | `biomeos-types`, `chrono`, `uuid` |
| biomeos-atomic-deploy | `biomeos-federation` |
| biomeos | `comfy-table`, `etcetera` |
| biomeos-test-utils | `futures`, `tower`, `uuid` |
| workspace root | `serde_bytes` |

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Dead dependencies | 14 | 0 |
| Build artifacts on disk | 62.7 GiB | 0 (cargo clean) |
| Tests | 8,570+ | 8,570+ (0 regressions) |
| Clippy | 0 warnings | 0 warnings |
| cargo deny | clean | clean |

---

## Allocation Optimization

`neural_router::registry::register_capability`:
- Eliminated redundant `capability.to_string()` when the owned `String` was available
- Pre-creates `Arc<str>` for capability/primal name and uses `Arc::clone` for cheap sharing
- Removed unfulfilled `#[expect(clippy::implicit_clone)]` at crate root

---

## Workspace Recovery

Discovered `Cargo.toml`, `Cargo.lock`, and 30+ source files missing from disk due to stale staged deletions from a previous incomplete refactoring attempt. All files restored from `git HEAD` before proceeding with debt work.

---

## Documentation Updated

- `CURRENT_STATUS.md`: Added dep cleanup deliverables, dead deps metric
- `START_HERE.md`, `DOCUMENTATION.md`, `QUICK_START.md`, `README.md`: Test count 8,564→8,570, added dead deps metric
- `CONTEXT.md`: Version → v4.51
- `infra/wateringHole/CHAIN_STATUS.md`: Added dead deps metric, upstream items statused
- `infra/wateringHole/handoffs/README.md`: Added 155m-c entry, upstream items statused

---

## Posture

**STANDBY-READY**. Zero biomeOS-owned blockers. All deep debt criteria verified clean.

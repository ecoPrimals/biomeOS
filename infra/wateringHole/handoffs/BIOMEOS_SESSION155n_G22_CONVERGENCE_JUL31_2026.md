# biomeOS Session 155n — G22 API Convergence (Steps 1+2)

**Date**: July 31, 2026 12:45 EDT
**Version**: v4.56
**Commits**: `4b48b83b`, `bd33e17d`
**Gate**: eastGate

---

## Summary

G22 (whitePaper API convergence) is a glacial goal that became achievable after the
coevolution contract (G21) proved biomeOS can speak both riboCipher and plain JSON-RPC
natively. This session implements the first two code steps.

---

## Changes

### G22 Step 1: NUCLEUS Dual-Server Architecture (`4b48b83b`)

NUCLEUS Full mode now launches the HTTP API server (axum/WebSocket) alongside the
Neural API server (JSON-RPC) in a single process.

**Before**: `biomeos api` and `biomeos neural-api` were separate entry points, requiring
two processes or explicit composition. In NUCLEUS mode, only the Neural API was launched.

**After**: `biomeos nucleus` launches both. Single binary, single process, dual protocol:
- `neural-api-{family}.sock` — JSON-RPC for primal IPC
- `biomeos-api-{family}.sock` — HTTP/WebSocket for UI, dashboards, external tools

### G22 Step 2: Socket Namespace Unified (`bd33e17d`)

Migrated 46 files from stale `/biomeos/` socket path convention to the canonical
`/membrane/` namespace. This closes the "socket dir mismatch" P3 and establishes
biomeOS as the single authority over `/run/membrane`.

Affected: doc comments, test fixtures, examples, constants. Zero production behavior
change (runtime paths already resolved via `SystemPaths` + `MEMBRANE_SUBDIR`).

### Deep Debt: Dead Dependency Removal (Round 3)

Removed 5 unused dependencies (total 39 across sessions):
- `toml` from biomeos-genome-deploy
- `hex`, `thiserror`, `tokio` from biomeos-genome-factory
- `thiserror` from biomeos-genomebin-v3
- `indexmap` from biomeos-types

### P3 Audit: /run/membrane Permission Reset

Confirmed RESOLVED in v4.53. The `freshly_created` guard in `nucleation.rs` prevents
`apply_dir_group_ownership()` from running on existing directories. Only two call sites
exist, both properly guarded. No remaining unguarded permission paths.

---

## Remaining G22 Steps

| Step | Status | Owner |
|------|--------|-------|
| 1. NUCLEUS dual-server | ✅ DONE | biomeOS |
| 2. Socket namespace unified | ✅ DONE | biomeOS |
| 3. Sovereign CI git-pull-before-build | PENDING | sporeGate/operational |
| 4. Full `biomeos api` mode absorption | PLANNED (glacial) | biomeOS |

Step 4 would make the standalone `biomeos api` mode launch a Neural API alongside
its HTTP server (mirror of what NUCLEUS does). This is low-priority since NUCLEUS is
the standard production deployment.

---

## Codebase Health

| Metric | Value |
|--------|-------|
| Tests | 8,458+ pass |
| Clippy | 0 warnings |
| Unsafe blocks | 0 |
| TODOs in prod | 0 |
| Mocks in prod | 0 |
| Dead deps | 0 |
| Hardcoded primal names | 0 |
| Stale socket path refs | 0 |

---

## For Upstream

biomeOS v4.56 is ready for depot rebuild. Key changes for gates:
- NUCLEUS now also serves HTTP API (no separate `biomeos api` needed)
- All socket paths in docs/configs should reference `membrane/` not `biomeos/`

The `/run/membrane` permission reset P3 can be closed — it was fixed in v4.53.
The "socket dir mismatch" P3 can be closed — all biomeOS refs unified.

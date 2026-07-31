# Handoffs — biomeOS

Session handoff documents for upstream overwatch audit.

Each file captures: what was done, metrics before/after, commits produced, and next-wave candidates.

## ⚡ STATUS: ALL biomeOS-OWNED P0/P1/P2/P3 RESOLVED — v4.55 STANDBY

**Version**: v4.55 | **Tests**: 8,570+ | **biomeOS P2/P3 open**: 0 | **Posture**: STANDBY-READY

### Chain 1: biomeOS Orchestration Lifecycle — COMPLETE (v4.44–v4.48)

| # | Item | Version | Commit |
|---|------|---------|--------|
| 1 | Graph executor riboCipher fix | v4.46 | `bd202674` |
| 2 | BTSP composition broker | v4.44 | (composition broker session) |
| 3 | Composition lifecycle (boot_order) | v4.48 | `076d4743` |
| 4 | Socket evaporation fix (persistence) | v4.46 | `bd202674` |
| 5 | Socket path unification (membrane/) | v4.46 | `bd202674` |

### P2 Divergences — ALL biomeOS-OWNED RESOLVED (v4.49 + v4.50 + v4.51)

| Divergence | Version | Fix |
|-----------|---------|-----|
| Capability wipe cycle (654→0→187→654) | v4.49 | 3-strike prune threshold |
| Neural API socket hardcoded to `membrane/` | v4.49 | Stale doc comments updated |
| API 403 on non-/health | v4.49 | Intentional Dark Forest (documented) |
| Socket evaporation (health ping format) | v4.50 | RPC ping tolerance — `Ok(_)` = alive |
| Binary path retention (blocks resurrection) | v4.50 | Auto-discovery probes plasmidBin |
| Socket ownership (multi-user) | v4.51 | `chown :membrane` post-bind + MEMBRANE_SOCKET_GROUP env |
| Socket evaporation (user-space deploys) | v4.52 | `binary_search_dirs()`: +~/.local/bin +~/.cargo/bin +$PATH |
| /run/membrane permission reset | v4.53 | Guard `apply_dir_group_ownership` behind `!exists()` |
| Sandbox false positive (orchestrator) | v4.53 | `composition.self_test` RPC endpoint |
| **Sandbox P2 ESCALATED (depot blocked)** | **v4.55** | **`composition.test_swap` — delegated validation** |

### P1 Divergences — ALL RESOLVED (v4.54)

| Divergence | Version | Fix |
|-----------|---------|-----|
| Respawn storm (175 procs/14 min) | v4.54 | Dual-protocol health ping: plain JSON-RPC first, BTSP fallback |
| Socket file deletion (50% survival) | v4.54 | PID ownership + confirmed kill before unlink |

### Upstream Items — Status per Wave 155n

| Issue | Owner | Status |
|-------|-------|--------|
| rootpulse.ledger | cellMembrane | **FIXED** (`0cfcce5`) |
| checksums.toml partial | sporeGate CI | **FIXED** (`0cfcce5`) |
| /run/membrane tmpfiles.d | cellMembrane | **FIXED** (`0cfcce5`) |
| golgi post-receive hook | golgiBody | **FIXED** (Wave 155n confirmed) |
| `membrane/` vs `biomeos/` socket dir | cellMembrane | **OPEN** — biomeOS scans both; needs primal launch env |
| cellMembrane not in sources.toml | cellMembrane | **P3 OPEN** |
| GNU depot incomplete (4/16) | sporeGate | **P3 OPEN** |

---

## Current Handoffs

| Session | Date | Focus |
|---------|------|-------|
| **155n** | Jul 31, 2026 | **Coevolution contract: composition.test_swap (P2 sandbox unblock)** |
| 155m-f | Jul 31, 2026 | Deep debt: 15 more dead deps removed (29 total), full audit clean |
| 155m-e | Jul 31, 2026 | P3 fixes: permission reset + composition.self_test sandbox endpoint |
| 155m-d | Jul 30, 2026 | P2 FINAL FIX: user-space binary discovery (socket evaporation closed) |
| 155m-c | Jul 30, 2026 | Deep debt: 14 dead deps removed, registry alloc optimization, disk recovery |
| 155m-b | Jul 30, 2026 | Socket ownership fix (v4.51) + upstream triage |
| 155m | Jul 30, 2026 | P2 socket evaporation + binary path retention fix (v4.50) |
| 155k | Jul 30, 2026 | P2 divergence fixes: capability wipe cycle, socket docs, Dark Forest clarification |
| 155j | Jul 30, 2026 | Composition lifecycle: cellMembrane boot_order integration, Chain 1 complete |
| 155i-c | Jul 29, 2026 | Deep debt cleanup: dead deps purged, test extraction, capability-based BTSP resolution |
| 155i-b | Jul 29, 2026 | NUCLEUS Orchestrator: riboCipher executor fix, socket unification, capability persistence, composition.start |
| 155i | Jul 29, 2026 | Composition Broker: riboCipher + BTSP executor + E2E validation + deep debt audit clean |
| 155d | Jul 28, 2026 | Live signal graph validation — tower.health + tower.mesh_status (19→26 signals) |
| 155b | Jul 27, 2026 | Plasmodium G8 multi-gate bonding, workload dispatch, dead monolith + dep purge |
| 151c | Jul 26, 2026 | Deep debt cleanup, 100% clippy --tests, Arc<str> hot path |
| 151b | Jul 26, 2026 | SDK BTSP handshake evolution, strict mode readiness |
| 150y | Jul 24, 2026 | Neural Router pool integration, chimera schema evolution |
| 150x | Jul 24, 2026 | UDS connection pool crate, transport listener |
| 150t | Jul 21, 2026 | Deep debt clippy pass, stale socket cleanup |
| 149b | Jul 18, 2026 | Gap resolution, capability domain tests |
| 144b | Jul 16, 2026 | Deep debt wave, sovereignty guardian |

## Convention

Filename: `BIOMEOS_SESSION{wave}_{FOCUS}_{DATE}.md`

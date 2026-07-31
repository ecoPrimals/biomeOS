# biomeOS — Chain Status for Overwatch

**Last Updated**: July 31, 2026 12:45 EDT
**Version**: v4.56
**Team**: biomeOS
**Gate**: eastGate

---

## Chain 1: biomeOS Orchestration Lifecycle — ✅ COMPLETE

**All 5 items shipped. No remaining P1 blockers for biomeOS.**

| # | Item | Priority | Status | Version | Commit |
|---|------|----------|--------|---------|--------|
| 1 | Graph executor riboCipher fix | P1 | ✅ SHIPPED | v4.46 | `bd202674` |
| 2 | BTSP composition broker for live E2E | P1 | ✅ SHIPPED | v4.44 | (composition broker) |
| 3 | Composition lifecycle management (boot_order) | P1 | ✅ SHIPPED | v4.48 | `076d4743` |
| 4 | Socket evaporation fix | P2 | ✅ SHIPPED | v4.46 | `bd202674` |
| 5 | Socket path unification | P2 | ✅ SHIPPED | v4.46 | `bd202674` |

### Evidence

- `send_ribocipher_jsonrpc_request()` in `neural_executor_node_impls.rs` (Item 1)
- `persist_capability_registry()` / `load_persisted_capability_registry()` in `neural_router/registry.rs` (Item 4)
- 30+ usages of `MEMBRANE_SUBDIR` constant across all socket paths (Item 5)
- `extract_boot_order()` + `composition.boot_order` RPC + `ManagedPrimal.boot_order_index` (Item 3)
- `composition.start` health-gated transitions with prerequisite checking (Item 2+3)

---

## Chain 2: bearDog Crypto + Provenance 7/7 — ⏳ BLOCKED (not biomeOS)

| # | Item | Priority | Owner | Status |
|---|------|----------|-------|--------|
| 1 | `crypto.sign_ed25519` real signing | P1 | **bearDog** | OPEN |
| 2 | bearDog Windows platform gating | P1 | **bearDog** | OPEN |

**biomeOS is not blocking Chain 2.** bearDog team owns both items.

---

## Chain 3: Windows Depot — ⏳ PARTIALLY COMPLETE (not biomeOS)

| # | Item | Owner | Status |
|---|------|-------|--------|
| 1 | beardog.exe | bearDog | OPEN (UnixStream not gated) |
| 2 | toadstool.exe | toadStool | ✅ FIXED (`2df71399b`) |
| 3 | coralreef.exe | coralReef | ✅ FIXED (`edcd696`) |

**biomeOS is not blocking Chain 3.** bearDog team owns the last item.

---

## P1 Divergences — ALL RESOLVED + GATE VALIDATED (Wave 155n)

| Divergence | Resolution | Version | Gate Validation |
|-----------|-----------|---------|-----------------|
| Respawn storm (175 procs/14 min, strandGate) | Dual-protocol health ping: plain JSON-RPC first, BTSP fallback | v4.54 | strandGate: 13 procs (1/primal) |
| Socket file deletion (50% survival, westGate) | PID ownership + confirmed kill before unlink | v4.54 | westGate: 31/31 sockets 225s |

---

## P2 Divergences — ALL biomeOS-OWNED RESOLVED (Wave 155k–155n)

| Divergence | Resolution | Version |
|-----------|-----------|---------|
| Capability wipe cycle (654→0→187→654) | 3-strike prune threshold | v4.49 |
| Neural API socket hardcoded to `membrane/` | Stale doc comments updated (code was correct) | v4.49 |
| API 403 on non-/health | Intentional Dark Forest sovereign behavior (documented) | v4.49 |
| Socket evaporation (health ping format) | RPC ping tolerance — `Ok(_)` = alive | v4.50 |
| Binary path retention (blocks resurrection) | Auto-discovery probes plasmidBin, stores path | v4.50 |
| Socket ownership (multi-user access) | `chown :membrane` post-bind + MEMBRANE_SOCKET_GROUP | v4.51 |
| Socket evaporation (user-space deploy paths) | `binary_search_dirs()` expanded: +~/.local/bin +~/.cargo/bin +$PATH | v4.52 |
| /run/membrane permission reset | Guard `apply_dir_group_ownership` behind `!exists()` check | v4.53 |
| Sandbox false positive (orchestrator isolation) | `composition.self_test` + **`composition.test_swap`** (delegated validation) | v4.53 + **v4.55** |

---

## P3 Divergences — ALL biomeOS-OWNED RESOLVED (Wave 155n)

| Divergence | Resolution | Version |
|-----------|-----------|---------|
| Zombie process reaping | Background `child.wait()` | v4.54 |
| Virtual service DEGRADED churn | Skip resurrection for external primals | v4.54 |
| `graphs_dir` default path | XDG fallback + BIOMEOS_GRAPHS_DIR env | v4.54 |
| riboCipher rejection at ERROR level | Demoted to debug (protocol negotiation) | v4.54 |
| `--version` reports 0.1.0 | Workspace version synced to 4.54.0 | v4.54 |
| **Socket dir mismatch (membrane/ vs biomeos/)** | **46 files unified to membrane/** | **v4.56** |

---

## G22: whitePaper API Convergence — IN PROGRESS

| Step | What | Status | Version |
|------|------|--------|---------|
| 1 | NUCLEUS serves both HTTP API + Neural API in single process | ✅ DONE | v4.56 (`4b48b83b`) |
| 2 | Socket namespace unified (all refs → membrane/) | ✅ DONE | v4.56 (`bd33e17d`) |
| 3 | Sovereign CI git-pull-before-build | PENDING (sporeGate/operational) | — |
| 4 | `biomeos api` mode absorbs Neural API (full unification) | PLANNED (glacial) | — |

---

## Coevolution Contract (G21/J19) — COMPLETE

| Capability | Status | How it works |
|-----------|--------|-------------|
| `composition.self_test` | ✅ v4.53 | Proves biomeOS functional without full composition |
| `composition.test_swap` | ✅ v4.55 | Running biomeOS validates replacement binary, reports pass/fail |
| **Mode gap fix** | ✅ v4.55 (`652cf8a7`) | Neural API accepts plain JSON-RPC (btsp_optional) |

**E2E ready**: cellMembrane's `validate_with_deps` can now call `composition.test_swap`
via plain JSON-RPC on the running Neural API socket. No more riboCipher gate blocking.
Sovereign CI should be fully automated for all 13 primals including biomeOS.

---

## Upstream Items — Status per Wave 155n

| Issue | Owner | Status |
|-------|-------|--------|
| ~~`membrane/` vs `biomeos/` socket dir~~ | ~~cellMembrane~~ | **CLOSED** — all biomeOS refs unified (v4.56); primals already use membrane/ |
| cellMembrane not in sources.toml | cellMembrane | **FIXED** — `0d39075` (J16 KILLED) |
| GNU depot incomplete (4/16) | sporeGate | **P3 OPEN** |
| golgi post-receive hook | golgiBody | **FIXED** (Wave 155n confirmed) |

---

## biomeOS Posture: STANDBY-READY (G22 Active)

biomeOS Chain 1 complete. **All P0/P1/P2/P3 biomeOS-owned blockers resolved.**
G22 convergence actively progressing: Steps 1+2 shipped.

### Metrics (v4.56)

| Metric | Value |
|--------|-------|
| Tests | 8,458+ pass, 0 failures |
| Clippy | 0 warnings (pedantic+nursery, --tests, -D warnings) |
| Unsafe blocks | 0 (forbid) |
| Largest prod file | 716 LOC |
| TODOs in prod | 0 |
| Mocks in prod | 0 |
| Dead code | 0 |
| Dead dependencies | 0 (39 removed total, cargo-machete verified) |
| Hardcoded primal names | 0 in production |
| Stale socket path refs | 0 (unified to membrane/) |
| cargo deny | clean |
| Version | v4.56 |
| biomeOS P0/P1/P2/P3 | ZERO biomeOS-owned open |

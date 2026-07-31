# biomeOS — Chain Status for Overwatch

**Last Updated**: July 31, 2026 08:30 EDT
**Version**: v4.53
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

## P2 Divergences — ALL biomeOS-OWNED RESOLVED (Wave 155k + 155m)

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
| Sandbox false positive (orchestrator) | `composition.self_test` RPC endpoint for sandbox validation | v4.53 |

---

## Upstream Items — Status per Wave 155m

| Issue | Owner | Status |
|-------|-------|--------|
| rootpulse.ledger | cellMembrane | **FIXED** (`0cfcce5`) — returns ok=true advisory |
| Sandbox false positive | cellMembrane | **FIXED** (`0cfcce5`) — ServerContract resolution |
| checksums.toml partial | sporeGate CI | **FIXED** (`0cfcce5`) — full disk scan |
| /run/membrane tmpfiles.d | cellMembrane | **FIXED** (`0cfcce5`) — tmpfiles.d shipped |
| cellMembrane not in sources.toml | cellMembrane | **P3 OPEN** |
| golgi post-receive hook | golgiBody | **P3 OPEN** |

---

## biomeOS Posture: STANDBY-READY

biomeOS Chain 1 is complete. **All 12 biomeOS-owned P2 divergences resolved.**
Team in **STANDBY** pending:
- NUCLEUS v4.51 redeploy on strandGate
- AlphaFold ~1TB ingestion through westGate Nest Atomic
- steamGate Tower deployment (user-space, gnu bins)

### Metrics (v4.53)

| Metric | Value |
|--------|-------|
| Tests | 8,570+ pass, 0 failures |
| Clippy | 0 warnings (pedantic+nursery, --tests, -D warnings) |
| Unsafe blocks | 0 (forbid) |
| Largest prod file | 716 LOC |
| TODOs in prod | 0 |
| Mocks in prod | 0 |
| Dead code | 0 |
| Dead dependencies | 0 (29 removed total, cargo-machete verified) |
| Hardcoded primal names | 0 in production |
| cargo deny | clean |
| Version | v4.53 |
| biomeOS P0/P1/P2/P3 | ZERO biomeOS-owned open |

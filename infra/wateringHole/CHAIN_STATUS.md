# biomeOS — Chain Status for Overwatch

**Last Updated**: July 30, 2026 08:15 EDT
**Version**: v4.48
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

## biomeOS Posture: STANDBY-READY

biomeOS Chain 1 is complete. Team transitions to **STANDBY** pending:
- bearDog `crypto.sign_ed25519` (unblocks Provenance 7/7 E2E through biomeOS)
- NUCLEUS lifecycle integration testing (once strandGate validates auto-startup)

### Metrics (v4.48)

| Metric | Value |
|--------|-------|
| Tests | 8,570 pass, 0 failures |
| Clippy | 0 warnings (pedantic+nursery, --tests) |
| Unsafe blocks | 0 (forbid) |
| Largest prod file | 716 LOC |
| TODOs in prod | 0 |
| Version | v4.48 |
| Commits since v4.44 | 5 (all pushed to origin main) |

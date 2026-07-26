# biomeOS Session 151b — SDK BTSP Handshake Evolution

**Date**: 2026-07-26
**Version**: v4.40
**Wave**: 151b (eastGate local)

---

## Summary

Evolved `biomeos-primal-sdk` to perform BTSP ClientHello handshake automatically
when `FAMILY_ID` is set and the target socket is family-scoped. This closes the
last gap in biomeOS's BTSP compliance — the SDK layer now works under strict mode
(`BIOMEOS_BTSP_ENFORCE=1`).

## Context

Wave 151b mandates: "every primal that talks to bearDog must evolve to BTSP."
sporeGate already deployed strict mode — legacy plaintext JSON-RPC is rejected.

biomeOS's **core infrastructure** (biomeos-core, biomeos-atomic-deploy) has had
full BTSP support for multiple waves (Phase 2 + Phase 3, 1229 LOC). However,
the **primal SDK** — used by downstream primals that depend on biomeOS — was
sending raw plaintext JSON-RPC. Under strict mode, this would fail.

## Changes

### New: `biomeos-primal-sdk/src/ipc/btsp_handshake.rs` (294 LOC)
- 4-step handshake: ClientHello → ServerHello → ChallengeResponse → HandshakeComplete
- Zero-crypto: all operations delegated to security provider via JSON-RPC
- Anti-recursion: provider calls bypass BTSP (provider socket is never family-scoped)
- Detection: `should_btsp(path)` checks `FAMILY_ID` + socket naming convention
- Fallback: handshake failure logs warning and retries plaintext (graceful degradation)

### Modified: `biomeos-primal-sdk/src/ipc/jsonrpc.rs`
- `send_jsonrpc_request()` now checks `should_btsp()` before connecting
- On success: sends JSON-RPC over post-handshake BufReader
- On failure: falls back to raw stream with warning
- New `send_jsonrpc_over_reader()` for post-handshake communication

### Modified: `biomeos-primal-sdk/src/ipc/mod.rs`
- Added `btsp_handshake` module

### Tests: 5 new unit tests
- `is_family_scoped_recognizes_family_sockets`
- `is_family_scoped_rejects_non_family`
- `should_btsp_requires_family_id_and_scoped_socket`
- `should_btsp_false_without_family_id`
- `should_btsp_ignores_default_family_id`

## BTSP Compliance Matrix (biomeOS)

| Layer | Status | Notes |
|-------|--------|-------|
| `biomeos-core` client | DONE | Phase 2 + Phase 3 (encrypted framing) |
| `biomeos-core` server | DONE | Accepts + validates handshake |
| `biomeos-atomic-deploy` router | DONE | `btsp_enforce()` gates dispatch |
| `biomeos-api` listener | DONE | ClientHello detection + redirect |
| `biomeos-primal-sdk` | **DONE (NEW)** | Auto-handshake for SDK consumers |

**biomeOS is FULLY BTSP COMPLIANT** — ready for eastGate deployment with
`BIOMEOS_BTSP_ENFORCE=1`.

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| SDK tests | 146 | 151 |
| Workspace tests | 8,610 | 8,616 |
| Clippy | PASS | PASS |
| BTSP coverage | core only | core + SDK |

## Next Steps

1. Deploy `BIOMEOS_BTSP_ENFORCE=1` on eastGate (bearDog team validates)
2. grapheneGate HSM validation via BTSP (bearDog team, eastGate)
3. Continue chimera library extraction (Phase 0)
4. Push toward 90% coverage

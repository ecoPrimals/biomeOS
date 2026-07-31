# biomeOS Session 155n — Coevolution Contract: composition.test_swap

**Date**: July 31, 2026
**Version**: v4.55
**Focus**: P2 sandbox false positive resolution via delegated validation

---

## Summary

Implemented `composition.test_swap` — the biomeOS side of the coevolution contract (G21/J19).
This allows the running biomeOS instance to validate its own replacement binary, enabling
cellMembrane to delegate sandbox validation to the live orchestrator instead of running an
isolated sandbox (which fails for broker primals that can't self-validate in isolation).

---

## What Was Done

### composition.test_swap RPC

New JSON-RPC method on the Neural API:

```
Method: composition.test_swap
Params: { binary_path: "/absolute/path/to/candidate", timeout_secs: 15 }
Returns: { validated: true/false, candidate_version, reason, self_test }
```

Flow:
1. Running biomeOS spawns candidate binary on temp socket (`/tmp/biomeos-test-swap/candidate-<pid>.sock`)
2. Waits for socket to appear (up to `timeout_secs`)
3. Calls `composition.self_test` on candidate via plain JSON-RPC
4. Evaluates response (`ok: true` = validated)
5. Kills candidate and cleans up temp socket
6. Returns validation result to caller

This enables:
- **J19 sandbox P2 resolution**: cellMembrane calls `composition.test_swap` on running Neural API
  instead of standalone sandbox → broker primals validated in context
- **Zero-downtime deploys**: validate before swap, rollback metadata available
- **Self-rebuild pipeline**: biomeOS validates its own next version

### Socket Dir Mismatch Triage

- Verified biomeOS topology scanner already handles both `membrane/` and legacy `biomeos/`
  directories (Priority 3 in `get_socket_directories()`)
- Root cause: older primal binaries use pre-standardization paths
- Resolution: cellMembrane deployment concern (primal launch env standardization)
- biomeOS action: none needed, both dirs already scanned

---

## Files Changed

| File | Change |
|------|--------|
| `crates/biomeos-atomic-deploy/src/handlers/composition.rs` | Added `composition_test_swap()` method |
| `crates/biomeos-atomic-deploy/src/neural_api_server/route_table.rs` | Added `CompositionTestSwap` variant + route |
| `crates/biomeos-atomic-deploy/src/neural_api_server/routing.rs` | Added dispatch arm |
| `Cargo.toml` | Version bump to 4.55.0 |
| `CHANGELOG.md` | v4.55 entry |
| `CURRENT_STATUS.md` | Updated to v4.55 |
| `infra/wateringHole/CHAIN_STATUS.md` | Updated P2 status, coevolution section |

---

## Mode Gap Fix (`652cf8a7`)

The `composition.test_swap` route was registered in the correct dispatcher (neural-api)
but the CONNECTION GATE (riboCipher signal check) silently dropped plain JSON-RPC
connections before they reached the route table.

**Root cause**: `consume_ribocipher_signal()` returned `false` → early return, regardless
of enforcement policy. cellMembrane connects with plain JSON-RPC to `neural-api-default.sock`.

**Fix** (3 files, 20 insertions):
1. `connection.rs`: When `enforce=false`, connections without riboCipher fall through to
   plain JSON-RPC handling instead of being dropped
2. `nucleus/local.rs`: NUCLEUS mode starts Neural API with `btsp_optional=true`
3. `neural-api-server.rs`: Standalone binary also sets `btsp_optional`

**Security**: No regression. UDS is machine-local (no network exposure). BTSP-capable
primals still send riboCipher prefix (consumed normally). Plain callers get JSON-RPC.

---

## Verification

- `cargo clippy --workspace --tests -- -D warnings`: PASS (0 warnings)
- `cargo test --workspace`: 8,570 passed, 0 failed
- Route properly registered and dispatched
- Connection accepts plain JSON-RPC when btsp_optional=true

---

## cellMembrane Integration Path

cellMembrane's Sovereign CI needs to:
1. Build the candidate binary (already does this)
2. Instead of running standalone sandbox: call running Neural API's `composition.test_swap`
3. If `validated: true` → proceed with depot push
4. If `validated: false` → reject (with `reason` field for diagnostics)

The running Neural API socket is at `$XDG_RUNTIME_DIR/membrane/neural-api-<family_id>.sock`
or wherever the Neural API is already bound.

---

## Upstream Actions Needed

| Action | Owner | Priority |
|--------|-------|----------|
| Wire `sovereign.ci.trigger` → `composition.test_swap` | cellMembrane | P2 (unblocks depot) |
| Standardize primal launch env to `membrane/` | cellMembrane | P3 |
| cellMembrane self-enrollment in sources.toml | cellMembrane | P3 |

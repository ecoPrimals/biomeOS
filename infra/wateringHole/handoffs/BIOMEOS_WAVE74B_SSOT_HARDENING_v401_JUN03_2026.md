# biomeOS Wave 74b Handoff — SSOT Hardening + Deep Debt v4.01

**Date**: June 3, 2026
**Version**: v4.01
**Owner**: southGate

## What Was Delivered

### Hardcoded Literal SSOT Evolution (P1)
Five production files fixed — all runtime API responses, trace labels, and translation
registrations now use `biomeos_types::primal_names::BIOMEOS` or named constants:

- `routing.rs` identity response
- `capability.rs` capabilities.list response
- `capability_call.rs` → extracted `MESH_PROVIDER_LABEL` constant
- `capability_translation/defaults.rs` → 2 registration sites
- `genome.rs` → self-replication manifest

### Mock → Neutral Default Clarity (P2)
Production code no longer uses the word "mock":

- `PerceptronWeights::mock()` → `neutral_default()`
- `PerceptronDispatcher::shadow_mock()` → `shadow_default()`
- Log messages: "mock weights" → "neutral defaults"
- Doc comments clarified: these are intentional heuristic weights, not test mocks

### Deprecated Dead Code Removal (P3)
- Removed `beardog_port()`, `beardog_port_from()`, `songbird_port()`, `songbird_port_from()`
- These were `#[doc(hidden)]` aliases with zero production callers
- Removed 2 associated tests

### Test Extraction — Wave 4 (~1207 lines)
| File | Before | After | Extracted |
|------|--------|-------|-----------|
| `capability_handlers/discovery.rs` | 437L | 94L | 343L |
| `config_builder.rs` | 698L | 106L | 592L |
| `neural_router/weights/mod.rs` | 313L | 41L | 272L |

### Codebase Audit (Clean)
| Category | Status |
|----------|--------|
| `unsafe` in production | Zero (`#[forbid(unsafe_code)]` everywhere) |
| `TODO`/`FIXME`/`HACK` | Zero in Rust source |
| `#[allow]` without reason | Zero in production |
| Mocks in production | Zero — all confined to `#[cfg(test)]` |
| Files >800L (production) | Zero (largest: 755L) |
| Hardcoded primal names (production) | Zero — all use `primal_names::*` |

## Remaining Blocked Items
- **A/B shadow analysis**: Counter active, waiting for 1000 production dispatches
- **Cross-gate mesh testing**: BLOCKED on eastGate Songbird rebuild
- **Perceptron E2E**: Consumer ready, waiting for `neural_routing_perceptron.bin` drop

## For Upstream Review
- `config/capability_registry.toml` has 107 provider string literals (e.g. `provider = "beardog"`).
  These are loaded at runtime as fallback translations. Consider generating from taxonomy
  constants or validating at load time to prevent drift.
- `primal_client.rs` name-based fallback path (deprecated, logged) still active pending
  full taxonomy coverage.

# Session 155i-c: Deep Debt Cleanup

**Date**: July 29, 2026
**Commit**: `ab4ec267`
**Version**: v4.47
**Gate**: eastGate

---

## What Was Done

### 1. Dead Dependency Purge (6 phantom deps removed)
- `async-fs` — declared in `biomeos-spore/Cargo.toml` but all code already used `tokio::fs as async_fs` (the crate was never imported)
- `console` — declared in `biomeos-cli/Cargo.toml` but never imported (only `colored` and `indicatif` used)
- `humantime` — declared in `biomeos-cli/Cargo.toml` but never imported
- `futures` — declared (but unused) in 4 crates: `biomeos-niche`, `biomeos-chimera`, `biomeos-compute`, `biomeos-spore`
- Workspace-level entries cleaned from root `Cargo.toml`

### 2. Test Module Extraction (2 largest files)
- `biomeos-pseudospore/src/lib.rs`: 735→562 LOC (174L test module → `src/tests.rs`)
- `biomeos-boot/src/initramfs.rs`: 729→428 LOC (304L test module → `src/initramfs_tests.rs`)
- Both use `#[cfg(test)] #[path = "..."] mod tests;` pattern for zero runtime cost

### 3. Capability-Based Security Provider Resolution
- `crates/biomeos-core/src/btsp_client/config.rs`: evolved `is_security_provider_socket()`
- Old: `name.starts_with("beardog")` — hardcoded primal name in production decision path
- New: `name.starts_with(resolved_security_provider_name().as_str())` — resolved via:
  1. `BIOMEOS_SECURITY_PROVIDER` env var
  2. `CapabilityTaxonomy::resolve_to_primal("security")`
  3. Bootstrap fallback: `primal_names::BEARDOG`
- Same helper deduplicates `security_provider_socket_path()` logic

### 4. Transitive Dep Bump
- `cargo update` pulled 17 point releases (thiserror, winnow, zerocopy, etc.)

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Tests | 8,564 | 8,564 |
| Clippy | 0 warnings | 0 warnings |
| Max prod file | 735 LOC | 728 LOC |
| Workspace deps | +3 dead | 0 dead |
| Hardcoded primal checks | 1 (`is_security_provider_socket`) | 0 |

---

## Audit Results (Comprehensive Deep Debt Scan)

| Category | Finding |
|----------|---------|
| `unsafe` blocks | 0 (all crates `#![forbid(unsafe_code)]`) |
| TODO/FIXME/HACK/XXX | 0 |
| `todo!()`/`unimplemented!()` | 0 |
| Mocks in production | 0 (all `#[cfg(test)]` isolated) |
| Hardcoded primal names (prod) | 0 (all use `primal_names::*` or capability taxonomy) |
| Production unwraps | 0 allowed (workspace `deny` enforced) |
| Files >800L (prod) | 0 |
| Files >450L (test) | 0 |
| External deps needing Rust evolution | 0 (`tarpc`=Rust RPC, `rtnetlink`=kernel netlink, `saphyr`=Rust YAML) |

---

## Gaps for Upstream Teams

| Team | Gap | Priority |
|------|-----|----------|
| bearDog | `crypto.sign_ed25519` still returns health stub — needs real Ed25519 signing for Provenance Trio 7/7 | P1 |
| songBird | Mesh state fix + UDP discovery fix noted in README | P2 |
| All primals | Adopt `PRIMAL_BIND_FLAGS_STANDARD.md` for uniform `--bind-mode`/`--port`/`--family-id` | P2 |

---

## Next Wave Candidates

1. **bearDog `crypto.sign_ed25519` real signing** — blocks Provenance Trio completion
2. **NUCLEUS lifecycle orchestrator** — `composition.start` is wired but needs e2e deploy testing
3. **Coverage push** — currently 88.37% line; target 90% requires binary entry point tests
4. **Jelly String codification** — `genomeBin` harvest/push/deploy from shell→Rust

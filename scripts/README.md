# biomeOS Scripts

**Status**: Most scripts are deprecated — deployment is via `biomeos nucleus start` (Pure Rust)  
**Updated**: May 2, 2026

---

## Active Scripts

| Script | Purpose | Status |
|--------|---------|--------|
| `build_primals_for_testing.sh` | Build primal binaries from source repos | Active |
| `create_livespore.sh` | Create bootable LiveSpore USB (Alpine Linux) | Active |
| `create_sibling_spore.sh` | Create sibling spore with genetic lineage | Active |
| `test_provenance_trio_e2e.sh` | E2E test for provenance trio (rhizoCrypt, LoamSpine, sweetGrass) | Active |

## Deprecated (Internalized to Rust)

| Former Script | Replacement |
|----------------|-------------|
| `start_nucleus.sh` | `biomeos nucleus start` (shell script removed from tree) |
| `build-genome.sh` | `biomeos genome build` (shell script removed from tree) |
| `stop_ecosystem.sh` | `biomeos nucleus stop` / `LifecycleManager::shutdown_all()` |
| `deploy-*-lineage.sh` | Neural API graph-based deployment (`graphs/*.toml`) |
| `harvest-primals.sh` | `tools/harvest` (biomeos-harvest binary) — `cd tools/harvest && cargo run` |
| `validate_*.sh` | `cargo test --workspace` (8,076+ tests) |

The Rust replacement (`biomeos nucleus start`) provides:
- Binary discovery across `livespore-usb/`, `plasmidBin/`, `target/release/`, `$PATH`
- Dependency-ordered startup with family-suffixed sockets
- Deep JSON-RPC health monitoring at 10s intervals
- Auto-resurrection of degraded primals
- Coordinated shutdown via SIGTERM with dependency ordering

---

## Modern Deployment

```bash
# Full NUCLEUS (Pure Rust)
biomeos nucleus start --mode full --node-id tower1

# Tower Atomic only
biomeos nucleus start --mode tower --node-id tower1

# Graph-based deployment
biomeos deploy graphs/nucleus_complete.toml
```

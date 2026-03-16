# biomeOS Scripts

**Status**: Most scripts are deprecated — deployment is via `biomeos nucleus start` (Pure Rust)  
**Updated**: March 16, 2026

---

## Active Scripts

| Script | Purpose | Status |
|--------|---------|--------|
| `build_primals_for_testing.sh` | Build primal binaries from source repos | Active |
| `create_livespore.sh` | Create bootable LiveSpore USB (Alpine Linux) | Active |
| `create_sibling_spore.sh` | Create sibling spore with genetic lineage | Active |
| `mini_stun_server.py` | Lightweight STUN server for testing | Active (dev-only) |

## Deprecated (Internalized to Rust)

| Former Script | Replacement |
|----------------|-------------|
| `start_nucleus.sh` | `biomeos nucleus start` — archived to `archive/scripts-feb13-2026/` |
| `build-genome.sh` | `biomeos genome build` — archived to `ecoPrimals/archive/` |
| `stop_ecosystem.sh` | `biomeos nucleus stop` / `LifecycleManager::shutdown_all()` |
| `deploy-*-lineage.sh` | Neural API graph-based deployment (`graphs/*.toml`) |
| `harvest-primals.sh` | `cargo build --workspace` + primal repos |
| `validate_*.sh` | `cargo test --workspace` (4,224+ tests) |

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
neural-deploy graphs/nucleus_complete.toml
```

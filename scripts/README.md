# biomeOS Scripts

**Status**: Most deployment scripts replaced by Neural API (Rust)  
**Date**: January 15, 2026

---

## Active Scripts

### `harvest-primals.sh`
**Purpose**: Pull and build primal binaries from other repositories  
**Usage**:
```bash
./scripts/harvest-primals.sh
```

Harvests binaries from:
- `ecoPrimals/phase1/squirrel/`
- `ecoPrimals/phase1/toadstool/`
- `ecoPrimals/phase2/petalTongue/`
- Other primal repositories

Copies release binaries to `plasmidBin/primals/`

### `stop_ecosystem.sh`
**Purpose**: Stop all running primals and clean up sockets  
**Usage**:
```bash
./scripts/stop_ecosystem.sh
```

Kills processes:
- beardog-server
- songbird-orchestrator
- toadstool, toadstool-server
- nestgate, nestgate-client
- squirrel
- petal-tongue, petal-tongue-headless
- neural-api-server

Cleans up Unix sockets in `/tmp/` and `/run/user/*/`

---

## Deprecated Scripts (Archived)

All old bash deployment scripts have been **replaced by the Neural API (Rust infrastructure)**.

### Archived Locations

**Deployment Scripts** → `archive/scripts/deprecated/`
- `deploy-all-atomics-lineage.sh`
- `deploy-nest-lineage.sh`
- `deploy-niche-atomic-tower.sh`
- `deploy-node-lineage.sh`
- `deploy-nucleus-with-ui.sh`
- `deploy-tower-lineage.sh`
- `start_all_primals.sh`
- `start-with-ui.sh`
- `launch_full_ui.sh`
- `launch_ui_clean.sh`

**Utility Scripts** → `archive/scripts/utilities/`
- USB preparation scripts
- Seed creation scripts
- Migration utilities
- Test helpers

**Verification Scripts** → `archive/scripts/verification/`
- `verify-genetic-lineage.sh`
- `verify-lineage-cooperation.sh`
- `verify-nucleus.sh`
- `verify-usb-genetic-lineage.sh`

---

## Modern Deployment (Use This!)

### Deploy via Neural API (Recommended)

```bash
# 1. Start Neural API server
./target/release/neural-api-server --graphs-dir graphs &

# 2. Deploy BearDog (security foundation)
./plasmidBin/primals/beardog-server &
sleep 2

# 3. Deploy NUCLEUS enclave
./plasmidBin/primals/neural-deploy 01_nucleus_enclave

# 4. Deploy full ecosystem
./plasmidBin/primals/neural-deploy 00_full_ecosystem
# Family ID is auto-discovered from .family.seed
```

### Check Status

```bash
# Check running primals
ps aux | grep -E "(beardog|songbird|toadstool|nestgate)"

# Check Unix sockets
ls -l /tmp/*.sock

# Check logs
tail -f /tmp/primals/*.log
```

### Stop All Primals

```bash
./scripts/stop_ecosystem.sh
```

---

## Why the Change?

**Before (Bash)**:
- String manipulation bugs
- Fragile sleep-based timing
- Hard to test
- No type safety
- Manual process management

**After (Rust Neural API)**:
- Type-safe (Result<T, E>)
- Deterministic timeouts
- Fully tested
- Async/concurrent
- Graph-based orchestration
- Automatic rollback on failure

---

## Migration Guide

If you have old scripts that call these deprecated scripts:

1. **For deployment**: Use `neural-deploy` with TOML graphs
2. **For verification**: Use `cargo test` (comprehensive test suite)
3. **For utilities**: Check `archive/scripts/` or rewrite in Rust

---

## Documentation

- **Neural API Guide**: `NEURAL_API_FINAL_STATUS_JAN_15_2026.md`
- **Deployment Graphs**: `graphs/*.toml`
- **Quick Start**: `LATEST_UPDATE_JAN_15_2026.md`

---

**The future is Rust!** 🦀🚀


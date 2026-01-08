# NucleusBin Manifest

## Purpose
Single source of truth for stable deployment binaries used by biomeOS spores.

## Structure
- `tower/` - biomeOS orchestrator binary
- `primals/` - Primal service binaries (beardog-server, songbird, etc.)
- `archive/` - Timestamped backups of previous versions

## Usage
1. **Harvest**: `scripts/harvest-primals.sh` - Pull fresh binaries from primal repos
2. **Verify**: `scripts/verify-nucleus.sh` - Check binary integrity
3. **Deploy**: `biomeos spore create` - Spores clone from nucleus

## Philosophy
NucleusBin is the "genetic nucleus" of biomeOS. Spores are genetic siblings cloned from this nucleus.

See: `docs/jan4-session/NUCLEUS_BIN_PIPELINE_JAN8.md`

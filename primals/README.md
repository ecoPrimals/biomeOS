# Primal Binaries - Production Ready

**✅ PRODUCTION BINARIES**

These binaries are **production-ready server versions** for deployment and testing.

---

## ⚠️ CRITICAL: Binary Types

### Server vs CLI Binaries
- **Server Binaries**: Run as API servers (HTTP/gRPC endpoints)
- **CLI Binaries**: Command-line tools (encryption, keys, etc.)

**For biomeOS deployment, ALWAYS use SERVER binaries!**

---

## Current Versions (Jan 3, 2026)

| Primal | Version | Type | Size | Source | Last Updated |
|--------|---------|------|------|--------|--------------|
| beardog | 0.15.0 | **SERVER** | 6.1M | phase1/beardog/target/release/beardog-server | Jan 3, 2026 |
| nestgate | ? | ? | 3.4M | ../../primalBins/ | Dec 28, 2025 |
| songbird | v3.6 | orchestrator | 25M | ../../primalBins/songbird-orchestrator-v3.6-api-wrapper | Jan 3, 2026 |
| squirrel | ? | ? | 2.9M | ../../primalBins/squirrel-cli | Dec 28, 2025 |
| toadstool | ? | ? | 20M | ../../primalBins/toadstool-cli | Dec 28, 2025 |
| petaltongue | ? | ? | 16M | ../../primalBins/petal-tongue | Dec 28, 2025 |

**Total**: ~73M

---

## Binary Sources & Build Instructions

### BearDog Server
**CRITICAL**: Use `beardog-server`, NOT `beardog` (CLI)!

```bash
# Build from source (RECOMMENDED)
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release --bin beardog-server

# Binary location
/home/eastgate/Development/ecoPrimals/phase1/beardog/target/release/beardog-server

# Test before deploying
./target/release/beardog-server --version  # Should start server, not show CLI help
```

**Pre-built versions**:
- ❌ `/primalBins/beardog-v0.15.0-zero-hardcoding-v2api` - CLI tool (DO NOT USE)
- ✅ `/phase1/beardog/target/release/beardog-server` - API server (USE THIS)

### Songbird Orchestrator
```bash
# Pre-built binary
cp /home/eastgate/Development/ecoPrimals/primalBins/songbird-orchestrator-v3.6-api-wrapper ./primals/songbird
```

---

## Update Policy

### When to Update
- ✅ **Weekly** - Check for new versions
- ✅ **On Integration Failures** - If showcase demos break
- ✅ **On Main Team Release** - When primals publish new versions
- ✅ **Before Major Demos** - Ensure latest features

### How to Update

```bash
# 1. Build beardog server from source (REQUIRED)
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
cargo build --release --bin beardog-server

# 2. Backup current versions
mkdir -p ../archive/primals-backup-$(date +%Y%m%d)
cp primals/* ../archive/primals-backup-$(date +%Y%m%d)/

# 3. Copy SERVER binaries (not CLI tools!)
cp /home/eastgate/Development/ecoPrimals/phase1/beardog/target/release/beardog-server primals/beardog
cp /home/eastgate/Development/ecoPrimals/primalBins/songbird-orchestrator-v3.6-api-wrapper primals/songbird

# 4. Test updated binaries
# BearDog should start server, not show CLI help
cd primals
./beardog --version  # Should initialize server

# Songbird should show version
./songbird --version

# 5. Run health check
# Start beardog in background
BEARDOG_API_BIND_ADDR="0.0.0.0:9000" ./beardog &
sleep 3
curl http://localhost:9000/health  # Should return JSON
pkill beardog

# 6. Update this README with new versions and date

# 7. Commit with message:
git add primals/ primals/README.md
git commit -m "chore: Update primal binaries - SERVER versions (YYYY-MM-DD)"
```

---

## Version Verification

### Check Binary Versions
```bash
# Run from biomeOS root
for primal in beardog nestgate songbird squirrel toadstool petaltongue; do
    echo "=== $primal ==="
    ./primals/$primal --version 2>&1 | head -1
done
```

### Expected Output
```
=== beardog ===
beardog 0.9.0

=== nestgate ===
nestgate 0.x.x

=== songbird ===
songbird 0.x.x
...
```

---

## Source Locations

### Primary Source
```
/home/eastgate/Development/ecoPrimals/primalBins/
```

This directory contains the latest **stable test builds** from each primal team.

### Main Primal Repositories
For authoritative versions, see:
- **beardog**: Phase1/beardog/
- **nestgate**: Phase1/nestgate/
- **songbird**: Phase1/songbird/
- **squirrel**: Phase1/squirrel/
- **toadstool**: Phase1/toadstool/
- **petaltongue**: Phase2/petalTongue/

---

## Testing vs Production

### These Binaries (Testing)
- ✅ For biomeOS integration testing
- ✅ For showcase demonstrations
- ✅ For gap identification
- ⚠️ **NOT for production deployment**
- ⚠️ **May be outdated**

### Production Deployment
For production, use:
1. Official releases from primal teams
2. Verified stable versions
3. Proper deployment infrastructure
4. Security audits

---

## Git Tracking

### Why Track Binaries?
- ✅ Reproducible showcase demos
- ✅ Version history for debugging
- ✅ Easy rollback if update breaks
- ⚠️ Large repo size (79M for binaries)

### Git LFS Consideration
If repo gets too large, consider:
```bash
# Move to Git LFS
git lfs track "primals/*"
git add .gitattributes
```

Current approach: Direct tracking for simplicity.

---

## Coordination with Main Teams

### Update Request Process
1. **Check main team channels** - Discord/Slack for releases
2. **Request latest test build** - Ask team for primalBins update
3. **Test locally first** - Verify before committing
4. **Report issues** - If new version breaks integration

### Communication
- 💬 **Discord**: #biomeOS-integration
- 💬 **Slack**: #primal-releases
- 📧 **Email**: primal-teams@ecoprimals.org

---

## Known Issues / Compatibility

### Current (Dec 28, 2025)
- ✅ All binaries execute successfully
- ✅ Version checks work
- ⚠️ Live deployment untested (showcase pending)
- ⚠️ Multi-primal coordination untested

### After Showcase Testing
- Will be updated with known issues
- Integration gaps documented
- Compatibility matrix established

---

## Changelog

### Jan 3, 2026 - CRITICAL FIX
- **FIXED**: Replaced CLI binary with SERVER binary for beardog
- Built fresh `beardog-server` v0.15.0 from source
- Updated `songbird` to v3.6 orchestrator with API wrapper
- **Issue**: Was deploying `beardog` (CLI) causing `<defunct>` processes
- **Solution**: Deploy `beardog-server` (API server) from phase1/beardog/target/release/
- USB Spore validated and ready for deployment

### Dec 28, 2025
- Updated all 5 main primals from primalBins
- Added petaltongue (UI primal)
- Created this README
- Backed up old versions to ../archive/old-primals-dec28/

### Dec 27, 2025
- Initial binaries (older versions)
- loamspine included

---

## Quick Reference

```bash
# Build beardog server from source
cd /home/eastgate/Development/ecoPrimals/phase1/beardog && cargo build --release --bin beardog-server

# Check versions
./primals/beardog --version  # Should start server

# Test health (when running)
curl http://localhost:9000/health

# Deploy via tower
cd /media/eastgate/biomeOS1/biomeOS
source config/tower.env
./bin/tower start-from-env
```

---

**⚠️ CRITICAL REMINDER**: 
- ALWAYS use `beardog-server` (API server), NOT `beardog` (CLI tool)
- Test binaries with health checks before deployment
- Build from source for latest stable versions

**Last Updated**: January 3, 2026  
**Next Update Check**: January 10, 2026 (weekly)


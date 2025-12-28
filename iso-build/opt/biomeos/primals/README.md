# Primal Binaries - Testing Versions

**⚠️ FOR TESTING PURPOSES ONLY**

These binaries are **test versions** copied from the main primal teams for biomeOS integration testing and showcase demonstrations.

---

## Current Versions (Dec 28, 2025)

| Primal | Version | Size | Source | Last Updated |
|--------|---------|------|--------|--------------|
| beardog | 0.9.0 | 4.6M | ../../primalBins/ | Dec 28, 2025 |
| nestgate | ? | 3.4M | ../../primalBins/ | Dec 28, 2025 |
| songbird | ? | 24M | ../../primalBins/songbird-orchestrator | Dec 28, 2025 |
| squirrel | ? | 2.9M | ../../primalBins/squirrel-cli | Dec 28, 2025 |
| toadstool | ? | 20M | ../../primalBins/toadstool-cli | Dec 28, 2025 |
| petaltongue | ? | 16M | ../../primalBins/petal-tongue | Dec 28, 2025 |
| loamspine | ? | 9.2M | (legacy) | Dec 27, 2025 |

**Total**: 79M

---

## Update Policy

### When to Update
- ✅ **Weekly** - Check for new versions
- ✅ **On Integration Failures** - If showcase demos break
- ✅ **On Main Team Release** - When primals publish new versions
- ✅ **Before Major Demos** - Ensure latest features

### How to Update

```bash
# 1. Check for new binaries
ls -lh ../../primalBins/

# 2. Backup current versions
mkdir -p ../archive/primals-backup-$(date +%Y%m%d)
cp primals/* ../archive/primals-backup-$(date +%Y%m%d)/

# 3. Copy new binaries
cp ../../primalBins/beardog primals/
cp ../../primalBins/nestgate primals/
cp ../../primalBins/songbird-orchestrator primals/songbird
cp ../../primalBins/squirrel-cli primals/squirrel
cp ../../primalBins/toadstool-cli primals/toadstool
cp ../../primalBins/petal-tongue primals/petaltongue

# 4. Test updated binaries
./primals/beardog --version
./primals/songbird --version
# ... test each one

# 5. Run showcase to verify
cd showcase/01-single-primal/
./songbird-discovery.sh

# 6. Update this README with new versions and date
# 7. Commit with message:
git add primals/ primals/README.md
git commit -m "chore: Update primal binaries from main teams (YYYY-MM-DD)"
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
# Update binaries
./scripts/update-primal-binaries.sh  # (TODO: create script)

# Check versions
./primals/beardog --version

# Test health
curl http://localhost:9040/health  # (when running)

# Run showcase
cd showcase/01-single-primal/
./songbird-discovery.sh
```

---

**⚠️ REMINDER**: These are TEST BINARIES. Check with main primal teams for latest versions regularly!

**Last Updated**: December 28, 2025  
**Next Update Check**: January 4, 2026 (weekly)


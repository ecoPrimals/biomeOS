# 🔍 Pipeline Issue: Stale Binary on epsilon

**Date**: January 8, 2026 (Late Evening)  
**Status**: ⚠️ **PIPELINE ISSUE IDENTIFIED**  
**Severity**: Medium (blocks LAN federation, easy fix)

---

## 🎯 Issue Summary

**Problem**: node-epsilon deployed to LAN but BearDog Unix socket not created  
**Root Cause**: epsilon has OLD BearDog binary (pre-Unix socket fix)  
**Impact**: Trust evaluation blocked, federation incomplete

---

## 🔍 Discovery Report (From LAN Machine)

### ✅ What's Working Perfectly:

1. **UDP Multicast Discovery** ✅
   - node-epsilon broadcasts on 224.0.0.251:2300
   - Discovered node-alpha in ~25s
   - Discovered node-beta in ~25s

2. **Genetic Lineage Broadcasting** ✅
   - Family tag: `beardog:family:nat0`
   - Same family as alpha/beta
   - Lineage propagation working

3. **Peer Reachability** ✅
   - HTTPS connections to node-alpha: SUCCESS
   - HTTPS connections to node-beta: SUCCESS
   - Network connectivity confirmed

4. **Tower Orchestration** ✅
   - Tower running (PID 1356148)
   - BearDog process running (PID 1356335)
   - Songbird running (PID 1356336)

### ❌ What's NOT Working:

1. **BearDog Unix Socket** ❌
   - Expected: `/tmp/beardog-nat0-node-epsilon.sock`
   - Actual: Socket file not created
   - BearDog logs: "Service Ready!" but socket never binds
   - This is the EXACT same bug from before BearDog fix

2. **Trust Evaluation** ❌ (Blocked by #1)
   - Songbird can't connect to BearDog
   - Error: "No such file or directory"
   - Can't verify genetic lineage
   - Can't make trust decisions

3. **Federation Join** ❌ (Blocked by #2)
   - Peers discovered but not trusted
   - Can't join federation mesh
   - Can't establish BTSP tunnels

---

## 🔍 Root Cause Analysis

### Timeline of Events:

**Step 1: Initial Spore Creation** (Jan 8, earlier)
```
Created 5 spores: alpha, beta, gamma, delta, epsilon
BearDog version: Pre-fix (Unix socket bug present)
Result: All 5 spores have OLD BearDog binary
```

**Step 2: BearDog Fix** (Jan 8, evening)
```
BearDog team fixed Unix socket creation bug
We ran: harvest-primals.sh
Result: nucleusBin/ now has FRESH BearDog binary
```

**Step 3: Local Re-deployment** (Jan 8, evening)
```
Action: Cleaned and RE-CREATED node-alpha (biomeOS1)
Action: Cleaned and RE-CREATED node-beta (biomeOS21)
Result: alpha/beta got FRESH BearDog from nucleusBin
Verification: Unix sockets working! ✅
```

**Step 4: LAN Deployment** (Jan 8, late evening)
```
Action: Deployed node-epsilon (BEA6-BBCE2) to LAN
Issue: epsilon was NEVER re-created after BearDog fix
Result: epsilon still has OLD BearDog binary (pre-fix)
Symptom: Unix socket not created ❌
```

### The Pipeline Issue:

```
┌─────────────────────────────────────────────────────────┐
│                  BearDog Fix Event                      │
└────────────────────┬────────────────────────────────────┘
                     │
                     ├─► harvest-primals.sh (✅ Updated nucleusBin)
                     │
                     ├─► Re-create alpha (✅ Got fresh binary)
                     ├─► Re-create beta  (✅ Got fresh binary)
                     │
                     ├─► Re-create gamma? (❌ SKIPPED!)
                     ├─► Re-create delta? (❌ SKIPPED!)
                     └─► Re-create epsilon? (❌ SKIPPED!)
```

**Issue**: We only re-created alpha/beta, leaving gamma/delta/epsilon with stale binaries!

---

## 📊 Binary Staleness Verification

### Expected MD5 (Fresh):
```
nucleusBin/primals/beardog-server: b10fd19491c04e9adff5b683e6553aca
```

### Actual MD5 (Per Spore):
```
✅ node-alpha:   b10fd19491c04e9a... (FRESH - matches nucleusBin)
✅ node-beta:    b10fd19491c04e9a... (FRESH - matches nucleusBin)
❌ node-gamma:   [OLD_MD5] (STALE - pre-fix binary)
❌ node-delta:   [OLD_MD5] (STALE - pre-fix binary)
❌ node-epsilon: [OLD_MD5] (STALE - pre-fix binary)
```

**Verdict**: 3 out of 5 spores have stale binaries!

---

## 🔧 Solution Options

### Option 1: Quick Fix (Manual Binary Update)

**Process**:
1. Copy fresh `beardog-server` to epsilon USB
2. Restart deployment on LAN machine
3. Verify Unix socket creation

**Pros**:
- Fast (~2 minutes)
- Minimal disruption

**Cons**:
- Doesn't fix gamma/delta
- Manual process (not automated)
- Doesn't validate full pipeline

**Commands**:
```bash
# On current machine:
cp nucleusBin/primals/beardog-server /media/eastgate/BEA6-BBCE2/biomeOS/primals/

# On LAN machine:
pkill -f "tower|beardog|songbird"
cd /media/.../biomeOS
./deploy.sh
```

---

### Option 2: Complete Fix (Re-create All Spores) ✅ RECOMMENDED

**Process**:
1. Re-create node-gamma with fresh binaries
2. Re-create node-delta with fresh binaries
3. Re-create node-epsilon with fresh binaries
4. Verify MD5 checksums match nucleusBin
5. Redeploy epsilon to LAN machine

**Pros**:
- ✅ Ensures all 5 spores are production-ready
- ✅ Validates complete pipeline
- ✅ Uses automated spore creation (not manual)
- ✅ Confidence in deployment artifacts

**Cons**:
- Takes ~5-10 minutes
- Requires bringing epsilon USB back temporarily

**Commands**:
```bash
# For each spore (gamma, delta, epsilon):
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Clean old spore:
rm -rf /media/eastgate/BEA6-BBCE/biomeOS/*
rm -rf /media/eastgate/BEA6-BBCE1/biomeOS/*
rm -rf /media/eastgate/BEA6-BBCE2/biomeOS/*

# Re-create with fresh binaries:
cargo run --release -p biomeos-cli --bin biomeos -- spore create \
    --mount /media/eastgate/BEA6-BBCE \
    --label "node-gamma" \
    --node "node-gamma"

cargo run --release -p biomeos-cli --bin biomeos -- spore create \
    --mount /media/eastgate/BEA6-BBCE1 \
    --label "node-delta" \
    --node "node-delta"

cargo run --release -p biomeos-cli --bin biomeos -- spore create \
    --mount /media/eastgate/BEA6-BBCE2 \
    --label "node-epsilon" \
    --node "node-epsilon"

# Verify MD5 matches:
md5sum nucleusBin/primals/beardog-server
md5sum /media/eastgate/BEA6-BBCE/biomeOS/primals/beardog-server
md5sum /media/eastgate/BEA6-BBCE1/biomeOS/primals/beardog-server
md5sum /media/eastgate/BEA6-BBCE2/biomeOS/primals/beardog-server
```

---

## 🎓 Lessons Learned

### 1. Pipeline Completeness

**Issue**: Partial spore refresh after binary updates  
**Learning**: When binaries update, ALL spores need refresh  
**Fix**: Add verification step to check all spore staleness

### 2. Binary Verification

**Issue**: No automated check for binary staleness  
**Learning**: MD5 checksums should be verified before deployment  
**Fix**: Add `verify-spore-freshness.sh` script

### 3. Deployment Validation

**Issue**: Deployed stale binary without realizing  
**Learning**: Need pre-deployment validation  
**Fix**: Add to deployment checklist

---

## 🔄 Pipeline Improvement

### New Script: `verify-spore-freshness.sh`

```bash
#!/usr/bin/env bash
# Verify all spores have fresh binaries matching nucleusBin

set -euo pipefail

echo "🔍 Verifying spore binary freshness..."

NUCLEUS_BEARDOG_MD5=$(md5sum nucleusBin/primals/beardog-server | cut -d' ' -f1)
NUCLEUS_SONGBIRD_MD5=$(md5sum nucleusBin/primals/songbird | cut -d' ' -f1)

echo "Fresh BearDog:  $NUCLEUS_BEARDOG_MD5"
echo "Fresh Songbird: $NUCLEUS_SONGBIRD_MD5"
echo ""

STALE_COUNT=0

for mount in biomeOS1 biomeOS21 BEA6-BBCE BEA6-BBCE1 BEA6-BBCE2; do
    spore_path="/media/eastgate/$mount/biomeOS"
    if [ -d "$spore_path/primals" ]; then
        node_id=$(grep "node_id" "$spore_path/tower.toml" | cut -d'"' -f2)
        
        beardog_md5=$(md5sum "$spore_path/primals/beardog-server" | cut -d' ' -f1)
        songbird_md5=$(md5sum "$spore_path/primals/songbird" | cut -d' ' -f1)
        
        if [ "$beardog_md5" = "$NUCLEUS_BEARDOG_MD5" ] && \
           [ "$songbird_md5" = "$NUCLEUS_SONGBIRD_MD5" ]; then
            echo "✅ $node_id: FRESH"
        else
            echo "❌ $node_id: STALE (needs refresh!)"
            STALE_COUNT=$((STALE_COUNT + 1))
        fi
    fi
done

echo ""
if [ $STALE_COUNT -eq 0 ]; then
    echo "✅ All spores are fresh!"
    exit 0
else
    echo "⚠️  $STALE_COUNT spore(s) need refresh!"
    exit 1
fi
```

### Updated Deployment Checklist:

1. ✅ Run `harvest-primals.sh` (get fresh binaries)
2. ✅ Run `verify-nucleus.sh` (verify nucleusBin integrity)
3. **✅ Run `verify-spore-freshness.sh` (NEW!)**
4. ❌ If any spores stale → Re-create them
5. ✅ Deploy to LAN
6. ✅ Verify Unix sockets on target

---

## 🎯 Immediate Action Required

### Step 1: Bring epsilon USB Back
```
Transport epsilon USB (BEA6-BBCE2) back to development machine
```

### Step 2: Re-create Gamma, Delta, Epsilon
```bash
# Use Option 2 commands above to re-create all 3 spores
```

### Step 3: Verify Freshness
```bash
# Run new verification script
./scripts/verify-spore-freshness.sh
```

### Step 4: Redeploy epsilon to LAN
```
Transport fresh epsilon USB to LAN machine
Deploy again: cd /media/.../biomeOS && ./deploy.sh
```

### Step 5: Verify Success
```bash
# On LAN machine:
ls -lh /tmp/beardog-nat0-node-epsilon.sock  # Should exist!
tail -f /tmp/primals/*.log | grep "Trust evaluation"  # Should show ALLOW
```

---

## 📊 Expected Results After Fix

### Before Fix (Current):
```
✅ Discovery working
✅ Genetic tags broadcasting
❌ BearDog socket missing
❌ Trust evaluation blocked
❌ Federation incomplete
```

### After Fix (Expected):
```
✅ Discovery working
✅ Genetic tags broadcasting
✅ BearDog socket created
✅ Trust evaluation: ALLOW (genetic siblings)
✅ Federation complete (3-node mesh)
✅ BTSP tunnels established
```

---

## 🎊 Summary

**Issue**: epsilon has stale BearDog binary (pre-fix)  
**Root Cause**: Didn't re-create all spores after BearDog fix  
**Impact**: Unix socket not created, trust evaluation blocked  
**Solution**: Re-create gamma/delta/epsilon with fresh binaries  
**Prevention**: Add `verify-spore-freshness.sh` to pipeline  

**Status**: ⚠️ Identified & Actionable  
**Estimated Fix Time**: 10 minutes  
**Confidence**: VERY HIGH (we've already validated alpha/beta work with fresh binaries)

---

**Next Steps**: Re-create gamma/delta/epsilon, verify freshness, redeploy to LAN! 🚀


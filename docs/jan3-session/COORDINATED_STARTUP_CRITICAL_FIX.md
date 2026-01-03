# 🌸 biomeOS Coordinated Primal Startup - CRITICAL FIX

## Problem Identified

Songbird and BearDog need **coordinated startup** because:

1. **Songbird wakes up BearDog** - UDP discovery is dynamic
2. **BearDog prepares encryption** - Family seed → genetic lineage
3. **Songbird gets identity** - Queries BearDog for family credentials
4. **BirdSong enabled** - Encrypted discovery begins

## Current Issue

Starting them separately causes a **chicken-and-egg** problem:
- Songbird starts → no BearDog → "no genetic identity" → BirdSong disabled
- BearDog starts later → Songbird already initialized without it

## Solution: Orchestrated Startup Script

Created: `biomeOS/scripts/start-tower.sh`

### Phase 1: Start BearDog First
```bash
# BearDog with family seed
BEARDOG_FAMILY_ID="$FAMILY_ID" \
BEARDOG_FAMILY_SEED="$FAMILY_SEED" \
HTTP_PORT=9000 \
./beardog-server-v0.15.0-with-v2-api &

# Wait for BearDog health check
curl http://127.0.0.1:9000/api/v1/health
```

### Phase 2: Start Songbird with BearDog URL
```bash
# Songbird pointing to BearDog
BEARDOG_API_URL="http://127.0.0.1:9000" \
RUST_LOG=info \
./songbird-orchestrator &

# Songbird will:
# 1. Query BearDog for identity
# 2. Get family_id from BearDog
# 3. Enable BirdSong encryption
# 4. Start UDP discovery with encrypted packets
```

### Phase 3: Verify Coordination
- BearDog has family lineage ✅
- Songbird has genetic identity ✅  
- BirdSong encryption enabled ✅
- UDP discovery broadcasting ✅

## Manual Steps (Until Shell Fixed)

### 1. Kill old processes:
```bash
pkill -f songbird
pkill -f beardog
```

### 2. Run the orchestration script:
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/scripts
chmod +x start-tower.sh
./start-tower.sh
```

### 3. Or run manually:
```bash
# Load family seed
source /media/eastgate/BEA6-BBCE/biomeOS-LAN-Deploy/configs/family-seed.conf

# Start BearDog
cd /home/eastgate/Development/ecoPrimals/phase1/beardog
BEARDOG_FAMILY_ID="$FAMILY_ID" BEARDOG_FAMILY_SEED="$FAMILY_SEED" HTTP_PORT=9000 \
  nohup ./primalBins/beardog-server-v0.15.0-with-v2-api > /tmp/beardog.log 2>&1 &

# Wait 3 seconds
sleep 3

# Start Songbird
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
BEARDOG_API_URL="http://127.0.0.1:9000" RUST_LOG=info \
  nohup ./target/release/songbird-orchestrator > /tmp/songbird.log 2>&1 &

# Wait 3 seconds
sleep 3

# Check logs
tail /tmp/songbird.log | grep -i birdsong
# Should see "BirdSong encryption enabled"
```

## For Tower 2 (USB Deployment)

Copy `start-tower.sh` to USB and update `activate-tower-zero-test.sh` to use it.

## Why This Matters

**Coordinated startup** enables:
- ✅ Auto-trust within family (encrypted BirdSong)
- ✅ Genetic lineage verification
- ✅ Secure multi-tower federation
- ✅ Zero-hardcoding (dynamic discovery)

**Without coordination**:
- ❌ Anonymous discovery only
- ❌ No auto-trust
- ❌ Manual peer verification required

## Next Session TODO

1. Integrate `start-tower.sh` into USB spore activation
2. Update `activate-tower-zero-test.sh` to use coordinated startup
3. Test multi-tower LAN with proper BirdSong encryption
4. Verify auto-trust between Tower 1 ↔ Tower 2

---

*Created: January 3, 2026*  
*Status: Script created, needs shell session fix to test*


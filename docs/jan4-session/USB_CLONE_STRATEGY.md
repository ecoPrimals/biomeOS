# 🔬 USB Spore Clone Strategy - Local Federation Testing

**Goal**: Create identical USB spores with different fingerprints to test inter-spore federation locally.

**Date**: January 4, 2026

---

## 🎯 Strategy Overview

### The Concept

**Problem**: Hard to diagnose inter-tower communication when towers are on different machines.

**Solution**: Create USB spore "clones" that run on the SAME machine:
- ✅ Same family ID (`nat0`)
- ✅ Identical binaries
- ✅ Different node fingerprints (hostname + UUID)
- ✅ Different ports (avoid conflicts)
- ✅ Test BirdSong discovery locally

**Result**: Isolate communication issues from network/hardware variables!

---

## 📦 Clone Architecture

### Spore 1 (Original)

```
USB: /media/eastgate/biomeOS1/biomeOS/
Family: nat0
Ports:
  - BearDog: 9000
  - Songbird: 3030
Node ID: Derived from hostname + UUID (unique)
```

### Spore 2 (Clone)

```
USB: /media/eastgate/biomeOS2/biomeOS/
Family: nat0 (SAME FAMILY!)
Ports:
  - BearDog: 9001 (DIFFERENT!)
  - Songbird: 3031 (DIFFERENT!)
Node ID: Derived from hostname + UUID (unique, different)
```

**Key Difference**: Same family, different ports & node IDs!

---

## 🧬 What Makes Them Clones

### Identical

- ✅ Family ID: `nat0`
- ✅ Family Seed: `Nat0C/G/b4B7u06n0r14SuZXrp/IZ/38fZHh8aJQMVg=`
- ✅ Binaries: tower, beardog, songbird (exact same)
- ✅ Capabilities: Security, Discovery
- ✅ BirdSong protocol version

### Different

- ⚡ Node ID: Each derives unique ID from hostname + UUID
- ⚡ Encryption Tag: `beardog:family:nat0:hostname_UUID` (UUID differs)
- ⚡ Ports: 9000/3030 vs 9001/3031
- ⚡ Mount point: biomeOS1 vs biomeOS2

---

## 🔍 Expected Behavior

### Discovery Phase

**Songbird 1** (port 3030):
```
1. Broadcasts UDP multicast: "I'm family:nat0, node:X"
2. Listens for responses
3. Discovers Songbird 2 on same LAN
```

**Songbird 2** (port 3031):
```
1. Broadcasts UDP multicast: "I'm family:nat0, node:Y"
2. Listens for responses
3. Discovers Songbird 1 on same LAN
```

**Expected**: Both discover each other via UDP multicast!

### Trust Evaluation Phase

**BearDog 1** receives identity from Songbird 2:
```json
{
  "encryption_tag": "beardog:family:nat0:hostname_UUID2",
  "family_id": "nat0"
}
```

**BearDog 1 Evaluates**:
```
My family: nat0
Their family: nat0
Match? YES!
Trust Level: HIGH (same family)
Action: AUTO-ACCEPT ✅
```

**Expected**: Automatic trust and federation!

### Federation Phase

```
Tower 1 (Spore 1)        Tower 2 (Spore 2)
     |                         |
     |-- UDP Multicast ------->|  Discovery
     |<----- Response ---------|
     |                         |
     |-- Identity Request ---->|  Trust
     |<--- Identity + Proof ---|
     |                         |
     |-- Trust Established --->|  Federation
     |<---- Federated! --------|
```

**Expected**: Full federation within 30 seconds!

---

## 🛠️ Implementation Plan

### Step 1: Format Second USB

```bash
# Unmount if mounted
sudo umount /media/eastgate/BEA6-BBCE

# Format as ext4
sudo mkfs.ext4 -L biomeOS2 /dev/sdb1

# Mount
sudo mkdir -p /media/eastgate/biomeOS2
sudo mount /dev/sdb1 /media/eastgate/biomeOS2
sudo chown -R eastgate:eastgate /media/eastgate/biomeOS2
```

### Step 2: Clone Spore Structure

```bash
# Copy entire spore
cp -r /media/eastgate/biomeOS1/biomeOS /media/eastgate/biomeOS2/

# Verify
ls -la /media/eastgate/biomeOS2/biomeOS/
```

### Step 3: Modify Spore 2 Ports

**Edit `tower.toml` on Spore 2**:

```toml
[[primals]]
binary = "./primals/beardog"
provides = ["Security", "Encryption"]
requires = []
http_port = 9001  # CHANGED from 9000

[primals.env]
BEARDOG_API_BIND_ADDR = "0.0.0.0:9001"  # CHANGED from 9000
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_FAMILY_SEED = "Nat0C/G/b4B7u06n0r14SuZXrp/IZ/38fZHh8aJQMVg="

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]
http_port = 3031  # CHANGED from 3030

[primals.env]
SONGBIRD_HTTP_PORT = "3031"  # CHANGED from 3030
RUST_LOG = "info"
```

### Step 4: Deploy Both Spores

**Terminal 1** (Spore 1):
```bash
cd /media/eastgate/biomeOS1/biomeOS
./deploy.sh
```

**Terminal 2** (Spore 2):
```bash
cd /media/eastgate/biomeOS2/biomeOS
./deploy.sh
```

### Step 5: Verify Both Running

```bash
# Check processes
ps aux | grep -E "tower|beardog|songbird"

# Should see 6 processes:
# - 2x tower
# - 2x beardog (ports 9000, 9001)
# - 2x songbird (ports 3030, 3031)
```

### Step 6: Check Health

```bash
# Spore 1
curl http://localhost:9000/health
curl http://localhost:9000/identity

# Spore 2
curl http://localhost:9001/health
curl http://localhost:9001/identity
```

### Step 7: Monitor Discovery

```bash
# Watch for federation in logs
# (if we can access them - currently redirected to /dev/null)

# Check BearDog for trusted peers
# (API endpoint TBD)
```

---

## 🧪 Test Cases

### Test 1: Discovery

**Verify**: Both Songbirds discover each other
```bash
# Query Songbird 1
curl http://localhost:3030/peers

# Query Songbird 2
curl http://localhost:3031/peers

# Expected: Each sees the other
```

### Test 2: Family Verification

**Verify**: Both BearDogs report same family
```bash
# BearDog 1
curl http://localhost:9000/identity | jq '.family_id'
# Expected: "nat0"

# BearDog 2
curl http://localhost:9001/identity | jq '.family_id'
# Expected: "nat0"
```

### Test 3: Different Fingerprints

**Verify**: Different encryption tags
```bash
# BearDog 1
curl http://localhost:9000/identity | jq '.encryption_tag'
# Expected: "beardog:family:nat0:hostname_UUIDX"

# BearDog 2
curl http://localhost:9001/identity | jq '.encryption_tag'
# Expected: "beardog:family:nat0:hostname_UUIDY"

# UUIDX ≠ UUIDY (different fingerprints!)
```

### Test 4: Trust Establishment

**Verify**: Automatic trust due to same family
```bash
# Check if BearDog 1 trusts BearDog 2
# (API endpoint TBD)

# Expected: HIGH trust (same family)
```

### Test 5: Federation

**Verify**: Communication between spores
```bash
# Send request from Spore 1 to Spore 2's BearDog
# (via Songbird discovery)

# Expected: Successful encrypted communication
```

---

## 📊 Debugging Advantages

### What We Can Isolate

**✅ Same Machine**:
- No network latency
- No firewall issues
- No router/switch problems
- No DNS issues

**✅ Controlled Environment**:
- Access to both spores
- Monitor both in real-time
- Kill/restart easily
- Identical software versions

**✅ Debug Logs**:
- Both terminals visible
- Can check process status
- Can inspect env vars
- Can modify configs easily

### What We Can Test

1. **BirdSong Discovery**: Do they find each other via UDP multicast?
2. **Family Matching**: Do they recognize same family?
3. **Trust Evaluation**: Do they trust each other automatically?
4. **Encryption Tags**: Are unique fingerprints generated?
5. **API Communication**: Can they talk to each other?
6. **Health Monitoring**: Do they detect each other's health?

---

## 🎯 Success Criteria

### Minimum Success

- ✅ Both spores deploy successfully
- ✅ Different ports (9000/9001, 3030/3031)
- ✅ Same family (`nat0`)
- ✅ Different encryption tags (unique UUIDs)
- ✅ Both report healthy

### Full Success

- ✅ Minimum success criteria
- ✅ Both Songbirds discover each other
- ✅ Identity exchange successful
- ✅ Automatic trust established
- ✅ Federation working
- ✅ Can communicate between spores

---

## 🚀 Benefits

### For Debugging

1. **Eliminate Network Variables**: Same machine = no network issues
2. **Real-Time Monitoring**: Watch both spores simultaneously
3. **Fast Iteration**: Kill/restart/modify quickly
4. **Controlled Testing**: Same software, same environment

### For Development

1. **Test Federation Locally**: No need for multiple physical towers
2. **Validate BirdSong Protocol**: Prove discovery works
3. **Test Trust Evaluation**: Verify family-based trust
4. **Debug Communication**: See exactly what's being exchanged

### For Production

1. **Confidence**: If it works locally, it should work remotely
2. **Validation**: Prove the architecture before deploying
3. **Documentation**: Capture working config for deployment
4. **Troubleshooting**: Known-good baseline for comparison

---

## 📝 Next Steps

1. ✅ Format second USB as ext4 (biomeOS2)
2. ✅ Clone spore structure
3. ✅ Modify tower.toml for different ports
4. ✅ Deploy Spore 1
5. ✅ Deploy Spore 2
6. ✅ Verify both healthy
7. 🔍 Monitor discovery
8. 🔍 Verify federation
9. 📊 Document results

---

**Status**: Ready to implement USB spore clone strategy for local federation testing!

🎊 **This is a brilliant diagnostic approach!**


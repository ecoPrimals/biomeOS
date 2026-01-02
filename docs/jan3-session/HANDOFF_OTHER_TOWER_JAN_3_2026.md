# 🚀 Two-Tower Live Testing - Handoff for Other Tower

**Date**: January 3, 2026  
**From**: Tower 1 (pop-os) - biomeOS Development Team  
**To**: Tower 2 Agent (Other LAN Tower)  
**Purpose**: Deploy biomeOS USB v6.0 and establish genetic lineage federation

---

## 📊 Current Status

**Tower 1 (pop-os)** is deployed and ready:
- ✅ BearDog v0.10.0 running (localhost:9000)
- ✅ Songbird v6.0 running (0.0.0.0:8080, UDP 2300)
- ✅ Genetic lineage enabled (Family: iidn)
- ✅ Encryption tag: `beardog:family:iidn:pop-os_338b213a`
- ✅ USB Family Seed: `ecoPrimals-20260101-6b50f574`

**Waiting for**: Tower 2 to deploy from same USB package

---

## 🎯 Your Mission

Deploy the biomeOS USB v6.0 package on your tower and establish automatic federation with Tower 1.

**Expected Outcome**:
- Both towers discover each other via UDP multicast (30-60 seconds)
- Both towers verify genetic lineage via BearDog (same family = auto-trust)
- Automatic mesh formation via Songbird tarpc RPC
- Secure-by-default P2P coordination

---

## 📦 USB Package Details

**Version**: v6.0 (Production Ready)  
**Location**: Should be at `/media/*/biomeOS-LAN-Deploy` after plugging in USB  
**Family Seed**: `ecoPrimals-20260101-6b50f574` (same as Tower 1)

**Contents**:
- `primals/beardog-server-v0.10.0-universal` (6.0M, Universal Trust API)
- `primals/songbird-orchestrator` (24M, Zero Hardcoding + Generic Trust)
- `scripts/auto-deploy-v6.sh` (Genetic lineage deployment)
- `secrets/family-genesis.key` (USB family seed)

---

## 🚀 Deployment Steps

### Step 1: Copy USB to Local Directory

```bash
# Find USB mount point
ls -d /media/*/biomeOS-LAN-Deploy

# Copy to local directory
cp -r /media/*/biomeOS-LAN-Deploy ~/biomeOS-Deploy
cd ~/biomeOS-Deploy

# Set permissions
chmod +x scripts/*.sh primals/* biomeOS/*
```

### Step 2: Deploy Services

```bash
# Run auto-deploy script
./scripts/auto-deploy-v6.sh
```

**What this does**:
1. Reads USB family seed from `secrets/family-genesis.key`
2. Mixes with local machine entropy (unique for this tower)
3. Starts BearDog with genetic lineage enabled
4. Starts Songbird with BearDog trust integration
5. Begins UDP multicast discovery for Tower 1

### Step 3: Verify Deployment

```bash
# Check processes
ps aux | grep -E 'beardog-server|songbird-orchestrator' | grep -v grep

# Check ports
ss -tulpn | grep -E '9000|8080|2300'

# Check BearDog identity
curl -s http://localhost:9000/api/v1/trust/identity | jq .

# Check BearDog health
curl -s http://localhost:9000/api/v1/health | jq .
```

**Expected**:
- BearDog running on `localhost:9000`
- Songbird running on `0.0.0.0:8080` (tarpc) and `0.0.0.0:2300` (UDP multicast)
- BearDog identity shows family_id: `iidn` (same as Tower 1)
- Encryption tag: `beardog:family:iidn:<your-hostname>_<unique-id>`

---

## 📡 Automatic Discovery Process

**Timeline**:
1. **0-10 seconds**: Services start, BearDog creates child lineage
2. **10-30 seconds**: Songbird begins UDP multicast discovery
3. **30-60 seconds**: Tower 1 discovered, BearDog trust evaluation begins
4. **60-90 seconds**: Trust verified (same family = auto-accept), tarpc connection established
5. **90+ seconds**: Federation complete, mesh operational

**What's happening under the hood**:
```
Tower 2 (You)                Tower 1 (pop-os)
     |                            |
     |-- UDP multicast probe ---->|
     |<-- UDP multicast response -|
     |                            |
     |-- BearDog: evaluate trust ->| (family check)
     |<-- BearDog: auto-accept ---|
     |                            |
     |-- Songbird: tarpc connect ->|
     |<-- Songbird: federation OK -|
     |                            |
     |===== MESH ESTABLISHED =====|
```

---

## 🔍 Monitoring & Verification

### Watch for Connections

```bash
# Watch for tarpc connections on port 8080
watch -n 2 'ss -tn | grep 8080'

# Monitor Songbird logs
tail -f /tmp/songbird-orchestrator.log

# Monitor BearDog logs
tail -f /tmp/beardog-server.log
```

### Look for These Log Messages

**Songbird logs (successful discovery)**:
```
🔍 Peer discovered: pop-os at <IP>
🔐 Evaluating trust via BearDog...
✅ AUTO-ACCEPT: Same genetic family (iidn)
🤝 Federation established with pop-os
```

**BearDog logs (trust evaluation)**:
```
POST /api/v1/trust/evaluate
Peer tag: beardog:family:iidn:pop-os_338b213a
Decision: auto_accept (same_family)
Confidence: 1.0
```

---

## ✅ Success Criteria

You'll know the federation is working when:

1. ✅ **Both towers running**: BearDog + Songbird on both towers
2. ✅ **Same family ID**: Both towers show family_id: `iidn`
3. ✅ **Different encryption tags**: Each tower has unique tag (but same family prefix)
4. ✅ **TCP connection established**: `ss -tn | grep 8080` shows ESTABLISHED connection
5. ✅ **Logs show auto-accept**: Trust evaluation logs show "auto_accept (same_family)"

---

## 🐛 Troubleshooting

### Services won't start

```bash
# Check if old processes are running
ps aux | grep -E 'beardog|songbird'

# Kill old processes if needed
pkill -f beardog-server
pkill -f songbird-orchestrator

# Wait and retry
sleep 3
./scripts/auto-deploy-v6.sh
```

### Ports already in use

```bash
# Find what's using the ports
sudo lsof -i :9000
sudo lsof -i :8080
sudo lsof -i :2300

# Kill the processes
sudo kill <PID>
```

### Discovery not working

```bash
# Check if UDP port 2300 is listening
ss -ulpn | grep 2300

# Check if multicast is working
sudo tcpdump -i any port 2300 -c 10

# Verify both towers are on same network
ping <tower1-ip>

# Check firewall (might need to open ports)
sudo ufw status
```

### Trust evaluation failing

```bash
# Check BearDog identity
curl -s http://localhost:9000/api/v1/trust/identity | jq .

# Verify family_id matches Tower 1 ("iidn")
# If different, USB seed might be missing or different

# Check USB seed
cat ~/biomeOS-Deploy/secrets/family-genesis.key | jq .
```

---

## 📞 Report Back to Tower 1

After deployment, report back with:

1. **Your tower name**: `hostname`
2. **Your IP address**: `hostname -I | awk '{print $1}'`
3. **Your encryption tag**: `curl -s http://localhost:9000/api/v1/trust/identity | jq -r .data.encryption_tag`
4. **Federation status**: `ss -tn | grep 8080 | grep ESTAB` (should show connection to Tower 1)

---

## 🎯 Key Points to Remember

1. **Same USB = Same Family**: Both towers use the same USB family seed, so they'll auto-trust each other
2. **Unique Identities**: Each tower mixes USB seed with local entropy for unique encryption tag
3. **Automatic Discovery**: UDP multicast on port 2300 (works across routers!)
4. **Genetic Lineage**: BearDog handles all trust evaluation cryptographically
5. **Zero Manual Config**: Everything is automatic - just run the script and wait 60-90 seconds

---

## 📊 Expected Final State

```
Tower 1 (pop-os)              Tower 2 (Your Tower)
├─ IP: <tower1-ip>            ├─ IP: <tower2-ip>
├─ Family: iidn                ├─ Family: iidn
├─ Tag: beardog:family:iidn:  ├─ Tag: beardog:family:iidn:
│   pop-os_338b213a           │   <your-hostname>_<unique>
├─ BearDog: localhost:9000    ├─ BearDog: localhost:9000
├─ Songbird: 0.0.0.0:8080     ├─ Songbird: 0.0.0.0:8080
└─ Discovery: 0.0.0.0:2300    └─ Discovery: 0.0.0.0:2300
         |                             |
         |<====== MESH ======>|
         |  (auto-trust via    |
         |   genetic lineage)  |
```

---

**Status**: ✅ **READY FOR YOUR DEPLOYMENT**  
**Timeline**: ~2-3 minutes for full federation  
**Support**: Monitor logs and report any issues

🚀 **Good luck! Let's see this historic two-tower genetic lineage mesh in action!** 🚀


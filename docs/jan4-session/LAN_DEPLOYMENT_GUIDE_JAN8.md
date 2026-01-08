# 🌐 LAN Deployment Guide - Cross-Subnet Federation

**Date**: January 8, 2026 (Late Evening)  
**Status**: 🚀 **READY FOR LAN DEPLOYMENT**  
**Purpose**: Deploy genetic sibling node to different LAN for federation testing

---

## ✅ Prerequisites Verified

### Local Testing Complete ✅
- ✅ 2 nodes running locally (node-alpha, node-beta)
- ✅ Port-free architecture validated
- ✅ UDP multicast discovery active
- ✅ Unix sockets working (BearDog + Songbird)
- ✅ Genetic lineage verified (unique siblings)

### Available Spores ✅
- ✅ **node-gamma** (BEA6-BBCE) - Available for LAN
- ✅ **node-delta** (BEA6-BBCE1) - Available for LAN
- ✅ **node-epsilon** (BEA6-BBCE2) - Available for LAN
- 🔒 node-alpha (biomeOS1) - Running locally
- 🔒 node-beta (biomeOS21) - Running locally

### Pipeline Validated ✅
- ✅ NucleusBin build system working
- ✅ Capability-based deployment
- ✅ Self-contained USB spores
- ✅ deploy.sh tested and working

---

## 🎯 Recommended LAN Deployment

### Use: **node-epsilon** (BEA6-BBCE2)

**Why node-epsilon?**
1. Latest spore (most recent validation)
2. Preserves node-gamma and node-delta for future testing
3. Genetic sibling of running nodes (same family: `nat0`)
4. Full stack validated (tower, BearDog, Songbird)

**Genetic Info**:
```
Node ID: node-epsilon
Family: nat0
Seed: 6e32319ece57c20a89c99daa94126fea...
Relationship: Genetic sibling to node-alpha & node-beta
```

---

## 📋 LAN Deployment Steps

### Step 1: Prepare USB Spore (Already Done! ✅)

The USB is already prepared with:
- ✅ `tower` orchestrator
- ✅ `beardog-server` (Unix socket mode)
- ✅ `songbird` (UDP multicast + Unix socket)
- ✅ `deploy.sh` (self-bootable script)
- ✅ `.family.seed` (unique genetic identity)
- ✅ `tower.toml` (configuration)

### Step 2: Physical USB Transfer

```bash
# On current machine:
1. Safely eject USB: BEA6-BBCE2 (node-epsilon)
2. Remove USB from machine
3. Transport to target LAN machine
4. Insert USB into target machine
```

### Step 3: Mount USB on Target Machine

```bash
# On target LAN machine (Linux):
# USB should auto-mount, or manually:
sudo mkdir -p /media/biomeOS-epsilon
sudo mount /dev/sdX1 /media/biomeOS-epsilon  # Replace sdX1 with your USB device

# Verify mount:
ls -la /media/biomeOS-epsilon/biomeOS/
```

### Step 4: Deploy on Target Machine

```bash
# Navigate to spore:
cd /media/biomeOS-epsilon/biomeOS

# Make deploy script executable (if needed):
chmod +x deploy.sh
chmod +x bin/tower
chmod +x primals/*

# Deploy:
./deploy.sh

# The script will:
# 1. Start tower orchestrator
# 2. Tower spawns BearDog (port-free mode)
# 3. Tower spawns Songbird (UDP multicast + Unix socket)
# 4. Services start broadcasting discovery
```

### Step 5: Verify Deployment

```bash
# Check processes:
ps aux | grep -E "tower|beardog|songbird" | grep -v grep

# Expected output:
# ./bin/tower run --config tower.toml
# ./primals/beardog-server
# ./primals/songbird

# Check Unix sockets:
ls -lh /tmp/*epsilon*.sock

# Expected:
# /tmp/beardog-nat0-node-epsilon.sock
# /tmp/songbird-nat0-node-epsilon.sock

# Check logs:
tail -f /tmp/primals/*.log
```

---

## 🌐 Network Requirements

### Firewall Rules (Target Machine)

**Required Ports**:
```bash
# UDP Multicast Discovery:
# Port: 2300 (UDP)
# Multicast Group: 224.0.0.251
# Direction: Inbound & Outbound
sudo ufw allow 2300/udp

# HTTPS Federation (fallback):
# Port: 8080 (TCP) - Songbird HTTPS endpoint
sudo ufw allow 8080/tcp

# tarpc RPC (primary):
# Port: 8091 (TCP) - High-performance binary RPC
sudo ufw allow 8091/tcp
```

**Multicast Routing**:
```bash
# Ensure multicast is enabled on network interface:
sudo ip link set dev eth0 multicast on  # or wlan0, etc.

# Add multicast route (if needed):
sudo ip route add 224.0.0.0/4 dev eth0  # or wlan0, etc.
```

### Network Configuration

**Same Subnet** (Easiest):
- If both machines are on same subnet (e.g., 192.168.1.x/24)
- UDP multicast will work automatically
- No additional routing needed

**Different Subnet** (More Complex):
- Requires multicast routing between subnets
- May need IGMP support on routers
- Test connectivity with `ping` and `nc` first

**Cross-Site** (Advanced):
- Requires VPN or BTSP tunnel
- UDP multicast won't work across internet
- Falls back to HTTPS + BTSP encrypted tunnel

---

## 🧪 Verification & Testing

### 1. Check UDP Multicast Broadcasting

```bash
# On target machine:
sudo tcpdump -i any -n udp port 2300

# Expected output (every 30s):
# IP 192.168.1.X.2300 > 224.0.0.251.2300: UDP, length Y
```

### 2. Verify Discovery (Wait ~30-60 seconds)

```bash
# Check Songbird logs for peer discoveries:
tail -f /tmp/primals/*.log | grep -E "Discovered peer|peer discovery|Federation"

# Expected:
# "Discovered peer: node-alpha (192.168.1.144:8080)"
# "Discovered peer: node-beta (192.168.1.144:8081)"
# "Federation bridge: Processing 2 peers"
```

### 3. Test Trust Evaluation

```bash
# Watch logs for trust evaluation:
tail -f /tmp/primals/*.log | grep -E "Trust|family|genetic"

# Expected:
# "Trust evaluation: node-alpha (family: nat0) -> ALLOW"
# "Genetic lineage verified: same family"
# "BTSP tunnel candidate: node-alpha"
```

### 4. Test Connectivity

```bash
# From source machine (node-alpha/beta):
# Check if epsilon appears in discovered peers
curl -s http://localhost:3000/api/topology | jq '.nodes[] | select(.id | contains("epsilon"))'

# From target machine (node-epsilon):
# Check if alpha/beta appear
curl -s http://localhost:3000/api/topology | jq '.nodes[] | select(.id | contains("alpha") or contains("beta"))'
```

---

## 🎯 Expected Behavior

### Discovery Timeline

**0-30 seconds**: Node starts broadcasting
- ✅ UDP multicast packets sent every 30s
- ✅ Discovery listener active

**30-60 seconds**: First peer discoveries
- ✅ node-epsilon discovers node-alpha & node-beta
- ✅ node-alpha & node-beta discover node-epsilon

**60-90 seconds**: Trust evaluation
- ✅ Genetic lineage verified (family: nat0)
- ✅ Trust decisions made (siblings -> ALLOW)
- ✅ BTSP tunnel candidates identified

**90-120 seconds**: Federation established
- ✅ Peers added to federation
- ✅ BTSP tunnels established (if needed)
- ✅ Full mesh connectivity

### Success Indicators

1. **Processes Running** ✅
   ```
   tower, beardog-server, songbird all running
   ```

2. **Unix Sockets Created** ✅
   ```
   /tmp/beardog-nat0-node-epsilon.sock
   /tmp/songbird-nat0-node-epsilon.sock
   ```

3. **UDP Multicast Active** ✅
   ```
   tcpdump shows packets to 224.0.0.251:2300
   ```

4. **Peer Discoveries** ✅
   ```
   Logs show "Discovered peer: node-alpha", etc.
   ```

5. **Trust Evaluation** ✅
   ```
   Logs show "family: nat0", "Trust: ALLOW"
   ```

6. **Federation Mesh** ✅
   ```
   Topology API shows all 3 nodes connected
   ```

---

## 🚨 Troubleshooting

### Issue: No UDP Multicast Packets

**Symptoms**:
- `tcpdump` shows no packets on port 2300
- No peer discoveries in logs

**Solutions**:
```bash
# Check multicast is enabled:
ip link show eth0 | grep MULTICAST

# Enable if disabled:
sudo ip link set dev eth0 multicast on

# Check firewall:
sudo ufw status | grep 2300

# Verify Songbird is running:
ps aux | grep songbird
```

### Issue: Peers Discovered But Not Trusted

**Symptoms**:
- Logs show "Discovered peer: X"
- But logs also show "Trust evaluation: DENY"

**Cause**: Different family tags (not genetic siblings)

**Solutions**:
```bash
# Check family seeds match:
sha256sum /media/biomeOS-epsilon/biomeOS/.family.seed
sha256sum /media/biomeOS1/biomeOS/.family.seed  # on source machine

# If different families, need to use sibling derivation
# (Seeds should be unique but from same parent)

# Check tower.toml family_id:
grep family_id /media/biomeOS-epsilon/biomeOS/tower.toml
# Should be: family_id = "nat0"
```

### Issue: BearDog Not Starting

**Symptoms**:
- No `/tmp/beardog-nat0-node-epsilon.sock`
- Logs show "HSM provider" errors

**Solutions**:
```bash
# Check BearDog logs:
find /tmp/primals -name "*.log" -exec grep -l "BearDog" {} \;

# Verify binary is executable:
ls -lh /media/biomeOS-epsilon/biomeOS/primals/beardog-server

# Check environment:
grep BEARDOG /media/biomeOS-epsilon/biomeOS/deploy.sh
```

---

## 📊 Network Architecture

### Same LAN (Current Setup)

```
┌──────────────────────────────────────────────────────────┐
│                  Local Network (192.168.1.x/24)          │
│                                                           │
│  ┌──────────────┐    UDP Multicast     ┌──────────────┐ │
│  │ node-alpha   │◄────224.0.0.251────►│ node-epsilon │ │
│  │ (current PC) │      :2300           │ (target PC)  │ │
│  │              │                      │              │ │
│  │ - BearDog    │    BTSP Tunnel       │ - BearDog    │ │
│  │ - Songbird   │◄───(encrypted)─────►│ - Songbird   │ │
│  │ - Tower      │                      │ - Tower      │ │
│  └──────────────┘                      └──────────────┘ │
│         ▲                                      ▲         │
│         │         UDP Multicast Discovery      │         │
│         └──────────────────┬───────────────────┘         │
│                            │                             │
│                    ┌───────▼────────┐                    │
│                    │   node-beta    │                    │
│                    │  (current PC)  │                    │
│                    └────────────────┘                    │
│                                                           │
└──────────────────────────────────────────────────────────┘
```

### Communication Protocols

**Discovery** (Anonymous):
- UDP Multicast 224.0.0.251:2300
- 30-second broadcast interval
- No encryption (public discovery)

**Trust Evaluation** (Via BearDog):
- Unix socket IPC (local)
- Genetic lineage verification
- Family tag comparison

**Federation** (Authenticated):
- Primary: tarpc (8091) - Binary RPC
- Secondary: HTTPS (8080) - JSON API
- Encrypted: BTSP tunnel (via BearDog)

---

## 🎓 Key Concepts

### Genetic Lineage Trust Model

1. **Discovery**: Anonymous UDP multicast
   - All nodes broadcast presence
   - Family tag included: `beardog:family:nat0`

2. **Trust Evaluation**: Genetic verification
   - Receiving node checks family tag
   - If same family → BearDog verifies lineage
   - If verified → Trust level: ALLOW

3. **Federation**: Secure communication
   - BTSP tunnel established (encrypted)
   - tarpc for high-performance RPC
   - Continuous trust re-evaluation

### Port-Free Architecture

**Local (within tower)**:
- Unix sockets only
- File system permissions
- Zero network exposure

**Network (between towers)**:
- UDP multicast (discovery only)
- tarpc/HTTPS (federation)
- BTSP tunnels (encrypted P2P)

**Security Levels**:
1. Unix Socket IPC: Level 5 (highest)
2. BTSP Tunnel: Level 4
3. tarpc: Level 3
4. HTTPS: Level 2
5. UDP Multicast: Level 1 (discovery only)

---

## 🎊 Ready for LAN Deployment!

### Quick Checklist

- ✅ Local testing complete (2 nodes)
- ✅ USB spore ready (node-epsilon)
- ✅ Deployment script validated
- ✅ Network requirements documented
- ✅ Troubleshooting guide provided
- ✅ Genetic lineage verified

### Deployment Summary

1. **Eject USB**: BEA6-BBCE2 (node-epsilon)
2. **Transport**: Move to target LAN machine
3. **Mount**: Insert USB, auto-mount or manual mount
4. **Deploy**: `cd /media/.../biomeOS && ./deploy.sh`
5. **Verify**: Check processes, sockets, logs
6. **Wait**: 30-60s for discovery
7. **Confirm**: Peers discovered, trust evaluated, federation established

### Expected Timeline

- **5 minutes**: Full deployment and discovery
- **10 minutes**: Federation mesh established
- **15 minutes**: BTSP tunnels active

---

**Status**: ✅ **READY FOR LAN DEPLOYMENT**  
**Recommended Spore**: node-epsilon (BEA6-BBCE2)  
**Risk Level**: Low (validated pipeline)

🌐 **Let's federate across the LAN!** 🧬🐻🐦🌱


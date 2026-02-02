# USB ↔ PIXEL STUN HANDSHAKE TEST PLAN

**Date**: February 2, 2026  
**Goal**: Validate cross-device STUN discovery and handshake  
**Status**: 🚀 **IN PROGRESS**

═══════════════════════════════════════════════════════════════════

## ✅ **INFRASTRUCTURE STATUS**

### **USB System** ✅ **READY**

```
BearDog:  ✅ Running (Alpha: 301235, Beta: 301649)
Songbird: ✅ Running (PID: 364477)
Socket:   /run/user/1000/biomeos/songbird-debug.sock
STUN:     ⏳ Testing now
```

---

### **Pixel System** ⚡ **PARTIAL**

```
BearDog:  ✅ Running (PID: 5457)
          ✅ TESTED: crypto.blake3_hash working
          ✅ Port forward: localhost:9900 → Pixel:9900
Songbird: ⏳ Starting HTTP-only mode
HTTP:     0.0.0.0:8080 (should be listening)
STUN:     ⏳ Will use Songbird's built-in STUN client
```

---

## 🎯 **TEST PHASES**

### **Phase 1: STUN Discovery** ⏳ **IN PROGRESS**

**USB Side**:
```bash
# Via Songbird IPC
echo '{"jsonrpc":"2.0","method":"stun.get_public_address",
"params":{"server":"stun.l.google.com:19302"},"id":1}' \
  | nc -U /run/user/1000/biomeos/songbird-debug.sock
```

**Expected**: Public IP + port for USB

**Pixel Side**:
```bash
# Via Songbird HTTP API (if IPC unavailable)
# Or direct STUN test via Songbird binary
adb shell "cd /data/local/tmp && \
  echo '{\"jsonrpc\":\"2.0\",\"method\":\"stun.get_public_address\",
  \"params\":{\"server\":\"stun.l.google.com:19302\"},\"id\":1}' \
  | nc 127.0.0.1 [SONGBIRD_IPC_PORT]"
```

**Expected**: Public IP + port for Pixel

---

### **Phase 2: Manual Connection Test** ⏳ **PENDING**

**Objective**: Verify direct connectivity between discovered addresses

**Test**:
```bash
# From USB to Pixel's public address
nc -vz [PIXEL_PUBLIC_IP] [PIXEL_PUBLIC_PORT]

# From Pixel to USB's public address
adb shell "nc -vz [USB_PUBLIC_IP] [USB_PUBLIC_PORT]"
```

**Expected**: Connection established or timeout (NAT type dependent)

---

### **Phase 3: BirdSong Exchange** ⏳ **PENDING**

**Objective**: Use BearDog genetics to exchange STUN results

**USB → Pixel**:
```bash
# Encode USB's public address with BearDog crypto
echo '{"jsonrpc":"2.0","method":"crypto.encrypt",
"params":{"data":"[USB_IP:PORT_BASE64]","recipient":"pixel_tower"},"id":1}' \
  | nc -U /run/user/1000/biomeos/beardog-alpha.sock
```

**Pixel → USB**:
```bash
# Encode Pixel's public address with BearDog crypto
adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"crypto.encrypt\",
\"params\":{\"data\":\"[PIXEL_IP:PORT_BASE64]\",\"recipient\":\"alpha_tower\"},\"id\":1}' \
  | nc 127.0.0.1 9900"
```

**Exchange**: Manual for first test, then via Dark Forest broadcast

---

### **Phase 4: Dark Forest Broadcast** ⏳ **PENDING**

**Objective**: Autonomous discovery via BirdSong

**USB Broadcast**:
```bash
# Announce presence on family channel
echo '{"jsonrpc":"2.0","method":"birdsong.broadcast",
"params":{"family":"dark_forest","message":"[ENCRYPTED_DISCOVERY]"},"id":1}' \
  | nc -U /run/user/1000/biomeos/songbird-debug.sock
```

**Pixel Listen**:
```bash
# Listen for family broadcasts
adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"birdsong.listen\",
\"params\":{\"family\":\"dark_forest\"},\"id\":1}' \
  | nc 127.0.0.1 [SONGBIRD_PORT]"
```

**Expected**: Mutual discovery, lineage verification

---

### **Phase 5: Lineage Verification** ⏳ **PENDING**

**Objective**: Verify genetic relationship before handshake

**USB Verify Pixel**:
```bash
# Check if Pixel's signature matches family genetics
echo '{"jsonrpc":"2.0","method":"genetics.verify_lineage",
"params":{"node":"pixel_tower","signature":"[PIXEL_SIG]"},"id":1}' \
  | nc -U /run/user/1000/biomeos/beardog-alpha.sock
```

**Expected**: `true` (same family) or `false` (reject)

---

### **Phase 6: Complete Handshake** ⏳ **PENDING**

**Objective**: Establish trusted P2P connection

**Steps**:
1. Exchange ephemeral keys
2. Establish encrypted channel
3. Verify mutual capabilities
4. Complete trust escalation
5. Mark as federated peer

**Expected**: Persistent P2P connection, elevated trust level

---

## 📊 **CURRENT STATUS**

**Completed**:
- ✅ TCP transport implemented
- ✅ BearDog on Pixel operational
- ✅ USB TOWER operational
- ✅ Port forwarding working

**In Progress**:
- ⏳ USB STUN discovery
- ⏳ Pixel Songbird HTTP startup
- ⏳ Pixel STUN discovery

**Pending**:
- Manual connection test
- BirdSong exchange
- Dark Forest broadcast
- Lineage verification
- Complete handshake

---

## 🚀 **NEXT IMMEDIATE STEPS**

1. **Verify USB STUN** (now)
2. **Start Pixel Songbird HTTP** (now)
3. **Test Pixel STUN** (5 min)
4. **Manual connection test** (10 min)
5. **Exchange via BearDog crypto** (15 min)

**Timeline**: ~30 min to Phase 3 complete

═══════════════════════════════════════════════════════════════════

**Status**: 🚀 Infrastructure ready, testing STUN discovery now!

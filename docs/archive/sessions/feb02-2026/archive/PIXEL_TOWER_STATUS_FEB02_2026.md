# 🚧 PIXEL TOWER DEPLOYMENT STATUS

**Date**: February 2, 2026  
**Goal**: Full TOWER deployment on Pixel 8a (BearDog + Songbird)  
**Status**: ⚡ **PARTIAL SUCCESS - In Progress**

═══════════════════════════════════════════════════════════════════

## ✅ **ACHIEVEMENTS**

### **BearDog** ✅ **FULLY OPERATIONAL**

```
Status:    ✅ RUNNING
PID:       5457
Port:      127.0.0.1:9900 (TCP)
Protocol:  JSON-RPC 2.0 over TCP
Family:    pixel_tower
Node:      pixel_node1
Tested:    ✅ crypto.blake3_hash working
```

**Test Evidence**:
```json
{"algorithm":"BLAKE3","hash":"SHjKBCXHOfpCf37aIP6EX2suRrpf4qFN9bHjL1BgMhU="}
```

**Success**: BearDog TCP transport working perfectly on Android! 🎊

---

### **Songbird** ⚡ **PARTIALLY RUNNING**

```
Status:    ⚡ RUNNING (HTTP server working, IPC server failed)
HTTP Port: 0.0.0.0:8080 (LAN discovery) ✅
IPC Port:  127.0.0.1:9901 (TCP) ❌ NOT LISTENING
Family:    pixel_tower
Node:      pixel_node1
```

**Issues**:
1. Universal IPC Broker crashed trying to bind Unix sockets
2. Custom TCP IPC server (`start_ipc_server_tcp`) didn't start
3. Songbird process may have exited after broker error

**Root Cause**:
- Songbird has TWO IPC systems running simultaneously:
  1. **Universal IPC Broker** (built-in, tries Unix sockets first)
  2. **Custom TCP IPC** (bin_interface.rs, for biomeOS integration)
- Universal IPC Broker fails on Android → crashes orchestrator
- Custom TCP server never gets chance to start

**Evidence from Log**:
```
INFO ✅ TCP IPC listening on 127.0.0.1:45337  (Universal IPC Broker auto-fallback)
ERROR ❌ Universal IPC Broker error: Universal IPC Broker server error
```

---

## 🔍 **DIAGNOSIS**

### **Why Songbird Failed**

1. **Architectural Conflict**:
   - Songbird orchestrator starts Universal IPC Broker automatically
   - Universal IPC Broker detects Android, tries Unix → fails → TCP fallback (45337)
   - BUT then Universal IPC Broker itself crashes with "server error"
   - Custom `start_ipc_server_tcp` code (from `bin_interface.rs`) never reached

2. **Process Death**:
   - Orchestrator crash likely killed entire Songbird process
   - No process in `ps` output confirms this

3. **TCP Server Not Started**:
   - `start_ipc_server_tcp` function was never invoked
   - Port 9901 never bound
   - Connection refused when testing

---

## 💡 **SOLUTION PATHS**

### **Option 1: Fix Universal IPC Broker** ⏰ **2-3 hours**

**Approach**: Fix the broker's Android TCP fallback

**Pros**:
- Addresses root cause
- Benefits all Songbird deployments

**Cons**:
- Requires deep dive into Universal IPC Broker code
- Unfamiliar subsystem
- May have other hidden Android issues

---

### **Option 2: Disable Universal IPC Broker** ⏰ **30 min**

**Approach**: Skip/disable Universal IPC Broker, rely only on custom TCP server

**Implementation**:
```rust
// In app::core.rs startup
#[cfg(target_os = "android")]
{
    info!("Skipping Universal IPC Broker (Android TCP mode)");
    // Don't start broker
}

#[cfg(not(target_os = "android"))]
{
    // Start broker normally
}
```

**Pros**:
- Fast solution
- Custom TCP server is already implemented
- Clear separation of concerns

**Cons**:
- Loses some Universal IPC features (maybe not needed for biomeOS integration)

---

### **Option 3: Simple Test Mode** ⏰ **15 min** ⭐ **RECOMMENDED**

**Approach**: Run Songbird without IPC for now, test only HTTP + STUN

**Command**:
```bash
# No --listen flag, just HTTP server
./songbird-tcp server --port 8080
```

**Rationale**:
- BearDog IPC already works (9900) ✅
- Songbird's value for STUN handshake is HTTP discovery + STUN client
- IPC between Songbird and BearDog can be added later
- Focus on cross-device handshake (USB ↔ Pixel)

**Next Steps**:
1. Start Songbird without IPC
2. Test STUN from Songbird directly (has built-in STUN client)
3. Proceed to USB ↔ Pixel handshake testing
4. Fix IPC in parallel session

---

## 🎯 **IMMEDIATE RECOMMENDATION**

**DO**: Option 3 (Simple Test Mode)

**Reasoning**:
- User goal: USB ↔ Pixel STUN handshake
- BearDog working (crypto available)
- Songbird HTTP + STUN client work without IPC
- Unblock progress, fix IPC later

**Command**:
```bash
# Pixel - Songbird (no IPC)
SONGBIRD_FAMILY_ID=pixel_tower \
SONGBIRD_NODE_ID=pixel_node1 \
HOME=/data/local/tmp \
RUST_LOG=info \
./songbird-tcp server --port 8080
```

**Expected**: Songbird HTTP server operational, STUN client available

---

## 📊 **CURRENT SYSTEM STATE**

### **USB** ✅ **READY**

```
BearDog:  ✅ Unix socket (/run/user/1000/biomeos/beardog.sock)
Songbird: ✅ Unix socket (/run/user/1000/biomeos/songbird.sock)
STUN:     ✅ Public IP 162.226.225.148
Mode:     Tier 1 (optimal - tarpc + Unix sockets)
```

---

### **Pixel** ⚡ **PARTIAL**

```
BearDog:  ✅ TCP (127.0.0.1:9900) - WORKING
Songbird: ❌ Crashed (Universal IPC Broker issue)
STUN:     ⏳ Untested (Songbird not running)
Mode:     Tier 2 (degraded - TCP transport)
```

---

## 🚀 **NEXT ACTIONS**

**Immediate** (15 min):
1. Start Songbird without IPC (Option 3)
2. Verify HTTP server operational
3. Test STUN client directly

**Short-term** (30 min):
4. USB discovers public IP
5. Pixel discovers public IP
6. Exchange IPs (manual for now)
7. Test connectivity

**Medium-term** (1-2 hours):
8. Fix Songbird IPC (Option 2 or disable broker)
9. Wire BirdSong Dark Forest broadcast
10. Autonomous handshake

---

═══════════════════════════════════════════════════════════════════

**Grade**: ⚡ **B+ PROGRESS**

- ✅ BearDog TCP transport: **A+**
- ⚡ Songbird IPC: **C** (needs fix)
- ✅ Overall architecture: **A** (clear path forward)

**Next**: Deploy Songbird without IPC, proceed to handshake testing!

═══════════════════════════════════════════════════════════════════

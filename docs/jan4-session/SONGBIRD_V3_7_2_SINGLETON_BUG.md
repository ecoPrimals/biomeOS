# 🚨 Critical Bug Report: Songbird v3.7.2 Singleton Check Too Aggressive

**Date**: January 4, 2026 17:11 EST  
**Severity**: **CRITICAL** - Blocks multi-spore deployment  
**Status**: 🔴 **BLOCKING FRACTAL SCALING**

---

## 🎯 Issue Summary

Songbird v3.7.2-complete has a **singleton check that is too aggressive**, preventing multiple Songbird instances from running on the same machine, **even when they have different `NODE_ID`s and should use different socket paths**.

This **blocks multi-spore/multi-tower deployment** on a single machine, which is critical for:
- Local development and testing
- Multi-VM deployments on a single host
- Container orchestration (Kubernetes, Docker Compose)
- Fractal scaling validation

---

## 📊 Evidence

### **Scenario**
Two USB spores (biomeOS1 and biomeOS2) deployed on the same machine:
- **Spore 1**: `SONGBIRD_NODE_ID=tower1` → Socket: `/tmp/songbird-nat0-tower1.sock`
- **Spore 2**: `SONGBIRD_NODE_ID=tower2` → Socket: `/tmp/songbird-nat0-tower2.sock`

### **Expected Behavior**
Both Songbird instances start successfully with unique socket paths.

### **Actual Behavior**
- **Spore 1**: ✅ Starts successfully (PID 1316456, socket: `/tmp/songbird-nat0-tower1.sock`)
- **Spore 2**: ❌ Crashes immediately with error:

```
Error: Another Songbird instance is already running (PID: 1316456)
```

### **Process Status**
```bash
$ ps aux | grep "[s]ongbird"
eastgate 1316456  0.0  0.3  ./primals/songbird  ✅ Running (Spore 1)
eastgate 1316659  0.0  0.0  [songbird] <defunct>  ❌ Zombie (Spore 2)
```

### **Socket Status**
```bash
$ ls /tmp/songbird*.sock
/tmp/songbird-nat0-tower1.sock  ✅ Only Spore 1's socket exists
# Expected: /tmp/songbird-nat0-tower2.sock  ❌ Never created!
```

### **Environment Variables** (Spore 2 Songbird - from defunct PID)
```bash
SONGBIRD_FAMILY_ID=nat0     ✅ Correct
SONGBIRD_NODE_ID=tower2     ✅ Correct (unique!)
SONGBIRD_MULTICAST_ADDR=239.255.42.99:4242
SONGBIRD_DISCOVERY_INTERVAL=5
```

**Conclusion**: Environment variables are correct, socket paths should be unique, but Songbird refuses to start due to singleton check.

---

## 🔍 Root Cause Analysis

### **Hypothesis**
Songbird v3.7.2 implements a singleton check (likely using a pidfile, lock file, or global mutex) that **does not account for `NODE_ID`**.

**What it should do**:
```rust
// Allow multiple instances with different NODE_IDs
let lock_file = format!("/tmp/songbird-{}-{}.lock", family_id, node_id);
```

**What it's probably doing**:
```rust
// Global singleton check (wrong!)
let lock_file = "/tmp/songbird.lock";  // ❌ Not unique per NODE_ID
// OR
let pidfile = "/var/run/songbird.pid"; // ❌ Not unique per NODE_ID
```

### **Why This Was Added**
The singleton check was likely added to prevent:
- Accidental double-starts on the same machine
- Port conflicts (when using HTTP)
- Socket path collisions

These are **valid concerns for single-instance deployments**, but the check must be **scoped per NODE_ID** for multi-instance support.

---

## 🛠️ Proposed Fix

### **Option 1: NODE_ID-scoped Singleton Check** (Recommended)

```rust
// In songbird-orchestrator/src/app/core.rs or similar

// Get NODE_ID and FAMILY_ID as we already do for socket paths
let family_id = SafeEnv::get("SONGBIRD_FAMILY_ID")
    .ok()
    .or_else(|| std::env::var("FAMILY_ID").ok())
    .unwrap_or_else(|| "default".to_string());

let node_id = SafeEnv::get("SONGBIRD_NODE_ID")
    .ok()
    .or_else(|| std::env::var("NODE_ID").ok())
    .or_else(|| std::env::var("SPORE_ID").ok())
    .unwrap_or_else(|| "default".to_string());

// Use unique lock file per instance
let lock_file = format!("/tmp/songbird-{}-{}.lock", family_id, node_id);

// Now the singleton check allows multiple instances with different NODE_IDs!
let lock = FileLock::try_lock(&lock_file)?;
```

### **Option 2: Remove Singleton Check Entirely**

Since Songbird v3.7.2 already uses **unique socket paths** per `NODE_ID`, the socket itself acts as a natural singleton enforcer:

```rust
// In Unix socket server initialization
// If socket bind fails, another instance is already running on THIS node
match UnixListener::bind(&socket_path) {
    Ok(listener) => listener,
    Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
        return Err(format!(
            "Another Songbird instance with NODE_ID={} is already running",
            node_id
        ));
    }
    Err(e) => return Err(e.into()),
}
```

**This approach**:
- ✅ Prevents duplicate NODE_IDs (real problem)
- ✅ Allows different NODE_IDs (desired behavior)
- ✅ No separate lock file needed
- ✅ Clear error message

---

## 🎯 Acceptance Criteria

After fix, validate:

1. **Single Spore Deployment** (regression test)
   - ✅ Songbird starts successfully
   - ✅ Cannot start second instance with same NODE_ID
   - ✅ Clear error message if attempted

2. **Multi-Spore Deployment** (new requirement)
   - ✅ Spore 1 (NODE_ID=tower1) starts successfully
   - ✅ Spore 2 (NODE_ID=tower2) starts successfully
   - ✅ Both create unique sockets:
     - `/tmp/songbird-nat0-tower1.sock`
     - `/tmp/songbird-nat0-tower2.sock`
   - ✅ Both respond to JSON-RPC pings
   - ✅ No zombie processes

3. **Error Handling** (edge cases)
   - ✅ Duplicate NODE_ID → Clear error message
   - ✅ Missing NODE_ID → Falls back to legacy `/tmp/songbird.sock` (singleton)
   - ✅ Permission denied → Clear error message

---

## 📋 Test Script

```bash
#!/usr/bin/env bash
# test-multi-songbird.sh

set -e

# Clean state
pkill -9 songbird || true
rm -f /tmp/songbird*.sock /tmp/songbird*.lock

echo "Test 1: Single Instance"
export SONGBIRD_FAMILY_ID=nat0
export SONGBIRD_NODE_ID=tower1
./songbird &
PID1=$!
sleep 3
test -S /tmp/songbird-nat0-tower1.sock && echo "✅ Test 1 PASS" || echo "❌ Test 1 FAIL"

echo "Test 2: Duplicate NODE_ID (should fail)"
./songbird &
PID2=$!
sleep 3
ps -p $PID2 > /dev/null && echo "❌ Test 2 FAIL (should have exited)" || echo "✅ Test 2 PASS"

echo "Test 3: Different NODE_ID (should succeed)"
export SONGBIRD_NODE_ID=tower2
./songbird &
PID3=$!
sleep 3
test -S /tmp/songbird-nat0-tower2.sock && echo "✅ Test 3 PASS" || echo "❌ Test 3 FAIL"

# Cleanup
kill $PID1 $PID3
rm -f /tmp/songbird*.sock
```

---

## 📊 Impact Assessment

### **Who This Affects**
- ✅ **biomeOS**: Blocks multi-spore USB deployment (critical for fractal scaling demo)
- ✅ **Development**: Cannot run multiple test instances locally
- ✅ **CI/CD**: Cannot run parallel tests
- ✅ **Production**: Blocks multi-tenant or multi-environment deployments on single host

### **Workarounds** (None Viable)
1. ❌ **Run on separate machines**: Defeats purpose of fractal scaling validation
2. ❌ **Use VMs**: Adds unnecessary complexity for testing
3. ❌ **Disable singleton check**: Not possible without source code change

---

## 🚀 Request to Songbird Team

### **Priority**: 🔴 **CRITICAL**

This blocks:
- Multi-spore deployment validation
- Fractal scaling demonstration
- Production readiness certification

### **Recommended Fix**: **Option 2** (Socket-based singleton check)

**Rationale**:
- Simpler implementation (already have socket path logic)
- Natural singleton enforcement (OS-level)
- Clear error messages
- No additional lock files to manage

### **Timeline Request**
- **Hours** (if Option 2: socket-based check)
- **Days** (if Option 1: lock file approach)

---

## 📚 Related Issues

### **Previous Fixes**
- **v3.7.1-multispore**: Fixed socket path collision → Created `/tmp/songbird-{family}-{node}.sock`
- **v3.7.2-complete**: Added atomic readiness flag, improved test quality
- **v3.7.2-complete**: **Introduced singleton check (too aggressive)** ← THIS ISSUE

### **Why This Wasn't Caught in v3.7.2 Testing**
- Tests likely ran in isolation (single instance)
- Multi-instance tests may be missing from test suite

**Recommendation**: Add integration test for multi-instance deployment:
```rust
#[tokio::test]
async fn test_multiple_songbirds_different_nodes() {
    let songbird1 = spawn_songbird("nat0", "tower1").await;
    let songbird2 = spawn_songbird("nat0", "tower2").await;
    
    assert!(songbird1.is_healthy().await);
    assert!(songbird2.is_healthy().await);
    assert_ne!(songbird1.socket_path(), songbird2.socket_path());
}
```

---

## 🎊 biomeOS Status (Meanwhile)

### **What We Achieved**
- ✅ **Deep technical debt resolved**: Eliminated bash orchestration
- ✅ **Modern Rust patterns**: Pure async/await, TOML config
- ✅ **Clean env var passing**: No metadata pollution
- ✅ **Concurrent startup**: Wave-based orchestration
- ✅ **Production-ready** (for single-spore deployments)

### **What's Blocked**
- ⏳ **Multi-spore federation**: Waiting for Songbird v3.7.3
- ⏳ **Fractal scaling validation**: Waiting for Songbird v3.7.3

### **Current Status**
```
Tower: 2/2 working ✅
BearDog: 2/2 working ✅ (port-free architecture validated!)
Songbird: 1/2 working ⚠️ (blocked by singleton check)
```

---

## 📝 Summary for Songbird Team

**In One Sentence**:  
Songbird v3.7.2's singleton check prevents multiple instances with different `NODE_ID`s from running on the same machine, blocking multi-spore deployment—please scope the check per `NODE_ID` or use socket binding as the singleton enforcer.

**Binary Details**:
- Path: `/home/eastgate/Development/ecoPrimals/plasmidBin/songbird-orchestrator-v3.7.2-complete`
- SHA256: `fda781508a3a3b326266f9fa4d4c6ed0b2097b74a467f970e0ed091e1a73c150`
- Size: 25MB
- Version: v3.7.2-complete

**Error Message**:
```
Error: Another Songbird instance is already running (PID: 1316456)
```

**Expected Next Version**: v3.7.3 with multi-instance support! 🚀

---

**Status**: 🔴 **CRITICAL BUG**  
**Reporter**: biomeOS Team  
**Date**: January 4, 2026 17:11 EST  
**Urgency**: High (blocks fractal scaling validation)


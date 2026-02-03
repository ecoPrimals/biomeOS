# 🎯 PIXEL NODE ATOMIC - TCP Fallback Status
## Feb 1, 2026 - NODE Deployment Progress

**Date**: February 1, 2026  
**Platform**: Pixel 8a (GrapheneOS/Android)  
**Target**: NODE Atomic (TOWER + toadstool)  
**Status**: 🟡 **IN PROGRESS** - toadstool needs investigation

═══════════════════════════════════════════════════════════════════

## ✅ CURRENT STATUS

### **TOWER Atomic** ✅ **OPERATIONAL**

**beardog**:
- PID: 31020
- TCP Port: 33765
- Discovery: `/data/local/tmp/run/beardog-ipc-port` ✅
- Status: Running with TCP fallback

**songbird**:
- PID: 31159
- TCP Port: 36343
- Discovery: `/data/local/tmp/run/songbird-ipc-port` ✅  
- HTTP Port: 8080
- Status: Running with TCP fallback

**TOWER Grade**: ✅ **A++** (Fully operational)

═══════════════════════════════════════════════════════════════════

## 🟡 toadstool STATUS

### **Process Running** ✅

**toadstool**:
- PID: 31207
- Status: Running but IPC failed
- Error: "Permission denied (os error 13)"

---

### **Error Analysis** ⚠️

**Log Output**:
```
tarpc server binding to Unix socket: "/data/local/tmp/run/biomeos/toadstool.sock"
ERROR tarpc server error: Permission denied (os error 13)
ERROR JSON-RPC server error: Permission denied (os error 13)
```

**Issue**: Permission denied when binding Unix sockets

**Expected Behavior**: Should trigger TCP fallback (isomorphic pattern)

**Actual Behavior**: Server fails without falling back to TCP

---

### **Root Cause Analysis** 🔍

**toadstool Has Isomorphic TCP Fallback**:

**File**: `crates/runtime/display/src/ipc/server.rs`

**Pattern**:
```rust
pub async fn start(self: Arc<Self>) -> Result<()> {
    // 1. TRY Unix socket first
    match self.clone().try_unix_server().await {
        Ok(()) => Ok(()),
        
        // 2. DETECT platform constraints
        Err(e) if self.is_platform_constraint(&e) => {
            // 3. ADAPT to TCP fallback
            self.start_tcp_fallback().await
        }
        
        // 4. Real error
        Err(e) => Err(e)
    }
}
```

**Issue**: "Permission denied" error might not be recognized as a platform constraint!

**Hypothesis**: `is_platform_constraint()` needs to check for "Permission denied" on Android

═══════════════════════════════════════════════════════════════════

## 🎯 DIAGNOSIS

### **Error Type** ⚠️

**Error Code**: 13 (EACCES - Permission denied)

**Platform**: Android/SELinux

**Possible Causes**:
1. SELinux blocking socket creation
2. Directory permissions issue
3. App sandbox restrictions

**Expected Pattern**: This should trigger TCP fallback!

---

### **Comparison with beardog & songbird** ✅

**beardog Error**:
```
Failed to bind socket on Unix (filesystem): /data/local/tmp/run/biomeos/beardog.sock
```
**Result**: ✅ TCP fallback triggered

**songbird Error**:
```
Failed to bind Unix socket: /data/local/tmp/run/biomeos/songbird.sock
```
**Result**: ✅ TCP fallback triggered

**toadstool Error**:
```
Permission denied (os error 13)
```
**Result**: ❌ No TCP fallback

**Difference**: Error message format might not match the platform constraint check!

═══════════════════════════════════════════════════════════════════

## 🔧 POTENTIAL FIX

### **Option 1: Check `is_platform_constraint()` Implementation**

**File**: `crates/runtime/display/src/ipc/server.rs`

**Need to verify**: Does it check for "Permission denied" (error code 13)?

**Expected**:
```rust
fn is_platform_constraint(&self, error: &DisplayError) -> bool {
    let msg = error.to_string().to_lowercase();
    
    msg.contains("permission denied") ||  // ✅ This check!
    msg.contains("address already in use") ||
    msg.contains("not supported") ||
    msg.contains("protocol not available")
}
```

---

### **Option 2: Directory Permissions**

**Issue**: Maybe the directory itself has permission issues

**Test**:
```bash
adb shell "ls -la /data/local/tmp/run/biomeos/"
adb shell "touch /data/local/tmp/run/biomeos/test-toadstool.sock"
```

**If this fails**: Directory permissions need fixing

---

### **Option 3: Server Implementation Difference**

**toadstool Uses**: Two protocols (tarpc + JSON-RPC)

**beardog/songbird Use**: Single protocol

**Hypothesis**: Maybe the error happens in a different code path that doesn't check for platform constraints?

═══════════════════════════════════════════════════════════════════

## 🎯 NEXT STEPS

### **Immediate Investigation** (10 min)

**1. Check Directory Permissions**:
```bash
adb shell "ls -la /data/local/tmp/run/biomeos/"
adb shell "touch /data/local/tmp/run/biomeos/test.sock"
adb shell "rm /data/local/tmp/run/biomeos/test.sock"
```

**2. Check Error Code Handling**:
```bash
# Read is_platform_constraint implementation
grep -A 20 "is_platform_constraint" \
  /path/to/toadstool/crates/runtime/display/src/ipc/server.rs
```

**3. Test Manual TCP Mode**:
```bash
# If toadstool has a TCP-only flag, test it
adb shell "TOADSTOOL_TCP_ONLY=1 ./toadstool server"
```

---

### **Short Term Fix** (30 min)

**If `is_platform_constraint()` missing "permission denied"**:

**Add to check**:
```rust
fn is_platform_constraint(&self, error: &DisplayError) -> bool {
    let msg = error.to_string().to_lowercase();
    
    // Android/SELinux errors
    msg.contains("permission denied") ||     // os error 13
    msg.contains("operation not permitted") || // os error 1
    
    // Other platform errors
    msg.contains("address already in use") ||
    msg.contains("not supported") ||
    msg.contains("protocol not available") ||
    msg.contains("address family not supported")
}
```

**Build & Deploy**:
```bash
cd toadstool
cargo build --release --target aarch64-unknown-linux-musl
adb push target/aarch64-unknown-linux-musl/release/toadstool /data/local/tmp/
```

═══════════════════════════════════════════════════════════════════

## 📊 ATOMIC STATUS MATRIX

| Atomic | beardog | songbird | toadstool | Status | Grade |
|--------|---------|----------|-----------|--------|-------|
| **TOWER** | ✅ TCP:33765 | ✅ TCP:36343 | N/A | Operational | **A++** |
| **NODE** | ✅ TCP:33765 | ✅ TCP:36343 | 🟡 PID 31207 | **Blocked** | **75%** |

**Blocker**: toadstool TCP fallback not triggering

═══════════════════════════════════════════════════════════════════

## 🎯 WORKAROUND OPTIONS

### **Option A: Fix toadstool** (Preferred)
- Update `is_platform_constraint()` to recognize "permission denied"
- Rebuild and redeploy
- Time: 30 minutes

### **Option B: Test on Different Platform**
- Deploy NODE atomic to USB (where Unix sockets work)
- Validate full NODE functionality there
- Come back to Pixel after fix
- Time: 15 minutes

### **Option C: Use TCP-Only Mode** (If Available)
- Force toadstool to use TCP directly
- Skip Unix socket attempt entirely
- Time: 5 minutes

═══════════════════════════════════════════════════════════════════

## 📋 SUMMARY

**TOWER Atomic**: ✅ **OPERATIONAL ON PIXEL!**
- beardog + songbird both running with TCP fallback
- Discovery files created
- Grade: A++

**NODE Atomic**: 🟡 **75% COMPLETE**
- TOWER operational ✅
- toadstool process running ✅
- toadstool IPC failed (permission denied) ⏳
- TCP fallback not triggered ❌

**Recommendation**: Investigate `is_platform_constraint()` implementation in toadstool's display server to ensure "Permission denied" error triggers TCP fallback.

═══════════════════════════════════════════════════════════════════

**Created**: February 1, 2026  
**Status**: 🟡 **IN PROGRESS**  
**TOWER**: ✅ Operational  
**NODE**: 🟡 Needs toadstool fix

**Next**: Investigate toadstool error handling for "Permission denied"

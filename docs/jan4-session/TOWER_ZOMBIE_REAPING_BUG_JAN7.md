# 🐛 Tower Zombie Reaping Bug - Root Cause Analysis

**Date**: January 7, 2026  
**Component**: `biomeos-core/src/primal_impls.rs`  
**Severity**: 🔴 **CRITICAL** - Production blocker  
**Status**: ✅ **ROOT CAUSE IDENTIFIED**

---

## 🎯 The Bug

**Tower spawns child processes but NEVER reaps dead ones, creating zombies.**

### Code Location: `primal_impls.rs`

```rust
// Line 148-167: start() function
async fn start(&self) -> BiomeResult<()> {
    // ... setup code ...
    
    let child = cmd.spawn()?;  // ← Spawn child process
    
    *process_guard = Some(child);  // ← Store Child handle
    
    info!("✅ Primal {} process started", self.id);
    Ok(())
    // ← NEVER calls child.wait() to reap zombies!
}

// Line 187-209: health_check() function  
async fn health_check(&self) -> BiomeResult<HealthStatus> {
    let process_guard = self.process.lock().await;
    if process_guard.is_some() {  // ← WRONG! Just checks if handle exists
        Ok(HealthStatus::Healthy { /* ... */ })
    } else {
        Ok(HealthStatus::Unhealthy { /* ... */ })
    }
    // ← NEVER checks if process actually exited!
    // ← NEVER calls try_wait() to reap zombies!
}
```

### What's Wrong

1. **`start()`**: Spawns child, stores handle, but **never calls `.wait()`**
2. **`health_check()`**: Only checks if handle exists, **not if process is alive**
3. **Result**: When child exits, `Child` handle remains, zombie persists forever

---

## 🧟 Why Zombies Happen

### Normal Process Lifecycle
```
1. Parent spawns child
2. Child runs
3. Child exits
4. Child becomes zombie (waiting for parent to collect exit status)
5. Parent calls wait() → zombie is reaped
6. Process fully removed from process table
```

### What Tower Does (BROKEN)
```
1. Tower spawns Songbird
2. Songbird crashes immediately
3. Songbird becomes zombie
4. Tower NEVER calls wait()
5. Zombie persists FOREVER
6. New Songbird sees lock file → exits immediately → NEW zombie
7. Repeat infinitely...
```

---

## 🔍 Evidence

### Zombie Processes
```bash
$ ps aux | grep songbird
eastgate  145654  0.6  0.0      0     0 ?        ZN   16:22   [songbird] <defunct>
eastgate  145798  0.5  0.0      0     0 ?        ZN   16:22   [songbird] <defunct>
eastgate  146732  0.0  0.0      0     0 ?        ZN   16:23   [songbird] <defunct>
# ... 7 total zombies ...
```

### Parent Processes (Tower)
```bash
$ ps --ppid 145623  # Tower PID
  PID STAT COMMAND
145653 SNl  beardog-server    # ← Healthy
145654 ZN   songbird <defunct> # ← Zombie!
```

### Songbird Error
```
Error: Another Songbird instance with NODE_ID=nat0-node-alpha is already running (PID: 145654)
```
- PID 145654 is a **zombie** (already dead)
- But lock file remains, so new Songbird thinks it's running
- New Songbird exits, becomes zombie, repeat...

---

## 🔧 The Fix

### Solution: Use `try_wait()` in health_check

```rust
async fn health_check(&self) -> BiomeResult<HealthStatus> {
    let mut process_guard = self.process.lock().await;
    
    if let Some(child) = process_guard.as_mut() {
        // Check if process actually exited
        match child.try_wait() {
            Ok(Some(exit_status)) => {
                // Process exited! Reap the zombie
                info!("⚠️  Primal {} exited with status: {:?}", self.id, exit_status);
                *process_guard = None;  // Clear the handle
                
                Ok(HealthStatus::Unhealthy {
                    reason: format!("Primal {} exited: {:?}", self.id, exit_status),
                    since: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    consecutive_failures: 1,
                    recovery_attempts: 0,
                })
            },
            Ok(None) => {
                // Process still running
                Ok(HealthStatus::Healthy {
                    last_check: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    consecutive_successes: 1,
                })
            },
            Err(e) => {
                // Error checking status
                warn!("Failed to check process status for {}: {}", self.id, e);
                Ok(HealthStatus::Unhealthy {
                    reason: format!("Failed to check status: {}", e),
                    since: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
                    consecutive_failures: 1,
                    recovery_attempts: 0,
                })
            }
        }
    } else {
        // No process handle
        Ok(HealthStatus::Unhealthy {
            reason: format!("Primal {} not running", self.id),
            since: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            consecutive_failures: 1,
            recovery_attempts: 0,
        })
    }
}
```

### What This Does

1. **`try_wait()`**: Non-blocking check if child exited
2. **Reaps zombie**: If exited, collects exit status (reaps zombie)
3. **Updates state**: Clears handle, marks unhealthy
4. **Enables recovery**: Health monitor can restart the primal

---

## 🎯 Why This Fixes Everything

### Before (BROKEN)
```
Songbird exits → zombie
health_check() → "Healthy" (wrong!)
Tower keeps zombie forever
New deployment blocked
```

### After (FIXED)
```
Songbird exits → zombie
health_check() → try_wait() reaps zombie → "Unhealthy"
Health monitor can restart if configured
No zombies accumulate
New deployment works
```

---

## 🚀 Implementation Plan

1. ✅ **Identified root cause**
2. 🔧 **Fix `health_check()` to use `try_wait()`**
3. ✅ **Add logging for process exits**
4. ✅ **Clear handle when process exits**
5. 🧪 **Test with intentional crashes**
6. 📝 **Document zombie prevention**

---

## 🎊 Why This is Important

### Current Impact
- ❌ Cannot deploy without reboot
- ❌ Zombies accumulate infinitely
- ❌ Manual cleanup required
- ❌ Production blocker

### After Fix
- ✅ Automatic zombie reaping
- ✅ Health monitoring detects exits
- ✅ Auto-restart possible (if configured)
- ✅ No manual intervention
- ✅ Production ready

---

## 📚 Linux Process Internals

### What is a Zombie?
- Process that has exited but not been reaped
- Holds PID in process table
- Cannot be killed (already dead!)
- Only parent can reap via `wait()`

### Why Reaping is Critical
```c
// In C, normal parent behavior:
pid_t pid = fork();
if (pid == 0) {
    exec(...);  // Child runs
} else {
    wait(&status);  // Parent reaps ← THIS IS MANDATORY
}
```

In Rust:
```rust
let mut child = Command::new("binary").spawn()?;
child.wait()?;  // ← Or try_wait() for non-blocking
```

**Tower was missing this `wait()` call!**

---

## 🎯 Recommendation

**Implement the fix immediately!**

This is a fundamental process management bug that affects:
- All primal orchestration
- Production deployments
- Health monitoring accuracy
- System stability

**ETA**: 10 minutes to implement and test  
**Priority**: 🔴 **CRITICAL**  
**Complexity**: Low (simple fix, big impact)

---

**biomeOS can't be production-ready without proper zombie reaping!** 🐛 → ✅


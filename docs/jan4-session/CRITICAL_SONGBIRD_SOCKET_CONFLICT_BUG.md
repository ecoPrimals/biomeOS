# 🐛 CRITICAL BUG: Songbird Socket Conflict

**Date**: January 4, 2026  
**Status**: ⚠️ **BUG IDENTIFIED - NEEDS FIX**

---

## 🔍 Issue Summary

**Problem**: Spore 2's Songbird crashes immediately due to Unix socket conflict.

**Root Cause**: Songbird is hardcoding `/tmp/songbird.sock` instead of using a unique path like BearDog does (`/tmp/beardog-{family}-{node}.sock`).

---

## 📊 Evidence

### Process Count
```bash
Tower: 2 processes ✅ (one per spore)
BearDog: 2 processes ✅ (one per spore, each with ~20 threads)
Songbird: 1 process ❌ (only Spore 1! Spore 2 crashed!)
```

### Socket Conflict
```bash
$ ls -la /tmp/songbird.sock
srwxrwxr-x 1 eastgate eastgate 0 Jan  4 11:26 /tmp/songbird.sock

$ lsof /tmp/songbird.sock
COMMAND      PID     USER   FD   TYPE             DEVICE SIZE/OFF     NODE NAME
songbird 1238584 eastgate   12u  unix 0x0000000000000000      0t0 51615879 /tmp/songbird.sock
```

**Only Spore 1's Songbird (PID 1238584) has the socket!**

### Spore 2 Log
```
2026-01-04T16:27:07.792515Z  INFO ... ✅ Primal 7a349af1-d02b-45fc-ab60-074e7c30f607 process started
2026-01-04T16:27:07.792522Z  INFO ... ✅ Primal 7a349af1-d02b-45fc-ab60-074e7c30f607 is healthy and running
```

**Tower THINKS Songbird started successfully, but the process crashes immediately after due to socket conflict!**

---

## 🎯 Why This Breaks Federation

### Expected Architecture
```
Spore 1:
  BearDog → /tmp/beardog-nat0-tower1.sock ✅
  Songbird → /tmp/songbird-nat0-tower1.sock ❌ (should be, but isn't!)

Spore 2:
  BearDog → /tmp/beardog-nat0-tower2.sock ✅
  Songbird → /tmp/songbird-nat0-tower2.sock ❌ (should be, but crashes!)
```

### Actual Architecture (BROKEN)
```
Spore 1:
  BearDog → /tmp/beardog-nat0-tower1.sock ✅
  Songbird → /tmp/songbird.sock ✅ (hardcoded, no unique ID!)

Spore 2:
  BearDog → /tmp/beardog-nat0-tower2.sock ✅
  Songbird → /tmp/songbird.sock ❌ (conflicts with Spore 1, CRASHES!)
```

---

## 🐞 Root Cause Analysis

### BearDog (CORRECT Implementation)
BearDog v0.17.0 port-free uses:
```rust
// Socket path includes family + node ID
let socket_path = format!("/tmp/beardog-{}-{}.sock", family_id, node_id);
```

**Result**: Unique socket per instance → No conflicts ✅

### Songbird (BROKEN Implementation)
Songbird v3.8 is hardcoding:
```rust
// Hardcoded socket path, no unique ID!
let socket_path = "/tmp/songbird.sock";
```

**Result**: Same socket for all instances → Conflict → Second instance crashes ❌

---

## 💥 Impact

### What Works
- ✅ Single spore deployment (no conflicts)
- ✅ BearDog on both spores (unique sockets)
- ✅ Songbird on Spore 1 (first to claim the socket)

### What Breaks
- ❌ Dual spore federation (Spore 2's Songbird crashes)
- ❌ Multi-tower deployment (only first tower gets Songbird)
- ❌ Fractal scaling (limited to 1 Songbird per machine)

### Why We Didn't Catch It Earlier
1. **Silent failure**: Tower reports "started successfully" before Songbird crashes
2. **No error logs**: Songbird crashes too quickly to log the error
3. **Tests passed**: Unix socket IPC tests only used one instance
4. **Misleading success**: Spore 1 works perfectly, masking the issue

---

## ✅ Solution

### For Songbird Team
Update `songbird-orchestrator` to use unique socket paths:

```rust
// BEFORE (broken):
let socket_path = "/tmp/songbird.sock";

// AFTER (fixed):
let family_id = env::var("SONGBIRD_FAMILY_ID").unwrap_or_else(|_| "default".to_string());
let node_id = env::var("SONGBIRD_NODE_ID").unwrap_or_else(|_| hostname::get().unwrap().to_string_lossy().to_string());
let socket_path = format!("/tmp/songbird-{}-{}.sock", family_id, node_id);
```

### For biomeOS Team
Update `tower.toml` to pass `SONGBIRD_NODE_ID`:

```toml
[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]

[primals.env]
SONGBIRD_FAMILY_ID = "nat0"
SONGBIRD_NODE_ID = "tower1"  # ← ADD THIS!
RUST_LOG = "info"
```

---

## 🎯 Validation Plan

1. **Update Songbird** to use unique socket paths
2. **Update tower.toml** to pass `SONGBIRD_NODE_ID`
3. **Rebuild Songbird** binary
4. **Update USB spores** with new binary
5. **Kill and redeploy** both spores
6. **Verify**:
   ```bash
   ls -la /tmp/songbird*.sock
   # Expected:
   # /tmp/songbird-nat0-tower1.sock
   # /tmp/songbird-nat0-tower2.sock
   ```
7. **Test federation** with 2 running Songbirds

---

## 📊 Why This Is Critical

This bug **completely breaks** the Sovereign Primal Architecture's core promise:
- ❌ **Fractal Scaling**: Can't run multiple towers on same machine
- ❌ **Zero Conflicts**: Socket conflicts violate the design
- ❌ **Reproducibility**: Different behavior on Spore 1 vs Spore 2

**Priority**: 🔥 **CRITICAL** - Blocks multi-tower deployment

---

## 🎓 Key Insight

**BearDog got it right, Songbird needs to follow the same pattern!**

Every primal needs:
- Family ID (for family membership)
- Node ID (for unique identity)
- Unique socket path: `/tmp/{primal}-{family}-{node}.sock`

This is the **genetic lineage** pattern in action!

---

**Next Steps**: Hand off to Songbird team for immediate fix. 🚨


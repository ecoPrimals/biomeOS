# 🔍 Songbird v3.21.1 Issue Root Cause

**Date**: January 11, 2026 23:44  
**Status**: Issue Identified  

---

## 🎯 Issue

Songbird v3.21.1 binary fails with:
```
Error: Failed to create Unix socket server
Caused by: invalid socket address
```

---

## 🔬 Root Cause Analysis

### Code Location
`crates/songbird-orchestrator/src/ipc/server.rs` line 191:

```rust
let server = Server::builder()
    .build(self.socket_path.to_str().unwrap())  // ← Issue here
    .await
    .context("Failed to create Unix socket server")?;
```

### Problem
The `jsonrpsee::server::Server::builder().build()` method expects Unix socket paths in a specific format:

**Expected**: `unix:///run/user/1000/songbird-nat0.sock`  
**Provided**: `/run/user/1000/songbird-nat0.sock`

The missing `unix://` prefix causes "invalid socket address" error.

---

## ✅ Socket Configuration IS Correct

The socket path determination (lines 91-137) is **perfect**:
- ✅ `SONGBIRD_SOCKET` override working
- ✅ 3-tier fallback working
- ✅ XDG runtime directory working
- ✅ Directory creation working
- ✅ Stale socket removal working

**The socket config v3.21.1 is fully compliant!**

---

## 🔧 Solution

The Songbird team needs to update line 191 to:

```rust
// Prefix Unix socket path with "unix://" for jsonrpsee
let socket_uri = format!("unix://{}", self.socket_path.display());

let server = Server::builder()
    .build(&socket_uri)  // ← Use prefixed URI
    .await
    .context("Failed to create Unix socket server")?;
```

---

## 📊 Current Status

| Component | Status | Note |
|-----------|--------|------|
| **Socket Config** | ✅ PERFECT | v3.21.1 compliant |
| **Path Resolution** | ✅ PERFECT | All 3 tiers working |
| **Server Binding** | ❌ BROKEN | Needs `unix://` prefix |
| **BearDog** | ✅ WORKING | Successfully deployed |

**Fix Required**: 1 line change in Songbird  
**Complexity**: Trivial  
**Impact**: Unblocks all atomic deployment  

---

## 🎯 Workaround

For immediate testing, we can:
1. Use BearDog alone to verify it works (DONE ✅)
2. Wait for Songbird team to fix the URI format
3. Or modify Songbird ourselves locally for testing

---

**BearDog v0.16.1 proves the entire socket standardization works!**


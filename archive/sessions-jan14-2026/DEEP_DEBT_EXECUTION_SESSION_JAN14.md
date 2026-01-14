# 🚀 Deep Debt Execution Session - January 14, 2026

**Date**: January 14, 2026  
**Status**: ⚡ **IN PROGRESS**  
**Goal**: Execute on deep debt solutions - modern idiomatic Rust

---

## 🎯 Execution Priorities

### **Audit Results**

| Category | Count | Priority |
|----------|-------|----------|
| **HTTP references** | 85 files | 🚨 CRITICAL |
| **unsafe/todo!/unimplemented!/mock** | 378 matches in 56 files | 🔴 HIGH |
| **Mocks in production** | 43 in test-utils (✅), ~20 in production (🚨) | 🔴 HIGH |

---

## ⚡ Phase 1: HTTP → Unix Socket (IN PROGRESS)

### **biomeOS API Server Evolution**

**Current State** (Port 3000, HTTP):
```rust
// Line 250-251 in crates/biomeos-api/src/main.rs
let listener = tokio::net::TcpListener::bind(config.bind_addr).await?;
axum::serve(listener, app).await?;
```

**Target State** (Unix Socket, JSON-RPC):
```rust
// Unix socket path
let socket_path = format!("/run/user/{}/biomeos-api.sock", nix::unistd::getuid());

// Remove old socket
let _ = std::fs::remove_file(&socket_path);

// Bind Unix listener
let listener = tokio::net::UnixListener::bind(&socket_path)?;

// Set permissions (0600 - owner only)
#[cfg(unix)]
{
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&socket_path, fs::Permissions::from_mode(0o600))?;
}

// Serve over Unix socket
axum_unix_socket::serve(listener, app).await?;
```

**Changes Required**:
1. Add `axum-unix-socket` or custom Unix socket adapter
2. Update `Config` to use socket path instead of bind address
3. Update all log messages
4. Add temporary HTTP bridge (if PetalTongue needs it)
5. Update integration tests

**Estimated**: 4-6 hours

---

## 📊 Deep Debt Audit

### **1. unsafe Code** (Priority: HIGH)

**Found**: Minimal usage (good!)

Most are in tests or unavoidable syscalls. Need to audit each one.

**Action**: Audit and document justification for each `unsafe` block

---

### **2. todo!/unimplemented!** (Priority: HIGH)

**Found**: Many in production code

**Examples**:
- `crates/biomeos-ui/src/suggestions.rs` - 2 instances
- `crates/biomeos-api/src/handlers/topology.rs` - 1 instance  
- `crates/biomeos-graph/src/executor.rs` - 12 instances

**Action**: Replace with proper implementations or feature flags

---

### **3. Mocks in Production** (Priority: CRITICAL)

**Found**: 20+ instances outside test-utils

**Action**: Evolve to real implementations using runtime discovery

---

## 🎊 Completed Deep Debt (Previous Sessions)

### ✅ **Genetic Lineage**
- Verified in BearDog v0.16.1
- Verified in Songbird v3.22.0
- Production ready!

### ✅ **atomic-deploy Evolution**
- Evolved from hardcoded launcher to discovery-based orchestrator
- Created `primal_discovery.rs` (socket scanning)
- Created `primal_coordinator.rs` (TRUE PRIMAL orchestrator)

### ✅ **Hardcoding Elimination** (Jan 13)
- Eliminated primal name hardcoding
- Evolved to environment-based discovery
- Port/localhost hardcoding reduced

### ✅ **Client Module** (Jan 13)
- Fixed all compilation errors
- Implemented PrimalClient trait
- Updated all primal clients

---

## 🔧 Current Work

**Starting**: biomeOS API Unix socket evolution

**Files to Modify**:
1. `crates/biomeos-api/Cargo.toml` - Add dependencies
2. `crates/biomeos-api/src/main.rs` - Unix socket server
3. `crates/biomeos-api/src/state.rs` - Update Config
4. `crates/biomeos-api/tests/*.rs` - Update tests

**Branch Strategy**: Create feature branch for Unix socket work

---

**Created**: January 14, 2026  
**Status**: ⚡ EXECUTING  
**Next Update**: After Unix socket implementation


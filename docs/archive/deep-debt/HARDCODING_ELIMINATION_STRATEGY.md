# 🎯 Hardcoding Elimination Strategy

**Date:** January 30, 2026  
**Status:** IN PROGRESS  
**Philosophy:** Runtime discovery, zero assumptions, platform-agnostic

---

## 🔍 **Analysis Complete**

### **Hardcoding Patterns Found**

**Category 1: Hardcoded Paths (HIGH PRIORITY)**
- `/tmp/{primal}.sock` - 12+ instances
- `/run/user/{uid}/` - 6+ instances  
- `/var/run/biomeos/` - 2+ instances
- Absolute filesystem paths

**Category 2: Hardcoded Network (MEDIUM PRIORITY)**
- `localhost` / `127.0.0.1` - 15+ instances
- Hardcoded ports (development/testing)

**Category 3: Discovery Mechanism (COMPLETE!)**
- ✅ `socket_discovery.rs` already implements runtime discovery!
- ⚠️ Not being used everywhere yet

---

## ✅ **GOOD NEWS: Solution Already Exists!**

The `biomeos-core/src/socket_discovery.rs` module is a **complete deep debt solution**:

```rust
/// Socket Discovery - Capability-Based Runtime Discovery
///
/// **Deep Debt Solution**: Replaces hardcoded `/tmp/{primal}.sock` paths
/// 
/// Discovery Order:
/// 1. Environment variable hint (e.g., `BEARDOG_SOCKET`)
/// 2. XDG_RUNTIME_DIR (e.g., `/run/user/1000/biomeos/beardog-nat0.sock`)
/// 3. Family-scoped /tmp (e.g., `/tmp/beardog-nat0.sock`)
/// 4. Capability registry query via Neural API
```

**This is EXACTLY the platform-agnostic discovery we need!**

---

## 📋 **Execution Plan**

### **Phase 1: Adopt Existing Solution**

Replace hardcoded paths with `SocketDiscovery`:

**Files to Update:**

1. **`atomic_client.rs`** (Lines 239-256)
   ```rust
   // OLD (hardcoded)
   let candidates = vec![
       format!("/tmp/{}.sock", primal_lower),
       format!("/tmp/{}-server.sock", primal_lower),
   ];
   
   // NEW (discovery)
   let discovery = SocketDiscovery::new(family_id);
   let socket = discovery.discover_primal(primal_name).await?;
   ```

2. **`primal_orchestrator.rs`** (Line 152)
   - Replace hardcoded Unix socket parsing with discovery

3. **`primal_impls.rs`** (Lines 118-175)
   - Replace `/tmp/primals/` fallback with proper discovery

4. **`deployment_mode.rs`** (Lines 159, 359)
   - Replace hardcoded `/run/user/` and `/tmp/biomeos`
   - Use `SocketDiscovery::xdg_runtime_dir()` or equivalent

5. **`capability_registry.rs`** (Lines 588, 608 - tests)
   - Update test fixtures to use discovery

### **Phase 2: Network Address Discovery**

**Localhost Usage Analysis:**

**Acceptable (Tests/Development):**
- Test fixtures using `localhost` - OK
- Development fallbacks with warnings - OK  
- Examples/documentation - OK

**Must Fix (Production):**
- Production code using hardcoded `localhost` - NOT OK
- Missing environment variable fallbacks - NOT OK

**Strategy:**
```rust
// OLD (hardcoded)
let endpoint = "http://localhost:8900";

// NEW (discoverable)
let endpoint = env::var("PRIMAL_ENDPOINT")
    .or_else(|_| discover_primal_endpoint("primal-name"))
    .unwrap_or_else(|_| {
        warn!("Using localhost fallback - set PRIMAL_ENDPOINT for production");
        "http://localhost:8900".to_string()
    });
```

### **Phase 3: Complete Platform Agnostic**

**Integration with TRUE ecoBin v2.0:**

```rust
// Use platform-agnostic discovery
let discovery = PlatformSocketDiscovery::new();
let transport = discovery.discover_transport("primal-name").await?;

match transport {
    Transport::UnixSocket { path } => { /* Linux, macOS */ },
    Transport::AbstractSocket { name } => { /* Android */ },
    Transport::NamedPipe { name } => { /* Windows */ },
    Transport::Tcp { host, port } => { /* Universal fallback */ },
}
```

---

## 🎯 **Implementation**

### **Step 1: Update atomic_client.rs**

Current hardcoded discovery:
```rust
pub fn discover_primal(primal_name: &str) -> Option<PathBuf> {
    let candidates = vec![
        format!("/tmp/{}.sock", primal_lower),
        format!("/tmp/{}-server.sock", primal_lower),
        format!("/var/run/biomeos/{}.sock", primal_lower),
        format!("/run/biomeos/{}.sock", primal_lower),
    ];
    // Check each candidate...
}
```

**Evolution:**
```rust
pub async fn discover_primal(
    primal_name: &str,
    family_id: &str
) -> Result<PathBuf> {
    let discovery = SocketDiscovery::new(family_id);
    let socket = discovery.discover_primal(primal_name).await?;
    Ok(socket.path)
}
```

**Benefits:**
- ✅ Uses environment hints first
- ✅ Respects XDG_RUNTIME_DIR
- ✅ Family-namespaced paths
- ✅ Capability-based fallback
- ✅ Platform-agnostic

---

## 📊 **Impact Assessment**

### **Files Requiring Updates**

**Critical (Production Code):**
1. `atomic_client.rs` - Primary IPC client
2. `primal_orchestrator.rs` - Primal lifecycle
3. `primal_impls.rs` - Primal implementations
4. `deployment_mode.rs` - Runtime path detection
5. `config_builder.rs` - Configuration defaults

**Medium (Tests/Dev):**
6. `capability_registry.rs` - Test fixtures
7. `adaptive_client.rs` - Test fixtures
8. `primal_adapter/tests_extended.rs` - Test fixtures

**Low (Already Using Discovery):**
- `socket_discovery.rs` ✅ - Already correct!
- Tests that explicitly test hardcoded behavior

---

## ✅ **Success Criteria**

**Zero Hardcoded Paths:**
- [ ] No `/tmp/` in production code
- [ ] No `/run/user/` in production code
- [ ] No `/var/run/` in production code
- [ ] All use `SocketDiscovery` or equivalent

**Environment-First:**
- [ ] Environment variables checked first
- [ ] Clear warnings when using fallbacks
- [ ] Production rejects hardcoded localhost

**Platform-Agnostic:**
- [ ] Works on Linux (Unix sockets)
- [ ] Works on Android (abstract sockets - future)
- [ ] Works on Windows (named pipes - future)
- [ ] XDG-compliant on all platforms

**Capability-Based:**
- [ ] Discover by capability, not name
- [ ] No hardcoded primal names
- [ ] Runtime service discovery

---

## 🔥 **The Evolution**

### **Before (Hardcoded)**

```rust
// Assumes /tmp always exists and is writable
let socket = PathBuf::from("/tmp/beardog.sock");

// Assumes localhost is always the right address
let endpoint = "http://localhost:8080";

// Assumes /run/user exists
let runtime_dir = format!("/run/user/{}", uid);
```

**Problems:**
- ❌ Breaks on Android (no /tmp filesystem access)
- ❌ Breaks on Windows (no Unix sockets)
- ❌ Not configurable via environment
- ❌ Not family-namespaced (conflicts)
- ❌ Platform-specific assumptions

### **After (Discovery)**

```rust
// Discovers at runtime, respects environment
let discovery = SocketDiscovery::new(family_id);
let socket = discovery.discover_primal("beardog").await?;

// Or by capability (better!)
let crypto_provider = discovery.discover_capability("crypto").await?;

// Platform-agnostic path detection
let runtime_dir = SocketDiscovery::platform_runtime_dir()?;
```

**Benefits:**
- ✅ Works on Linux, Android, Windows (with appropriate transport)
- ✅ Respects environment variables
- ✅ XDG-compliant
- ✅ Family-namespaced (no conflicts)
- ✅ Capability-based (no hardcoded names)
- ✅ Platform-agnostic

---

## 📝 **Next Steps**

1. ✅ Analysis complete
2. ✅ Solution identified (`SocketDiscovery` exists!)
3. ⏭️ Update `atomic_client.rs` to use `SocketDiscovery`
4. ⏭️ Update `primal_orchestrator.rs` 
5. ⏭️ Update `primal_impls.rs`
6. ⏭️ Update `deployment_mode.rs`
7. ⏭️ Update test fixtures
8. ⏭️ Integration tests
9. ⏭️ Validate on Linux, Android (Termux), Windows (WSL)

---

**Created:** January 30, 2026  
**Status:** Strategy complete, implementation starting  
**Goal:** Zero hardcoded paths, 100% runtime discovery

🎯🦀✨ **TRUE PRIMAL - Runtime Discovery, Zero Assumptions!** ✨🦀🎯

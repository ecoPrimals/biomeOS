# 🧬 Primal Socket Configuration Issues - Handoff

**Date**: January 11, 2026  
**Priority**: HIGH - Blocking atomic deployment  
**Status**: 🔴 Requires primal team fixes  

---

## 🎯 **Executive Summary**

During atomic deployment testing, we discovered that primals are **not consistently respecting socket configuration**. This blocks the capability-based atomic architecture (Tower, Node, Nest).

**Impact**: Cannot deploy atomics live until primal socket handling is standardized.

---

## 🔴 **Critical Issues**

### **Issue 1: BearDog Ignores Socket Environment Variable**

**Observed Behavior**:
```bash
# We set:
BEARDOG_SOCKET=/run/user/1000/beardog-nat0.sock

# BearDog creates:
/tmp/beardog-nat0-default.sock
```

**Expected Behavior**:
- Respect `BEARDOG_SOCKET` environment variable
- Create socket at specified path
- Support XDG runtime directory (`/run/user/<uid>/`)

**Logs**:
```
🔌 Step 4: Configuring Unix Socket IPC...
   Socket Path: /tmp/beardog-nat0-default.sock  ← Hardcoded!
   Family ID: nat0
   Node ID: default
```

**Fix Required** (BearDog team):
1. Check for `BEARDOG_SOCKET` env var first
2. Fall back to `/run/user/<uid>/beardog-<family>.sock` if not set
3. Only use `/tmp/` as last resort (less secure)

---

### **Issue 2: Songbird Cannot Bind to XDG Socket Path**

**Observed Behavior**:
```bash
# We set:
SONGBIRD_SOCKET=/run/user/1000/songbird-nat0.sock

# Songbird fails:
Error: Failed to create Unix socket server
Caused by:
    invalid socket address
```

**Expected Behavior**:
- Bind to XDG-compliant socket path
- Create parent directories if needed
- Support `/run/user/<uid>/` paths

**Fix Required** (Songbird team):
1. Ensure socket path parent directories exist
2. Call `std::fs::create_dir_all()` for socket directory
3. Remove old socket file if it exists before binding
4. Support full absolute paths (not just filenames)

**Code Pattern** (Rust):
```rust
use std::path::Path;
use std::fs;

let socket_path = std::env::var("SONGBIRD_SOCKET")
    .unwrap_or_else(|_| format!("/run/user/{}/songbird-{}.sock", uid, family_id));

// Ensure parent directory exists
if let Some(parent) = Path::new(&socket_path).parent() {
    fs::create_dir_all(parent)?;
}

// Remove old socket if exists
let _ = fs::remove_file(&socket_path);

// Now bind
let listener = UnixListener::bind(&socket_path)?;
```

---

### **Issue 3: ToadStool Likely Has Same Issue**

**Not Yet Tested** but likely similar to BearDog:
- May ignore environment variables
- May use hardcoded `/tmp/` paths
- May not support XDG runtime directory

**Required** (ToadStool team):
- Add `TOADSTOOL_SOCKET` environment variable support
- Default to `/run/user/<uid>/toadstool-<family>.sock`
- Test with biomeOS launcher

---

### **Issue 4: NestGate Requires `service start` Command**

**Current** (works but inconsistent):
```bash
nestgate service start
```

**Preferred** (consistent with other primals):
```bash
# Just run the binary, configure via env vars
NESTGATE_SOCKET=/run/user/1000/nestgate-nat0.sock \
NESTGATE_FAMILY_ID=nat0 \
nestgate
```

**Fix Required** (NestGate team):
- Support running without `service start` subcommand
- Make `service start` optional (backward compat)
- Default behavior: start server immediately

---

## 📊 **Primal Socket Configuration Matrix**

| Primal | Env Var Support | XDG Compliant | Issues |
|--------|-----------------|---------------|--------|
| **BearDog** | ❌ Ignores | ❌ Uses `/tmp/` | Hardcoded path |
| **Songbird** | ❓ Unclear | ❌ Fails to bind | Invalid socket address |
| **ToadStool** | ❓ Untested | ❓ Untested | Unknown |
| **NestGate** | ✅ Yes | ✅ Yes | Requires `service start` |
| **Squirrel** | ❓ Untested | ❓ Untested | Unknown |

**Target**: All ✅✅

---

## 🎯 **Standardized Socket Configuration**

### **Environment Variables** (All Primals)

```bash
# Socket path (absolute)
<PRIMAL>_SOCKET=/run/user/<uid>/<primal>-<family>.sock

# Family ID
<PRIMAL>_FAMILY_ID=<family>

# Optional: Node ID (for multi-instance)
<PRIMAL>_NODE_ID=<node>
```

### **Fallback Logic** (Priority Order)

1. **`<PRIMAL>_SOCKET`** env var (highest priority)
2. **XDG Runtime Directory**: `/run/user/<uid>/<primal>-<family>.sock`
3. **Temp Directory** (last resort): `/tmp/<primal>-<family>-<node>.sock`

### **Path Construction Rules**

```rust
fn get_socket_path(primal_name: &str) -> String {
    // 1. Check env var
    if let Ok(path) = std::env::var(&format!("{}_SOCKET", primal_name.to_uppercase())) {
        return path;
    }
    
    // 2. XDG runtime directory
    let uid = nix::unistd::getuid();
    let family_id = std::env::var(&format!("{}_FAMILY_ID", primal_name.to_uppercase()))
        .unwrap_or_else(|_| "default".to_string());
    
    let xdg_path = format!("/run/user/{}/{}-{}.sock", uid, primal_name, family_id);
    if Path::new(&format!("/run/user/{}", uid)).exists() {
        return xdg_path;
    }
    
    // 3. Fallback to /tmp
    let node_id = std::env::var(&format!("{}_NODE_ID", primal_name.to_uppercase()))
        .unwrap_or_else(|_| "default".to_string());
    format!("/tmp/{}-{}-{}.sock", primal_name, family_id, node_id)
}
```

---

## 🚀 **Testing Requirements**

Each primal team should verify:

### **Test 1: Environment Variable Override**
```bash
export <PRIMAL>_SOCKET=/tmp/test-socket.sock
export <PRIMAL>_FAMILY_ID=test0
./<primal>

# Verify socket exists at /tmp/test-socket.sock
ls -lh /tmp/test-socket.sock
```

### **Test 2: XDG Runtime Directory**
```bash
export <PRIMAL>_FAMILY_ID=xdg0
./<primal>

# Verify socket exists at /run/user/<uid>/<primal>-xdg0.sock
ls -lh /run/user/$(id -u)/<primal>-xdg0.sock
```

### **Test 3: Fallback to /tmp**
```bash
# No env vars set
./<primal>

# Verify socket exists somewhere in /tmp
ls -lh /tmp/<primal>-*
```

### **Test 4: Socket Cleanup**
```bash
# Create old socket
touch /tmp/test-socket.sock

# Start primal
export <PRIMAL>_SOCKET=/tmp/test-socket.sock
./<primal>

# Should remove old socket and create new one (no "address already in use" error)
```

---

## 📚 **Implementation Checklist**

### **For Each Primal Team**:

- [ ] Add `<PRIMAL>_SOCKET` environment variable support
- [ ] Add `<PRIMAL>_FAMILY_ID` environment variable support
- [ ] Add `<PRIMAL>_NODE_ID` environment variable support (optional)
- [ ] Implement 3-tier fallback logic (env var → XDG → /tmp)
- [ ] Create parent directories for socket path (`create_dir_all`)
- [ ] Remove old socket file before binding
- [ ] Test all 4 scenarios above
- [ ] Document socket configuration in primal's README
- [ ] Update any hardcoded paths to use dynamic configuration

---

## 🎊 **Why This Matters**

### **Atomic Architecture Depends On This**

```
Tower  = BearDog + Songbird
Node   = BearDog + Songbird + ToadStool
Nest   = BearDog + Songbird + NestGate
NUCLEUS = Tower + Node + Nest
```

**Each atomic needs**:
- Unique socket paths per instance
- Capability-based discovery
- No hardcoded endpoints

**Without standardized sockets**:
- Cannot deploy multiple atomics on same machine
- Cannot test Tower ↔ Tower interactions
- Cannot verify genetic lineage connections
- Cannot federate nodes

---

## 🔗 **Related Documents**

- `BIOMEOS_ATOMICS_ARCHITECTURE.md` - Atomic architecture
- `PRIMAL_LAUNCHER_README.md` - Launcher implementation
- `NICHE_DEPLOYMENT_STATUS.md` - Current deployment status

---

## 📞 **Contact**

**biomeOS Team**: Ready to test as soon as fixes are deployed  
**Timeline**: Please prioritize - blocking production deployment  
**Testing**: We have launcher ready to test immediately after fixes  

---

**Different orders of the same architecture.** 🍄🐸

**Let's get these sockets standardized so we can deploy live atomics!** 🦀



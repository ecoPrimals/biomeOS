# 🤝 Handoff: BearDog Socket Path Standardization

**To:** BearDog Team  
**From:** biomeOS Integration Team  
**Date:** January 30, 2026  
**Priority:** HIGH  
**Issue:** Socket path discovery mismatch blocking NUCLEUS integration

---

## 🎯 **Summary**

During NUCLEUS integration testing, we discovered that biomeOS cannot reliably discover BearDog's Unix socket. This blocks all Tower Atomic deployments and cross-primal communication.

**Root Cause:** BearDog's socket location doesn't match biomeOS discovery expectations.

**Solution:** Standardize BearDog socket creation to use XDG-compliant paths.

---

## 📊 **Current Situation**

### **biomeOS Discovery Logic**

biomeOS looks for BearDog socket in this order:

1. `$BEARDOG_SOCKET` environment variable
2. `/run/user/$UID/biomeos/beardog.sock` (XDG-compliant)
3. `/tmp/beardog-*.sock` (fallback scan)

### **What We Observed**

During testing:
- ✅ BearDog process started successfully (PID confirmed)
- ❌ Socket not found at expected XDG location
- ❌ Songbird couldn't connect to BearDog for TLS operations

**Error:**
```
BearDog RPC error: Failed to connect to BearDog at /tmp/neural-api-nat0.sock: 
No such file or directory (os error 2)
```

---

## 💡 **Proposed Solution**

### **Socket Path Standard**

BearDog should create its Unix socket at:

```
/run/user/$UID/biomeos/beardog.sock
```

Where:
- `$UID` = User ID (e.g., 1000 for first user)
- `biomeos` = Shared directory for all biomeOS primals
- `beardog.sock` = Consistent, discoverable name

### **Implementation Example**

```rust
use std::env;
use std::path::PathBuf;

fn get_socket_path() -> PathBuf {
    // 1. Check environment variable (highest priority)
    if let Ok(socket) = env::var("BEARDOG_SOCKET") {
        return PathBuf::from(socket);
    }
    
    // 2. Check BIOMEOS_SOCKET_DIR (shared standard)
    let socket_dir = env::var("BIOMEOS_SOCKET_DIR")
        .unwrap_or_else(|_| {
            // 3. Default to XDG-compliant runtime directory
            let uid = unsafe { libc::getuid() };
            format!("/run/user/{}/biomeos", uid)
        });
    
    // Ensure directory exists
    if let Err(e) = std::fs::create_dir_all(&socket_dir) {
        eprintln!("⚠️  Failed to create socket directory: {}", e);
    }
    
    PathBuf::from(socket_dir).join("beardog.sock")
}

// In BearDog startup:
fn main() -> Result<()> {
    let socket_path = get_socket_path();
    
    info!("🐻 BearDog starting...");
    info!("   Socket: {}", socket_path.display());
    
    // Create Unix socket listener
    let listener = UnixListener::bind(&socket_path)?;
    
    info!("✅ BearDog ready at {}", socket_path.display());
    
    // ... rest of server logic
}
```

---

## 🔧 **Changes Needed**

### **1. Socket Path Configuration**

Add support for `BIOMEOS_SOCKET_DIR` environment variable alongside existing `BEARDOG_SOCKET`.

**Priority:** HIGH  
**Effort:** 30 minutes  
**Location:** Socket initialization code

### **2. Startup Logging**

Log the actual socket path during startup for debugging.

**Priority:** MEDIUM  
**Effort:** 5 minutes

**Example:**
```rust
info!("✅ BearDog ready!");
info!("   Socket: {}", socket_path.display());
info!("   Family: {}", family_id);
info!("   PID: {}", std::process::id());
```

### **3. Documentation**

Update BearDog README with socket configuration details.

**Priority:** MEDIUM  
**Effort:** 15 minutes

**Sections to add:**
- Socket Configuration
- Environment Variables (`BEARDOG_SOCKET`, `BIOMEOS_SOCKET_DIR`)
- XDG Compliance

---

## ✅ **Success Criteria**

After implementation:

1. **Socket Creation**
   ```bash
   $ ./beardog server
   # Creates: /run/user/1000/biomeos/beardog.sock
   ```

2. **Environment Variable Override**
   ```bash
   $ BEARDOG_SOCKET=/tmp/test.sock ./beardog server
   # Creates: /tmp/test.sock
   ```

3. **Shared Directory Support**
   ```bash
   $ BIOMEOS_SOCKET_DIR=/custom/path ./beardog server
   # Creates: /custom/path/beardog.sock
   ```

4. **Discoverable by biomeOS**
   ```bash
   $ ls /run/user/$(id -u)/biomeos/beardog.sock
   # Should exist and be accessible
   ```

5. **Cross-Primal Connection**
   - Songbird can connect to BearDog for TLS operations
   - biomeOS Nucleus can initialize Identity layer
   - Integration tests pass

---

## 🧪 **Testing**

### **Manual Test**

```bash
# 1. Start BearDog
cd beardog
cargo run --release -- server

# 2. Verify socket exists
ls -la /run/user/$(id -u)/biomeos/beardog.sock

# 3. Test connection
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/beardog.sock

# Expected: {"jsonrpc":"2.0","result":{"status":"healthy"},"id":1}
```

### **Integration Test**

```bash
# From biomeOS:
cd ../biomeOS
./scripts/quick_start_nucleus_test.sh

# Should now succeed at Tower Atomic deployment phase
```

---

## 📚 **Background: Why XDG?**

### **XDG Base Directory Specification**

The XDG spec defines standard locations for user-specific data:
- **Runtime files**: `/run/user/$UID/` (sockets, PIDs, temp files)
- **Config files**: `~/.config/`
- **Data files**: `~/.local/share/`

**Benefits:**
1. ✅ Predictable locations (no scanning needed)
2. ✅ Per-user isolation (multi-user systems)
3. ✅ Automatic cleanup on logout
4. ✅ Standards-compliant (used by systemd, Wayland, etc.)

### **Why `/biomeos/` Subdirectory?**

Groups all biomeOS primal sockets together:
```
/run/user/1000/
├── biomeos/
│   ├── beardog.sock
│   ├── songbird.sock
│   ├── nestgate.sock
│   ├── toadstool.sock
│   └── squirrel.sock
├── pulse/  (PulseAudio)
├── systemd/  (systemd user services)
└── ...
```

**Benefits:**
- Easy discovery (one directory to scan)
- Clear ownership (all biomeOS-related)
- Permissions isolation (700 on directory)

---

## 🤔 **FAQ**

### **Q: What if `/run/user/$UID/` doesn't exist?**

**A:** Fallback to `/tmp/beardog.sock` or `/tmp/biomeos/beardog.sock`.

Most modern Linux systems create this automatically, but portable fallback is good practice.

### **Q: Do we need to change existing deployments?**

**A:** No immediate changes required. The `BEARDOG_SOCKET` env var will continue to work.

New deployments should use the standard location, and we can migrate gradually.

### **Q: What about Docker/containers?**

**A:** Mount a shared volume:
```bash
docker run -v /tmp/biomeos:/run/user/1000/biomeos beardog
```

Or set `BEARDOG_SOCKET` to container-accessible path.

### **Q: Performance impact?**

**A:** None. Unix socket performance is identical regardless of filesystem location.

`/run/user/` is actually faster (tmpfs) than regular filesystems.

---

## 📞 **Contact**

**Questions?** Reach out via:
- biomeOS integration channel
- File issue in biomeOS repo with `[beardog]` tag
- Direct message biomeOS integration lead

**Timeline:**
- **Immediate:** biomeOS workaround deployed (env vars set)
- **This week:** BearDog implements socket standard
- **Next sprint:** Remove workarounds, test production

---

## 🎯 **Action Items**

- [ ] **BearDog Team**: Implement `get_socket_path()` function
- [ ] **BearDog Team**: Add `BIOMEOS_SOCKET_DIR` support
- [ ] **BearDog Team**: Update startup logging
- [ ] **BearDog Team**: Update README documentation
- [ ] **BearDog Team**: Test with biomeOS integration
- [ ] **biomeOS Team**: Verify BearDog discovery works
- [ ] **biomeOS Team**: Update integration tests
- [ ] **Both Teams**: Document socket standard in ecosystem guide

---

## 🙏 **Thank You!**

This is a simple change with huge impact. Standardizing socket paths will:
- ✅ Unblock NUCLEUS integration
- ✅ Simplify deployment
- ✅ Improve debugging
- ✅ Enable production readiness

We appreciate your collaboration in making the biomeOS ecosystem more robust!

**🦀✨ Together we build TRUE PRIMAL architecture! ✨🦀**

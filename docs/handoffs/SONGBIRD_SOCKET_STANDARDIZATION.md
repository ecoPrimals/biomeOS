# 🤝 Handoff: Songbird Socket Path Standardization

**To:** Songbird Team  
**From:** biomeOS Integration Team  
**Date:** January 30, 2026  
**Priority:** HIGH  
**Issue:** Socket discovery working but name inconsistency detected

---

## 🎯 **Summary**

During NUCLEUS integration testing, Songbird was successfully discovered at `/run/user/1000/biomeos/songbird.sock` ✅

However, biomeOS discovery logic was initially looking for `songbird-orchestrator.sock` (binary name) instead of `songbird.sock` (primal name).

**Status:** ✅ biomeOS discovery fixed  
**Action:** Confirm Songbird follows socket naming standard

---

## 📊 **Current Situation**

### **What's Working** ✅

- Songbird creates socket at: `/run/user/1000/biomeos/songbird.sock`
- biomeOS discovers it successfully (after fix)
- Squirrel integration test found Songbird
- Health checks pass

### **What Needs Confirmation**

Does Songbird consistently use:
1. Socket directory: `/run/user/$UID/biomeos/`
2. Socket name: `songbird.sock` (not `songbird-orchestrator.sock`)
3. Environment variable support: `SONGBIRD_SOCKET`, `BIOMEOS_SOCKET_DIR`

---

## 💡 **Socket Path Standard**

### **Expected Location**

```
/run/user/$UID/biomeos/songbird.sock
```

### **Implementation (if not already done)**

```rust
use std::env;
use std::path::PathBuf;

fn get_socket_path() -> PathBuf {
    // 1. Check environment variable
    if let Ok(socket) = env::var("SONGBIRD_SOCKET") {
        return PathBuf::from(socket);
    }
    
    // 2. Check shared socket directory
    let socket_dir = env::var("BIOMEOS_SOCKET_DIR")
        .unwrap_or_else(|_| {
            let uid = unsafe { libc::getuid() };
            format!("/run/user/{}/biomeos", uid)
        });
    
    std::fs::create_dir_all(&socket_dir).ok();
    
    PathBuf::from(socket_dir).join("songbird.sock")
}
```

---

## ✅ **Success Criteria**

1. **Socket created at standard location:**
   ```bash
   $ ./songbird server
   # Creates: /run/user/1000/biomeos/songbird.sock
   ```

2. **Consistent naming:**
   - ✅ `songbird.sock` (primal name)
   - ❌ NOT `songbird-orchestrator.sock` (binary name)

3. **Environment variable support:**
   ```bash
   $ SONGBIRD_SOCKET=/tmp/test.sock ./songbird server
   # Creates: /tmp/test.sock
   ```

4. **Startup logging:**
   ```bash
   ✅ Songbird ready!
      Socket: /run/user/1000/biomeos/songbird.sock
      Family: nat0
      Capabilities: http, discovery, secure_http
   ```

---

## 🧪 **Testing**

### **Quick Verification**

```bash
# 1. Start Songbird
./songbird server

# 2. Check socket location
ls -la /run/user/$(id -u)/biomeos/songbird.sock

# 3. Test health endpoint
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird.sock
```

### **Integration with biomeOS**

```bash
# Should now pass Tower Atomic deployment:
cd ../biomeOS
./scripts/quick_start_nucleus_test.sh
```

---

## 📋 **Action Items**

- [ ] **Songbird Team**: Confirm socket path logic
- [ ] **Songbird Team**: Verify socket name is `songbird.sock`
- [ ] **Songbird Team**: Add startup logging for socket path
- [ ] **Songbird Team**: Update README with socket configuration
- [ ] **biomeOS Team**: Document Songbird socket standard
- [ ] **Both Teams**: Test cross-primal discovery (Songbird ↔ BearDog)

---

## 🎉 **Great Job!**

Songbird is already very close to (or meeting) the standard! This handoff is primarily for confirmation and documentation.

**Thank you for maintaining XDG compliance!** 🦀✨

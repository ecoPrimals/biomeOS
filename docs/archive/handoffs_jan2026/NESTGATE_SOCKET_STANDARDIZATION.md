# 🤝 Handoff: NestGate Socket Path Standardization

**To:** NestGate Team  
**From:** biomeOS Integration Team  
**Date:** January 30, 2026  
**Priority:** HIGH  
**Issue:** Socket not visible during NUCLEUS integration testing

---

## 🎯 **Summary**

During NUCLEUS integration testing:
- ✅ NestGate process started successfully (PID confirmed)
- ❌ Socket not found at expected XDG location
- ❌ Cannot test model persistence functionality

**Root Cause:** Socket location unknown or not following standard.

**Solution:** Implement XDG-compliant socket creation.

---

## 📊 **Current Situation**

### **What We Observed**

```bash
$ ps aux | grep nestgate
eastgate  152229  ... /path/to/nestgate server

$ ls /run/user/1000/biomeos/nestgate.sock
ls: cannot access: No such file or directory
```

**Status:** NestGate running but socket not discoverable.

---

## 💡 **Proposed Solution**

### **Socket Path Standard**

```
/run/user/$UID/biomeos/nestgate.sock
```

### **Implementation Example**

```rust
use std::env;
use std::path::PathBuf;

fn get_socket_path() -> PathBuf {
    // 1. Check environment variable
    if let Ok(socket) = env::var("NESTGATE_SOCKET") {
        return PathBuf::from(socket);
    }
    
    // 2. Check shared directory
    let socket_dir = env::var("BIOMEOS_SOCKET_DIR")
        .unwrap_or_else(|_| {
            let uid = unsafe { libc::getuid() };
            format!("/run/user/{}/biomeos", uid)
        });
    
    std::fs::create_dir_all(&socket_dir).ok();
    
    PathBuf::from(socket_dir).join("nestgate.sock")
}

// In NestGate startup:
fn main() -> Result<()> {
    let socket_path = get_socket_path();
    
    info!("🏰 NestGate starting...");
    info!("   Socket: {}", socket_path.display());
    info!("   Storage: {}", storage_path.display());
    
    let listener = UnixListener::bind(&socket_path)?;
    
    info!("✅ NestGate ready at {}", socket_path.display());
    
    // ... server logic
}
```

---

## ✅ **Success Criteria**

1. **Socket created at standard location:**
   ```bash
   $ ./nestgate server
   # Creates: /run/user/1000/biomeos/nestgate.sock
   ```

2. **Environment variable support:**
   ```bash
   $ NESTGATE_SOCKET=/tmp/test.sock ./nestgate server
   # Creates: /tmp/test.sock
   ```

3. **Discoverable by biomeOS:**
   ```bash
   $ ls -la /run/user/$(id -u)/biomeos/nestgate.sock
   srwxrwxr-x 1 user user 0 Jan 30 01:00 nestgate.sock
   ```

4. **JSON-RPC communication:**
   ```bash
   $ echo '{"jsonrpc":"2.0","method":"storage.list","id":1}' | \
     nc -U /run/user/$(id -u)/biomeos/nestgate.sock
   {"jsonrpc":"2.0","result":{"items":[]},"id":1}
   ```

5. **Integration with Squirrel:**
   - Squirrel can discover NestGate
   - Model metadata can be cached
   - Storage operations succeed

---

## 🧪 **Testing**

### **Manual Test**

```bash
# 1. Start NestGate
cd nestgate
cargo run --release -- server

# 2. Verify socket
ls -la /run/user/$(id -u)/biomeos/nestgate.sock

# 3. Test storage
echo '{"jsonrpc":"2.0","method":"storage.store","params":{"key":"test","value":"data"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/nestgate.sock
```

### **Integration Test**

```bash
# From biomeOS:
cd ../biomeOS
./scripts/quick_start_nucleus_test.sh

# Should now succeed at Nest Atomic deployment phase
```

---

## 📚 **Use Cases This Enables**

1. **Model Persistence** (Squirrel AI):
   ```rust
   // Squirrel can cache model metadata to NestGate
   squirrel.cache_model("llama-3-8b", metadata).await?;
   ```

2. **NUCLEUS Nest Atomic**:
   - Tower (BearDog + Songbird) + NestGate
   - Full storage capabilities
   - Provenance tracking

3. **Cross-Primal Coordination**:
   - Any primal can store data via NestGate
   - Capability-based discovery
   - Semantic routing

---

## 📋 **Action Items**

- [ ] **NestGate Team**: Implement `get_socket_path()` function
- [ ] **NestGate Team**: Add `BIOMEOS_SOCKET_DIR` support
- [ ] **NestGate Team**: Add startup logging
- [ ] **NestGate Team**: Update README documentation
- [ ] **NestGate Team**: Test with biomeOS integration
- [ ] **biomeOS Team**: Add NestGate discovery logic (if missing)
- [ ] **biomeOS Team**: Update integration tests
- [ ] **Both Teams**: Document socket standard

---

## 🎉 **Impact**

This change unblocks:
- ✅ Nest Atomic deployment
- ✅ Model persistence for AI workloads
- ✅ Full NUCLEUS + AI integration
- ✅ Production readiness

Thank you for your collaboration! **🦀✨**

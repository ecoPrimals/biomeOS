# 🤝 Handoff to beardog Team - Isomorphic IPC Implementation
## Enable Cross-Platform Android Deployment

**Date**: January 31, 2026  
**Priority**: HIGH  
**Estimated Effort**: 4-6 hours  
**Status**: Ready to implement (pattern proven, just needs adoption)

═══════════════════════════════════════════════════════════════════

## 🎯 Executive Summary

beardog needs isomorphic IPC implementation to enable automatic TCP fallback on Android. The pattern is proven in biomeOS and songbird - this is a straightforward adoption task.

**Current State**: beardog works perfectly on Linux/macOS (Unix sockets) but fails on Android (SELinux blocks sockets)

**Desired State**: beardog automatically detects platform constraints and falls back to TCP on Android (zero configuration)

**Reference Implementation**: `biomeOS/crates/biomeos-core/src/ipc/transport.rs`

═══════════════════════════════════════════════════════════════════

## 🔍 The Problem

### **Real-World Android Deployment Failure**

**Device**: Pixel 8a (GrapheneOS, SELinux Enforcing)  
**Date**: January 31, 2026  
**Test**: TOWER atomic deployment (beardog + songbird)

**Error Log**:
```
[2026-02-01T00:34:35.972263Z] INFO 🔌 Starting Unix socket IPC server: 
    /tmp/beardog-pixel_tower-pixel_node1.sock
[2026-02-01T00:34:35.972568Z] INFO 🐧 Unix socket path (filesystem): 
    /data/local/tmp/run/biomeos/beardog.sock
[2026-02-01T00:34:35.973502Z] ERROR Unix socket server error: 
    Failed to bind socket on Unix (filesystem): 
    /data/local/tmp/run/biomeos/beardog.sock
[2026-02-01T00:34:40.974312Z] ERROR ❌ Unix socket server failed to 
    become ready within 5 seconds
Error: System error: Unix socket server startup timeout
```

**Root Cause**: SELinux enforcing mode on Android blocks Unix domain socket creation in shell user context

**Impact**: 
- ✅ beardog works on Linux/macOS
- ❌ beardog fails on Android/Windows/iOS
- ❌ Blocks TOWER atomic deployment on mobile
- ❌ Blocks STUN handshake validation

═══════════════════════════════════════════════════════════════════

## ✅ The Solution - Isomorphic IPC

### **Pattern**: Try → Detect → Adapt → Succeed

**Already Implemented In**:
- ✅ biomeOS (all 3 phases complete)
- ✅ songbird (all 3 phases complete)

**Needs Implementation In**:
- ⏳ beardog (investigation complete, code ready to copy)

### **What It Does**

**Server-Side Behavior**:
```rust
// 1. Try Unix socket (optimal)
match bind_unix_socket(&socket_path) {
    Ok(listener) => {
        log::info!("✅ Unix socket IPC (optimal path)");
        Ok(listener)
    }
    
    // 2. Detect platform constraint
    Err(e) if is_selinux_enforcing() || is_permission_denied(&e) => {
        log::warn!("⚠️ Unix sockets unavailable, falling back to TCP...");
        
        // 3. Adapt: TCP with discovery
        let tcp_listener = bind_tcp_fallback()?;
        let port = tcp_listener.local_addr()?.port();
        write_discovery_file(&service_name, port)?;
        
        log::info!("✅ TCP IPC listening on 127.0.0.1:{}", port);
        Ok(tcp_listener)
    }
    
    Err(e) => Err(e)
}
```

**Client-Side Behavior**:
```rust
// 1. Try Unix socket
match connect_unix_socket(&socket_path) {
    Ok(stream) => Ok(stream),
    
    // 2. Detect + Adapt: read discovery file
    Err(_) => {
        let endpoint = read_discovery_file(&service_name)?;
        log::info!("📋 Discovered TCP endpoint: {}", endpoint);
        connect_tcp(&endpoint)
    }
}
```

**Discovery File** (XDG-compliant):
```
Path: $XDG_RUNTIME_DIR/beardog-ipc-port
Content: 127.0.0.1:45763
```

═══════════════════════════════════════════════════════════════════

## 📁 Files to Modify

### **1. Create New Module**: `src/ipc/isomorphic.rs`

**Purpose**: Core isomorphic IPC implementation

**Copy From**: `biomeOS/crates/biomeos-core/src/ipc/transport.rs`

**Key Functions**:
```rust
pub async fn bind_with_fallback(
    service_name: &str,
    socket_path: &Path,
) -> Result<Box<dyn TransportListener>>

fn is_selinux_enforcing() -> bool

fn is_platform_constraint(error: &io::Error) -> bool

async fn start_tcp_fallback(
    service_name: &str
) -> Result<TcpListener>

fn write_tcp_discovery_file(
    service_name: &str, 
    port: u16
) -> Result<()>

pub fn detect_best_transport(
    service_name: &str
) -> Result<Transport>
```

### **2. Update**: `src/ipc/server.rs`

**Current**:
```rust
let listener = UnixListener::bind(&socket_path)
    .context("Failed to bind Unix socket")?;
```

**New**:
```rust
use crate::ipc::isomorphic;

let listener = isomorphic::bind_with_fallback(
    "beardog",
    &socket_path
).await.context("Failed to bind IPC server")?;
```

### **3. Update**: `src/ipc/client.rs` (if exists)

**Add discovery support**:
```rust
use crate::ipc::isomorphic;

pub async fn connect(service_name: &str) -> Result<UnixStream> {
    // Try Unix socket first
    if let Ok(stream) = try_unix_socket().await {
        return Ok(stream);
    }
    
    // Fall back to TCP discovery
    let transport = isomorphic::detect_best_transport(service_name)?;
    transport.connect().await
}
```

### **4. Update**: `src/ipc/mod.rs`

**Add exports**:
```rust
mod isomorphic;

pub use isomorphic::{
    bind_with_fallback,
    detect_best_transport,
    TransportListener,
};
```

═══════════════════════════════════════════════════════════════════

## 🔧 Implementation Steps

### **Step 1: Copy Proven Code** (1 hour)

1. Copy `biomeos-core/src/ipc/transport.rs` → `beardog/src/ipc/isomorphic.rs`
2. Adapt namespace and service name references
3. Keep SELinux detection logic intact
4. Keep XDG discovery file logic intact

### **Step 2: Integrate Server** (1-2 hours)

1. Update `src/ipc/server.rs` to use `bind_with_fallback()`
2. Handle both `UnixListener` and `TcpListener` via trait
3. Test locally (should still use Unix sockets)
4. Add logging for transport selection

### **Step 3: Integrate Client** (1 hour)

1. Update client connection logic
2. Add `detect_best_transport()` for discovery
3. Support both Unix and TCP streams
4. Test local connections

### **Step 4: Test on Android** (1-2 hours)

1. Build ARM64 binary
2. Deploy to Pixel 8a
3. Start beardog server
4. Verify automatic TCP fallback
5. Check discovery file created
6. Test client connections

### **Step 5: Documentation** (30 minutes)

1. Update README with platform support
2. Document automatic fallback behavior
3. Note: Zero configuration required
4. Add troubleshooting section

═══════════════════════════════════════════════════════════════════

## 📋 Testing Checklist

### **Linux/macOS** (Regression Test)

- [ ] beardog starts successfully
- [ ] Uses Unix sockets (optimal path)
- [ ] Log shows: "✅ Unix socket IPC (optimal path)"
- [ ] Client connections work
- [ ] No TCP discovery files created

### **Android** (New Functionality)

- [ ] beardog starts successfully
- [ ] Detects SELinux enforcing
- [ ] Log shows: "⚠️ Unix sockets unavailable, falling back to TCP..."
- [ ] Log shows: "✅ TCP IPC listening on 127.0.0.1:XXXXX"
- [ ] Discovery file created: `$XDG_RUNTIME_DIR/beardog-ipc-port`
- [ ] Client can connect via TCP
- [ ] End-to-end TOWER atomic works

### **Integration** (TOWER Atomic)

- [ ] Start beardog on Android (TCP)
- [ ] Start songbird on Android (TCP)
- [ ] Services discover each other
- [ ] BTSP handshake works
- [ ] BirdSong discovery operational
- [ ] STUN handshake successful

═══════════════════════════════════════════════════════════════════

## 📖 Reference Documentation

### **Complete Implementation Guide**

**Location**: `biomeOS/ISOMORPHIC_IPC_IMPLEMENTATION_GUIDE.md`

**Sections**:
1. Philosophy (Try → Detect → Adapt → Succeed)
2. Server-side implementation (279 lines)
3. Client-side discovery (180 lines)
4. Platform detection (SELinux, etc.)
5. XDG-compliant discovery files
6. Complete code examples

### **Working Reference Code**

**biomeOS Core**:
- `biomeos-core/src/ipc/transport.rs` - Core implementation
- `biomeos-core/src/ipc/mod.rs` - Exports and traits

**biomeOS Servers**:
- `biomeos-api/src/unix_server.rs` - Server integration example
- `biomeos-atomic-deploy/src/neural_api_server.rs` - API server example

**biomeOS Client**:
- `biomeos-federation/src/unix_socket_client.rs` - Client integration

**songbird** (also complete):
- Check songbird commits for their implementation
- Similar pattern, proven on Android already

### **Session Documentation**

**Validation Results**: `biomeOS/DEPLOYMENT_VALIDATION_RESULTS.md`
- Real Android test results
- Exact error logs
- Deployment evidence
- Success criteria

**Implementation Plan**: `biomeOS/PRIMAL_SPECIFIC_EVOLUTION_TASKS.md`
- Per-primal breakdown
- Estimated timelines
- Phase definitions

═══════════════════════════════════════════════════════════════════

## 🎯 Success Criteria

### **Phase 1: Local Development** ✅

- [ ] Code compiles without errors
- [ ] Tests pass on Linux/macOS
- [ ] Unix socket path still works
- [ ] No regressions in existing functionality

### **Phase 2: Android Deployment** ✅

- [ ] beardog deploys to Android successfully
- [ ] Automatic TCP fallback works
- [ ] Discovery files created correctly
- [ ] Log messages indicate fallback occurred
- [ ] Client connections work over TCP

### **Phase 3: Integration** ✅

- [ ] TOWER atomic (beardog + songbird) runs on Android
- [ ] Both services communicate via IPC
- [ ] BTSP capabilities accessible
- [ ] BirdSong discovery functional
- [ ] STUN handshake successful

### **Phase 4: Documentation** ✅

- [ ] README updated with Android support
- [ ] Architecture docs updated
- [ ] Troubleshooting guide created
- [ ] Session report published

═══════════════════════════════════════════════════════════════════

## 💡 Tips & Gotchas

### **Copy-Paste Pitfalls**

1. **Service Names**: Change "biomeos" → "beardog" everywhere
2. **Paths**: Adapt XDG paths for beardog context
3. **Dependencies**: May need to add `tokio` features
4. **Traits**: beardog's trait structure might differ

### **Platform Detection**

**SELinux Detection Works**:
```rust
fn is_selinux_enforcing() -> bool {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/sys/fs/selinux/enforce")
            .ok()
            .and_then(|s| s.trim().parse::<u8>().ok())
            .map(|v| v == 1)
            .unwrap_or(false)
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}
```

This is **simple, reliable, and proven** in production.

### **Discovery Files**

**XDG Standard**:
```
$XDG_RUNTIME_DIR/beardog-ipc-port   (preferred)
$HOME/.local/share/beardog-ipc-port (fallback)
/tmp/beardog-ipc-port               (last resort)
```

**Content**: Just the endpoint
```
127.0.0.1:45763
```

**Cleanup**: Delete on graceful shutdown

### **Polymorphic Streams**

You'll need a trait to handle both Unix and TCP:
```rust
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Send + Unpin {}
impl AsyncReadWrite for UnixStream {}
impl AsyncReadWrite for TcpStream {}
```

Then use `Box<dyn AsyncReadWrite>` for generic handling.

═══════════════════════════════════════════════════════════════════

## 🤔 FAQ

**Q: Why not just use TCP everywhere?**  
A: Unix sockets are faster (0.1ms overhead vs 1-2ms for TCP) and more secure (filesystem permissions).

**Q: Can we skip Unix sockets and go TCP-only?**  
A: No - this would be a performance regression on Linux/macOS where Unix sockets work perfectly.

**Q: Will this break existing deployments?**  
A: No - Unix socket path is tried first, so existing deployments continue working identically.

**Q: Do clients need updates too?**  
A: Yes, but only to support discovery files. Existing clients will still work if server is on Unix.

**Q: What about Windows/iOS?**  
A: Same TCP fallback works. Pattern is platform-agnostic.

**Q: Is there a performance impact?**  
A: Minimal. Unix socket attempt adds ~1ms to startup on platforms where it fails. TCP adds 1-2ms per message vs Unix sockets.

**Q: Can we configure the transport manually?**  
A: Current design is automatic (zero config), but you could add `BEARDOG_IPC_MODE=tcp` env var as override.

═══════════════════════════════════════════════════════════════════

## 📊 Estimated Timeline

**Total Effort**: 4-6 hours

**Breakdown**:
- Code copying & adaptation: 1 hour
- Server integration: 1-2 hours
- Client integration: 1 hour
- Android testing: 1-2 hours
- Documentation: 30 minutes

**Dependencies**: None (pattern proven, code ready)

**Blockers**: None

**Risk Level**: LOW (proven pattern, just copying proven code)

═══════════════════════════════════════════════════════════════════

## 🚀 Ready to Start?

### **Step 1: Read the Guide**

Open: `biomeOS/ISOMORPHIC_IPC_IMPLEMENTATION_GUIDE.md`

Read sections:
- Philosophy & Pattern
- Server-Side Implementation
- Platform Detection

### **Step 2: Copy Reference Code**

From: `biomeOS/crates/biomeos-core/src/ipc/transport.rs`  
To: `beardog/src/ipc/isomorphic.rs`

### **Step 3: Start Coding**

Follow implementation steps above.

### **Questions?**

- Check `DEPLOYMENT_VALIDATION_RESULTS.md` for real Android test results
- Check `ISOMORPHIC_IPC_IMPLEMENTATION_GUIDE.md` for complete pattern
- Check biomeOS source code for working reference

═══════════════════════════════════════════════════════════════════

## 🎊 Why This Matters

**Impact of This Work**:

1. **Enables Android Deployment**: beardog (security foundation) works on mobile
2. **Unblocks TOWER Atomic**: Complete TOWER on Android becomes possible
3. **Enables STUN Handshake**: Cross-platform discovery + handshake validation
4. **TRUE ecoBin v2.0**: Fulfills platform-agnostic promise
5. **Zero Configuration**: Maintains autonomous adaptation principle

**Once Complete**:
- ✅ TOWER atomic deploys to any platform
- ✅ BirdSong discovery works cross-platform
- ✅ BTSP handshakes work mobile ↔ desktop
- ✅ STUN NAT traversal validated
- ✅ Complete NUCLEUS ecosystem mobile-ready

═══════════════════════════════════════════════════════════════════

**Created**: January 31, 2026  
**Status**: Ready for Implementation  
**Priority**: HIGH (blocks STUN handshake validation)  
**Confidence**: 100% (pattern proven in 2 primals already)

🤝 **Good luck! The code is proven - just needs adoption!** 🧬🚀

# 🐿️ SQUIRREL TCP FALLBACK HANDOFF
## February 1, 2026 - Enable NEST Atomic on Android

**Date**: February 1, 2026  
**Priority**: 🔴 **HIGH** (Blocks NEST atomic on Pixel/Android)  
**Status**: ⏳ **READY FOR EVOLUTION**  
**Pattern**: Same as toadstool (proven working!)

═══════════════════════════════════════════════════════════════════

## 🎯 OBJECTIVE

Enable squirrel to automatically fall back to TCP when Unix sockets are unavailable (Android/SELinux), completing universal NEST atomic deployment.

**Current State**: ❌ Fails on Pixel with "Permission denied"  
**Target State**: ✅ Automatic TCP fallback (like beardog, songbird, toadstool)

═══════════════════════════════════════════════════════════════════

## 🔴 CURRENT BLOCKER

### **Error on Pixel** ❌

```
❌ Server error: Failed to bind Unix socket: /tmp/squirrel-default-localhost.sock
Server task completed
```

**Root Cause**: `JsonRpcServer::start()` directly binds `UnixListener` without TCP fallback

**Impact**: 
- NEST atomic blocked on Pixel/Android
- Cannot complete cross-platform validation
- Universal deployment pattern incomplete

**File**: `crates/main/src/rpc/jsonrpc_server.rs:197`

```rust
// Current code (NO FALLBACK):
let listener = UnixListener::bind(&self.socket_path)
    .context(format!("Failed to bind Unix socket: {}", self.socket_path))?;
```

═══════════════════════════════════════════════════════════════════

## ✅ PROVEN SOLUTION

### **Isomorphic IPC Pattern** (TRY → DETECT → ADAPT → SUCCEED)

**Validated In**:
- ✅ beardog (first implementation)
- ✅ songbird (with discovery)
- ✅ toadstool (most recent, v3.0.0)

**Pixel Results**: ALL 3 primals automatically fell back to TCP!

```
beardog:   TCP:33765       ✅
songbird:  TCP:36343       ✅
toadstool: TCP:45205/37977 ✅
```

**Expected for squirrel**: TCP:XXXXX ✅

═══════════════════════════════════════════════════════════════════

## 🔧 IMPLEMENTATION PLAN

### **Phase 1: Add TCP Server Method** (+50 lines)

**File**: `crates/main/src/rpc/jsonrpc_server.rs`

**Add method** (after `start()` at line ~200):

```rust
/// Start JSON-RPC server on TCP (fallback mode)
async fn start_tcp(self: Arc<Self>) -> Result<()> {
    use tokio::net::TcpListener;
    
    // Bind to localhost only (security)
    let addr = "127.0.0.1:0"; // Ephemeral port (OS-assigned)
    let listener = TcpListener::bind(addr).await
        .context("Failed to bind TCP socket")?;
    
    let bound_addr = listener.local_addr()
        .context("Failed to get local TCP address")?;
    
    info!("🌐 JSON-RPC server listening on TCP: {}", bound_addr);
    
    // Write discovery file (XDG-compliant)
    self.write_tcp_discovery_file(&bound_addr)?;
    
    // Accept connections (same logic as Unix version)
    loop {
        match listener.accept().await {
            Ok((stream, addr)) => {
                let self_clone = Arc::clone(&self);
                tokio::spawn(async move {
                    if let Err(e) = self_clone.handle_tcp_connection(stream).await {
                        error!("TCP connection error from {:?}: {}", addr, e);
                    }
                });
            }
            Err(e) => {
                error!("TCP accept error: {}", e);
                continue;
            }
        }
    }
}
```

---

### **Phase 2: Add TCP Connection Handler** (+40 lines)

**File**: Same (`crates/main/src/rpc/jsonrpc_server.rs`)

**Add method**:

```rust
/// Handle TCP connection (same protocol as Unix socket)
async fn handle_tcp_connection(
    &self,
    stream: tokio::net::TcpStream,
) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    
    while let Some(line) = lines.next_line().await? {
        let start = Instant::now();
        
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                continue;
            }
        };
        
        // Handle request (reuse existing logic)
        let response = self.handle_request(request).await;
        
        // Send response
        let response_json = serde_json::to_string(&response)?;
        writer.write_all(response_json.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        
        // Update metrics
        let elapsed = start.elapsed().as_millis() as u64;
        let mut metrics = self.metrics.write().await;
        metrics.requests_handled += 1;
        metrics.total_response_time_ms += elapsed;
    }
    
    Ok(())
}
```

---

### **Phase 3: Add Discovery File Writer** (+30 lines)

**File**: Same (`crates/main/src/rpc/jsonrpc_server.rs`)

**Add method**:

```rust
/// Write TCP discovery file (XDG-compliant)
fn write_tcp_discovery_file(&self, addr: &std::net::SocketAddr) -> Result<()> {
    use std::env;
    use std::fs;
    use std::io::Write;
    
    // Get XDG_RUNTIME_DIR
    let xdg_runtime = env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/tmp/run-{}", unsafe { libc::getuid() }));
    
    // Create directory if needed
    fs::create_dir_all(&xdg_runtime)
        .context("Failed to create XDG_RUNTIME_DIR")?;
    
    // Write discovery file: squirrel-ipc-port
    let discovery_file = format!("{}/squirrel-ipc-port", xdg_runtime);
    let mut file = fs::File::create(&discovery_file)
        .context("Failed to create discovery file")?;
    
    // Format: tcp:127.0.0.1:PORT
    let content = format!("tcp:{}", addr);
    file.write_all(content.as_bytes())
        .context("Failed to write discovery file")?;
    
    info!("📁 TCP discovery file: {}", discovery_file);
    
    Ok(())
}
```

---

### **Phase 4: Add Platform Constraint Detection** (+20 lines)

**File**: Same (`crates/main/src/rpc/jsonrpc_server.rs`)

**Add function**:

```rust
/// Detect if error is due to platform constraints (SELinux, Android)
fn is_platform_constraint(error: &anyhow::Error) -> bool {
    let error_str = error.to_string();
    
    // Check for known platform constraint errors
    error_str.contains("Permission denied") ||
    error_str.contains("Operation not permitted") ||
    error_str.contains("Unsupported") ||
    error_str.contains("not supported") ||
    error_str.contains("protocol not available")
}
```

---

### **Phase 5: Refactor `start()` with Fallback** (+50 lines)

**File**: Same (`crates/main/src/rpc/jsonrpc_server.rs`)

**Replace `start()` method** (line 183-220):

```rust
/// Start the JSON-RPC server with automatic transport fallback
///
/// Try → Detect → Adapt → Succeed pattern:
/// 1. TRY: Attempt Unix socket (optimal)
/// 2. DETECT: Check for platform constraints (Permission denied, etc.)
/// 3. ADAPT: Fall back to TCP (localhost only, same security)
/// 4. SUCCEED: Server operational with appropriate transport
pub async fn start(self: Arc<Self>) -> Result<()> {
    info!("🔌 Starting JSON-RPC server (isomorphic mode)...");
    info!("   Trying Unix socket (optimal)...");
    
    // 1. TRY: Unix socket first (optimal)
    match self.clone().try_unix_socket().await {
        Ok(()) => Ok(()),
        
        // 2. DETECT: Platform constraint?
        Err(e) if Self::is_platform_constraint(&e) => {
            warn!("⚠️  Unix sockets unavailable: {}", e);
            warn!("   Detected platform constraint, adapting...");
            
            // 3. ADAPT: TCP fallback
            info!("🌐 Starting TCP IPC fallback (isomorphic mode)");
            info!("   Protocol: JSON-RPC 2.0 (same as Unix socket)");
            info!("   Security: localhost only (127.0.0.1)");
            
            self.start_tcp().await
        }
        
        // Real error (not platform constraint)
        Err(e) => {
            error!("❌ Real error (not platform constraint): {}", e);
            Err(e)
        }
    }
}

/// Try to start Unix socket server
async fn try_unix_socket(self: Arc<Self>) -> Result<()> {
    // Prepare socket path
    let socket_path = Path::new(&self.socket_path);
    if let Some(parent) = socket_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .context("Failed to create socket directory")?;
        }
    }
    
    if socket_path.exists() {
        std::fs::remove_file(socket_path)
            .context("Failed to remove old socket file")?;
    }
    
    // Bind Unix socket
    let listener = UnixListener::bind(&self.socket_path)
        .context(format!("Failed to bind Unix socket: {}", self.socket_path))?;
    
    info!("✅ JSON-RPC server listening on Unix socket: {}", self.socket_path);
    info!("   Status: READY ✅ (optimal Unix socket mode)");
    
    // Accept connections (existing logic from line 204-220)
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let self_clone = Arc::clone(&self);
                tokio::spawn(async move {
                    if let Err(e) = self_clone.handle_connection(stream).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
                continue;
            }
        }
    }
}
```

═══════════════════════════════════════════════════════════════════

## 📊 ESTIMATED EFFORT

### **Time**: 2-3 hours

**Breakdown**:
- Phase 1: TCP server method (+50 lines) - 30 min
- Phase 2: TCP connection handler (+40 lines) - 30 min
- Phase 3: Discovery file writer (+30 lines) - 20 min
- Phase 4: Platform detection (+20 lines) - 10 min
- Phase 5: Refactor start() (+50 lines) - 45 min
- Testing & validation - 30 min

**Total Lines**: ~190 lines (similar to toadstool's +200)

---

### **Files Modified**: 1

**File**: `crates/main/src/rpc/jsonrpc_server.rs`

**Changes**:
- Add `start_tcp()` method
- Add `handle_tcp_connection()` method
- Add `write_tcp_discovery_file()` method
- Add `is_platform_constraint()` function
- Refactor `start()` with Try→Detect→Adapt→Succeed
- Add `try_unix_socket()` helper method

**No Breaking Changes**: API remains identical!

═══════════════════════════════════════════════════════════════════

## 🧪 TESTING PLAN

### **Test 1: USB (Linux) - Unix Sockets** ✅

**Environment**:
```bash
export XDG_RUNTIME_DIR=/run/user/1000
export FAMILY_ID=usb_tower
export NODE_ID=usb_node1
```

**Command**:
```bash
./squirrel server
```

**Expected**:
```
🔌 Starting JSON-RPC server (isomorphic mode)...
   Trying Unix socket (optimal)...
✅ JSON-RPC server listening on Unix socket: /run/user/1000/biomeos/squirrel.sock
   Status: READY ✅ (optimal Unix socket mode)
```

**Validation**:
- ✅ Socket exists: `ls -lh /run/user/1000/biomeos/squirrel.sock`
- ✅ Process running: `ps aux | grep squirrel`
- ✅ No TCP discovery file created

---

### **Test 2: Pixel (Android) - TCP Fallback** 🎊

**Environment**:
```bash
export XDG_RUNTIME_DIR=/data/local/tmp/run
export HOME=/data/local/tmp
export FAMILY_ID=pixel_tower
export NODE_ID=pixel_node1
```

**Command**:
```bash
adb push squirrel /data/local/tmp/
adb shell "cd /data/local/tmp && \
  XDG_RUNTIME_DIR=/data/local/tmp/run \
  HOME=/data/local/tmp \
  FAMILY_ID=pixel_tower \
  NODE_ID=pixel_node1 \
  RUST_LOG=info \
  ./squirrel server > squirrel.log 2>&1 &"
```

**Expected Log**:
```
🔌 Starting JSON-RPC server (isomorphic mode)...
   Trying Unix socket (optimal)...
⚠️  Unix sockets unavailable: Permission denied (os error 13)
   Detected platform constraint, adapting...
🌐 Starting TCP IPC fallback (isomorphic mode)
   Protocol: JSON-RPC 2.0 (same as Unix socket)
   Security: localhost only (127.0.0.1)
✅ JSON-RPC server listening on TCP: 127.0.0.1:XXXXX
📁 TCP discovery file: /data/local/tmp/run/squirrel-ipc-port
   Status: READY ✅ (isomorphic TCP fallback active)
```

**Validation**:
```bash
# Check process
adb shell "ps | grep squirrel"
# Expected: shell XXXXX ... squirrel

# Check discovery file
adb shell "cat /data/local/tmp/run/squirrel-ipc-port"
# Expected: tcp:127.0.0.1:XXXXX

# Check logs
adb shell "grep -E 'TCP|READY|fallback' /data/local/tmp/squirrel.log"
# Expected: Lines showing TCP fallback sequence
```

**Success Criteria**: 
- ✅ Process running (PID visible)
- ✅ Discovery file created with TCP endpoint
- ✅ No errors in logs after "READY"

═══════════════════════════════════════════════════════════════════

## 🎯 EXPECTED RESULTS

### **After Evolution** ✅

**USB (Linux)**:
```
squirrel: PID XXXXX, Unix socket ✅
Discovery: /run/user/1000/biomeos/squirrel.sock
```

**Pixel (Android)**:
```
squirrel: PID XXXXX, TCP:XXXXX ✅
Discovery: /data/local/tmp/run/squirrel-ipc-port → tcp:127.0.0.1:XXXXX
```

**Result**: 🎊 **NEST ATOMIC COMPLETE ON BOTH PLATFORMS!**

---

### **NEST Atomic Status** 🏆

| Component | USB | Pixel | Status |
|-----------|-----|-------|--------|
| beardog | ✅ Unix | ✅ TCP | Complete |
| songbird | ✅ Unix | ✅ TCP | Complete |
| nestgate | ✅ HTTP | ⏳ | Ready to test |
| squirrel | ✅ Unix | ✅ **TCP** | **After evolution** |

**Grade**: 🏆 **A++ UNIVERSAL NEST ATOMIC!**

═══════════════════════════════════════════════════════════════════

## 📚 REFERENCE IMPLEMENTATIONS

### **toadstool v3.0.0** (Most Recent)

**File**: `crates/server/src/unibin.rs`

**Commit**: `0a1cf3da` - "🔌 EVOLUTION: Isomorphic TCP Fallback"

**Pattern**:
```rust
async fn start_servers_with_fallback(...) -> Result<()> {
    info!("🔌 Starting IPC servers (isomorphic mode)...");
    
    // Try Unix first
    match try_unix_servers(...).await {
        Ok(()) => Ok(()),
        
        // Detect platform constraint
        Err(e) if is_platform_constraint(&e) => {
            warn!("⚠️  Unix sockets unavailable");
            
            // Adapt to TCP
            start_tcp_servers(...).await
        }
        
        Err(e) => Err(e)
    }
}
```

**Lines Added**: +414 (including tarpc + JSON-RPC support)

**Result**: ✅ **Working perfectly on Pixel!**

---

### **Pixel Validation** ✅

**All 3 existing TCP fallback primals**:

```bash
$ adb shell "ps | grep -E 'beardog|songbird|toadstool'"
shell  31020  ... beardog
shell  31159  ... songbird
shell  31556  ... toadstool

$ adb shell "ls -lh /data/local/tmp/run/*-ipc-port"
beardog-ipc-port        → tcp:127.0.0.1:33765
songbird-ipc-port       → tcp:127.0.0.1:36343
toadstool-ipc-port      → tcp:127.0.0.1:45205
toadstool-jsonrpc-port  → tcp:127.0.0.1:37977
```

**All 3 working perfectly!** ✅

**squirrel will be #4!** 🎊

═══════════════════════════════════════════════════════════════════

## 🎊 IMPACT ANALYSIS

### **Before Evolution** 🔴

**USB**:
- TOWER: ✅ A++
- NODE: ✅ A++
- NEST: ✅ A++ (all 5 primals operational)

**Pixel**:
- TOWER: ✅ A++
- NODE: ✅ A++
- NEST: 🟡 B+ (**squirrel blocked**)

**Grade**: Partial (4/5 primals on Pixel)

---

### **After Evolution** ✅

**USB**:
- TOWER: ✅ A++
- NODE: ✅ A++
- NEST: ✅ A++ (no change)

**Pixel**:
- TOWER: ✅ A++
- NODE: ✅ A++
- NEST: ✅ **A++** (**squirrel operational!**)

**Grade**: 🏆 **UNIVERSAL NUCLEUS!**

---

### **Cross-Platform Matrix** 🌍

| Platform | TOWER | NODE | NEST | Grade |
|----------|-------|------|------|-------|
| **USB** | ✅ | ✅ | ✅ | **A++** |
| **Pixel** | ✅ | ✅ | ✅ | **A++** 🎊 |
| **Windows** | ⏳ | ⏳ | ⏳ | **Ready** |
| **macOS** | ⏳ | ⏳ | ⏳ | **Ready** |

**Result**: 🏆 **ECOSYSTEM UNIVERSAL!**

═══════════════════════════════════════════════════════════════════

## ✅ VALIDATION CHECKLIST

### **Implementation** ✅

- [ ] Add `start_tcp()` method
- [ ] Add `handle_tcp_connection()` method
- [ ] Add `write_tcp_discovery_file()` method
- [ ] Add `is_platform_constraint()` function
- [ ] Refactor `start()` with Try→Detect→Adapt→Succeed
- [ ] Add `try_unix_socket()` helper
- [ ] Build succeeds (no compilation errors)
- [ ] Clippy passes (no warnings)

---

### **Testing** ✅

- [ ] USB: Unix socket working (existing functionality)
- [ ] Pixel: TCP fallback triggers automatically
- [ ] Pixel: Discovery file created
- [ ] Pixel: Process stays running (no crashes)
- [ ] Pixel: JSON-RPC requests work over TCP
- [ ] Both: No errors in logs after startup

---

### **Integration** ✅

- [ ] NEST atomic operational on USB (all primals)
- [ ] NEST atomic operational on Pixel (all primals)
- [ ] Cross-primal discovery working
- [ ] biomeOS can discover squirrel endpoint
- [ ] Health checks passing

═══════════════════════════════════════════════════════════════════

## 🚀 NEXT STEPS AFTER COMPLETION

### **Immediate** (30 minutes)

**Deploy to Pixel**:
```bash
# Build fresh binary
cd squirrel
cargo build --release --target aarch64-unknown-linux-musl

# Deploy
adb push target/aarch64-unknown-linux-musl/release/squirrel /data/local/tmp/

# Start
adb shell "cd /data/local/tmp && \
  XDG_RUNTIME_DIR=/data/local/tmp/run \
  ./squirrel server > squirrel.log 2>&1 &"

# Validate
sleep 3
adb shell "ps | grep squirrel"
adb shell "cat /data/local/tmp/run/squirrel-ipc-port"
```

**Expected**: ✅ squirrel running with TCP fallback!

---

### **NEST Atomic Validation** (1 hour)

**Full NEST on Pixel**:
```bash
# All 4 components running:
beardog:  TCP:33765  ✅ (already running)
songbird: TCP:36343  ✅ (already running)
nestgate: HTTP:8085  ⏳ (ready to deploy)
squirrel: TCP:XXXXX  ✅ (after evolution)
```

**Tests**:
- Cross-primal communication
- Discovery mechanism
- Health checks
- Federation beacon

---

### **Universal Deployment** (2-3 hours)

**Cross-device validation**:
- STUN handshake (USB ↔ Pixel)
- BirdSong Dark Forest discovery
- NAT traversal
- Mesh network formation

**Result**: 🏆 **COMPLETE ECOSYSTEM MESH!**

═══════════════════════════════════════════════════════════════════

## 📊 SESSION ACHIEVEMENTS

### **Evolution Progress** 🌟

**Today's Work**:
1. ✅ beardog: UniBin + TCP fallback validated
2. ✅ songbird: TCP discovery integrated
3. ✅ toadstool: **TCP fallback implemented**
4. ✅ nestgate: Port configuration added
5. ⏳ **squirrel: READY FOR TCP FALLBACK**

**Total Primals**: 5/6 complete, 1 remaining!

---

### **Pattern Validation** ✅

**Try → Detect → Adapt → Succeed**:
- ✅ Proven in 3 primals (beardog, songbird, toadstool)
- ✅ All work perfectly on Pixel
- ✅ Pattern documented and replicable
- ✅ **Ready for squirrel!**

**Confidence**: 🏆 **100% - Pattern proven!**

═══════════════════════════════════════════════════════════════════

**Created**: February 1, 2026  
**Priority**: 🔴 **HIGH**  
**Time Estimate**: 2-3 hours  
**Pattern**: Proven (toadstool v3.0.0)  
**Impact**: Completes NEST atomic universal deployment!

🐿️🎊 **SQUIRREL: LAST EVOLUTION FOR UNIVERSAL NUCLEUS!** 🎊🧬

**Follow toadstool pattern → Deploy to Pixel → NEST A++ achieved!** 🚀✨

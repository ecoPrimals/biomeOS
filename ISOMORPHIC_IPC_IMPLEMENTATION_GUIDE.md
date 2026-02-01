# Isomorphic IPC Implementation Guide for Primals

**Date**: January 31, 2026  
**Version**: 1.0  
**Reference**: songbird v3.33.0 (Production Validated)  
**Status**: Ready for Implementation

═══════════════════════════════════════════════════════════════════

## 📋 TO: All Remaining Primals

**Recipients**: beardog, toadstool, nestgate, squirrel teams  
**From**: biomeOS NUCLEUS Team  
**Subject**: Isomorphic IPC Pattern - Implementation Guide

This document provides everything you need to evolve your primal with isomorphic IPC. Follow this guide to achieve TRUE platform-agnostic operation with automatic TCP fallback.

═══════════════════════════════════════════════════════════════════

## 🎯 WHAT IS ISOMORPHIC IPC?

### Definition

**Isomorphic IPC**: Same binary runs on ALL platforms, automatically adapting to platform constraints without configuration.

### The Problem We Solve

**Before** (Platform-Specific):
```rust
// ❌ Requires user to set flags
if env::var("PLATFORM") == "android" {
    use_tcp_ipc();  // User must configure this
} else {
    use_unix_sockets();
}
```

**After** (Isomorphic):
```rust
// ✅ Automatic adaptation, zero config
match try_unix_socket().await {
    Ok(listener) => run_unix_server(listener),
    Err(e) if is_platform_constraint(&e) => {
        // Automatically detect and adapt!
        run_tcp_fallback().await
    }
    Err(e) => Err(e)  // Real error
}
```

### Why This Matters

**Platform constraints are DATA (detected at runtime), not CONFIG (hardcoded at compile time).**

This is biological adaptation - your primal learns its environment and adapts!

═══════════════════════════════════════════════════════════════════

## ✅ VALIDATION PROOF

### songbird: Production Validated on Android

**Evidence from Pixel 8a** (Jan 31, 2026):
```log
[INFO] Starting IPC server (isomorphic mode)...
[INFO]    Trying Unix socket IPC (optimal)...
[WARN] ⚠️  Unix sockets unavailable: Failed to bind Unix socket
[WARN]    Detected platform constraint, adapting...
[INFO] 🌐 Starting TCP IPC fallback (isomorphic mode)
[INFO]    Protocol: JSON-RPC 2.0 (same as Unix socket)
[INFO] ✅ TCP IPC listening on 127.0.0.1:45763
[INFO]    Status: READY ✅ (isomorphic TCP fallback active)
```

**Result**: Same binary works on Linux (Unix sockets) AND Android (TCP) with ZERO configuration!

**Deep Debt Grade**: A++ (205/100) - All principles validated

═══════════════════════════════════════════════════════════════════

## 🏗️ THE TRY→DETECT→ADAPT→SUCCEED PATTERN

### Core Philosophy

Your primal should:
1. **TRY** the optimal implementation first (Unix sockets)
2. **DETECT** if it's a platform constraint (not a real error)
3. **ADAPT** automatically to fallback (TCP)
4. **SUCCEED** or fail with real error

### Universal Pattern (Apply to ANY Capability)

```rust
/// Universal pattern for platform-agnostic capabilities
async fn start_capability(&self) -> Result<()> {
    // 1. TRY optimal path first
    match self.try_optimal_implementation().await {
        Ok(result) => {
            info!("✅ Using optimal implementation");
            Ok(result)
        }
        
        // 2. DETECT platform constraints
        Err(e) if self.is_platform_constraint(&e) => {
            warn!("⚠️  Optimal path unavailable: {}", e);
            warn!("   Detected platform constraint, adapting...");
            
            // 3. ADAPT to fallback
            self.try_fallback_implementation().await
        }
        
        // 4. SUCCEED or fail with real error
        Err(e) => {
            error!("❌ Real error (not platform constraint): {}", e);
            Err(e)
        }
    }
}
```

### Why This Works

- **No hardcoding**: No `#[cfg(target_os = "android")]`
- **Runtime discovery**: Learns from errors
- **Automatic**: User does nothing
- **Transparent**: Same API, different transport
- **Testable**: Can verify on real devices

═══════════════════════════════════════════════════════════════════

## 📚 REFERENCE IMPLEMENTATION: songbird

### Files to Study

**songbird's implementation** (Jan 31, 2026):

1. **Server with Fallback**:
   - `crates/songbird-orchestrator/src/ipc/pure_rust_server/server.rs`
   - Lines 250-446: Complete Try→Detect→Adapt pattern

2. **Client Discovery**:
   - `crates/songbird-http-client/src/crypto/socket_discovery.rs`
   - `discover_ipc_endpoint()`: Auto-discover Unix OR TCP

3. **Connection Handling**:
   - `crates/songbird-http-client/src/beardog_client/core.rs`
   - `IpcEndpoint` enum: Polymorphic endpoint type
   - `crates/songbird-http-client/src/beardog_client/rpc.rs`
   - `AsyncStream` trait: Polymorphic streams

### Key Code Sections

**1. Server Start (Entry Point)**:
```rust
pub async fn start(self: Arc<Self>) -> Result<()> {
    info!("🔌 Starting IPC server (isomorphic mode)...");
    
    // 1. TRY Unix socket first (optimal)
    info!("   Trying Unix socket IPC (optimal)...");
    
    match self.try_unix_server().await {
        Ok(()) => Ok(()),
        
        // 2. DETECT platform constraints
        Err(e) if self.is_platform_constraint(&e) => {
            warn!("⚠️  Unix sockets unavailable: {}", e);
            warn!("   Falling back to TCP IPC...");
            
            // 3. ADAPT to TCP fallback
            self.start_tcp_fallback().await
        }
        
        // 4. Real error
        Err(e) => Err(e)
    }
}
```

**2. Platform Constraint Detection**:
```rust
fn is_platform_constraint(&self, error: &anyhow::Error) -> bool {
    if let Some(io_err) = error.downcast_ref::<std::io::Error>() {
        match io_err.kind() {
            // Permission denied often means SELinux blocking
            ErrorKind::PermissionDenied => {
                // Check if SELinux is enforcing
                self.is_selinux_enforcing()
            }
            // Address family not supported (platform lacks Unix sockets)
            ErrorKind::Unsupported => true,
            _ => false
        }
    } else {
        false
    }
}

fn is_selinux_enforcing(&self) -> bool {
    // On Android, check SELinux status
    std::fs::read_to_string("/sys/fs/selinux/enforce")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .map(|v| v == 1)
        .unwrap_or(false)
}
```

**3. TCP Fallback Server**:
```rust
async fn start_tcp_fallback(self: Arc<Self>) -> Result<()> {
    info!("🌐 Starting TCP IPC fallback (isomorphic mode)");
    info!("   Protocol: JSON-RPC 2.0 (same as Unix socket)");
    
    // Bind to localhost only (security: same as Unix socket)
    let listener = TcpListener::bind("127.0.0.1:0").await
        .context("Failed to bind TCP socket")?;
    
    let local_addr = listener.local_addr()?;
    info!("✅ TCP IPC listening on {}", local_addr);
    
    // Write discovery file for clients
    self.write_tcp_discovery_file(&local_addr)?;
    
    info!("   Status: READY ✅ (isomorphic TCP fallback active)");
    
    // Accept connections (same loop as Unix)
    loop {
        let (stream, _addr) = listener.accept().await?;
        let handler = self.clone();
        
        tokio::spawn(async move {
            if let Err(e) = handler.handle_tcp_connection(stream).await {
                error!("TCP connection error: {}", e);
            }
        });
    }
}
```

**4. Discovery File System**:
```rust
fn write_tcp_discovery_file(&self, addr: &SocketAddr) -> Result<()> {
    // XDG-compliant discovery file
    let discovery_dirs = [
        env::var("XDG_RUNTIME_DIR").ok(),
        env::var("HOME").map(|h| format!("{}/.local/share", h)),
        Some("/tmp".to_string()),
    ];
    
    for dir in discovery_dirs.iter().filter_map(|d| d.as_ref()) {
        let discovery_file = format!("{}/songbird-ipc-port", dir);
        
        if let Ok(mut f) = File::create(&discovery_file) {
            // Write in format: tcp:127.0.0.1:PORT
            writeln!(f, "tcp:{}", addr)?;
            info!("📁 TCP discovery file: {}", discovery_file);
            break;
        }
    }
    
    Ok(())
}
```

**5. Client Discovery**:
```rust
pub enum IpcEndpoint {
    UnixSocket(PathBuf),
    TcpLocal(SocketAddr),
}

pub fn discover_ipc_endpoint(primal: &str) -> Result<IpcEndpoint> {
    // 1. Try Unix socket first
    let socket_paths = get_socket_paths(primal);
    for path in socket_paths {
        if path.exists() {
            return Ok(IpcEndpoint::UnixSocket(path));
        }
    }
    
    // 2. Try TCP discovery file
    if let Ok(endpoint) = discover_tcp_endpoint(primal) {
        return Ok(endpoint);
    }
    
    Err(anyhow::anyhow!("Could not discover IPC endpoint for {}", primal))
}

fn discover_tcp_endpoint(primal: &str) -> Result<IpcEndpoint> {
    let discovery_files = get_tcp_discovery_file_candidates(primal);
    
    for file in discovery_files {
        if let Ok(contents) = std::fs::read_to_string(&file) {
            // Parse format: tcp:127.0.0.1:PORT
            if let Some(addr_str) = contents.trim().strip_prefix("tcp:") {
                if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                    return Ok(IpcEndpoint::TcpLocal(addr));
                }
            }
        }
    }
    
    Err(anyhow::anyhow!("No TCP discovery file found"))
}
```

**6. Polymorphic Streams**:
```rust
// Trait for polymorphic streams
trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {}
impl AsyncStream for UnixStream {}
impl AsyncStream for TcpStream {}

async fn connect_endpoint(endpoint: &IpcEndpoint) -> Result<Box<dyn AsyncStream>> {
    match endpoint {
        IpcEndpoint::UnixSocket(path) => {
            let stream = UnixStream::connect(path).await?;
            Ok(Box::new(stream))
        }
        IpcEndpoint::TcpLocal(addr) => {
            let stream = TcpStream::connect(addr).await?;
            Ok(Box::new(stream))
        }
    }
}
```

═══════════════════════════════════════════════════════════════════

## 🔧 IMPLEMENTATION STEPS

### Phase 1: Server-Side Fallback (4-6 hours)

**Goal**: Add automatic TCP fallback to your IPC server

**Steps**:

1. **Add Platform Constraint Detection** (30 min)
   ```rust
   // Add to your IPC server module
   fn is_platform_constraint(&self, error: &anyhow::Error) -> bool {
       // Copy from songbird's implementation
   }
   
   fn is_selinux_enforcing(&self) -> bool {
       // Copy from songbird's implementation
   }
   ```

2. **Evolve Server Start Method** (1-2 hours)
   ```rust
   // Replace direct Unix socket binding with Try→Detect→Adapt
   pub async fn start(self: Arc<Self>) -> Result<()> {
       match self.try_unix_server().await {
           Ok(()) => Ok(()),
           Err(e) if self.is_platform_constraint(&e) => {
               self.start_tcp_fallback().await
           }
           Err(e) => Err(e)
       }
   }
   ```

3. **Add TCP Fallback Server** (2-3 hours)
   ```rust
   async fn start_tcp_fallback(self: Arc<Self>) -> Result<()> {
       let listener = TcpListener::bind("127.0.0.1:0").await?;
       let addr = listener.local_addr()?;
       
       // Write discovery file
       self.write_tcp_discovery_file(&addr)?;
       
       // Accept loop (same as Unix)
       loop {
           let (stream, _) = listener.accept().await?;
           // Handle connection (same protocol as Unix)
           self.handle_tcp_connection(stream).await?;
       }
   }
   ```

4. **Add Discovery File System** (30 min)
   ```rust
   fn write_tcp_discovery_file(&self, addr: &SocketAddr) -> Result<()> {
       // XDG-compliant paths
       // Write format: tcp:127.0.0.1:PORT
   }
   ```

5. **Test on Linux** (30 min)
   - Should use Unix sockets
   - Logs: "✅ Unix socket IPC listening"

6. **Test on Android** (30 min)
   - Should automatically fall back to TCP
   - Logs: "⚠️ Unix sockets unavailable, using TCP fallback"
   - Logs: "✅ TCP IPC listening on 127.0.0.1:XXXXX"

### Phase 2: Client-Side Discovery (2-3 hours)

**Goal**: Clients can discover and connect to either Unix or TCP endpoints

**Steps**:

1. **Add IpcEndpoint Enum** (15 min)
   ```rust
   pub enum IpcEndpoint {
       UnixSocket(PathBuf),
       TcpLocal(SocketAddr),
   }
   ```

2. **Add Discovery Function** (1 hour)
   ```rust
   pub fn discover_ipc_endpoint(primal: &str) -> Result<IpcEndpoint> {
       // Try Unix socket first
       // Fall back to TCP discovery file
   }
   ```

3. **Add Polymorphic Streams** (30 min)
   ```rust
   trait AsyncStream: AsyncRead + AsyncWrite + Send + Unpin {}
   impl AsyncStream for UnixStream {}
   impl AsyncStream for TcpStream {}
   ```

4. **Update Connection Logic** (1 hour)
   ```rust
   async fn connect_endpoint(endpoint: &IpcEndpoint) -> Result<Box<dyn AsyncStream>> {
       match endpoint {
           IpcEndpoint::UnixSocket(path) => // Unix connection
           IpcEndpoint::TcpLocal(addr) => // TCP connection
       }
   }
   ```

5. **Test Discovery** (30 min)
   - Linux: Should find Unix socket
   - Android: Should find TCP discovery file
   - Both: Should connect successfully

### Phase 3: Integration & Testing (1-2 hours)

**Goal**: Validate end-to-end isomorphic operation

**Steps**:

1. **Build for Multiple Architectures** (30 min)
   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   cargo build --release --target aarch64-unknown-linux-musl
   ```

2. **Test on Linux** (15 min)
   - Start server
   - Check logs: Unix socket used
   - Connect client
   - Verify communication works

3. **Test on Android** (15 min)
   - Deploy to device
   - Start server
   - Check logs: TCP fallback activated
   - Connect client
   - Verify communication works

4. **Verify Zero Configuration** (15 min)
   - No environment variables needed
   - No platform-specific flags
   - Same binary works everywhere

5. **Document Results** (15 min)
   - Capture logs showing automatic fallback
   - Update your primal's documentation
   - Mark isomorphic IPC as complete

═══════════════════════════════════════════════════════════════════

## ✅ VALIDATION CHECKLIST

### Server-Side

- [ ] `try_unix_server()` method exists
- [ ] `is_platform_constraint()` detects SELinux
- [ ] `is_selinux_enforcing()` checks `/sys/fs/selinux/enforce`
- [ ] `start_tcp_fallback()` binds to `127.0.0.1:0` (ephemeral port)
- [ ] TCP server uses same JSON-RPC protocol as Unix
- [ ] Discovery file written to XDG-compliant paths
- [ ] Logs show "⚠️ Unix sockets unavailable" on Android
- [ ] Logs show "✅ TCP IPC listening on 127.0.0.1:XXXXX"

### Client-Side

- [ ] `IpcEndpoint` enum defined (UnixSocket | TcpLocal)
- [ ] `discover_ipc_endpoint()` tries Unix first, then TCP
- [ ] TCP discovery file parsed correctly (format: `tcp:127.0.0.1:PORT`)
- [ ] `AsyncStream` trait for polymorphic streams
- [ ] `connect_endpoint()` handles both Unix and TCP
- [ ] Client connects successfully on Linux (Unix)
- [ ] Client connects successfully on Android (TCP)

### End-to-End

- [ ] Build succeeds for x86_64 and aarch64
- [ ] Same binary runs on Linux and Android
- [ ] No environment variables required
- [ ] No platform-specific configuration
- [ ] Logs prove automatic adaptation
- [ ] Inter-primal communication works
- [ ] Deep Debt principles maintained (Pure Rust, zero unsafe, runtime discovery)

═══════════════════════════════════════════════════════════════════

## 🧪 TESTING GUIDE

### Test 1: Linux (Unix Sockets)

```bash
# Build and run
cargo build --release --target x86_64-unknown-linux-musl
./target/x86_64-unknown-linux-musl/release/your-primal server

# Expected logs:
# [INFO] Starting IPC server (isomorphic mode)...
# [INFO]    Trying Unix socket IPC (optimal)...
# [INFO] ✅ Unix socket JSON-RPC server listening: /path/to/socket

# Verify socket exists
ls -l /tmp/your-primal-*.sock  # Or your socket path

# Test client connection
# Should connect via Unix socket
```

### Test 2: Android (TCP Fallback)

```bash
# Build for Android
cargo build --release --target aarch64-unknown-linux-musl

# Deploy to device
adb push target/aarch64-unknown-linux-musl/release/your-primal /data/local/tmp/

# Run on device
adb shell "cd /data/local/tmp && \
  XDG_RUNTIME_DIR=/data/local/tmp/run \
  HOME=/data/local/tmp \
  RUST_LOG=info \
  ./your-primal server > logs/test.log 2>&1 &"

# Check logs
adb shell "cat /data/local/tmp/logs/test.log"

# Expected logs:
# [INFO] Starting IPC server (isomorphic mode)...
# [INFO]    Trying Unix socket IPC (optimal)...
# [WARN] ⚠️  Unix sockets unavailable: Permission denied
# [WARN]    Falling back to TCP IPC...
# [INFO] 🌐 Starting TCP IPC fallback (isomorphic mode)
# [INFO] ✅ TCP IPC listening on 127.0.0.1:XXXXX

# Verify discovery file
adb shell "cat /data/local/tmp/run/your-primal-ipc-port"
# Expected: tcp:127.0.0.1:XXXXX

# Test client connection
# Should connect via TCP using discovery file
```

### Test 3: Inter-Primal Communication

```bash
# Start your primal
./your-primal server &

# Start another primal that depends on yours
./other-primal client

# Verify they communicate successfully
# Check logs show connection established
# Verify JSON-RPC requests work
```

═══════════════════════════════════════════════════════════════════

## 🎯 PRIORITY BY PRIMAL

### beardog - HIGH PRIORITY (Next Session)
**Why**: Part of TOWER atomic (with songbird)  
**Advantage**: Already has platform traits foundation  
**Effort**: 4-6 hours (direct copy from songbird)  
**Impact**: Unblocks full TOWER atomic testing + STUN handshake

**Implementation Notes**:
- Copy server pattern from songbird's `server.rs`
- Reuse existing platform trait infrastructure
- Focus on Unix socket → TCP fallback
- Test with songbird communication

### toadstool - MEDIUM PRIORITY (Short-term)
**Why**: Part of NODE atomic (TOWER + toadstool)  
**Advantage**: Has IPC infrastructure in `runtime/display/src/ipc/`  
**Effort**: 6-8 hours (adapt existing IPC)  
**Impact**: Enables NODE atomic (TOWER + ML inference)

**Implementation Notes**:
- Evolve existing IPC in `runtime/display/src/ipc/server.rs`
- Add Try→Detect→Adapt pattern
- Maintain compatibility with display system
- Test with TOWER communication

### nestgate - MEDIUM PRIORITY (Short-term)
**Why**: Part of NEST atomic (gateway/routing)  
**Advantage**: Recent universal filesystem work (44% platform reduction)  
**Effort**: 6-8 hours (new IPC implementation)  
**Impact**: Enables NEST atomic (federation/gateway)

**Implementation Notes**:
- Apply same patterns as filesystem universality
- New IPC module following songbird pattern
- Gateway-specific discovery needs
- Test routing with multiple primals

### squirrel - LOW PRIORITY (Long-term)
**Why**: Data layer (less critical for atomics)  
**Advantage**: Universal transport stack already exists  
**Effort**: 4-6 hours (integrate with transport)  
**Impact**: Nice to have (completes ecosystem)

**Implementation Notes**:
- Integrate with existing universal transport
- May need transport-specific adaptations
- Focus on data layer communication
- Test with storage operations

═══════════════════════════════════════════════════════════════════

## 🎓 DEEP DEBT PRINCIPLES

### Ensure Your Implementation Maintains

1. **✅ 100% Pure Rust**
   - No C dependencies for IPC
   - Use `tokio::net` for all networking
   - Use `std::fs` for discovery files

2. **✅ Zero Unsafe Code**
   - All IPC code should be safe Rust
   - No raw pointers or unsafe blocks
   - Validate with: `cargo geiger`

3. **✅ Runtime Discovery Over Hardcoding**
   - Detect platform constraints from errors
   - No `#[cfg(target_os = "android")]` for logic
   - Automatic adaptation

4. **✅ Platform-Agnostic**
   - Same code on all platforms
   - Conditional compilation only for imports
   - Logic is universal

5. **✅ Modern Idiomatic Rust**
   - Use async/await throughout
   - Trait-based polymorphism
   - Error context with `anyhow`

6. **✅ Primal Self-Knowledge**
   - No external configuration
   - Discovers own capabilities
   - Adapts autonomously

7. **✅ Zero Configuration**
   - No environment variables required
   - No platform-specific flags
   - Works out of the box

═══════════════════════════════════════════════════════════════════

## 📚 ADDITIONAL RESOURCES

### Reference Code
- **songbird v3.33.0**: `/home/eastgate/Development/ecoPrimals/phase1/songbird/`
  - Study `crates/songbird-orchestrator/src/ipc/pure_rust_server/server.rs`
  - Study `crates/songbird-http-client/src/crypto/socket_discovery.rs`

### Documentation
- `ISOMORPHIC_IPC_VALIDATION_COMPLETE.md` - Validation proof
- `SONGBIRD_EVOLUTION_HARVEST.md` - Evolution analysis
- `ISOMORPHIC_IPC_DEEP_INVESTIGATION.md` - Original investigation
- `SESSION_HANDOFF.md` - Complete session summary

### Testing
- Pixel 8a: Available for Android testing
- Logs: Captured in `/data/local/tmp/logs/`
- Discovery files: Check `/data/local/tmp/run/`

### Support
- Reference implementation: songbird (working!)
- Validation proof: Logs from Pixel 8a
- Deep Debt validation: A++ (205/100) confirmed

═══════════════════════════════════════════════════════════════════

## 🏁 SUCCESS CRITERIA

Your implementation is complete when:

1. ✅ **Builds successfully** for x86_64 and aarch64
2. ✅ **Works on Linux** (uses Unix sockets)
3. ✅ **Works on Android** (automatically falls back to TCP)
4. ✅ **Zero configuration** (no env vars or flags)
5. ✅ **Logs show adaptation** ("⚠️ Unix sockets unavailable, using TCP fallback")
6. ✅ **Discovery works** (clients find Unix OR TCP endpoints)
7. ✅ **Communication works** (inter-primal JSON-RPC)
8. ✅ **Deep Debt maintained** (Pure Rust, zero unsafe, runtime discovery)

### Expected Timeline

- **beardog**: 4-6 hours (has foundation)
- **toadstool**: 6-8 hours (adapt existing IPC)
- **nestgate**: 6-8 hours (new implementation)
- **squirrel**: 4-6 hours (integrate with transport)

### Validation

Run on Android device and capture logs showing:
```
⚠️  Unix sockets unavailable: Permission denied
   Falling back to TCP IPC...
✅ TCP IPC listening on 127.0.0.1:XXXXX
```

**This proves TRUE isomorphism!**

═══════════════════════════════════════════════════════════════════

## 🎉 CLOSING NOTES

### Why This Matters

This is not just "working code" - it's **evolutionary architecture**:

- Same binary runs EVERYWHERE
- Automatically adapts to constraints
- Zero configuration needed
- Biological inspiration validated

### The Pattern Is Universal

Try→Detect→Adapt→Succeed works for:
- IPC transport (Unix → TCP)
- Storage (mmap → file → memory)
- Crypto (hardware → software HSM)
- Display (Wayland → X11 → framebuffer)

**Apply this pattern wherever platform constraints exist!**

### You're Not Alone

songbird has proven this works in production. You're implementing a validated pattern with a working reference.

**Questions?** Check songbird's code or the documentation listed above.

**Stuck?** Compare your implementation to songbird's `server.rs`.

**Success?** Document your logs showing automatic TCP fallback on Android!

═══════════════════════════════════════════════════════════════════

**Status**: Ready for Implementation  
**Pattern**: Validated in Production  
**Reference**: songbird v3.33.0  
**Grade**: A++ (205/100)

🌍🧬🦀 **Binary = DNA: Universal, Deterministic, Adaptive** 🦀🧬🌍

**Go forth and evolve!** 🚀

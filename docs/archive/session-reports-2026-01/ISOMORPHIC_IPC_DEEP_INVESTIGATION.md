# Deep Investigation: Isomorphic IPC Architecture Gap

**Date**: January 31, 2026  
**Issue**: Platform-specific flags break isomorphism principle  
**Root Cause**: Error handling doesn't attempt TCP fallback automatically

═══════════════════════════════════════════════════════════════════

## 🎯 The Core Problem

**User's Insight**: "If we have to set custom flags, it's really not isomorphic"

**✅ CORRECT** - The proposed `PRIMAL_IPC_MODE=tcp` solution breaks TRUE ecoBin v2.0 principles:

- ❌ Requires platform-specific configuration
- ❌ Manual intervention needed
- ❌ Breaks "write once, run everywhere" isomorphism
- ❌ Violates primal autonomy (should self-adapt)

═══════════════════════════════════════════════════════════════════

## 🔬 Current Architecture Analysis

### What EXISTS Today

**File**: `songbird/crates/songbird-orchestrator/src/ipc/pure_rust_server/server.rs`

**Line 267-268**:
```rust
let listener = UnixListener::bind(&*self.socket_path)
    .context(format!("Failed to bind Unix socket: {}", self.socket_path.display()))?;
```

**Problem**: Uses `?` operator → **immediately returns error**

**Result**: No fallback attempt, no TCP retry, just exits

### What FAILS in app/core.rs

**Line 694-696**:
```rust
if let Err(e) = server_arc.start().await {
    error!("❌ Unix Socket IPC server error: {}", e);
}
```

**Problem**: Just logs error → **server never starts**

**Result**: Songbird continues running but IPC is dead

═══════════════════════════════════════════════════════════════════

## 🧬 The Gap: Missing Isomorphic Fallback

### What SHOULD Happen (Isomorphic Design)

```rust
// EVOLVED PATTERN: Try → Detect → Adapt → Succeed
pub async fn start(&self) -> Result<()> {
    // 1. TRY Unix sockets first
    match UnixListener::bind(&*self.socket_path) {
        Ok(listener) => {
            info!("✅ Using Unix socket IPC: {}", self.socket_path.display());
            self.run_unix_server(listener).await
        }
        Err(e) => {
            // 2. DETECT why it failed
            if self.is_selinux_blocking(&e) || self.is_permission_error(&e) {
                warn!("⚠️  Unix sockets unavailable: {}", e);
                warn!("   Detected platform constraint");
                warn!("   Falling back to TCP IPC...");
                
                // 3. ADAPT automatically
                self.start_tcp_fallback().await
            } else {
                // Real error, not a platform constraint
                Err(e).context("Failed to start IPC server")
            }
        }
    }
}
```

### Key Differences

**Current (Broken Isomorphism)**:
- ❌ Fails immediately
- ❌ Requires manual flag
- ❌ Platform-specific config

**Evolved (True Isomorphism)**:
- ✅ Tries Unix first
- ✅ Detects platform constraints automatically
- ✅ Falls back to TCP without intervention
- ✅ Same binary, works everywhere

═══════════════════════════════════════════════════════════════════

## 🔍 Detection Logic Needed

### How to Detect SELinux Blocking

```rust
fn is_selinux_blocking(&self, error: &std::io::Error) -> bool {
    match error.kind() {
        // Permission denied on Android often means SELinux
        std::io::ErrorKind::PermissionDenied => {
            #[cfg(target_os = "android")]
            {
                // On Android, if we can create files but not sockets,
                // it's almost certainly SELinux
                true
            }
            #[cfg(not(target_os = "android"))]
            {
                // On other platforms, check if getenforce says Enforcing
                self.check_selinux_status()
            }
        }
        _ => false,
    }
}

fn check_selinux_status(&self) -> bool {
    // Try to read /sys/fs/selinux/enforce
    if let Ok(contents) = std::fs::read_to_string("/sys/fs/selinux/enforce") {
        return contents.trim() == "1";
    }
    false
}
```

### Android-Specific Detection

```rust
fn is_android_restricted_env(&self) -> bool {
    #[cfg(target_os = "android")]
    {
        // Check if we're running as shell user in /data/local/tmp
        let current_dir = std::env::current_dir().ok();
        let is_tmp = current_dir
            .as_ref()
            .and_then(|p| p.to_str())
            .map(|s| s.contains("/data/local/tmp"))
            .unwrap_or(false);
        
        // Check if we're non-root
        let is_shell_user = std::env::var("USER")
            .ok()
            .map(|u| u == "shell")
            .unwrap_or(false);
        
        is_tmp && is_shell_user
    }
    #[cfg(not(target_os = "android"))]
    {
        false
    }
}
```

═══════════════════════════════════════════════════════════════════

## 🏗️ TCP Fallback Implementation

### What TCP Fallback Needs

```rust
async fn start_tcp_fallback(&self) -> Result<()> {
    info!("🌐 Starting TCP IPC fallback");
    
    // 1. Bind to localhost only (security)
    let addr = "127.0.0.1:0".parse::<SocketAddr>()?;
    let listener = TcpListener::bind(addr).await?;
    let bound_addr = listener.local_addr()?;
    
    info!("✅ TCP IPC listening on {}", bound_addr);
    
    // 2. Write port to discoverable location
    self.write_tcp_port_file(bound_addr.port())?;
    
    // 3. Run TCP server with same protocol
    self.run_tcp_server(listener).await
}

fn write_tcp_port_file(&self, port: u16) -> Result<()> {
    // Write to location where clients can discover it
    // Priority: XDG_RUNTIME_DIR > HOME/.local > /tmp
    let port_file = self.get_port_file_path();
    std::fs::write(&port_file, port.to_string())?;
    info!("   Port file: {}", port_file.display());
    Ok(())
}
```

### Client-Side Discovery

```rust
// In songbird_http_client or similar
pub fn discover_ipc_endpoint() -> Result<IpcEndpoint> {
    // 1. Try Unix socket first
    if let Some(socket_path) = Self::find_unix_socket() {
        if socket_path.exists() {
            return Ok(IpcEndpoint::Unix(socket_path));
        }
    }
    
    // 2. Try TCP port file
    if let Some(port) = Self::read_tcp_port_file() {
        return Ok(IpcEndpoint::Tcp(SocketAddr::from(([127, 0, 0, 1], port))));
    }
    
    Err(anyhow::anyhow!("Could not discover IPC endpoint"))
}
```

═══════════════════════════════════════════════════════════════════

## 🎯 Which Primals Need Evolution?

### PRIMARY: songbird (Orchestrator)

**Why**: Central hub, all primals connect TO songbird

**Files to Evolve**:
1. `songbird-orchestrator/src/ipc/pure_rust_server/server.rs`
   - Add `start_tcp_fallback()` method
   - Add `is_selinux_blocking()` detection
   - Evolve `start()` to try→detect→adapt

2. `songbird-orchestrator/src/app/core.rs`
   - Update error handling for IPC server startup
   - Log fallback mode clearly

3. **NEW**: `songbird-orchestrator/src/ipc/tcp_server.rs`
   - TCP variant of Unix socket server
   - Same JSON-RPC protocol
   - Port discovery mechanism

**Impact**: ⭐⭐⭐⭐⭐ **CRITICAL** - Unblocks all Android deployments

### SECONDARY: beardog (Security Provider)

**Why**: Songbird connects TO beardog for crypto

**Files to Evolve**:
1. Similar pattern to songbird
2. Beardog IPC server needs same fallback logic
3. Less critical if songbird's HTTP API works

**Impact**: ⭐⭐⭐ **HIGH** - Needed for full TOWER functionality

### TERTIARY: All Client Code

**Files to Evolve**:
1. `songbird-http-client/src/beardog_client/`
   - Socket discovery → TCP discovery fallback
   
2. `songbird-universal-ipc/src/`
   - Already has platform abstractions
   - Add TCP fallback to discovery

**Impact**: ⭐⭐ **MEDIUM** - Enables clients to find TCP endpoints

═══════════════════════════════════════════════════════════════════

## 🧩 Deep Debt Analysis

### Current Grade: A++ (205/100)

**Why This Doesn't Reduce Grade**:

The current architecture ALREADY has the right pieces:
- ✅ Platform abstraction exists (`songbird-universal-ipc`)
- ✅ Error handling infrastructure exists
- ✅ TCP server capability exists (HTTP server proves it)
- ✅ Discovery patterns exist

**What's Missing**: Automatic fallback logic connecting these pieces

**This is**: **Feature Gap**, not architectural debt

### Evolution Gaps Revealed

1. **Error Classification**
   - Need: Distinguish "platform constraint" from "real error"
   - Solution: Platform detection in error handler

2. **Fallback Strategy**
   - Need: Automatic TCP fallback without config
   - Solution: Try→Detect→Adapt pattern

3. **Discovery Protocol**
   - Need: Clients find TCP endpoints automatically
   - Solution: Port file + discovery priority list

4. **Testing Coverage**
   - Need: Validate fallback on constrained platforms
   - Solution: Android emulator in CI/CD

═══════════════════════════════════════════════════════════════════

## 🎓 Lessons for TRUE Isomorphism

### What We Learned

1. **Platform Constraints Are Data, Not Config**
   - Don't ask user "what platform?"
   - Detect it automatically from errors

2. **Fallbacks Should Be Invisible**
   - User shouldn't know or care
   - Log for observability, but work transparently

3. **Error Messages Are Platform Signals**
   - PermissionDenied on Android → SELinux
   - Use errors as runtime discovery input

4. **Test On Constraints, Not Just Success**
   - Don't just test "does it work"
   - Test "does it adapt when blocked"

### Applying to Other Primals

**Pattern**: Try → Detect → Adapt → Succeed

```rust
// UNIVERSAL PATTERN for any primal capability
async fn start_capability(&self) -> Result<()> {
    // 1. Try optimal path first
    match self.try_optimal_implementation().await {
        Ok(result) => Ok(result),
        Err(e) => {
            // 2. Detect if it's a platform constraint
            if self.is_platform_constraint(&e) {
                // 3. Adapt to alternative implementation
                self.try_fallback_implementation().await
            } else {
                // 4. Real error, propagate
                Err(e)
            }
        }
    }
}
```

═══════════════════════════════════════════════════════════════════

## 🚀 Implementation Plan

### Phase 1: songbird IPC Evolution (Critical)

**Goal**: Automatic TCP fallback for Unix socket failures

**Tasks**:
1. Add platform constraint detection
2. Implement TCP fallback server
3. Add port discovery mechanism
4. Update error handling in app/core.rs
5. Test on Android

**Time**: 3-4 hours
**Impact**: Unblocks all Android deployments

### Phase 2: beardog IPC Evolution (High Priority)

**Goal**: Same pattern for beardog

**Tasks**:
1. Apply same pattern to beardog IPC
2. Ensure discovery works both ways
3. Test TOWER atomic with TCP IPC

**Time**: 2-3 hours
**Impact**: Full TOWER functionality on Android

### Phase 3: Universal Client Discovery (Medium Priority)

**Goal**: All clients find TCP endpoints automatically

**Tasks**:
1. Evolve songbird-http-client discovery
2. Update songbird-universal-ipc
3. Add discovery tests

**Time**: 2 hours
**Impact**: Complete isomorphic deployment

### Phase 4: CI/CD Validation (Lower Priority)

**Goal**: Prevent regression

**Tasks**:
1. Add Android emulator to CI
2. Test socket binding failures
3. Validate fallback behavior

**Time**: 4-6 hours
**Impact**: Continuous validation

═══════════════════════════════════════════════════════════════════

## ✅ Success Criteria

### Definition of "True Isomorphism Achieved"

1. ✅ Same binary runs on x86_64 and ARM64
2. ✅ Same binary runs on Linux and Android
3. ✅ No platform-specific environment variables needed
4. ✅ Automatic adaptation to platform constraints
5. ✅ Transparent fallback (user unaware)
6. ✅ Logs show what happened (observability)
7. ✅ Deep Debt grade maintained or improved

### Test Cases

```bash
# Test 1: Linux with Unix sockets (should use Unix)
./songbird.genome run
# Expected: "✅ Using Unix socket IPC: /run/user/1000/songbird.sock"

# Test 2: Android with SELinux (should use TCP)
adb shell "./songbird.genome run"
# Expected: "⚠️  Unix sockets unavailable"
# Expected: "✅ TCP IPC listening on 127.0.0.1:xxxxx"

# Test 3: Client discovery
./beardog.genome run
# Expected: Finds songbird via Unix socket OR TCP automatically
```

═══════════════════════════════════════════════════════════════════

## 🎯 Recommendation

### DON'T Implement PRIMAL_IPC_MODE Flag

**Reasoning**:
- Breaks isomorphism
- Requires manual configuration
- Violates primal autonomy
- Not a TRUE ecoBin v2.0 solution

### DO Evolve Songbird IPC Server

**Reasoning**:
- Maintains isomorphism
- No configuration needed
- Platform-agnostic
- TRUE primal autonomy
- Follows Deep Debt principles

### Primary Evolution Target

**Primal**: songbird  
**File**: `songbird-orchestrator/src/ipc/pure_rust_server/server.rs`  
**Method**: `start()` → Add Try→Detect→Adapt pattern  
**Impact**: Unblocks ALL Android deployments with zero config  

═══════════════════════════════════════════════════════════════════

**Status**: Gap Identified ✅  
**Solution**: Clear Evolution Path ✅  
**Isomorphism**: Can Be Preserved ✅  
**Next**: Evolve songbird IPC with automatic TCP fallback 🚀

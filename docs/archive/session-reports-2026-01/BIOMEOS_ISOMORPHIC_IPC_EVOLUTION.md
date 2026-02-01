# biomeOS Isomorphic IPC Evolution Plan

**Date**: January 31, 2026  
**Status**: Analysis Complete → Ready for Evolution  
**Deep Debt Grade**: Current B+ → Target A++

═══════════════════════════════════════════════════════════════════

## 🔍 CURRENT STATE ANALYSIS

### What biomeOS Already Has ✅

1. **Platform-Agnostic IPC Module** (`biomeos-core/src/ipc/`)
   - `transport.rs`: Universal transport abstraction
   - Supports Unix sockets, abstract sockets (Android), TCP localhost
   - `detect_best_transport()` function for platform detection
   - `AsyncReadWrite` trait for polymorphic streams

2. **Socket Discovery** (`biomeos-core/src/socket_discovery.rs`)
   - XDG-compliant discovery
   - Environment variable hints
   - Family-scoped isolation
   - Capability registry queries
   - Caching system

3. **Unix Socket Server** (`biomeos-api/src/unix_server.rs`)
   - JSON-RPC 2.0 over Unix sockets
   - Owner-only permissions (0600)
   - Axum router serving

4. **Unix Socket Client** (`biomeos-federation/src/unix_socket_client.rs`)
   - JSON-RPC 2.0 client
   - Connection handling

5. **Neural API Server** (`biomeos-atomic-deploy/src/neural_api_server.rs`)
   - Graph orchestration
   - Unix socket listener
   - Deployment coordination

### What's Missing ❌

biomeOS has GREAT platform abstraction foundations, but lacks the **Try→Detect→Adapt→Succeed** pattern:

1. **No Automatic TCP Fallback**
   - Server code directly binds Unix sockets without trying/detecting/adapting
   - If Unix socket binding fails (like on Android SELinux), server crashes
   - No detection of platform constraints vs real errors

2. **Hardcoded Platform Detection**
   - Uses compile-time `#[cfg(target_os = "android")]`
   - Doesn't detect *runtime* constraints (like SELinux enforcement)
   - Abstract sockets assumed always available on Android (not true!)

3. **No Discovery Files for TCP**
   - When TCP is used, clients don't know how to find the port
   - No XDG-compliant TCP discovery file system

4. **Client-Side Missing Automatic Discovery**
   - Clients assume Unix socket works
   - No automatic retry with TCP discovery

5. **Split Architecture**
   - Server and client logic separate (not bad, but needs coordination)
   - No shared `IpcEndpoint` enum like songbird

═══════════════════════════════════════════════════════════════════

## 🎯 EVOLUTION STRATEGY

### Phase 1: Add Isomorphic Server Pattern
**Duration**: 3-4 hours  
**Files**: 4-5 files

### Phase 2: Add Client Discovery 
**Duration**: 2-3 hours  
**Files**: 3-4 files

### Phase 3: Evolution Atomic Deployment
**Duration**: 2-3 hours  
**Files**: 2-3 files

**Total**: 7-10 hours across 9-12 files

═══════════════════════════════════════════════════════════════════

## 📋 PHASE 1: ISOMORPHIC SERVER PATTERN (3-4 hours)

### Goal
Evolve biomeOS servers to automatically fall back to TCP when Unix sockets fail due to platform constraints.

### Files to Modify

#### 1. `biomeos-core/src/ipc/transport.rs` (HIGH PRIORITY)

**Current State**: Has platform detection, but no Try→Detect→Adapt pattern

**Evolution**:
```rust
// Add to Transport struct
impl Transport {
    /// Start server with automatic platform adaptation
    pub async fn bind_with_fallback(&self) -> Result<Box<dyn TransportListener>> {
        // 1. TRY optimal transport first
        match self.bind().await {
            Ok(listener) => {
                info!("✅ Using optimal transport: {:?}", self.transport_type);
                Ok(listener)
            }
            
            // 2. DETECT platform constraints
            Err(e) if self.is_platform_constraint(&e) => {
                warn!("⚠️  Optimal transport unavailable: {}", e);
                warn!("   Detected platform constraint, adapting...");
                
                // 3. ADAPT to TCP fallback
                self.start_tcp_fallback().await
            }
            
            // 4. Real error
            Err(e) => Err(e)
        }
    }
    
    /// Detect if error is due to platform constraint
    fn is_platform_constraint(&self, error: &anyhow::Error) -> bool {
        if let Some(io_err) = error.downcast_ref::<std::io::Error>() {
            match io_err.kind() {
                ErrorKind::PermissionDenied => {
                    // Check SELinux enforcement
                    is_selinux_enforcing()
                }
                ErrorKind::Unsupported => true,
                _ => false
            }
        } else {
            false
        }
    }
    
    /// Start TCP fallback server
    async fn start_tcp_fallback(&self) -> Result<Box<dyn TransportListener>> {
        info!("🌐 Starting TCP IPC fallback (isomorphic mode)");
        
        // Bind ephemeral port
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let local_addr = listener.local_addr()?;
        
        info!("✅ TCP IPC listening on {}", local_addr);
        
        // Write discovery file for clients
        self.write_tcp_discovery_file(&local_addr)?;
        
        Ok(Box::new(TcpTransportListener { listener }))
    }
    
    /// Write TCP discovery file (XDG-compliant)
    fn write_tcp_discovery_file(&self, addr: &SocketAddr) -> Result<()> {
        // Get service name from transport type
        let service_name = match &self.transport_type {
            TransportType::UnixSocket { path } => {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
            }
            _ => "unknown"
        };
        
        // Try XDG locations
        let discovery_dirs = [
            env::var("XDG_RUNTIME_DIR").ok(),
            env::var("HOME").map(|h| format!("{}/.local/share", h)),
            Some("/tmp".to_string()),
        ];
        
        for dir in discovery_dirs.iter().filter_map(|d| d.as_ref()) {
            let discovery_file = format!("{}/{}-ipc-port", dir, service_name);
            
            if let Ok(mut f) = File::create(&discovery_file) {
                writeln!(f, "tcp:{}", addr)?;
                info!("📁 TCP discovery file: {}", discovery_file);
                return Ok(());
            }
        }
        
        Ok(())
    }
}

/// Check if SELinux is enforcing
fn is_selinux_enforcing() -> bool {
    std::fs::read_to_string("/sys/fs/selinux/enforce")
        .ok()
        .and_then(|s| s.trim().parse::<u8>().ok())
        .map(|v| v == 1)
        .unwrap_or(false)
}
```

**Lines to Add**: ~150 lines  
**Testing**: Test on Linux (should use Unix) and Android (should fall back to TCP)

#### 2. `biomeos-api/src/unix_server.rs` → `biomeos-api/src/isomorphic_server.rs`

**Current State**: Hardcoded Unix socket binding

**Evolution**:
```rust
//! Isomorphic IPC server for biomeOS API
//!
//! Automatically adapts between Unix socket and TCP based on platform constraints.

use biomeos_core::ipc::{Transport, TransportType, TransportListener};

/// Serve an Axum router with automatic platform adaptation
pub async fn serve_isomorphic<P: AsRef<Path>>(
    socket_path: P, 
    app: Router
) -> Result<()> {
    let socket_path = socket_path.as_ref();
    
    // Create transport for Unix socket
    let transport = Transport::new(TransportType::UnixSocket { 
        path: socket_path.to_path_buf() 
    });
    
    // Bind with automatic fallback
    let mut listener = transport.bind_with_fallback().await?;
    
    info!("📡 biomeOS API ready (isomorphic mode)");
    info!("   Protocol: JSON-RPC 2.0");
    info!("   Security: Localhost-only");
    
    // Accept connections
    loop {
        match listener.accept().await {
            Ok(stream) => {
                let app = app.clone();
                tokio::spawn(async move {
                    // Handle connection with Hyper
                    // (same code as before)
                });
            }
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
            }
        }
    }
}
```

**Lines to Change**: ~50 lines  
**New File**: Rename `unix_server.rs` to `isomorphic_server.rs`

#### 3. `biomeos-atomic-deploy/src/neural_api_server.rs`

**Current State**: Hardcoded UnixListener::bind()

**Evolution**:
```rust
// In NeuralApiServer::run() method

// OLD CODE (line ~135):
let listener = UnixListener::bind(&self.socket_path)
    .context("Failed to bind Neural API Unix socket")?;

// NEW CODE:
let transport = Transport::new(TransportType::UnixSocket {
    path: self.socket_path.clone()
});

let mut listener = transport.bind_with_fallback().await
    .context("Failed to bind Neural API")?;

info!("🧠 Neural API ready (isomorphic mode)");
info!("   Socket: {}", self.socket_path.display());

// Accept connections loop remains same, but use TransportListener trait
loop {
    match listener.accept().await {
        Ok(stream) => {
            // Handle connection (same as before)
        }
        Err(e) => {
            warn!("Failed to accept connection: {}", e);
        }
    }
}
```

**Lines to Change**: ~20 lines  
**Testing**: Deploy to Android and verify TCP fallback

#### 4. `biomeos-core/src/ipc/mod.rs`

**Current State**: Exports `detect_best_transport()`

**Evolution**: Add exports for new functions
```rust
pub use transport::{
    detect_best_transport,
    is_selinux_enforcing,  // NEW
    Transport, 
    TransportType,
    TransportListener,     // NEW
};
```

**Lines to Change**: ~5 lines

### Testing Phase 1

1. **Linux Desktop**: 
   ```bash
   cargo test -p biomeos-core --lib ipc
   cargo run -p biomeos-api
   # Should use Unix socket
   # Logs: "✅ Using optimal transport: UnixSocket"
   ```

2. **Android (Pixel 8a)**:
   ```bash
   # Deploy and run
   adb push target/aarch64-unknown-linux-musl/release/biomeos-api /data/local/tmp/
   adb shell "cd /data/local/tmp && ./biomeos-api"
   
   # Expected logs:
   # [WARN] ⚠️  Optimal transport unavailable: Permission denied
   # [INFO] 🌐 Starting TCP IPC fallback (isomorphic mode)
   # [INFO] ✅ TCP IPC listening on 127.0.0.1:XXXXX
   # [INFO] 📁 TCP discovery file: /data/local/tmp/run/biomeos-api-ipc-port
   ```

═══════════════════════════════════════════════════════════════════

## 📋 PHASE 2: CLIENT DISCOVERY (2-3 hours)

### Goal
Evolve clients to automatically discover and connect to either Unix or TCP endpoints.

### Files to Modify

#### 1. `biomeos-core/src/ipc/transport.rs` (Add client discovery)

**Evolution**:
```rust
/// IPC endpoint (Unix socket or TCP)
#[derive(Debug, Clone)]
pub enum IpcEndpoint {
    UnixSocket(PathBuf),
    TcpLocal(SocketAddr),
}

impl Transport {
    /// Discover and connect to IPC endpoint
    pub async fn connect_with_discovery(
        service_name: &str
    ) -> Result<Box<dyn AsyncReadWrite>> {
        // 1. Try Unix socket first
        if let Some(endpoint) = discover_unix_socket(service_name) {
            if let Ok(stream) = UnixStream::connect(&endpoint).await {
                info!("Connected via Unix socket: {}", endpoint.display());
                return Ok(Box::new(stream));
            }
        }
        
        // 2. Try TCP discovery file
        if let Some(port) = discover_tcp_port(service_name) {
            let addr = SocketAddr::from(([127, 0, 0, 1], port));
            if let Ok(stream) = TcpStream::connect(addr).await {
                info!("Connected via TCP: {}", addr);
                return Ok(Box::new(stream));
            }
        }
        
        Err(anyhow::anyhow!("Could not discover IPC endpoint for {}", service_name))
    }
}

/// Discover Unix socket path for service
fn discover_unix_socket(service_name: &str) -> Option<PathBuf> {
    // Try XDG runtime dir
    if let Some(runtime_dir) = env::var("XDG_RUNTIME_DIR").ok() {
        let socket_path = PathBuf::from(runtime_dir)
            .join("biomeos")
            .join(format!("{}.sock", service_name));
        if socket_path.exists() {
            return Some(socket_path);
        }
    }
    
    // Try /tmp
    let tmp_path = PathBuf::from(format!("/tmp/{}.sock", service_name));
    if tmp_path.exists() {
        return Some(tmp_path);
    }
    
    None
}

/// Discover TCP port from discovery file
fn discover_tcp_port(service_name: &str) -> Option<u16> {
    let discovery_files = [
        env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(|d| format!("{}/{}-ipc-port", d, service_name)),
        Some(format!("/tmp/{}-ipc-port", service_name)),
    ];
    
    for file in discovery_files.iter().filter_map(|f| f.as_ref()) {
        if let Ok(contents) = std::fs::read_to_string(file) {
            // Parse format: tcp:127.0.0.1:PORT
            if let Some(addr_str) = contents.trim().strip_prefix("tcp:") {
                if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                    return Some(addr.port());
                }
            }
        }
    }
    
    None
}
```

**Lines to Add**: ~100 lines

#### 2. `biomeos-federation/src/unix_socket_client.rs` → `biomeos-federation/src/isomorphic_client.rs`

**Evolution**:
```rust
//! Isomorphic IPC client for biomeOS
//!
//! Automatically discovers and connects to Unix socket or TCP endpoint.

use biomeos_core::ipc::Transport;

pub struct IsomorphicClient {
    service_name: String,
}

impl IsomorphicClient {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }
    
    /// Call a JSON-RPC method (automatically discovers endpoint)
    pub async fn call_method(&self, method: impl Into<String>, params: Value) -> Result<Value> {
        // Discover and connect
        let mut stream = Transport::connect_with_discovery(&self.service_name).await?;
        
        // Send JSON-RPC request
        let request = JsonRpcRequest::new(method, params);
        let request_str = serde_json::to_string(&request)? + "\n";
        
        stream.write_all(request_str.as_bytes()).await?;
        stream.flush().await?;
        
        // Read response
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        
        let response: JsonRpcResponse = serde_json::from_str(&line)?;
        
        if let Some(error) = response.error {
            return Err(anyhow::anyhow!("JSON-RPC error: {}", error.message));
        }
        
        response.result
            .ok_or_else(|| anyhow::anyhow!("Missing result"))
    }
}
```

**Lines to Change**: ~80 lines  
**New File**: Rename or create `isomorphic_client.rs`

#### 3. Update Socket Discovery

**Evolution** to `biomeos-core/src/socket_discovery.rs`:
```rust
// Add TCP discovery file checking
async fn discover_via_tcp_file(&self, primal_name: &str) -> Option<DiscoveredSocket> {
    let discovery_file = format!("{}-ipc-port", primal_name);
    
    // Try XDG runtime dir
    if let Some(runtime_dir) = env::var("XDG_RUNTIME_DIR").ok() {
        let file_path = PathBuf::from(runtime_dir).join(&discovery_file);
        if let Ok(contents) = std::fs::read_to_string(&file_path) {
            if let Some(addr_str) = contents.trim().strip_prefix("tcp:") {
                if let Ok(addr) = addr_str.parse::<SocketAddr>() {
                    return Some(DiscoveredSocket {
                        path: PathBuf::from(format!("tcp://{}", addr)),
                        discovered_via: DiscoveryMethod::TcpDiscoveryFile,
                        primal_name: Some(primal_name.to_string()),
                        capabilities: Vec::new(),
                    });
                }
            }
        }
    }
    
    // Try /tmp
    let tmp_file = PathBuf::from(format!("/tmp/{}", discovery_file));
    if let Ok(contents) = std::fs::read_to_string(&tmp_file) {
        // (same parsing logic)
    }
    
    None
}
```

**Lines to Add**: ~40 lines

### Testing Phase 2

1. **Test Client Discovery on Linux**:
   ```bash
   # Start server (should use Unix socket)
   cargo run -p biomeos-api &
   
   # Test client
   cargo test -p biomeos-federation test_isomorphic_client_discovery
   # Should find Unix socket
   ```

2. **Test Client Discovery on Android**:
   ```bash
   # Server should be using TCP fallback
   # Client should discover via TCP discovery file
   adb shell "cd /data/local/tmp && cat run/biomeos-api-ipc-port"
   # Expected: tcp:127.0.0.1:XXXXX
   ```

═══════════════════════════════════════════════════════════════════

## 📋 PHASE 3: ATOMIC DEPLOYMENT (2-3 hours)

### Goal
Evolve atomic deployment code to use isomorphic IPC.

### Files to Modify

#### 1. `biomeos-atomic-deploy/src/primal_launcher.rs`

**Current Issue**: Hardcodes socket paths in environment variables

**Evolution**:
```rust
impl PrimalLauncher {
    /// Launch primal with isomorphic IPC
    pub async fn launch_isomorphic(
        &mut self,
        primal_name: &str,
        env: HashMap<String, String>,
    ) -> Result<PrimalInstance> {
        let binary_path = self.find_binary(primal_name)?;
        
        info!("🚀 Launching {} (isomorphic mode)", primal_name);
        
        // DON'T hardcode socket path - let primal discover it!
        // Remove socket path from env
        
        // Add runtime discovery hint
        let mut env = env;
        env.insert("BIOMEOS_IPC_MODE".to_string(), "isomorphic".to_string());
        env.insert("PRIMAL_NAME".to_string(), primal_name.to_string());
        
        // Launch process
        let mut cmd = Command::new(&binary_path);
        cmd.args(&["server"]);  // Standard subcommand
        
        for (key, value) in env {
            cmd.env(key, value);
        }
        
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        let child = cmd.spawn()?;
        let pid = child.id().ok_or_else(|| anyhow::anyhow!("No PID"))?;
        
        info!("✅ Launched {} (PID: {})", primal_name, pid);
        
        // Wait for IPC endpoint to appear (discover it!)
        let socket_path = self.discover_primal_endpoint(primal_name).await?;
        
        Ok(PrimalInstance {
            primal_name: primal_name.to_string(),
            pid,
            socket_path,
            started_at: chrono::Utc::now(),
        })
    }
    
    /// Discover primal's IPC endpoint (Unix or TCP)
    async fn discover_primal_endpoint(&self, primal_name: &str) -> Result<PathBuf> {
        use tokio::time::{sleep, Duration};
        
        info!("🔍 Discovering IPC endpoint for {}", primal_name);
        
        // Poll for up to 10 seconds
        for attempt in 1..=50 {
            // Try Unix socket
            if let Some(socket) = discover_unix_socket(primal_name) {
                info!("✅ Discovered Unix socket: {}", socket.display());
                return Ok(socket);
            }
            
            // Try TCP discovery file
            if let Some(port) = discover_tcp_port(primal_name) {
                let pseudo_path = PathBuf::from(format!("tcp://127.0.0.1:{}", port));
                info!("✅ Discovered TCP endpoint: {}", pseudo_path.display());
                return Ok(pseudo_path);
            }
            
            if attempt % 10 == 0 {
                debug!("Still waiting for {} endpoint... (attempt {})", primal_name, attempt);
            }
            
            sleep(Duration::from_millis(200)).await;
        }
        
        Err(anyhow::anyhow!("Timeout discovering IPC endpoint for {}", primal_name))
    }
}
```

**Lines to Change**: ~100 lines  
**Impact**: Primals now discover their own IPC mechanism without hardcoded hints!

#### 2. `biomeos-atomic-deploy/src/primal_coordinator.rs`

**Evolution**: Update health checks to use isomorphic client

```rust
use biomeos_federation::IsomorphicClient;

impl PrimalCoordinator {
    /// Health check using isomorphic IPC
    async fn health_check_primal(&self, primal_name: &str) -> Result<HealthStatus> {
        let client = IsomorphicClient::new(primal_name);
        
        // Will automatically discover Unix OR TCP endpoint
        let response = client.call_method("health.check", json!({})).await?;
        
        // Parse health status
        Ok(serde_json::from_value(response)?)
    }
}
```

**Lines to Change**: ~30 lines

### Testing Phase 3

1. **Deploy TOWER atomic on Linux**:
   ```bash
   cargo run -p biomeos-atomic-deploy -- deploy tower
   # Both beardog and songbird should use Unix sockets
   # Logs: "✅ Using optimal transport: UnixSocket"
   ```

2. **Deploy TOWER atomic on Android**:
   ```bash
   # Deploy to Pixel 8a
   ./scripts/deploy-to-pixel.sh tower
   
   # Expected logs:
   # [INFO] 🚀 Launching beardog (isomorphic mode)
   # [WARN] ⚠️  Optimal transport unavailable: Permission denied
   # [INFO] 🌐 Starting TCP IPC fallback (isomorphic mode)
   # [INFO] ✅ TCP IPC listening on 127.0.0.1:XXXXX
   # [INFO] 🔍 Discovering IPC endpoint for beardog
   # [INFO] ✅ Discovered TCP endpoint: tcp://127.0.0.1:XXXXX
   ```

═══════════════════════════════════════════════════════════════════

## 📊 FILE SUMMARY

### Files to Create/Rename

1. `biomeos-api/src/isomorphic_server.rs` (new/renamed)
2. `biomeos-federation/src/isomorphic_client.rs` (new/renamed)

### Files to Modify

1. `biomeos-core/src/ipc/transport.rs` (150 new lines)
2. `biomeos-core/src/ipc/mod.rs` (5 lines changed)
3. `biomeos-core/src/socket_discovery.rs` (40 new lines)
4. `biomeos-api/src/lib.rs` (update exports)
5. `biomeos-api/src/main.rs` (use isomorphic server)
6. `biomeos-atomic-deploy/src/neural_api_server.rs` (20 lines changed)
7. `biomeos-atomic-deploy/src/primal_launcher.rs` (100 lines changed)
8. `biomeos-atomic-deploy/src/primal_coordinator.rs` (30 lines changed)

**Total**: ~10 files, ~475 new lines, ~55 lines changed

═══════════════════════════════════════════════════════════════════

## ✅ SUCCESS CRITERIA

### When Evolution is Complete

1. ✅ **Linux Desktop**: Uses Unix sockets (optimal)
2. ✅ **Android/SELinux**: Automatically falls back to TCP
3. ✅ **Zero Configuration**: No environment variables needed
4. ✅ **Automatic Discovery**: Clients find Unix OR TCP endpoints
5. ✅ **Logs Show Adaptation**: 
   ```
   [WARN] ⚠️  Optimal transport unavailable
   [INFO] 🌐 Starting TCP IPC fallback
   [INFO] ✅ TCP IPC listening on 127.0.0.1:XXXXX
   ```
6. ✅ **Discovery Files Created**: `/tmp/{service}-ipc-port` or `$XDG_RUNTIME_DIR/{service}-ipc-port`
7. ✅ **Atomic Deployment Works**: TOWER, NODE, NEST deploy on Android
8. ✅ **Deep Debt A++**: Pure Rust, zero unsafe, runtime discovery

═══════════════════════════════════════════════════════════════════

## 🎓 DEEP DEBT PRINCIPLES VALIDATION

### Before Evolution (B+)
- ✅ Platform-agnostic abstractions exist
- ❌ But: Hardcoded platform detection (#[cfg])
- ❌ But: No automatic adaptation to constraints
- ❌ But: Crashes on SELinux instead of adapting

### After Evolution (A++)
- ✅ Try→Detect→Adapt→Succeed pattern
- ✅ Runtime platform constraint detection
- ✅ Automatic TCP fallback
- ✅ Zero configuration required
- ✅ Discovery files for client adaptation
- ✅ True platform isomorphism

═══════════════════════════════════════════════════════════════════

## 🚀 IMPLEMENTATION ORDER

### Session 1: Core IPC Evolution (3-4 hours)
1. Evolve `biomeos-core/src/ipc/transport.rs`
2. Add `bind_with_fallback()` method
3. Add platform constraint detection
4. Add TCP discovery file writing
5. Test on Linux and Android

### Session 2: Server & Client (2-3 hours)
6. Evolve `biomeos-api/src/unix_server.rs` → `isomorphic_server.rs`
7. Evolve `biomeos-federation/src/unix_socket_client.rs` → `isomorphic_client.rs`
8. Update `biomeos-atomic-deploy/src/neural_api_server.rs`
9. Test end-to-end on both platforms

### Session 3: Deployment Integration (2-3 hours)
10. Evolve `biomeos-atomic-deploy/src/primal_launcher.rs`
11. Evolve `biomeos-atomic-deploy/src/primal_coordinator.rs`
12. Test TOWER atomic deployment on Linux and Android
13. Validate logs show automatic adaptation

**Total**: 7-10 hours

═══════════════════════════════════════════════════════════════════

## 📚 REFERENCE

### Pattern Source
- **Reference**: songbird v3.33.0 (production validated)
- **Evidence**: Logs from Pixel 8a showing automatic TCP fallback
- **Pattern**: Try→Detect→Adapt→Succeed

### Key Files to Study
- songbird: `crates/songbird-orchestrator/src/ipc/pure_rust_server/server.rs`
- songbird: `crates/songbird-http-client/src/crypto/socket_discovery.rs`
- songbird: `crates/songbird-http-client/src/beardog_client/rpc.rs`

### Advantage: biomeOS Already Has Foundation
Unlike other primals, biomeOS already has:
- ✅ Transport abstraction layer
- ✅ Platform detection logic
- ✅ Socket discovery system
- ✅ XDG-compliant paths

**We just need to add the Try→Detect→Adapt pattern!**

═══════════════════════════════════════════════════════════════════

**Status**: Ready for Implementation  
**Complexity**: Medium (has foundation, needs pattern)  
**Priority**: HIGH (unblocks all atomic deployments on Android)  
**Expected Grade**: A++ (full isomorphism + Deep Debt principles)

🌍🧬🦀 **Let's evolve biomeOS!** 🦀🧬🌍

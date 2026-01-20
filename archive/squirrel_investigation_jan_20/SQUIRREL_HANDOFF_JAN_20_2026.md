# Squirrel Team Handoff - Server Not Starting

**Date**: January 20, 2026  
**Priority**: CRITICAL  
**Blocking**: Full Tower + Squirrel deployment and API validation

---

## Root Cause Identified

**File**: `/home/eastgate/Development/ecoPrimals/phase1/squirrel/crates/main/src/main.rs`  
**Lines**: 129-132

```rust
// Start the server (this will block)
// api_server.start().await?; // DELETED - HTTP server removed

Ok(())
```

**Problem**: The HTTP server was deleted but NO replacement server startup code was added. The `run_server()` function:
1. Prints startup messages
2. Initializes components
3. Prints "✅ Squirrel AI/MCP Primal Ready!"
4. **Returns immediately** instead of starting a server
5. `main()` exits, killing the entire process

---

## Required Fix

### Option 1: Use tarpc Server (TCP-based)
File exists: `crates/main/src/rpc/tarpc_server.rs`

```rust
// In run_server() function, replace lines 129-132:

use crate::rpc::tarpc_server::SquirrelRpcServer;

// Create and start tarpc server
let rpc_server = SquirrelRpcServer::new();
let addr = format!("{}:{}", bind, port).parse()?;
info!("🚀 Starting tarpc server on {}", addr);
rpc_server.start(addr).await?;
```

###Option 2: Implement Unix Socket Server (Recommended for biomeOS)
Based on BearDog/Songbird pattern:

```rust
// In run_server() function, replace lines 129-132:

use tokio::net::UnixListener;
use std::path::Path;

// Determine socket path
let socket_path = socket.unwrap_or_else(|| {
    format!("/tmp/squirrel-{}.sock", node_id)
});

// Remove old socket if exists
let socket_path_buf = Path::new(&socket_path);
if socket_path_buf.exists() {
    std::fs::remove_file(&socket_path)?;
}

// Create Unix socket listener
let listener = UnixListener::bind(&socket_path)?;
info!("🚀 JSON-RPC server listening on {}", socket_path);

// Accept connections loop
loop {
    match listener.accept().await {
        Ok((stream, _addr)) => {
            tokio::spawn(async move {
                // Handle JSON-RPC requests here
                // Use existing SquirrelService trait implementation
            });
        }
        Err(e) => {
            error!("Failed to accept connection: {}", e);
        }
    }
}
```

### Option 3: Quick Minimal Fix (For Testing)
Just make it block instead of exiting:

```rust
// In run_server() function, replace lines 129-132:

use tokio::signal;

println!("🔌 Server ready (minimal mode - no RPC yet)");
println!("   Press Ctrl+C to stop");

// Block until Ctrl+C
signal::ctrl_c().await?;
println!("\n👋 Shutting down gracefully...");

Ok(())
```

---

## Additional Issues

### 1. `--socket` CLI Flag Ignored

**File**: `crates/main/src/main.rs`  
**Line**: 77

```rust
async fn run_server(
    port: u16,
    _daemon: bool, // Reserved for future daemon mode
    _socket: Option<String>,  // ← PREFIXED WITH _ = UNUSED!
    _bind: String,            // ← ALSO UNUSED!
    verbose: bool,
) -> Result<()> {
```

**Fix**: Remove underscores and actually use these parameters:
```rust
async fn run_server(
    port: u16,
    daemon: bool,
    socket: Option<String>,  // ← Use this!
    bind: String,             // ← Use this!
    verbose: bool,
) -> Result<()> {
```

### 2. Hardcoded Socket Path

**Line**: 125

```rust
println!("   Socket: /tmp/squirrel-{}.sock", node_id);
```

**Issue**: This path is hardcoded and printed, but never actually used. Should use the `socket` parameter:

```rust
let socket_path = socket.unwrap_or_else(|| {
    format!("/tmp/squirrel-{}.sock", node_id)
});
println!("   Socket: {}", socket_path);
```

---

## Testing After Fix

### Build and Harvest
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
cargo build --release --target x86_64-unknown-linux-musl

# Copy to plasmidBin
cp target/x86_64-unknown-linux-musl/release/squirrel \
   /home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin/primals/squirrel/squirrel-x86_64-musl
```

### Manual Test
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Ensure Tower Atomic is running
ls -lh /tmp/beardog-nat0.sock /tmp/songbird-nat0.sock

# Start Squirrel
export SONGBIRD_ENDPOINT="/tmp/songbird-nat0.sock"
export ANTHROPIC_API_KEY="sk-ant-REDACTED"

./plasmidBin/primals/squirrel/squirrel-x86_64-musl server \
  --socket /tmp/squirrel-nat0.sock &

# Verify it stays running
sleep 2
ps aux | grep squirrel | grep -v grep
ls -lh /tmp/squirrel-nat0.sock

# Test health check
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | nc -U /tmp/squirrel-nat0.sock
```

### Full Deployment Test
```bash
# Use biomeOS deployment script
python3 /home/eastgate/Development/ecoPrimals/phase2/biomeOS/scripts/deploy.py nat0

# Should show all 3 primals running
ls -lh /tmp/*-nat0.sock
# Expected:
# beardog-nat0.sock
# songbird-nat0.sock
# squirrel-nat0.sock
```

---

## Impact

**Currently Blocking**:
- ❌ Full Tower + Squirrel deployment
- ❌ API calls to Anthropic via Squirrel
- ❌ biomeOS Neural API primal orchestration validation
- ❌ NUCLEUS deployment testing

**Working (Tower Atomic)**:
- ✅ BearDog (security foundation)
- ✅ Songbird (communications, bonded to BearDog)
- ✅ Can test Songbird HTTP capabilities directly

---

## Recommended Approach

1. **Immediate** (15 minutes): Implement Option 3 (minimal blocking fix) for initial testing
2. **Short-term** (2 hours): Implement Option 2 (Unix socket server) for biomeOS integration
3. **Complete** (1 day): Full JSON-RPC service implementation with all endpoints

---

## Files for Reference

**Squirrel Source**:
- `ecoPrimals/phase1/squirrel/crates/main/src/main.rs` - Entry point (needs fix)
- `ecoPrimals/phase1/squirrel/crates/main/src/rpc/tarpc_server.rs` - Existing RPC server (TCP)
- `ecoPrimals/phase1/squirrel/crates/main/src/cli.rs` - CLI definition

**BearDog/Songbird Examples** (for Unix socket patterns):
- `ecoPrimals/phase1/beardog/src/main.rs`
- `ecoPrimals/phase1/songbird/...` (exact path varies)

**BiomeOS Integration**:
- `biomeOS/scripts/deploy.py` - Deployment script (ready to use after fix)
- `biomeOS/MANUAL_DEPLOYMENT_GUIDE_JAN_20_2026.md` - Deployment guide
- `biomeOS/SQUIRREL_CRASH_INVESTIGATION_JAN_20_2026.md` - Investigation details

---

## Questions?

Contact biomeOS team or check:
- Investigation doc: `biomeOS/SQUIRREL_CRASH_INVESTIGATION_JAN_20_2026.md`
- Deployment guide: `biomeOS/MANUAL_DEPLOYMENT_GUIDE_JAN_20_2026.md`

---

**Status**: Ready for Squirrel team to implement fix  
**ETA**: 15 minutes (minimal), 2 hours (complete)  
**Validation**: biomeOS team will test immediately after fix


# Squirrel Crash Investigation - January 20, 2026

## Status
**ISSUE**: Squirrel starts successfully, prints "Ready!", but immediately exits/crashes

## Tower Atomic Status
✅ **BearDog**: RUNNING  
- Socket: `/tmp/beardog-nat0.sock`
- PID: 3541252
- Family ID: nat0

✅ **Songbird**: RUNNING  
- Socket: `/tmp/songbird-nat0.sock`  
- PID: 3539276
- Bonded to BearDog

❌ **Squirrel**: CRASHES ON STARTUP  
- Socket: Creates `/tmp/squirrel-squirrel.sock` but process exits
- Log shows: "✅ Squirrel AI/MCP Primal Ready!" then immediately exits

## Reproduction

### Command Used
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
export SONGBIRD_ENDPOINT="/tmp/songbird-nat0.sock"
export ANTHROPIC_API_KEY="sk-ant-REDACTED"
./plasmidBin/primals/squirrel/squirrel-x86_64-musl server
```

### Log Output
```
🐿️  Squirrel AI/MCP Primal Starting...
   Version: 0.1.0
   Mode: Server
✅ UniBin Architecture v1.0.0
✅ Zero-HTTP Production Mode (v1.1.0)
✅ Modern Async Concurrent Rust

✅ Ecosystem Manager initialized
✅ Metrics Collector initialized
✅ Shutdown Manager initialized
✅ Modern architecture: Unix sockets + JSON-RPC + tarpc
   (No HTTP server - TRUE PRIMAL!)
🔌 Starting JSON-RPC server...
   Socket: /tmp/squirrel-squirrel.sock

✅ Squirrel AI/MCP Primal Ready!
```

### Process Status
```bash
$ ps aux | grep squirrel | grep -v grep
# (no output - process not running)
```

### Socket Status
```bash
$ ls -lh /tmp/squirrel-squirrel.sock
srwxrwxr-x 1 eastgate eastgate 0 Jan 19 21:22 /tmp/squirrel-squirrel.sock

$ echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | nc -U /tmp/squirrel-squirrel.sock
nc: unix connect failed: Connection refused
```

## Issues Discovered

### 1. --socket Flag Ignored
Squirrel ignores the `--socket` CLI flag:
```bash
./plasmidBin/primals/squirrel/squirrel-x86_64-musl server --socket /tmp/squirrel-nat0.sock
# Still creates: /tmp/squirrel-squirrel.sock
```

**Expected**: Socket created at `/tmp/squirrel-nat0.sock`  
**Actual**: Socket created at `/tmp/squirrel-squirrel.sock`

### 2. Immediate Process Exit
- Process prints "Ready!" but doesn't stay running
- Socket file exists but nothing listens on it
- No error messages in logs
- Exit appears to be clean (no panic/crash messages)

## Hypothesis

### Likely Causes
1. **Tokio runtime issue**: Server task completes immediately instead of blocking
2. **Main function exits**: After spawning server, main() returns instead of `.await`ing
3. **Signal handler**: Immediately receives shutdown signal
4. **Async runtime misconfiguration**: Runtime drops before tasks complete

### Code to Review
Check `ecoPrimals/phase1/squirrel/src/main.rs`:
- `server` subcommand handler
- Tokio runtime setup
- Server task spawning
- Main function's blocking/await behavior

Likely issue:
```rust
// ❌ WRONG: Spawns server but doesn't wait
tokio::spawn(server.run());
// main() exits here, killing the runtime

// ✅ RIGHT: Awaits server completion
server.run().await?;
```

## Deployment Script Updates Needed

### Python Script (`scripts/deploy.py`)
Already updated to use `--socket` flag, but Squirrel ignores it:
```python
subprocess.Popen([
    "./plasmidBin/primals/squirrel/squirrel-x86_64-musl", "server",
    "--socket", SQUIRREL_SOCKET  # ← This is ignored!
], env=env, stdout=log, stderr=log)
```

## Investigation Steps for Squirrel Team

### 1. Check Main Function
```bash
cd /home/eastgate/Development/ecoPrimals/phase1/squirrel
grep -A 20 "fn main" src/main.rs
```

Look for:
- Is the server `.await`ed?
- Is there a `tokio::runtime` being dropped early?
- Is `main()` returning before server completes?

### 2. Check Server Subcommand
```bash
grep -A 30 "Commands::Server" src/main.rs
```

Look for:
- How is the socket path parsed from CLI?
- Is the server task spawned or awaited?
- Are there any early returns?

### 3. Enable Debug Logging
```bash
RUST_LOG=debug ./plasmidBin/primals/squirrel/squirrel-x86_64-musl server 2>&1 | tee /tmp/squirrel-debug.log
```

### 4. Run in Foreground with strace
```bash
strace -f -o /tmp/squirrel-strace.log ./plasmidBin/primals/squirrel/squirrel-x86_64-musl server
```

Check for:
- `exit_group()` syscall - why is it exiting?
- Socket operations - is it binding and then closing?
- Signal handling - any signals received?

## Workaround for Testing

### Option 1: Test BearDog + Songbird Only
Tower Atomic is working! We can test HTTP capabilities via Songbird directly.

### Option 2: Use Old Squirrel Version
If there's a known-working version in git history, harvest that to `plasmidBin`.

### Option 3: Patch and Rebuild
Fix the issue in `phase1/squirrel`, rebuild, reharvest to `plasmidBin`.

## Environment
- **Binary**: `plasmidBin/primals/squirrel/squirrel-x86_64-musl`
- **Build Date**: Jan 20 10:43 (from `ls -lh`)
- **Source**: `ecoPrimals/phase1/squirrel/`
- **Rust Version**: Stable x86_64-unknown-linux-gnu
- **Target**: x86_64-unknown-linux-musl

## Next Steps

1. **Squirrel Team**: Investigate and fix the immediate exit issue
2. **Squirrel Team**: Fix `--socket` CLI flag not being respected
3. **biomeOS Team**: Continue with Tower Atomic testing (BearDog + Songbird)
4. **biomeOS Team**: Test Songbird HTTP capabilities directly

## Related Files
- `MANUAL_DEPLOYMENT_GUIDE_JAN_20_2026.md` - Deployment instructions
- `scripts/deploy.py` - Python deployment script
- `TOWER_ATOMIC_READY_JAN_20_2026.md` - Tower Atomic quick start
- `/tmp/squirrel-nat0.log` - Latest crash log

---

**Created**: January 20, 2026  
**Status**: Ready for Squirrel team investigation  
**Priority**: HIGH - Blocking full stack validation


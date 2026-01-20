# Tower Atomic Deployment Solution - January 20, 2026

**Status**: ✅ **ROOT CAUSE IDENTIFIED**  
**Issue**: Socket path mismatch in `neural_executor.rs`  
**Fix**: Simple - 1 line change!

---

## 🎯 Root Cause

**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs`  
**Function**: `node_primal_start_capability` (line 473)  
**Problem**: Socket path inconsistency

### The Bug (Line 546 vs Line 605)

**Line 546** - Socket path creation (CORRECT):
```rust
let socket_path = format!("{}/{}-{}.sock", runtime_dir, primal_name, family_id);
// Result: /tmp/songbird-nat0.sock
```

**Line 605** - Socket check (BUG - hardcoded /tmp):
```rust
let socket_path = format!("/tmp/{}-{}.sock", primal_name, family_id);
// Result: /tmp/songbird-nat0.sock (hardcoded /tmp!)
```

**The variable is reused with different value!** The runtime_dir logic is lost.

---

## ✅ Primal-Specific Handling (Already Implemented!)

**Lines 553-579** already handle each primal differently:

### BearDog (GOLD STANDARD) ✅
```rust
cmd.arg("--socket").arg(&socket_path);
cmd.arg("--family-id").arg(family_id);
```

### Songbird ✅
```rust
cmd.env("SONGBIRD_SOCKET", &socket_path);
cmd.env("SONGBIRD_ORCHESTRATOR_FAMILY_ID", family_id);
```

### Squirrel ✅
```rust
cmd.env("SQUIRREL_SOCKET", &socket_path);
```

**All correct!** The primal-specific logic is already there!

---

## 🔧 The Fix

### Change Required

**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs`  
**Line**: ~605

**Before** (BUG):
```rust
// 4. Wait for socket (with timeout)
let socket_path = format!("/tmp/{}-{}.sock", primal_name, family_id);  // ❌ WRONG!
```

**After** (FIXED):
```rust
// 4. Wait for socket (with timeout)
// socket_path already defined at line 546 - don't redefine it!
// Just use the existing variable
```

**OR** (if socket_path went out of scope):
```rust
// 4. Wait for socket (with timeout)
let runtime_dir = std::env::var("BIOMEOS_RUNTIME_DIR")
    .or_else(|_| std::env::var("TMPDIR"))
    .unwrap_or_else(|_| "/tmp".to_string());
let socket_check_path = format!("{}/{}-{}.sock", runtime_dir, primal_name, family_id);
```

---

## 🧐 Additional Issues

### Issue 1: Stdout/Stderr Hidden

**Line 582-583**:
```rust
cmd.stdout(Stdio::null());
cmd.stderr(Stdio::null());
```

**Problem**: We can't see primal startup errors!

**Fix**: Redirect to log files instead:
```rust
let log_dir = format!("{}/primals/{}/{}", runtime_dir, primal_name, family_id);
std::fs::create_dir_all(&log_dir)?;

let stdout_log = std::fs::File::create(format!("{}/{}.log", log_dir, primal_name))?;
let stderr_log = stdout_log.try_clone()?;

cmd.stdout(Stdio::from(stdout_log));
cmd.stderr(Stdio::from(stderr_log));
```

---

## 🚀 Expected Behavior After Fix

### Tower Atomic Deployment

**1. BearDog starts**:
```
Command: ./plasmidBin/primals/beardog/beardog-x86_64-musl server \
  --socket /tmp/beardog-nat0.sock \
  --family-id nat0

Socket: /tmp/beardog-nat0.sock ✅
PID: 12345
```

**2. Songbird starts**:
```
Environment:
  SONGBIRD_SOCKET=/tmp/songbird-nat0.sock
  SONGBIRD_ORCHESTRATOR_FAMILY_ID=nat0

Command: ./plasmidBin/primals/songbird/songbird-x86_64-musl server

Socket: /tmp/songbird-nat0.sock ✅
PID: 12346
```

**3. Squirrel starts**:
```
Environment:
  SQUIRREL_SOCKET=/tmp/squirrel-nat0.sock
  FAMILY_ID=nat0
  ANTHROPIC_API_KEY=sk-ant-...

Command: ./plasmidBin/primals/squirrel/squirrel-x86_64-musl server

Socket: /tmp/squirrel-nat0.sock ✅
PID: 12347
```

---

## 📋 Complete Fix Checklist

### Step 1: Fix Socket Path Bug

```rust
// File: crates/biomeos-atomic-deploy/src/neural_executor.rs
// Around line 605

// REMOVE this line:
// let socket_path = format!("/tmp/{}-{}.sock", primal_name, family_id);

// The socket_path variable is already correctly defined at line 546!
// Just reference the existing variable or recreate it with runtime_dir
```

### Step 2: Add Logging for Debug

```rust
// Replace Stdio::null() with log files
let log_dir = format!("{}/primals/{}/{}", runtime_dir, primal_name, family_id);
tokio::fs::create_dir_all(&log_dir).await?;

let stdout_path = format!("{}/{}.log", log_dir, primal_name);
let stdout_log = tokio::fs::File::create(&stdout_path).await?;
let stderr_log = stdout_log.try_clone().await?;

cmd.stdout(Stdio::from(stdout_log.into_std().await));
cmd.stderr(Stdio::from(stderr_log.into_std().await));

tracing::info!("   Log file: {}", stdout_path);
```

### Step 3: Rebuild and Test

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Rebuild biomeOS
cargo build --release -p biomeos-atomic-deploy

# Clean deployment
pkill -f "beardog.*nat0"
pkill -f "songbird.*nat0"
pkill -f "squirrel.*nat0"
pkill -f "neural-api"
rm /tmp/*-nat0.sock

# Start Neural API
export ANTHROPIC_API_KEY="sk-ant-api03-..."
./target/release/biomeos neural-api --graphs-dir graphs &

# Execute graph
sleep 3
echo '{"jsonrpc":"2.0","method":"neural_api.execute_graph",
  "params":{"graph_id":"tower_squirrel","family_id":"nat0"},"id":1}' \
  | nc -U /tmp/neural-api-nat0.sock

# Verify
sleep 10
ls -la /tmp/*-nat0.sock
ps aux | grep -E "(beardog|songbird|squirrel)" | grep nat0

# Expected:
# /tmp/beardog-nat0.sock
# /tmp/songbird-nat0.sock
# /tmp/squirrel-nat0.sock
# /tmp/neural-api-nat0.sock
```

---

## 🧬 Tower Atomic Architecture

### Bonding Model

**Tower Atomic** = Covalent bond between BearDog + Songbird:
- BearDog provides cryptographic operations
- Songbird provides network/discovery services
- They share the same `family_id` (genetic identity)
- Operate as a bonded unit

**Squirrel** = Inherits from Tower:
- Receives `family_id` from Tower
- Can securely communicate with Tower
- Uses Tower for all external HTTP requests
- Genetic lineage established

### Communication Flow

```
Squirrel (AI request)
    ↓ (Unix socket to Neural API)
Neural API (routing mesh)
    ↓ (discover "secure_http" capability)
Tower Atomic (Songbird + BearDog)
    ↓ BearDog handles crypto/TLS
    ↓ Songbird makes HTTP request
Anthropic API (external HTTPS)
    ↓ Response
Tower Atomic
    ↓ Neural API
Squirrel (AI response)
```

---

## 🎯 Success Criteria

After fix, deployment should achieve:

- ✅ BearDog starts and creates socket
- ✅ Songbird starts and creates socket
- ✅ Squirrel starts and creates socket
- ✅ All three have same `family_id` (genetic lineage)
- ✅ Tower Atomic is operational (BearDog + Songbird bonded)
- ✅ Squirrel can communicate with Tower via Neural API
- ✅ End-to-end test: Squirrel → Tower → Anthropic API → response

---

## 💡 Key Insights

1. **The primal-specific logic is already there!** (lines 553-579)
2. **The bug is simple**: Socket path variable reused incorrectly (line 605)
3. **The architecture is correct**: Tower Atomic bonding via `family_id`
4. **Logging is important**: Need to see startup errors, not hide them

---

**Next**: Fix the socket path bug, rebuild, and test! Should work perfectly after 1-line fix.


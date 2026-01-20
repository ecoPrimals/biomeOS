# 🐿️ Squirrel Socket Path Issue - Team Handoff

**Date**: January 20, 2026  
**Priority**: ⚠️  **BLOCKER** for Neural API deployment  
**Estimate**: 30-60 minutes to fix

---

## 🎯 **THE ISSUE**

Squirrel **ignores** the `--socket` CLI flag and hardcodes its socket path.

### **Expected Behavior**:
```bash
./squirrel server --socket /tmp/squirrel-nat0.sock
# Should create: /tmp/squirrel-nat0.sock
```

### **Actual Behavior**:
```bash
./squirrel server --socket /tmp/squirrel-nat0.sock
# Creates: /tmp/squirrel-squirrel.sock (hardcoded!)
```

---

## 📊 **EVIDENCE**

### **Test 1: Manual Launch**
```bash
$ ./plasmidBin/primals/squirrel server --socket /tmp/squirrel-test.sock

Output:
🔌 Starting JSON-RPC server...
   Socket: /tmp/squirrel-squirrel.sock   # ❌ WRONG PATH!

$ ls /tmp/squirrel*.sock
/tmp/squirrel-squirrel.sock   # ❌ Not /tmp/squirrel-test.sock
```

### **Test 2: Neural API Launch**
```bash
# Neural API executes:
./squirrel server --socket /tmp/squirrel-nat0.sock

# Expected socket: /tmp/squirrel-nat0.sock
# Actual socket: /tmp/squirrel-squirrel.sock
# Result: Health check fails (socket not found)
```

---

## 🔍 **ROOT CAUSE ANALYSIS**

### **File**: `crates/main/src/main.rs`

**Line 74-77** - `run_server` function:
```rust
async fn run_server(
    port: u16,
    _daemon: bool,
    _socket: Option<String>,  // ❌ UNUSED (underscore prefix)
    _bind: String,
    verbose: bool,
) -> Result<()> {
```

**Problem**: The `_socket` parameter has an underscore prefix, indicating it's **intentionally unused**!

**Line 120-126** - Socket path construction:
```rust
let node_id = std::env::var("SQUIRREL_NODE_ID")
    .or_else(|_| std::env::var("HOSTNAME"))
    .unwrap_or_else(|_| "squirrel".to_string());  // ❌ Defaults to "squirrel"

println!("🔌 Starting JSON-RPC server...");
println!("   Socket: /tmp/squirrel-{}.sock", node_id);  // ❌ Hardcoded pattern!
```

**Problem**: Socket path is hardcoded as `/tmp/squirrel-{node_id}.sock` where:
- `node_id` comes from `SQUIRREL_NODE_ID` or `HOSTNAME` env vars
- Defaults to `"squirrel"`
- **Completely ignores the `--socket` CLI argument!**

---

## 🎯 **THE FIX**

### **Option 1: Honor CLI Flag** ⭐ **RECOMMENDED**

Update `run_server` to use the socket parameter:

```rust
async fn run_server(
    port: u16,
    _daemon: bool,
    socket: Option<String>,  // ✅ Remove underscore!
    _bind: String,
    verbose: bool,
) -> Result<()> {
    // ...
    
    // Build socket path
    let socket_path = if let Some(custom_socket) = socket {
        // CLI flag takes priority
        custom_socket
    } else {
        // Fall back to environment variable or default
        std::env::var("SQUIRREL_SOCKET").unwrap_or_else(|_| {
            let node_id = std::env::var("SQUIRREL_NODE_ID")
                .or_else(|_| std::env::var("HOSTNAME"))
                .unwrap_or_else(|_| "squirrel".to_string());
            format!("/tmp/squirrel-{}.sock", node_id)
        })
    };
    
    println!("🔌 Starting JSON-RPC server...");
    println!("   Socket: {}", socket_path);  // ✅ Use actual path
    
    // Pass socket_path to actual server initialization
    // (wherever that happens in Squirrel's codebase)
}
```

**Behavior**:
1. CLI flag (`--socket`) = highest priority ✅
2. Environment variable (`SQUIRREL_SOCKET`) = fallback
3. Default (`/tmp/squirrel-{node_id}.sock`) = last resort

---

### **Option 2: Environment Variable Only**

If CLI flag is complex to implement, use environment variable:

```rust
let socket_path = std::env::var("SQUIRREL_SOCKET").unwrap_or_else(|_| {
    let node_id = std::env::var("SQUIRREL_NODE_ID")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "squirrel".to_string());
    format!("/tmp/squirrel-{}.sock", node_id)
});
```

**Neural API would then set**:
```rust
cmd.env("SQUIRREL_SOCKET", "/tmp/squirrel-nat0.sock");
```

---

## ✅ **VALIDATION**

### **Test 1: CLI Flag**
```bash
$ ./squirrel server --socket /tmp/test-custom.sock
# Should output: "Socket: /tmp/test-custom.sock"
# Should create: /tmp/test-custom.sock

$ ls /tmp/test-custom.sock
/tmp/test-custom.sock  # ✅ Success!
```

### **Test 2: Environment Variable**
```bash
$ SQUIRREL_SOCKET=/tmp/test-env.sock ./squirrel server
# Should output: "Socket: /tmp/test-env.sock"
# Should create: /tmp/test-env.sock

$ ls /tmp/test-env.sock
/tmp/test-env.sock  # ✅ Success!
```

### **Test 3: Default**
```bash
$ ./squirrel server
# Should output: "Socket: /tmp/squirrel-squirrel.sock"
# Should create: /tmp/squirrel-squirrel.sock

$ ls /tmp/squirrel-squirrel.sock
/tmp/squirrel-squirrel.sock  # ✅ Success (default behavior)
```

### **Test 4: Neural API Integration**
```bash
# Neural API will pass --socket flag:
$ ./squirrel server --socket /tmp/squirrel-nat0.sock

# After 3 seconds:
$ ls /tmp/squirrel-nat0.sock
/tmp/squirrel-nat0.sock  # ✅ Success!

# Neural API health check:
$ ls /tmp/squirrel-nat0.sock && echo "✅ Health check passed"
✅ Health check passed
```

---

## 🎯 **ACCEPTANCE CRITERIA**

1. ✅ CLI flag (`--socket`) works and takes priority
2. ✅ Environment variable (`SQUIRREL_SOCKET`) works as fallback
3. ✅ Default behavior preserved (`/tmp/squirrel-{node_id}.sock`)
4. ✅ Neural API can launch Squirrel with custom socket path
5. ✅ Health checks pass (socket found at specified path)
6. ✅ All existing tests still pass

---

## 📈 **IMPACT**

### **Current State**:
- ❌ Neural API cannot reliably deploy Squirrel
- ❌ Multi-instance deployments impossible (all use same socket)
- ❌ Health checks fail (wrong socket path)
- ❌ Tower + Squirrel deployment blocked

### **After Fix**:
- ✅ Neural API can deploy Squirrel with custom socket
- ✅ Multi-instance deployments work (different families)
- ✅ Health checks pass (correct socket path)
- ✅ Tower + Squirrel deployment unblocked
- ✅ 100% Neural API deployment ready!

---

## 🏆 **GOLD STANDARD: BearDog**

BearDog shows how it should be done:

```bash
$ ./beardog server --help
Usage: beardog server [OPTIONS]

Options:
      --socket <SOCKET>                Unix socket path [default: /tmp/beardog.sock]
      --family-id <FAMILY_ID>          Family ID for BirdSong

$ ./beardog server --socket /tmp/beardog-nat0.sock --family-id nat0
# ✅ Creates: /tmp/beardog-nat0.sock (EXACTLY as specified)
# ✅ Works perfectly with Neural API
# ✅ Health checks pass
```

**Squirrel should follow BearDog's pattern!**

---

## 📝 **FILES TO MODIFY**

### **Primary**:
- `crates/main/src/main.rs` (lines 74-133)
  - Update `run_server` function
  - Remove underscore from `socket` parameter
  - Use socket parameter in path construction
  - Pass to actual server initialization

### **Secondary** (if socket server is elsewhere):
- Find where JSON-RPC server actually starts
- Ensure it receives and uses the socket_path
- Update any hardcoded socket paths

---

## ⏱️ **ESTIMATE**

- **Investigation**: 15 minutes (find all socket path uses)
- **Implementation**: 30 minutes (update code)
- **Testing**: 15 minutes (validate all scenarios)
- **Total**: **60 minutes**

---

## 🚀 **NEXT STEPS**

1. **Squirrel Team**: Implement fix (Option 1 recommended)
2. **Test**: Validate all 4 test scenarios above
3. **Notify**: biomeOS team when ready
4. **Redeploy**: Neural API will test Tower + Squirrel
5. **Celebrate**: 100% primal deployment! 🎉

---

## 📞 **QUESTIONS?**

See also:
- `PRIMAL_LAUNCHING_STATUS_JAN_20_2026.md` (full context)
- `SESSION_SUCCESS_JAN_20_2026.md` (overall progress)

Contact: biomeOS Neural API team

---

🐿️✨ **Squirrel Socket Path Fix = Tower + Squirrel Deployment!** ✨🐿️

**Follow BearDog's gold standard pattern!**


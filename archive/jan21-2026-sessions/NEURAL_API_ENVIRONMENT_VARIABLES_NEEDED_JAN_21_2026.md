# Neural API - Environment Variable Support for Graph Deployment

**Date**: January 21, 2026 02:18 UTC  
**Status**: ⚠️ **FEATURE GAP** - Environment variables in graph TOML not passed to primals  
**Priority**: 🔴 **CRITICAL** - Blocks end-to-end AI testing

---

## 🎯 PROBLEM

Graph TOML files can specify environment variables for primal deployment:

```toml
[[nodes]]
id = "start-squirrel"
primal = { by_capability = "ai" }

[nodes.operation]
name = "start"

[nodes.operation.params]
mode = "server"
family_id = "nat0"

[nodes.operation.environment]   # ← THESE ARE NOT PASSED TO PRIMAL!
ANTHROPIC_API_KEY = "sk-ant-..."
CAPABILITY_REGISTRY_SOCKET = "/tmp/neural-api-nat0.sock"
```

**Issue**: `neural_executor.rs::node_primal_start_capability()` does NOT read or pass `nodes.operation.environment` to the spawned process.

---

## 📍 CURRENT IMPLEMENTATION

**File**: `crates/biomeos-atomic-deploy/src/neural_executor.rs`  
**Function**: `node_primal_start_capability()` (lines 473-650)

**Current Behavior**:
```rust
async fn node_primal_start_capability(
    node: &GraphNode,
    _context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // ... (capability discovery, binary path) ...
    
    let mut cmd = Command::new(&binary_full_path);
    cmd.arg(mode);
    
    // Hard-coded environment variables only:
    match primal_name {
        "squirrel" => {
            cmd.arg("--socket").arg(&socket_path);
            cmd.env("SERVICE_MESH_ENDPOINT", ...);  // ← Only this!
        }
        // ...
    }
    
    // ❌ MISSING: Pass node.operation.environment to cmd!
    
    cmd.spawn()?;
    Ok(...)
}
```

---

## ✅ REQUIRED FIX

### **1. Add Environment Variable Passthrough**

```rust
async fn node_primal_start_capability(
    node: &GraphNode,
    _context: &ExecutionContext,
) -> Result<serde_json::Value> {
    // ... (existing setup) ...
    
    let mut cmd = Command::new(&binary_full_path);
    cmd.arg(mode);
    
    // Primal-specific setup
    match primal_name {
        "squirrel" => {
            cmd.arg("--socket").arg(&socket_path);
            cmd.env("SERVICE_MESH_ENDPOINT", ...);
        }
        // ...
    }
    
    // ✅ NEW: Pass environment variables from graph TOML
    if let Some(ref operation) = node.operation {
        if let Some(ref env_map) = operation.environment {
            for (key, value) in env_map {
                tracing::debug!("   Setting env: {}={}", key, value);
                cmd.env(key, value);
            }
        }
    }
    
    cmd.spawn()?;
    Ok(...)
}
```

### **2. Update GraphOperation Structure**

**File**: `crates/biomeos-graph/src/graph.rs` or wherever `GraphOperation` is defined

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphOperation {
    pub name: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
    
    // ✅ NEW FIELD
    #[serde(default)]
    pub environment: Option<HashMap<String, String>>,
}
```

---

## 🧪 TESTING

### **Before Fix**:
```bash
# Squirrel starts without ANTHROPIC_API_KEY
$ cat /proc/$(pgrep squirrel)/environ | tr '\0' '\n' | grep ANTHROPIC
# (nothing)

# AI query fails
$ echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello"},"id":1}' | \
  nc -U /tmp/squirrel-nat0.sock
{"error": "No providers available"}
```

### **After Fix**:
```bash
# Squirrel starts WITH ANTHROPIC_API_KEY
$ cat /proc/$(pgrep squirrel)/environ | tr '\0' '\n' | grep ANTHROPIC
ANTHROPIC_API_KEY=sk-ant-api03-...

# AI query succeeds
$ echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"Hello"},"id":1}' | \
  nc -U /tmp/squirrel-nat0.sock
{"result": {"text": "Hello! I'm Claude...", ...}}
```

---

## 📊 IMPACT

### **Blocks**:
1. ✅ End-to-end AI testing (Squirrel → Songbird → Anthropic)
2. ✅ Graph-based deployments with external API keys
3. ✅ TRUE PRIMAL deployment pattern (no manual scripts!)

### **Enables**:
1. ✅ Pure Neural API deployment (no shell scripts!)
2. ✅ Environment-specific configuration (dev/staging/prod)
3. ✅ Secure API key management (from secrets management system)
4. ✅ Complete automation of primal lifecycle

---

## 🎯 ESTIMATED EFFORT

- **Code Changes**: 10-20 lines
- **Testing**: 30 minutes
- **Total Time**: 1-2 hours

---

## 📚 RELATED FILES

1. `crates/biomeos-atomic-deploy/src/neural_executor.rs` (main fix)
2. `crates/biomeos-graph/src/graph.rs` (data structure)
3. `graphs/tower_squirrel.toml` (example usage)
4. `SQUIRREL_ANTHROPIC_INTEGRATION_JAN_20_2026.md` (architecture)
5. `SESSION_COMPLETE_JAN_21_2026_PURE_RUST_AI.md` (context)

---

## ✅ ACCEPTANCE CRITERIA

- [ ] `GraphOperation` has optional `environment` field
- [ ] `node_primal_start_capability()` reads `operation.environment`
- [ ] Environment variables passed to spawned process via `cmd.env()`
- [ ] Existing graphs still work (backwards compatible)
- [ ] New graphs with environment variables work
- [ ] End-to-end AI query succeeds (Squirrel → Songbird → Anthropic)

---

## 🎊 HANDOFF

**To**: Neural API / biomeOS Core Team  
**Priority**: CRITICAL (blocks Pure Rust AI stack validation)  
**Estimated Time**: 1-2 hours

**Context**:
- Pure Rust AI stack is 100% ready architecturally
- Songbird v4.3.0 provides HTTP delegation
- Squirrel has event-driven discovery
- Only blocker is environment variable passing in Neural API

**This is the FINAL piece for end-to-end Pure Rust AI!** 🚀

---

*The ecological way: Deploy via graphs, configure via environment, discover via capabilities* 🧬🌍⚡


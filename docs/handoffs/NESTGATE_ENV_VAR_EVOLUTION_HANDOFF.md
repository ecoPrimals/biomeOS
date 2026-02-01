# 🔧 NestGate Runtime Port Configuration Evolution
## February 1, 2026 - Complete Environment Variable Integration

**Status**: ⚠️ **PARTIAL IMPLEMENTATION**  
**Priority**: 🟡 **MEDIUM** (Workaround available via CLI args)  
**Estimated Work**: 30-60 minutes

═══════════════════════════════════════════════════════════════════

## 🎯 **PROBLEM STATEMENT**

### **Current Behavior**

**CLI Argument Defaults**: ✅ Working
```bash
# This works (port specified via CLI):
nestgate daemon --port 8085 --bind 127.0.0.1
```

**Environment Variable Config**: ❌ **NOT WORKING**
```bash
# This is IGNORED:
export NESTGATE_API_PORT=8085
export NESTGATE_BIND=127.0.0.1
nestgate daemon  # Still binds to 8080!
```

### **Root Cause**

**Code Analysis**:

1. **Backend Logic**: ✅ **COMPLETE**
   - `NetworkConfig::from_env_with_alternatives()` reads environment variables
   - Implements 4-tier fallback (`NESTGATE_API_PORT` → `NESTGATE_HTTP_PORT` → `NESTGATE_PORT` → default)

2. **CLI Integration**: ❌ **MISSING**
   - CLI uses hardcoded defaults from `DEFAULT_API_PORT` (8080)
   - Environment variables are never read
   - Port/bind passed directly from CLI args to `run_daemon()`

**File**: `code/crates/nestgate-bin/src/cli.rs`

```rust
// LINE 69-72: Current (hardcoded defaults)
Daemon {
    /// Port to bind to (ignored in socket-only mode)
    #[arg(short, long, default_value_t = nestgate_core::defaults::network::DEFAULT_API_PORT)]
    port: u16,  // ❌ Always 8080, ignores NESTGATE_API_PORT!
    
    /// Bind address (ignored in socket-only mode)  
    #[arg(long, default_value = nestgate_core::defaults::network::DEFAULT_BIND_ADDRESS)]
    bind: String,  // ❌ Always 0.0.0.0, ignores NESTGATE_BIND!
    // ...
}
```

**File**: `code/crates/nestgate-bin/src/cli.rs`

```rust
// LINE 297-303: CLI handler passes values directly
Commands::Daemon { port, bind, dev, socket_only } => {
    // port and bind come from CLI args (hardcoded defaults)
    // Environment variables never consulted!
    crate::commands::service::run_daemon(port, &bind, dev, socket_only).await
}
```

### **Impact**

**Current Workaround**: ✅ Use CLI arguments
```bash
nestgate daemon --port $NESTGATE_API_PORT --bind $NESTGATE_BIND
```

**Problem**: Not following ecosystem pattern
- Other primals (beardog, songbird, toadstool) read env vars directly
- Requires different launch scripts for nestgate
- Less clean for NUCLEUS atomic deployments

═══════════════════════════════════════════════════════════════════

## ✅ **SOLUTION**

### **Approach: Environment-Aware CLI Defaults**

**Goal**: Make CLI read environment variables *before* setting default values.

### **Option 1: Dynamic Default via `value_parser`** (Recommended)

Clap supports custom value parsers that can read environment variables:

**File**: `code/crates/nestgate-bin/src/cli.rs`

```rust
// Add helper functions at module level:

/// Read port from environment with fallback to default
fn port_from_env_or_default() -> u16 {
    // Try environment variables in order
    std::env::var("NESTGATE_API_PORT")
        .or_else(|_| std::env::var("NESTGATE_HTTP_PORT"))
        .or_else(|_| std::env::var("NESTGATE_PORT"))
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(nestgate_core::defaults::network::DEFAULT_API_PORT)
}

/// Read bind address from environment with fallback to default
fn bind_from_env_or_default() -> String {
    std::env::var("NESTGATE_BIND")
        .or_else(|_| std::env::var("NESTGATE_BIND_ADDRESS"))
        .ok()
        .unwrap_or_else(|| nestgate_core::defaults::network::DEFAULT_BIND_ADDRESS.to_string())
}

// Update Daemon struct (line 69):
Daemon {
    /// Port to bind to (ignored in socket-only mode)
    /// Reads from: NESTGATE_API_PORT, NESTGATE_HTTP_PORT, or NESTGATE_PORT
    #[arg(
        short, 
        long, 
        default_value_t = port_from_env_or_default(),
        env = "NESTGATE_API_PORT"  // Clap will show this in --help
    )]
    port: u16,
    
    /// Bind address (ignored in socket-only mode)
    /// Reads from: NESTGATE_BIND or NESTGATE_BIND_ADDRESS
    #[arg(
        long, 
        default_value_t = bind_from_env_or_default(),
        env = "NESTGATE_BIND"  // Clap will show this in --help
    )]
    bind: String,
    // ...
}
```

**Benefits**:
- ✅ Minimal code changes (add 2 helper functions, modify 2 arg attributes)
- ✅ Preserves existing CLI arg behavior
- ✅ Automatic `--help` documentation shows env vars
- ✅ CLI args still override environment variables (correct precedence)

### **Option 2: Use Clap's Built-in `env` Attribute** (Simpler but limited)

```rust
Daemon {
    #[arg(short, long, env = "NESTGATE_API_PORT", default_value_t = 8080)]
    port: u16,
    
    #[arg(long, env = "NESTGATE_BIND", default_value = "0.0.0.0")]
    bind: String,
}
```

**Limitations**:
- ❌ Only reads ONE environment variable (no fallback chain)
- ❌ Loses multi-tier fallback (`NESTGATE_HTTP_PORT`, `NESTGATE_PORT`)

**Recommendation**: Use Option 1 for ecosystem consistency.

═══════════════════════════════════════════════════════════════════

## 🔍 **VERIFICATION PLAN**

### **Test 1: Environment Variable Only**

```bash
# Set environment variables
export NESTGATE_API_PORT=8085
export NESTGATE_BIND=127.0.0.1

# Start without CLI args
./nestgate daemon

# Expected log:
# 🏰 Starting NestGate daemon (UniBin mode)
#    Port: 8085, Bind: 127.0.0.1, Dev: false  ✅
```

### **Test 2: CLI Argument Override**

```bash
# Environment says 8085
export NESTGATE_API_PORT=8085

# CLI overrides to 9000
./nestgate daemon --port 9000

# Expected:
#    Port: 9000, Bind: 0.0.0.0, Dev: false  ✅
# (CLI args take precedence)
```

### **Test 3: Fallback Chain**

```bash
# Primary not set, try alternatives
unset NESTGATE_API_PORT
export NESTGATE_HTTP_PORT=7070

./nestgate daemon

# Expected:
#    Port: 7070, Bind: 0.0.0.0, Dev: false  ✅
```

### **Test 4: Default Fallback**

```bash
# No env vars set
unset NESTGATE_API_PORT
unset NESTGATE_HTTP_PORT
unset NESTGATE_PORT

./nestgate daemon

# Expected:
#    Port: 8080, Bind: 0.0.0.0, Dev: false  ✅
# (Uses DEFAULT_API_PORT)
```

### **Test 5: Help Documentation**

```bash
./nestgate daemon --help

# Expected output should include:
#   -p, --port <PORT>
#       Port to bind to (ignored in socket-only mode)
#       [env: NESTGATE_API_PORT]
#       [default: 8080]
```

═══════════════════════════════════════════════════════════════════

## 📊 **IMPLEMENTATION CHECKLIST**

### **Phase 1: Core Changes** (20 minutes)

- [ ] Add `port_from_env_or_default()` helper function
- [ ] Add `bind_from_env_or_default()` helper function
- [ ] Update `Daemon` struct with `default_value_t` calling helpers
- [ ] Add `env` attribute to show in `--help`
- [ ] Build and test locally

### **Phase 2: Testing** (20 minutes)

- [ ] Test 1: Env var only (no CLI args)
- [ ] Test 2: CLI override
- [ ] Test 3: Fallback chain (`NESTGATE_HTTP_PORT`)
- [ ] Test 4: Default fallback (no env)
- [ ] Test 5: Help documentation shows env vars

### **Phase 3: Documentation** (10 minutes)

- [ ] Update README with environment variable precedence
- [ ] Update CLI help text (automatic via `env` attribute)
- [ ] Add example launch scripts

### **Phase 4: Deployment** (10 minutes)

- [ ] Build ARM64 binary with fix
- [ ] Test on Pixel without CLI args
- [ ] Verify discovery works
- [ ] Update genome

**Total Estimated Time**: 60 minutes

═══════════════════════════════════════════════════════════════════

## 🎯 **PRECEDENCE ORDER** (After Fix)

```
1. CLI Arguments (highest priority)
   ./nestgate daemon --port 9000
   
2. NESTGATE_API_PORT environment variable
   export NESTGATE_API_PORT=8085
   
3. NESTGATE_HTTP_PORT environment variable (fallback)
   export NESTGATE_HTTP_PORT=7070
   
4. NESTGATE_PORT environment variable (fallback)
   export NESTGATE_PORT=6060
   
5. DEFAULT_API_PORT constant (8080) (lowest priority)
```

**This matches the pattern used by NetworkConfig::from_env_with_alternatives()!**

═══════════════════════════════════════════════════════════════════

## 📚 **RELATED FILES**

### **Files to Modify**

1. **`code/crates/nestgate-bin/src/cli.rs`** (lines 69-83)
   - Add helper functions
   - Update `Daemon` struct attributes

### **Reference Implementation**

2. **`code/crates/nestgate-core/src/config/environment/network.rs`** (lines 88-110)
   - `env_port_with_alternatives()` - Shows correct fallback pattern
   - `env_host_with_alternatives()` - Shows bind address pattern

### **Testing Files**

3. **`code/crates/nestgate-bin/src/main.rs`** (lines 48-54)
   - Legacy `nestgate-server` symlink handler (uses defaults directly)
   - May need similar fix for backward compatibility

═══════════════════════════════════════════════════════════════════

## 🔄 **CURRENT WORKAROUND** (Production Ready)

### **For NUCLEUS Deployments**

**USB liveSpore**:
```bash
#!/bin/bash
# launch_nest.sh
export NESTGATE_API_PORT=8085
./nestgate daemon --port $NESTGATE_API_PORT --bind 127.0.0.1
```

**Pixel 8a**:
```bash
# Via adb
adb shell "NESTGATE_API_PORT=8085 ./nestgate daemon --port 8085 --bind 127.0.0.1"
```

**Status**: ✅ **WORKS** (but not as clean as ecosystem pattern)

═══════════════════════════════════════════════════════════════════

## 🏆 **SUCCESS CRITERIA**

### **After Evolution**

```bash
# Simple deployment (like other primals):
export NESTGATE_API_PORT=8085
export NESTGATE_BIND=127.0.0.1
./nestgate daemon  # ✅ Binds to 8085!
```

**No CLI args required!** Matches beardog/songbird/toadstool pattern.

### **Benefits**

1. ✅ Consistent with ecosystem primals
2. ✅ Cleaner launch scripts
3. ✅ Better NUCLEUS atomic integration
4. ✅ CLI args still work (override)
5. ✅ Help docs show environment variables

═══════════════════════════════════════════════════════════════════

## 📈 **PRIORITY ASSESSMENT**

### **Priority**: 🟡 **MEDIUM**

**Why Not High**:
- ✅ Workaround exists (CLI args work fine)
- ✅ nestgate already deployed and operational
- ✅ Not blocking NUCLEUS atomic progress

**Why Not Low**:
- 🟡 Inconsistent with other primals
- 🟡 More complex launch scripts needed
- 🟡 User expectations (env vars should work)

**Recommendation**: 
- ✅ Complete after squirrel integration (NEST 100%)
- ✅ Include in next nestgate evolution phase
- ✅ Document workaround for now

═══════════════════════════════════════════════════════════════════

## 🎊 **CURRENT STATUS**

### **What Works** ✅

- ✅ Backend `NetworkConfig::from_env_with_alternatives()` (complete!)
- ✅ Runtime port configuration via CLI args
- ✅ Port conflict resolution (8085 vs songbird's 8080)
- ✅ JWT security validation
- ✅ HTTP API operational on Pixel

### **What Needs Evolution** ⏳

- ⏳ CLI environment variable integration (30-60min)
- ⏳ Help documentation (automatic via `env` attribute)
- ⏳ Precedence testing (CLI > env > default)

### **Impact**

**Current**: nestgate 100% operational with workaround  
**After Fix**: nestgate 100% operational with ecosystem consistency  
**Grade**: Current A, After Fix A++

═══════════════════════════════════════════════════════════════════

**Created**: February 1, 2026  
**Priority**: 🟡 MEDIUM (30-60 minutes)  
**Status**: ⚠️ Partial (backend complete, CLI integration needed)  
**Workaround**: ✅ Available (use CLI args)  

🔧 **NESTGATE: 95% COMPLETE - CLI ENV VAR INTEGRATION REMAINING!** 🔧

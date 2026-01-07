# 🐛 Songbird v3.18.0 Runtime Bug - Handoff

**Date**: January 7, 2026  
**From**: biomeOS Integration Team  
**To**: Songbird Development Team  
**Priority**: CRITICAL (Blocking Production)  

---

## 🔴 Critical Bug Found

**Songbird v3.18.0 crashes on startup** with runtime panic!

## 🐛 Error Details

**Panic Message**:
```
thread 'main' panicked at crates/songbird-orchestrator/src/app/connection_manager.rs:108:39:
Cannot start a runtime from within a runtime. This happens because a function (like `block_on`) 
attempted to block the current thread while the thread is being used to drive asynchronous tasks.
```

**Location**: `crates/songbird-orchestrator/src/app/connection_manager.rs:108:39`

**Symptoms**:
- Songbird starts successfully
- Initializes components
- Crashes during connection manager initialization
- Process becomes zombie immediately
- All deployments fail

## 🔍 Root Cause

**Likely Issue**: BTSP client initialization in `connection_manager.rs` is calling `block_on()` or similar blocking call from within an async context.

**File**: `crates/songbird-orchestrator/src/app/connection_manager.rs`
**Line**: 108:39

**Code Pattern** (probable):
```rust
// ❌ WRONG - Can't call block_on from within async runtime
pub fn new() -> Self {
    let btsp_client = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(async {
            // Async initialization
        });  // ← Line 108:39 - PANIC!
}
```

**Should Be**:
```rust
// ✅ CORRECT - Use async initialization
pub fn new() -> Self {
    // Constructor is sync, delay initialization
    Self {
        btsp_client: None,  // Initialize later in async context
        // ...
    }
}

pub async fn initialize(&mut self) {
    // ✅ Now we're in async context, can await
    self.btsp_client = Some(Self::init_btsp_client().await?);
}
```

## 📊 Test Environment

**Binary**: `songbird-orchestrator`  
**SHA256**: `1cae1931254c0e8884c502ea3fa12753b60016a3174a2d4f71da8bd22ca99baf`  
**Built**: 2026-01-07 14:01:13

**Deployment**: USB spore deployment (tower.toml)  
**Result**: All instances crash on startup

**Full Log**:
```
2026-01-07T19:07:07.355990Z  INFO songbird_orchestrator: 🚀 Starting Songbird Orchestrator...
2026-01-07T19:07:07.356003Z  INFO songbird_orchestrator:    Instance Lock: Enforced (PID file active)
2026-01-07T19:07:07.356029Z  INFO songbird_orchestrator::app::initialization: 🔧 Initializing orchestrator components...
2026-01-07T19:07:07.356040Z  INFO songbird_orchestrator::app::initialization:       Timeouts: Anonymous=3600s, Capability=86400s, Identity=604800s, Hardware=never

thread 'main' panicked at crates/songbird-orchestrator/src/app/connection_manager.rs:108:39:
Cannot start a runtime from within a runtime.

2026-01-07T19:07:07.356105Z  INFO songbird_orchestrator::process_manager: ✅ Instance lock released cleanly
```

## 🎯 Recommended Fix

### Option 1: Lazy Initialization (Preferred)

**Change `ConnectionManager::new()` to not initialize BTSP client**:

```rust
impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            peer_metadata: Arc::new(RwLock::new(HashMap::new())),
            rejected_peers: Arc::new(RwLock::new(HashMap::new())),
            btsp_client: None,  // ← Don't initialize yet
        }
    }
    
    // Call this from async context (in main or initialization)
    pub async fn initialize_btsp(&mut self) -> Result<()> {
        self.btsp_client = Self::discover_btsp_client().await.ok();
        Ok(())
    }
}
```

**Update `main.rs` or initialization code**:
```rust
async fn initialize_app() -> Result<()> {
    let mut connection_manager = ConnectionManager::new();
    
    // ✅ Now we're in async context
    connection_manager.initialize_btsp().await?;
    
    // ... rest of initialization
}
```

### Option 2: Remove Blocking Calls

**If BTSP initialization must happen in constructor**, make the constructor async:

```rust
impl ConnectionManager {
    pub async fn new() -> Result<Self> {  // ← async constructor
        let btsp_client = Self::discover_btsp_client().await.ok();
        
        Ok(Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            peer_metadata: Arc::new(RwLock::new(HashMap::new())),
            rejected_peers: Arc::new(RwLock::new(HashMap::new())),
            btsp_client,
        })
    }
}
```

## ✅ Verification Steps

1. Fix the runtime initialization issue
2. Rebuild Songbird
3. Test: `cargo test --lib -p songbird-orchestrator`
4. Deploy to USB spore
5. Verify no runtime panic
6. Verify BTSP or HTTPS connection established

## 🔄 Rollback Plan

**For Production**: Revert to v3.17.0 until fix is available

```bash
# Use stable v3.17.0
cp songbird-v3.17.0-backup /usr/local/bin/songbird-orchestrator

# SHA256: e4a10567ad79c30842aaf005c38e00f6914d34a88c6d21f1ee8ba30cee656750
```

**v3.17.0 Status**: ✅ Stable, working, zombie detection functional

## 📋 Impact

**Severity**: CRITICAL  
**Impact**: All v3.18.0 deployments fail  
**Workaround**: Revert to v3.17.0  
**Timeline**: Blocking BTSP testing until fixed

## 🎯 Status

**Current**: biomeOS reverted to v3.17.0 for testing  
**Next**: Songbird team fixes runtime panic  
**ETA**: Unknown (high priority fix needed)

---

## 📚 Additional Context

**What Was Working** (v3.17.0):
- ✅ Zombie detection
- ✅ Graceful shutdown (SIGTERM)
- ✅ Genetic trust federation
- ✅ USB spore deployment

**What Broke** (v3.18.0):
- ❌ Startup crashes with runtime panic
- ❌ All deployments fail
- ❌ Cannot test BTSP functionality

**Root Issue**: Async runtime management in connection manager initialization

---

**Handed Off To**: Songbird Development Team  
**Status**: CRITICAL BUG - BLOCKING  
**Priority**: Fix before v3.18.1 release  
**Confidence**: Issue is fixable (async initialization pattern needed)


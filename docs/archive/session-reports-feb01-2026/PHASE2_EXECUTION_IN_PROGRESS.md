# 🔥 Phase 2 START - Unsafe Code Evolution + Hardcode Elimination

**Date**: February 1, 2026 23:32  
**Status**: 🔴 **EXECUTION IN PROGRESS**  
**Focus**: Critical files + high-impact patterns

═══════════════════════════════════════════════════════════════════

## 📊 **AUDIT RESULTS**

### **1. Hardcoded Primal References** 🔴

**Found**: 70+ hardcoded "beardog" references in production code!

**Critical Files**:
1. `neural_api_server.rs` - 6 beardog hardcodes
2. `primal_spawner.rs` - 8 beardog hardcodes  
3. `capability_handlers.rs` - 8 beardog hardcodes
4. `neural_executor.rs` - 5 beardog hardcodes
5. `neural_router.rs` - 2 beardog hardcodes
6. `mode.rs` - 3 beardog hardcodes
7. 10+ more files with hardcodes

**Pattern Analysis**:
```rust
// PATTERN 1: Hardcoded socket paths (BAD!)
let beardog_socket = "/tmp/beardog-nat0.sock";

// PATTERN 2: Hardcoded primal names (BAD!)
cmd.env("SONGBIRD_SECURITY_PROVIDER", "beardog");
let beardog = self.find_primal_by_socket("beardog").await?;

// PATTERN 3: Hardcoded capability assumptions (BAD!)
let beardog_socket = context.get_socket_path("beardog").await;
```

### **2. Unsafe Code** 🟡

**Status**: `genome-extract` and `genomebin-v3` appear to be **already safe!**

From header comment in `genome-extract/src/main.rs`:
```rust
// Deep Debt Principles:
// - 100% Pure Rust (zero unsafe, zero C deps except libc)
```

**Next**: Verify other 26 files

---

## 🎯 **EVOLUTION STRATEGY**

### **Priority 1: Hardcode → Capability** (STARTING NOW)

**Target**: `neural_api_server.rs` bootstrap sequence

**Current** (Hardcoded):
```rust
// Line 441
let beardog_socket = nucleation.assign_socket("beardog", &self.family_id);

// Line 471
match Self::verify_primal_health(&beardog_socket, "beardog").await {
```

**Evolution** (Capability-based):
```rust
// Discover security provider by capability
let security_provider = discovery
    .find_capability("crypto.sign")
    .await?
    .or_else(|| nucleation.assign_unoccupied_socket("security", &self.family_id));

match Self::verify_capability_provider(&security_provider, "crypto.sign").await {
```

### **Priority 2: Primal Spawner Evolution**

**Target**: `primal_spawner.rs`

**Current** (8 hardcodes):
```rust
let beardog_socket = context.get_socket_path("beardog").await;
cmd.env("SONGBIRD_SECURITY_PROVIDER", "beardog");
```

**Evolution**:
```rust
let security_socket = context.get_capability_socket("crypto.sign").await;
cmd.env("SECURITY_PROVIDER_SOCKET", &security_socket);
cmd.env("SECURITY_CAPABILITIES", "crypto.sign,crypto.verify");
```

### **Priority 3: Neural Executor Evolution**

**Target**: `neural_executor.rs`

**Current** (find_beardog_socket function):
```rust
async fn find_beardog_socket(context: &ExecutionContext) -> Option<String> {
    // Hardcoded search for beardog
}
```

**Evolution**:
```rust
async fn find_security_provider(context: &ExecutionContext) -> Option<CapabilityProvider> {
    context.discovery.find_capability("crypto.sign").await
}
```

---

## 📝 **EXECUTION PLAN**

### **Session 1** (NOW - 2 hours):

1. ✅ Create `CapabilityDiscovery` trait
2. ✅ Evolve `neural_api_server.rs` bootstrap
3. ✅ Evolve `primal_spawner.rs` 
4. ✅ Test bootstrap with capability-based

### **Session 2** (Next - 2 hours):

1. Evolve `neural_executor.rs`
2. Evolve `capability_handlers.rs`
3. Evolve `mode.rs`
4. Integration tests

### **Session 3** (Complete - 2 hours):

1. Evolve remaining 7 files
2. Remove all hardcoded primal names
3. Full ecosystem test
4. Documentation

---

## 🏗️ **CAPABILITY DISCOVERY TRAIT**

**New**: `crates/biomeos-core/src/capability_discovery.rs`

```rust
//! Capability Discovery - Runtime primal discovery by capability
//!
//! Deep Debt: Zero hardcoded primal names

use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CapabilityProvider {
    pub capabilities: Vec<String>,
    pub socket: PathBuf,
    pub primal_name: Option<String>,  // Discovered, not hardcoded
}

#[async_trait::async_trait]
pub trait CapabilityDiscovery {
    /// Find provider for capability (e.g., "crypto.sign")
    async fn find_capability(&self, capability: &str) -> Result<CapabilityProvider>;
    
    /// Find all providers for capability
    async fn find_all_capabilities(&self, capability: &str) -> Result<Vec<CapabilityProvider>>;
    
    /// Check if capability is available
    async fn has_capability(&self, capability: &str) -> bool {
        self.find_capability(capability).await.is_ok()
    }
}
```

---

## 🎯 **SUCCESS METRICS**

### **This Session**:
- [ ] `CapabilityDiscovery` trait created
- [ ] 3 critical files evolved (neural_api_server, primal_spawner, neural_executor)
- [ ] Bootstrap works with capability-based discovery
- [ ] 70+ hardcodes → <20 hardcodes

### **Complete Evolution**:
- [ ] Zero hardcoded primal names in production
- [ ] All discovery capability-based
- [ ] Tests pass
- [ ] Documentation updated

---

## 📊 **PROGRESS TRACKING**

**Files to Evolve**: 17 files with hardcodes

| File | Hardcodes | Status |
|------|-----------|--------|
| neural_api_server.rs | 6 | ⏳ IN PROGRESS |
| primal_spawner.rs | 8 | ⏳ QUEUED |
| capability_handlers.rs | 8 | ⏳ QUEUED |
| neural_executor.rs | 5 | ⏳ QUEUED |
| mode.rs | 3 | ⏳ QUEUED |
| neural_router.rs | 2 | ⏳ QUEUED |
| ... | 38+ | ⏳ QUEUED |

**Total Hardcodes**: 70+  
**Target**: 0 (except tests)

═══════════════════════════════════════════════════════════════════

**Status**: 🔴 **PHASE 2 IN PROGRESS**  
**Time Started**: 23:32  
**ETA**: 2 hours for Session 1

🧬🦀✨ **HARDCODE ELIMINATION. CAPABILITY DISCOVERY. PRIMAL AUTONOMY.** ✨🦀🧬

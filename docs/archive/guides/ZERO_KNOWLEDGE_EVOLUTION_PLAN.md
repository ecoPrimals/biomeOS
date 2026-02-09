# Zero-Knowledge Evolution Plan

**Date**: December 24, 2025  
**Philosophy**: **Infant Discovery Pattern**  
**Status**: 🔄 **IN PROGRESS**

---

## 🎯 Core Principle

> "Each primal wakes up like an infant - knowing only itself, discovering everything through the universal adapter."

**NO** primal has hardcoded knowledge of other primals.  
**NO** 2^n connections.  
**ONLY** n → 1 → n through universal adapter (Songbird).

---

## 📋 Hardcoding Found & Removal Strategy

### ✅ GOOD NEWS: Already Clean

1. **No hardcoded primal name strings** in operational code ✅
2. **No hardcoded vendor strings** in core logic ✅  
3. **No hardcoded ports** in production code (only in docs/tests) ✅
4. **Endpoints removed** from constants (Dec 23) ✅

### ❌ PROBLEMATIC: Needs Evolution

1. **`PrimalType` convenience constructors** (lines 76-94 in `primal/core.rs`)
   - `PrimalType::toadstool()`
   - `PrimalType::songbird()`
   - etc.

2. **Primal type constants** (lines 304-322 in `constants.rs`)
   - `SQUIRREL_TYPE`
   - `TOADSTOOL_TYPE`
   - etc.

3. **Hardcoded plugin path** (line 264 in `constants.rs`)
   - `.squirrel/plugins` → should be `.{primal_name}/plugins`

---

## 🏗️ Evolution Strategy

### Phase 1: Remove Convenience Constructors (Now)

**Problem**: `PrimalType::toadstool()` encourages hardcoding

**Solution**: Replace with capability-based construction

```rust
// ❌ REMOVE these convenience constructors
impl PrimalType {
    pub fn toadstool() -> Self { ... }
    pub fn songbird() -> Self { ... }
    pub fn nestgate() -> Self { ... }
    pub fn beardog() -> Self { ... }
    pub fn squirrel() -> Self { ... }
}

// ✅ KEEP only generic constructors
impl PrimalType {
    pub fn new(category: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self
    pub fn with_metadata(...) -> Self
    pub fn community(...) -> Self
}
```

**Migration Path**:
```rust
// Before
let primal_type = PrimalType::toadstool();

// After (from discovery)
let compute_service = adapter.query_capability("compute").await?;
let primal_type = compute_service.primal_type;

// Or (for self-identification only)
let my_type = PrimalType::new("compute", env::var("PRIMAL_NAME")?, "1.0.0");
```

### Phase 2: Remove Primal Type Constants (Now)

**Problem**: `TOADSTOOL_TYPE` constant encourages hardcoding

**Solution**: Use capability strings instead

```rust
// ❌ REMOVE from constants.rs
pub mod primals {
    pub const SQUIRREL_TYPE: &str = "squirrel";
    pub const TOADSTOOL_TYPE: &str = "toadstool";
    pub const SONGBIRD_TYPE: &str = "songbird";
    pub const NESTGATE_TYPE: &str = "nestgate";
    pub const BEARDOG_TYPE: &str = "beardog";
    pub const BIOMEOS_TYPE: &str = "biomeos";
}

// ✅ REPLACE with capability constants
pub mod capabilities {
    pub const COMPUTE: &str = "compute";
    pub const STORAGE: &str = "storage";
    pub const SECURITY: &str = "security";
    pub const AI: &str = "ai";
    pub const DISCOVERY: &str = "discovery";
    pub const ORCHESTRATION: &str = "orchestration";
}
```

**Migration Path**:
```rust
// Before
use biomeos_types::constants::primals::TOADSTOOL_TYPE;
if primal_name == TOADSTOOL_TYPE { ... }

// After
use biomeos_types::constants::capabilities::COMPUTE;
let service = adapter.query_capability(COMPUTE).await?;
```

### Phase 3: Dynamic Plugin Paths (Now)

**Problem**: `.squirrel/plugins` hardcodes primal name

**Solution**: Dynamic path based on primal identity

```rust
// ❌ REMOVE
pub const DEFAULT_PLUGIN_DIR: &str = ".squirrel/plugins";

// ✅ REPLACE with function
pub fn default_plugin_dir(primal_name: &str) -> String {
    format!(".{}/plugins", primal_name)
}

// Or even better - use capability
pub fn default_plugin_dir_for_capability(capability: &str) -> String {
    format!(".{}/plugins", capability)
}
```

### Phase 4: Universal Adapter Bootstrapping (Next)

**Goal**: Zero-knowledge startup for all primals

**Implementation**:

```rust
/// Universal discovery bootstrap
pub struct DiscoveryBootstrap {
    /// Multiple discovery methods (fallback chain)
    methods: Vec<DiscoveryMethod>,
}

pub enum DiscoveryMethod {
    /// Explicit endpoint from environment
    Environment { var_name: String },
    
    /// mDNS/Bonjour discovery
    MDns { service_type: String },
    
    /// UDP broadcast discovery
    Broadcast { port: u16 },
    
    /// Well-known multicast address
    Multicast { address: String },
    
    /// DNS-SD service discovery
    DnsSd { domain: String },
}

impl DiscoveryBootstrap {
    async fn find_universal_adapter(&self) -> Result<UniversalAdapter> {
        for method in &self.methods {
            if let Ok(adapter) = self.try_method(method).await {
                return Ok(adapter);
            }
        }
        Err(anyhow!("No universal adapter found"))
    }
}
```

### Phase 5: Vendor Agnostic Adapters (Future)

**Goal**: Runtime detection, no hardcoding

```rust
/// Runtime adapter trait
#[async_trait]
pub trait RuntimeAdapter: Send + Sync {
    /// Detect if this runtime is available
    async fn detect() -> Result<bool>;
    
    /// Deploy a workload
    async fn deploy(&self, spec: &WorkloadSpec) -> Result<DeploymentHandle>;
    
    /// Scale a deployment
    async fn scale(&self, handle: &DeploymentHandle, replicas: u32) -> Result<()>;
}

/// Auto-detect runtime
pub async fn detect_runtime() -> Result<Box<dyn RuntimeAdapter>> {
    if KubernetesAdapter::detect().await? {
        return Ok(Box::new(KubernetesAdapter::new().await?));
    }
    
    if DockerAdapter::detect().await? {
        return Ok(Box::new(DockerAdapter::new().await?));
    }
    
    if SystemdAdapter::detect().await? {
        return Ok(Box::new(SystemdAdapter::new().await?));
    }
    
    // Fallback to process-based
    Ok(Box::new(ProcessAdapter::new()))
}
```

---

## 🔧 Specific Changes Required

### File 1: `crates/biomeos-types/src/primal/core.rs`

**Lines 76-94**: Remove convenience constructors

```diff
-    /// Convenience constructors for known primal types
-    pub fn toadstool() -> Self {
-        Self::new("compute", "toadstool", "1.0.0")
-    }
-
-    pub fn songbird() -> Self {
-        Self::new("orchestration", "songbird", "1.0.0")
-    }
-
-    pub fn nestgate() -> Self {
-        Self::new("storage", "nestgate", "1.0.0")
-    }
-
-    pub fn beardog() -> Self {
-        Self::new("security", "beardog", "1.0.0")
-    }
-
-    pub fn squirrel() -> Self {
-        Self::new("ai", "squirrel", "1.0.0")
-    }
```

**Add**: Capability-based helpers

```rust
/// Create a PrimalType from discovered service info
pub fn from_discovered(
    category: impl Into<String>,
    name: impl Into<String>,
    version: impl Into<String>,
) -> Self {
    Self::new(category, name, version)
}

/// Create a PrimalType for self-identification
pub fn identify_self(
    category: impl Into<String>,
    version: impl Into<String>,
) -> Self {
    let name = std::env::var("PRIMAL_NAME")
        .unwrap_or_else(|_| "unknown".to_string());
    Self::new(category, name, version)
}
```

### File 2: `crates/biomeos-types/src/constants.rs`

**Lines 304-322**: Replace primal constants with capabilities

```diff
-/// Primal type constants
-pub mod primals {
-    /// Squirrel primal type
-    pub const SQUIRREL_TYPE: &str = "squirrel";
-
-    /// Toadstool primal type
-    pub const TOADSTOOL_TYPE: &str = "toadstool";
-
-    /// Songbird primal type
-    pub const SONGBIRD_TYPE: &str = "songbird";
-
-    /// Nestgate primal type
-    pub const NESTGATE_TYPE: &str = "nestgate";
-
-    /// Beardog primal type
-    pub const BEARDOG_TYPE: &str = "beardog";
-
-    /// BiomeOS primal type
-    pub const BIOMEOS_TYPE: &str = "biomeos";
-}

+/// Capability constants for discovery
+/// These are used to query for services, not identify specific primals
+pub mod capabilities {
+    /// Compute and execution capability
+    pub const COMPUTE: &str = "compute";
+    
+    /// Storage and persistence capability
+    pub const STORAGE: &str = "storage";
+    
+    /// Security and cryptography capability
+    pub const SECURITY: &str = "security";
+    
+    /// AI and intelligence capability
+    pub const AI: &str = "ai";
+    
+    /// Discovery and service mesh capability
+    pub const DISCOVERY: &str = "discovery";
+    
+    /// Orchestration capability
+    pub const ORCHESTRATION: &str = "orchestration";
+    
+    /// UI and visualization capability
+    pub const VISUALIZATION: &str = "visualization";
+}
```

**Line 264**: Dynamic plugin directory

```diff
-    /// Default plugins directory
-    pub const DEFAULT_PLUGIN_DIR: &str = ".squirrel/plugins";

+    /// Get default plugins directory for a primal
+    pub fn default_plugin_dir(primal_name: &str) -> String {
+        format!(".{}/plugins", primal_name)
+    }
+    
+    /// Get default plugins directory for current primal (from env)
+    pub fn current_primal_plugin_dir() -> String {
+        let primal_name = std::env::var("PRIMAL_NAME")
+            .unwrap_or_else(|_| "unknown".to_string());
+        default_plugin_dir(&primal_name)
+    }
```

### File 3: Create `crates/biomeos-core/src/discovery_bootstrap.rs`

**New file** for zero-knowledge startup:

```rust
//! Discovery Bootstrap - Zero Knowledge Startup
//!
//! Handles primal startup with no hardcoded dependencies.
//! Each primal discovers the ecosystem through multiple fallback methods.

use anyhow::{Context, Result};
use std::env;

/// Bootstrap discovery for a primal with zero knowledge
pub struct DiscoveryBootstrap {
    /// Service name for mDNS discovery
    service_name: String,
}

impl DiscoveryBootstrap {
    /// Create new bootstrap with service name
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }
    
    /// Find universal adapter using fallback methods
    pub async fn find_universal_adapter(&self) -> Result<String> {
        // Method 1: Explicit environment variable
        if let Ok(endpoint) = env::var("DISCOVERY_ENDPOINT") {
            return Ok(endpoint);
        }
        
        if let Ok(endpoint) = env::var("SONGBIRD_ENDPOINT") {
            return Ok(endpoint);
        }
        
        // Method 2: mDNS discovery
        if let Ok(endpoint) = self.discover_via_mdns().await {
            return Ok(endpoint);
        }
        
        // Method 3: Broadcast discovery
        if let Ok(endpoint) = self.discover_via_broadcast().await {
            return Ok(endpoint);
        }
        
        // Method 4: Well-known multicast
        if let Ok(endpoint) = self.discover_via_multicast().await {
            return Ok(endpoint);
        }
        
        Err(anyhow::anyhow!(
            "No universal adapter found. Please set DISCOVERY_ENDPOINT environment variable."
        ))
    }
    
    async fn discover_via_mdns(&self) -> Result<String> {
        // TODO: Implement mDNS discovery
        Err(anyhow::anyhow!("mDNS discovery not yet implemented"))
    }
    
    async fn discover_via_broadcast(&self) -> Result<String> {
        // TODO: Implement broadcast discovery
        Err(anyhow::anyhow!("Broadcast discovery not yet implemented"))
    }
    
    async fn discover_via_multicast(&self) -> Result<String> {
        // TODO: Implement multicast discovery
        Err(anyhow::anyhow!("Multicast discovery not yet implemented"))
    }
}
```

---

## 📊 Impact Analysis

### Before (Hardcoded)

```rust
// Primal knows about other primals
let toadstool_type = PrimalType::toadstool();
let songbird_endpoint = "http://localhost:3000";
let plugins_dir = ".squirrel/plugins";

// Results in 2^n connections
ToadStool -> Songbird
ToadStool -> Squirrel
Songbird -> Squirrel
Squirrel -> NestGate
... (n² connections)
```

### After (Discovery)

```rust
// Primal only knows itself
let my_type = PrimalType::identify_self("compute", "1.0.0");

// Discover adapter
let adapter = DiscoveryBootstrap::new("universal-adapter")
    .find_universal_adapter()
    .await?;

// Query by capability (no primal names!)
let ai_service = adapter.query_capability("ai").await?;
let storage = adapter.query_capability("storage").await?;

// Results in n→1→n through adapter
All Primals -> Universal Adapter -> All Primals
(2n connections)
```

---

## ✅ Success Criteria

### Must Have
- [ ] Remove all PrimalType convenience constructors
- [ ] Remove all primal name constants
- [ ] Add capability constants
- [ ] Dynamic plugin directory
- [ ] Discovery bootstrap implemented

### Should Have
- [ ] Multiple discovery methods (env, mDNS, broadcast)
- [ ] Clear error messages when discovery fails
- [ ] Documentation for zero-knowledge startup
- [ ] Examples of capability-based discovery

### Nice to Have
- [ ] mDNS discovery fully working
- [ ] Broadcast discovery fully working
- [ ] Auto-detection of universal adapter
- [ ] Caching of discovered services

---

## 🎯 Next Actions

### This Session (Now)
1. [ ] Remove PrimalType convenience constructors
2. [ ] Replace primal constants with capability constants  
3. [ ] Make plugin directory dynamic
4. [ ] Update tests to use capabilities
5. [ ] Verify build passes

### Next Session
1. [ ] Implement DiscoveryBootstrap
2. [ ] Add environment variable support
3. [ ] Create capability-based examples
4. [ ] Update documentation

### This Week
1. [ ] Implement mDNS discovery
2. [ ] Test with real primals
3. [ ] Update all examples
4. [ ] Comprehensive documentation

---

**Status**: 🎯 Ready to Execute  
**Risk**: LOW (mostly removing code, adding flexibility)  
**Impact**: HIGH (true zero-knowledge startup)  
**Estimated Time**: 1-2 hours

---

*"Know thyself. Discover the world. Connect to all."*


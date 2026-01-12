# 🎯 Deep Debt Evolution Plan - January 10, 2026

**Goal**: Systematically evolve biomeOS to production-ready, modern idiomatic Rust  
**Principles**: Fast AND Safe, Agnostic, Capability-Based, Self-Knowledge Only

---

## 📊 **Debt Inventory**

### **Critical Issues** (Production Blockers)

| Category | Count | Severity | Est. Time |
|----------|-------|----------|-----------|
| **Hardcoded Values** | 183 | 🔴 HIGH | 12-16h |
| **Hardcoded Primal Names** | 120 | 🔴 HIGH | 8-12h |
| **Production Mocks** | 114 | 🟡 MEDIUM | 6-8h |
| **Unsafe Code** | 9 | 🟡 MEDIUM | 4-6h |
| **Large Files (>500 lines)** | 20 | 🟢 LOW | 16-24h |

**Total Estimated Time**: 46-66 hours

---

## 🔴 **Priority 1: Hardcoded Primal Names** (8-12 hours)

### **Current State**
120 instances of hardcoded primal names (`"beardog"`, `"songbird"`, etc.)

### **Problem**
Violates primal philosophy: **"Primal code only has self knowledge and discovers other primals in runtime"**

**Examples of violation**:
```rust
// BAD: Hardcoded primal name
if primal_name == "beardog" {
    connect_to_security_primal();
}

// GOOD: Capability-based
if primal.has_capability("security") {
    connect_to_security_primal();
}
```

### **Solution: Capability-Based Discovery**

#### **Phase 1: Define Capability Taxonomy** (2 hours)
Create `crates/biomeos-types/src/capabilities.rs`:

```rust
/// Well-known capabilities that primals can provide
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PrimalCapability {
    // Security & Cryptography
    Encryption,
    Identity,
    Trust,
    KeyManagement,
    
    // Communication & Networking
    Discovery,
    P2PFederation,
    Tunneling,
    Routing,
    
    // Compute & Execution
    WorkloadManagement,
    ResourceScheduling,
    ProcessIsolation,
    
    // Storage & Data
    DataStorage,
    Provenance,
    Compression,
    Replication,
    
    // User Interface
    Rendering,
    InputHandling,
    MultiModal,
    
    // Orchestration
    LifecycleManagement,
    HealthMonitoring,
    ConfigManagement,
    
    // Custom capability (use sparingly!)
    Custom(String),
}
```

#### **Phase 2: Evolution Pattern** (6-10 hours)

**Pattern 1: Direct Name → Capability**
```rust
// BEFORE
let beardog = registry.get("beardog")?;
let result = beardog.encrypt(data)?;

// AFTER
let security = registry.find_by_capability(PrimalCapability::Encryption)?;
let result = security.call("encrypt", json!({"data": data}))?;
```

**Pattern 2: Conditional Logic → Capability Query**
```rust
// BEFORE
if primal_type == "songbird" {
    discover_peers();
}

// AFTER
if primal.has_capability(PrimalCapability::Discovery) {
    primal.call("discover_peers", json!({}))?;
}
```

**Pattern 3: Hardcoded Socket Paths → Discovery**
```rust
// BEFORE
let socket_path = "/tmp/beardog-main.sock";
connect_to_unix_socket(socket_path)?;

// AFTER
let security = nucleus.discover(DiscoveryRequest {
    capability: "security",
    trust_level: Some(TrustLevel::High),
    ..Default::default()
})?;
connect_to_primal(security.endpoint)?;
```

### **Files to Evolve** (Top 20 by primal name references)
1. `crates/biomeos-core/src/clients/beardog.rs` (BearDog client - needs capability wrapper)
2. `crates/biomeos-spore/src/spore.rs` (Hardcoded primal deployment)
3. `crates/biomeos-core/src/primal_orchestrator.rs` (Orchestration logic)
4. `crates/biomeos-manifest/src/niche.rs` (Niche definitions)
5. ... (full list to be generated)

---

## 🔴 **Priority 2: Hardcoded Paths/Addresses** (12-16 hours)

### **Current State**
183 instances of hardcoded paths/addresses:
- `/tmp/` paths (Unix sockets, temp files)
- `localhost`, `127.0.0.1`, `0.0.0.0` (network addresses)
- `/var/` paths (system directories)

### **Problem**
- Not portable across systems
- Breaks in containers/sandboxes
- Violates XDG Base Directory spec
- Assumes specific network configuration

### **Solution: Configuration-Based + Discovery**

#### **Phase 1: Path Configuration** (4-6 hours)

Create `crates/biomeos-types/src/paths.rs`:

```rust
use std::path::PathBuf;
use dirs;

/// System paths following XDG Base Directory specification
#[derive(Debug, Clone)]
pub struct SystemPaths {
    /// Runtime directory (Unix sockets, PID files)
    /// Default: $XDG_RUNTIME_DIR or /tmp/biomeos-$USER/
    pub runtime_dir: PathBuf,
    
    /// Data directory (persistent state)
    /// Default: $XDG_DATA_HOME/biomeos or ~/.local/share/biomeos
    pub data_dir: PathBuf,
    
    /// Config directory
    /// Default: $XDG_CONFIG_HOME/biomeos or ~/.config/biomeos
    pub config_dir: PathBuf,
    
    /// Cache directory
    /// Default: $XDG_CACHE_HOME/biomeos or ~/.cache/biomeos
    pub cache_dir: PathBuf,
}

impl SystemPaths {
    pub fn new() -> Result<Self> {
        let runtime_dir = if let Some(xdg_runtime) = std::env::var_os("XDG_RUNTIME_DIR") {
            PathBuf::from(xdg_runtime).join("biomeos")
        } else {
            std::env::temp_dir().join(format!("biomeos-{}", whoami::username()))
        };
        
        let data_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("~/.local/share"))
            .join("biomeos");
            
        // ... similar for config_dir and cache_dir
        
        // Create directories if they don't exist
        std::fs::create_dir_all(&runtime_dir)?;
        std::fs::create_dir_all(&data_dir)?;
        
        Ok(Self {
            runtime_dir,
            data_dir,
            config_dir,
            cache_dir,
        })
    }
    
    /// Get Unix socket path for a primal
    pub fn primal_socket(&self, primal_id: &str) -> PathBuf {
        self.runtime_dir.join(format!("{}.sock", primal_id))
    }
}
```

#### **Phase 2: Network Configuration** (4-6 hours)

```rust
/// Network configuration (no hardcoded addresses!)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Bind address (default: auto-detect)
    #[serde(default = "default_bind_address")]
    pub bind_address: Option<String>,
    
    /// Discovery method
    #[serde(default)]
    pub discovery: DiscoveryMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// mDNS/DNS-SD (for local networks)
    MDNS,
    /// Consul (for distributed)
    Consul { address: String },
    /// Manual configuration
    Manual { peers: Vec<String> },
    /// Auto (try MDNS, fall back to manual)
    Auto,
}

fn default_bind_address() -> Option<String> {
    // Auto-detect best interface
    // Don't hardcode 0.0.0.0 or 127.0.0.1!
    None
}
```

#### **Phase 3: Evolution Pattern** (4-6 hours)

```rust
// BEFORE
let socket_path = "/tmp/beardog-main.sock";

// AFTER
let paths = SystemPaths::new()?;
let socket_path = paths.primal_socket("beardog-main");
```

---

## 🟡 **Priority 3: Production Mocks** (6-8 hours)

### **Current State**
114 instances of mock-related code outside `#[cfg(test)]`

### **Problem**
**"Mocks should be isolated to testing, and any in production should be evolved to complete implementations"**

### **Analysis Results**

#### **✅ GOOD: Test-Only Mocks**
- `crates/biomeos-test-utils/src/mock_primal.rs` - Test utilities ✅
- `crates/biomeos-graph/src/executor.rs` (in `#[cfg(test)]` block) - Test-only ✅

#### **🟡 NEEDS EVOLUTION: Production Mocks**

**File**: `crates/biomeos-core/src/primal_orchestrator.rs`
```rust
// Lines 494-562: MockPrimal in production code!
struct MockPrimal {
    name: String,
}

impl ManagedPrimal for MockPrimal {
    // Stub implementation for when real primals aren't available
}
```

**Problem**: This is a graceful degradation for development, but it's NOT a mock - it's **standalone mode**.

**Solution**: Rename and clarify purpose

```rust
// BEFORE (misleading name)
struct MockPrimal { ... }

// AFTER (clear purpose)
struct StandalonePrimal {
    name: String,
    capabilities: Vec<PrimalCapability>,
}

impl StandalonePrimal {
    /// Creates a standalone primal that operates without external dependencies.
    /// Used for:
    /// - Development without full primal stack
    /// - Minimal deployments
    /// - Graceful degradation
    /// 
    /// NOT a mock! This is a real implementation with limited functionality.
    pub fn new(name: String, capabilities: Vec<PrimalCapability>) -> Self {
        Self { name, capabilities }
    }
}
```

### **Evolution Plan**

1. **Rename `MockPrimal` → `StandalonePrimal`** (2 hours)
2. **Add capability discovery to standalone** (2 hours)
3. **Document when/why standalone is used** (1 hour)
4. **Ensure all mocks are in `#[cfg(test)]`** (1-2 hours)

---

## 🟡 **Priority 4: Unsafe Code** (4-6 hours)

### **Current State**
9 instances of `unsafe` (mostly in comments, needs verification)

### **Analysis**

Looking at the grep results, most are **comments** about having "no unsafe code":
```rust
//! 3. **Fast AND Safe**: Zero unsafe code, async/await throughout
```

**Action**: Verify no actual `unsafe` blocks exist

```bash
# Find ACTUAL unsafe blocks (not comments)
grep -rn "unsafe {" --include="*.rs" crates/ | grep -v test
```

### **If Unsafe Blocks Found**

**Evolution Pattern**:
1. **Understand why unsafe was used**
2. **Find safe alternative** (often exists in modern Rust!)
3. **Benchmark if performance-critical**
4. **Document trade-offs if unsafe is truly necessary**

**Common patterns**:
- `unsafe { ... }` for FFI → Use `bindgen` or safe wrapper
- `unsafe { ... }` for transmute → Use `bytemuck` or manual conversion
- `unsafe { ... }` for raw pointers → Use `&` references or `Pin`

---

## 🟢 **Priority 5: Large File Refactoring** (16-24 hours)

### **Principle**: **Smart Refactoring, Not Mechanical Splitting**

"Large files should be refactored smart rather than just split"

### **Top 5 Targets**

#### **1. `crates/biomeos-cli/src/tui/widgets.rs` (904 lines)**

**Analysis**:
- TUI widget implementations
- Multiple widget types in one file

**Smart Refactoring**:
```
crates/biomeos-cli/src/tui/widgets/
├── mod.rs (re-exports)
├── node_view.rs (node-specific widgets)
├── graph_view.rs (graph visualization)
├── metrics_view.rs (metrics display)
├── command_view.rs (command input)
└── common.rs (shared widget utilities)
```

**Time**: 4-5 hours

#### **2. `crates/biomeos-core/src/clients/beardog.rs` (895 lines)**

**Analysis**:
- BearDog client implementation
- Multiple API sections (identity, security, federation)

**Smart Refactoring**:
```
crates/biomeos-core/src/clients/beardog/
├── mod.rs (main client struct)
├── identity.rs (identity APIs)
├── security.rs (encryption/decryption APIs)
├── federation.rs (federation APIs)
├── trust.rs (trust evaluation APIs)
└── error.rs (BearDog-specific errors)
```

**Additional Evolution**: Replace hardcoded "beardog" references with capability-based discovery!

**Time**: 4-5 hours

#### **3. `crates/biomeos-spore/src/spore.rs` (807 lines)**

**Analysis**:
- Spore creation, deployment, incubation
- Multiple concerns in one file

**Smart Refactoring**:
```
crates/biomeos-spore/src/
├── creation.rs (spore creation logic)
├── deployment.rs (deployment to USB/disk)
├── incubation.rs (local incubation)
├── genetic.rs (genetic lineage)
└── validation.rs (spore verification)
```

**Time**: 4-5 hours

#### **4. `crates/biomeos-types/src/manifest/networking_services.rs` (772 lines)**

**Analysis**:
- Network service definitions
- Multiple service types

**Smart Refactoring**:
- Group by service domain
- Separate DNS, DHCP, Firewall, etc.

**Time**: 3-4 hours

#### **5. `crates/biomeos-types/src/manifest/storage.rs` (770 lines)**

**Analysis**:
- Storage configuration types
- Multiple storage backends

**Smart Refactoring**:
- Separate by backend (local, distributed, cloud)
- Common traits in mod.rs

**Time**: 3-4 hours

---

## 📋 **Execution Order** (Recommended)

### **Phase 1: Foundation** (10-14 hours)
1. ✅ Define `PrimalCapability` taxonomy
2. ✅ Create `SystemPaths` for XDG compliance
3. ✅ Verify no actual unsafe blocks exist
4. ✅ Rename `MockPrimal` → `StandalonePrimal`

### **Phase 2: Core Evolution** (16-20 hours)
5. Evolve hardcoded primal names → capability-based
6. Evolve hardcoded paths → configuration-based
7. Smart refactor: `beardog.rs` (combines multiple goals!)
8. Smart refactor: `spore.rs`

### **Phase 3: Polish** (12-16 hours)
9. Smart refactor: TUI widgets
10. Smart refactor: Types manifests
11. Comprehensive testing
12. Documentation updates

**Total Time**: 38-50 hours (spread across multiple sessions)

---

## 🎯 **Success Criteria**

### **Phase 1 Complete When**:
- ✅ `PrimalCapability` enum defined and documented
- ✅ `SystemPaths` implemented and tested
- ✅ Zero actual `unsafe` blocks (or all documented/justified)
- ✅ No "Mock" in production (only "Standalone")

### **Phase 2 Complete When**:
- ✅ <50 hardcoded primal names (down from 120)
- ✅ <50 hardcoded paths (down from 183)
- ✅ BearDog client is capability-based
- ✅ Spore deployment is path-agnostic

### **Phase 3 Complete When**:
- ✅ All files <500 lines
- ✅ Clear domain separation
- ✅ Comprehensive tests passing
- ✅ Documentation complete

---

## 🚀 **First Session Goals** (4-6 hours)

Let's start with **Phase 1: Foundation**:

1. **Define `PrimalCapability`** (1-2 hours)
2. **Create `SystemPaths`** (1-2 hours)
3. **Verify unsafe code** (30 min)
4. **Rename MockPrimal** (1 hour)
5. **Write tests** (1 hour)

**End Result**: Foundation complete, ready for Phase 2!

---

**Created**: January 10, 2026, 21:45  
**Next Review**: After Phase 1 completion  
**Status**: Ready to execute! 🚀


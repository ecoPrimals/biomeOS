# 🚀 **Integration Roadmap: Adaptive API Adapters → BiomeOS**

**Date**: December 26, 2025  
**Status**: Adapters Complete - Ready for Integration  
**Priority**: High - Core Infrastructure

---

## 🎯 **Current State**

### **✅ What's Complete**

1. **API Discovery** - All 5 Phase 1 primals tested
2. **CLI Adapter System** - Base class + 2 primal adapters
3. **REST Adapter System** - Discovery + 3 primal adapters
4. **Documentation** - Usage guides + discovery reports
5. **Compilation** - All code builds successfully

### **📊 Architecture Map**

| Primal | Type | Adapter | Status |
|--------|------|---------|--------|
| Songbird | CLI | `SongbirdAdapter` | ✅ Built |
| NestGate | REST | `NestGateAdapter` | ✅ Built |
| BearDog | CLI | `BearDogAdapter` | ✅ Built |
| ToadStool | REST | `ToadStoolAdapter` | ✅ Built |
| Squirrel | REST | `SquirrelAdapter` | ✅ Built |

---

## 🛣️ **Integration Roadmap**

### **Phase 1: Core Integration** (Next 2-3 days)

#### **1. Process Manager for CLI Adapters** 🔴 Critical

**Why**: CLI adapters currently block on long-running commands (e.g., `songbird tower start`)

**What to Build**:

```rust
// crates/biomeos-core/src/process_manager/mod.rs

use std::process::{Child, Command};
use std::collections::HashMap;

pub struct ProcessManager {
    processes: HashMap<String, Child>,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: HashMap::new(),
        }
    }
    
    pub fn spawn_primal(
        &mut self,
        name: &str,
        binary_path: &str,
        args: &[&str]
    ) -> Result<()> {
        let child = Command::new(binary_path)
            .args(args)
            .spawn()?;
        
        self.processes.insert(name.to_string(), child);
        Ok(())
    }
    
    pub fn stop_primal(&mut self, name: &str) -> Result<()> {
        if let Some(mut child) = self.processes.remove(name) {
            child.kill()?;
            child.wait()?;
        }
        Ok(())
    }
    
    pub fn is_running(&self, name: &str) -> bool {
        self.processes.contains_key(name)
    }
}
```

**Usage**:

```rust
// Instead of:
// songbird.start_tower(8080, true).await?; // Blocks!

// Do this:
let mut pm = ProcessManager::new();
pm.spawn_primal(
    "songbird-tower",
    "/path/to/songbird",
    &["tower", "start", "--port", "8080"]
)?;
```

**Files to Create**:
- `crates/biomeos-core/src/process_manager/mod.rs`
- `crates/biomeos-core/src/process_manager/lifecycle.rs`
- Tests

**Estimated Time**: 4-6 hours

---

#### **2. Adapter Registry** 🟡 Important

**Why**: Centralize adapter management and discovery

**What to Build**:

```rust
// crates/biomeos-core/src/api_adapter/registry.rs

use std::collections::HashMap;
use std::sync::Arc;

pub struct AdapterRegistry {
    cli_adapters: HashMap<String, Arc<dyn CliPrimal>>,
    rest_adapters: HashMap<String, Arc<dyn RestPrimal>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        AdapterRegistry {
            cli_adapters: HashMap::new(),
            rest_adapters: HashMap::new(),
        }
    }
    
    pub async fn register_primal_auto(
        &mut self,
        name: &str,
        binary_path_or_url: &str
    ) -> Result<()> {
        // Try CLI first
        if let Ok(adapter) = Self::try_cli(binary_path_or_url) {
            self.cli_adapters.insert(name.to_string(), Arc::new(adapter));
            return Ok(());
        }
        
        // Try REST discovery
        if let Ok(adapter) = Self::try_rest(binary_path_or_url).await {
            self.rest_adapters.insert(name.to_string(), Arc::new(adapter));
            return Ok(());
        }
        
        Err(anyhow!("Could not register primal"))
    }
}
```

**Files to Create**:
- `crates/biomeos-core/src/api_adapter/registry.rs`
- Add trait `CliPrimal` and `RestPrimal`
- Tests

**Estimated Time**: 3-4 hours

---

#### **3. BiomeOS Orchestration Layer** 🔴 Critical

**Why**: Main integration point for multi-primal workflows

**What to Build**:

```rust
// crates/biomeos-core/src/orchestration/mod.rs

use crate::api_adapter::registry::AdapterRegistry;
use crate::process_manager::ProcessManager;

pub struct BiomeOrchestrator {
    adapters: AdapterRegistry,
    processes: ProcessManager,
}

impl BiomeOrchestrator {
    pub fn new() -> Self {
        BiomeOrchestrator {
            adapters: AdapterRegistry::new(),
            processes: ProcessManager::new(),
        }
    }
    
    pub async fn start_ecosystem(&mut self) -> Result<()> {
        // 1. Start Songbird tower
        self.start_songbird().await?;
        
        // 2. Start NestGate
        self.start_nestgate().await?;
        
        // 3. Register services with Songbird
        self.register_all_services().await?;
        
        Ok(())
    }
    
    async fn start_songbird(&mut self) -> Result<()> {
        self.processes.spawn_primal(
            "songbird",
            "/path/to/songbird",
            &["tower", "start", "--port", "8080", "--federation"]
        )?;
        
        // Wait for ready
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        // Register adapter
        self.adapters.register_primal_auto(
            "songbird",
            "/path/to/songbird"
        ).await?;
        
        Ok(())
    }
}
```

**Files to Create**:
- `crates/biomeos-core/src/orchestration/mod.rs`
- `crates/biomeos-core/src/orchestration/startup.rs`
- `crates/biomeos-core/src/orchestration/shutdown.rs`
- Tests

**Estimated Time**: 6-8 hours

---

### **Phase 2: Enhanced Features** (Week 2)

#### **4. Health Monitoring**

```rust
pub struct HealthMonitor {
    check_interval: Duration,
    adapters: Arc<AdapterRegistry>,
}

impl HealthMonitor {
    pub async fn monitor_loop(&self) {
        loop {
            self.check_all_primals().await;
            tokio::time::sleep(self.check_interval).await;
        }
    }
    
    async fn check_all_primals(&self) {
        // Check each primal's health
        // Restart if needed
        // Alert on failures
    }
}
```

**Estimated Time**: 4-6 hours

---

#### **5. Configuration Management**

```rust
// Load from biome.yaml
pub struct BiomeConfig {
    pub primals: HashMap<String, PrimalConfig>,
}

pub struct PrimalConfig {
    pub name: String,
    pub type: PrimalType, // CLI or REST
    pub path_or_url: String,
    pub auto_start: bool,
    pub port: Option<u16>,
}
```

**Estimated Time**: 3-4 hours

---

#### **6. Error Recovery**

```rust
pub struct ErrorRecovery {
    max_retries: u32,
    backoff: Duration,
}

impl ErrorRecovery {
    pub async fn with_retry<F, T>(&self, f: F) -> Result<T>
    where
        F: Fn() -> Future<Output = Result<T>>,
    {
        // Retry logic with exponential backoff
    }
}
```

**Estimated Time**: 3-4 hours

---

### **Phase 3: Production Hardening** (Week 3-4)

#### **7. Integration Tests**

```rust
#[tokio::test]
async fn test_full_ecosystem_startup() {
    let mut orchestrator = BiomeOrchestrator::new();
    
    // Start all primals
    orchestrator.start_ecosystem().await.unwrap();
    
    // Verify all healthy
    assert!(orchestrator.check_health().await.unwrap());
    
    // Shutdown
    orchestrator.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_multi_primal_workflow() {
    // Test: Upload to NestGate → Encrypt with BearDog → 
    //       Process with ToadStool → Decrypt → Retrieve
}
```

**Estimated Time**: 8-10 hours

---

#### **8. Performance Optimization**

- Connection pooling for REST adapters
- Request batching
- Adapter response caching
- Parallel startup

**Estimated Time**: 6-8 hours

---

#### **9. Monitoring & Telemetry**

- Metrics collection (requests, latency, errors)
- OpenTelemetry integration
- Dashboards

**Estimated Time**: 6-8 hours

---

## 📅 **Timeline**

### **Week 1** (Immediate)
- ✅ Day 1-2: Process Manager + Adapter Registry
- ✅ Day 3-4: Basic Orchestration Layer
- ✅ Day 5: Initial integration tests

**Deliverable**: Can start/stop all Phase 1 primals programmatically

### **Week 2** (Enhanced Features)
- ✅ Day 6-7: Health monitoring
- ✅ Day 8-9: Configuration management
- ✅ Day 10: Error recovery

**Deliverable**: Production-ready orchestration with monitoring

### **Week 3-4** (Production Hardening)
- ✅ Day 11-14: Comprehensive integration tests
- ✅ Day 15-17: Performance optimization
- ✅ Day 18-20: Telemetry & monitoring

**Deliverable**: Production-ready system with full observability

---

## 🎯 **Success Criteria**

### **Phase 1 Complete When**:
- [ ] All Phase 1 primals can be started programmatically
- [ ] Services auto-register with Songbird
- [ ] Multi-primal workflows work end-to-end
- [ ] Graceful shutdown of all services

### **Phase 2 Complete When**:
- [ ] Health checks run automatically
- [ ] Failed primals auto-restart
- [ ] Configuration loaded from biome.yaml
- [ ] Error recovery tested

### **Phase 3 Complete When**:
- [ ] 90%+ test coverage
- [ ] Performance benchmarks pass
- [ ] Monitoring dashboards operational
- [ ] Production deployment successful

---

## 🚀 **Quick Start (Next Developer)**

### **1. Verify Current State**

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo build --release
cargo test
```

**Expected**: All builds, all tests pass

### **2. Read Documentation**

1. `docs/API_ADAPTER_QUICK_REF.md` (2 min)
2. `docs/API_ADAPTER_USAGE_GUIDE.md` (15 min)
3. `showcase/api-adapter-test-results/` (discoveries)

### **3. Start Building**

**First Task**: Process Manager

```bash
# Create module
mkdir -p crates/biomeos-core/src/process_manager
touch crates/biomeos-core/src/process_manager/mod.rs

# Update lib.rs
# Add: pub mod process_manager;

# Implement ProcessManager struct (see Phase 1.1 above)
```

---

## 🔍 **Key Decisions Needed**

### **1. Configuration Format**

**Option A**: YAML (current biome.yaml)
```yaml
primals:
  songbird:
    type: cli
    path: /path/to/songbird
    auto_start: true
    port: 8080
```

**Option B**: TOML
```toml
[primals.songbird]
type = "cli"
path = "/path/to/songbird"
auto_start = true
port = 8080
```

**Recommendation**: Stick with YAML (already used)

---

### **2. Process Management**

**Option A**: Built-in ProcessManager (simple)  
**Option B**: systemd integration (production)  
**Option C**: Docker/containers (cloud-native)

**Recommendation**: Start with A, add B/C later

---

### **3. Error Handling**

**Option A**: Fail fast (strict)  
**Option B**: Graceful degradation (resilient)  
**Option C**: Hybrid (critical services fail fast, optional services degrade)

**Recommendation**: Option C - Songbird/NestGate critical, others optional

---

## 📊 **Estimated Total Effort**

| Phase | Tasks | Hours | Calendar |
|-------|-------|-------|----------|
| **Phase 1** | Core Integration | 13-18h | Week 1 |
| **Phase 2** | Enhanced Features | 10-14h | Week 2 |
| **Phase 3** | Production Hardening | 20-26h | Week 3-4 |
| **Total** | | **43-58h** | **3-4 weeks** |

With 1 developer: **3-4 weeks**  
With 2 developers: **2-3 weeks**  
With 3 developers: **1.5-2 weeks**

---

## 💡 **Architecture Principles**

### **1. Separation of Concerns**
- **Adapters**: Know how to talk to primals
- **Process Manager**: Manages primal lifecycles
- **Orchestrator**: Coordinates multi-primal workflows

### **2. Fail Gracefully**
- CLI adapter timeout? Return error, don't crash
- REST discovery fails? Try cached adapter
- Primal unreachable? Queue requests, retry

### **3. Observable**
- Log all adapter calls
- Track success/failure rates
- Monitor response times
- Alert on anomalies

### **4. Testable**
- Mock adapters for unit tests
- Test fixtures for integration tests
- Chaos testing for resilience

---

## 📚 **Resources**

### **Code Examples**
- `docs/API_ADAPTER_USAGE_GUIDE.md` - Complete usage examples
- `showcase/02-primal-pairs/` - Multi-primal demo scripts
- `showcase/04-complete-ecosystem/` - All 5 primals together

### **Discovery Reports**
- `showcase/api-adapter-test-results/` - Real architectures discovered

### **Reference Implementations**
- Songbird's `showcase/` - Multi-tower federation
- ToadStool's `showcase/` - Compute orchestration

---

## 🎊 **Final Notes**

### **What's Already Built** ✅
- All 5 Phase 1 adapters (production-ready)
- CLI adapter base class (tested)
- Discovery system (127+ patterns)
- Comprehensive documentation

### **What's Needed** 📝
- Process manager (for CLI primals)
- Adapter registry (centralized management)
- Orchestration layer (multi-primal workflows)
- Integration tests (end-to-end validation)

### **Philosophy to Maintain** 🌟
> "We adapt to primals, not the other way around. Each primal's sovereignty is preserved through flexible integration."

---

🦀 **Pure Rust. Reality-Based Integration. Human Dignity First.**

**Ready to integrate. Ready to orchestrate. Ready to scale.** 🚀

---

*Next Step: Create `crates/biomeos-core/src/process_manager/mod.rs`*


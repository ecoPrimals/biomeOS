# BiomeOS Evolution Plan - December 24, 2025

**Status**: 🔄 **MAJOR RESCOPE REQUIRED**  
**Grade**: C+ → Target: A (Production-Ready Substrate)  
**Philosophy**: BiomeOS is the **substrate** that enables composition, not a reimplementation

---

## 🎯 Executive Summary

After reviewing the **actual capabilities** of production-ready primals (Songbird, ToadStool, NestGate, Squirrel, BearDog), it's clear that **BiomeOS has significant overlap and outdated assumptions**.

**Key Insight**: BiomeOS was paused and now contains systems that mature primals already provide better. We need to **prune, delegate, and evolve** to become a true composition substrate.

---

## 📊 Primal Capability Matrix (ACTUAL vs ASSUMED)

### Songbird (Production-Ready, Grade A)

**ACTUAL Capabilities**:
- ✅ **Universal Service Discovery** - mDNS + capability-based + zero hardcoding
- ✅ **Service Registry** - UPA (Universal Primal Adapter) with health checks
- ✅ **Gaming Auto-Configuration** - One-touch gaming network setup
- ✅ **AI-First Workload Classification** - Intelligent routing decisions
- ✅ **Federation Management** - Multi-node coordination
- ✅ **Load Balancing** - Intelligent request routing
- ✅ **Health Monitoring** - Real-time service health tracking
- ✅ **Protocol Escalation** - HTTP → WebSocket → gRPC → QUIC
- ✅ **Geolocation Discovery** - Can route based on location (with metadata)

**API Endpoints**:
```
POST /api/v1/services/register          # Service registration
GET  /api/v1/services/query/{capability} # Capability-based discovery
POST /api/gaming/setup                   # Gaming auto-config
POST /api/ai/classify                    # Workload classification
GET  /api/federation/nodes               # Federation status
GET  /api/health/{service_id}            # Health checks
```

### ToadStool (Production-Ready, Grade A)

**ACTUAL Capabilities**:
- ✅ **Multi-Runtime Execution** - Native, WASM, GPU, Python (roadmap)
- ✅ **Capability Discovery** - mDNS integrated, zero hardcoding
- ✅ **Distributed GPU** - 25 comprehensive tests, production-ready
- ✅ **Workload Orchestration** - Deploy, scale, monitor
- ✅ **Resource Management** - CPU, memory, GPU allocation
- ✅ **BYOB (Build Your Own Biome)** - Manifest-based deployment
- ✅ **Secure Execution** - Sandboxed environments

**API Pattern**:
```rust
// ToadStool handles ALL compute orchestration
toadstool.deploy_workload(manifest).await?;
toadstool.scale_service(service_id, replicas).await?;
toadstool.get_resource_usage(service_id).await?;
```

### NestGate (Production-Ready)

**ACTUAL Capabilities**:
- ✅ **Distributed Storage** - ZFS-backed, multi-node
- ✅ **Volume Provisioning** - Automated storage allocation
- ✅ **MCP Integration** - Model Context Protocol support
- ✅ **Zero-Copy Operations** - Memory-efficient data handling
- ✅ **Circuit Breakers** - Fault tolerance built-in
- ✅ **Adaptive Compression** - Intelligent data compression
- ✅ **Federation** - Multi-node storage coordination

**API Pattern**:
```rust
// NestGate handles ALL storage operations
nestgate.provision_volume(spec).await?;
nestgate.store_data(key, value).await?;
nestgate.get_data(key).await?;
```

### Squirrel (Production-Ready, Grade A++)

**ACTUAL Capabilities**:
- ✅ **AI Orchestration** - Multi-provider AI coordination
- ✅ **Agent Management** - Deploy, monitor, coordinate agents
- ✅ **Context Sharing** - Cross-agent context management
- ✅ **Capability Discovery** - Runtime AI service discovery
- ✅ **MCP Platform** - Model Context Protocol implementation
- ✅ **Zero Vendor Lock-in** - Provider-agnostic architecture
- ✅ **Optimization Analysis** - AI-powered system optimization

**API Pattern**:
```rust
// Squirrel handles ALL AI operations
squirrel.discover_by_capability("ai-inference").await?;
squirrel.process_ai_request(request).await?;
squirrel.optimize_system(metrics).await?;
```

### BearDog (Production-Ready, Grade A+)

**ACTUAL Capabilities**:
- ✅ **Cryptography** - Ed25519, X25519, ChaCha20-Poly1305, AES-GCM
- ✅ **BTSP Protocol** - Secure transport with forward secrecy
- ✅ **BirdSong Privacy** - Lineage-based encryption
- ✅ **Mesh Networking** - libp2p + quinn
- ✅ **Key Management** - Self-enforcing constraints
- ✅ **Zero-Knowledge Bootstrap** - Secure node onboarding
- ✅ **Ecosystem Genetics** - Trust evolution and symbiosis

**API Pattern**:
```rust
// BearDog handles ALL security operations
beardog.encrypt(data, recipient).await?;
beardog.establish_btsp_tunnel(peer).await?;
beardog.verify_lineage(key).await?;
```

---

## ❌ What BiomeOS Should NOT Do (Primal Overlap)

### 1. ❌ Service Discovery & Registry
**Current**: BiomeOS has mock discovery in `discovery.rs`  
**Reality**: **Songbird provides this** via UPA registry + mDNS  
**Action**: **DELETE** - Delegate 100% to Songbird

```rust
// WRONG (BiomeOS doing discovery):
manager.discover_by_location(lat, lon, radius).await?;

// RIGHT (Delegate to Songbird):
songbird.query_services_by_capability("compute").await?;
```

### 2. ❌ Resource Metrics & Monitoring
**Current**: BiomeOS has mock metrics in `operations.rs:508`  
**Reality**: **Songbird provides health monitoring**, **ToadStool provides resource metrics**  
**Action**: **DELETE** - Query primals for real metrics

```rust
// WRONG (BiomeOS mocking metrics):
result.insert("cpu_percent", json!(15.5));  // Mock!

// RIGHT (Query actual primal):
let metrics = toadstool.get_resource_usage(service_id).await?;
```

### 3. ❌ AI Optimization Analysis
**Current**: BiomeOS has mock AI analysis in `ai.rs:259`  
**Reality**: **Squirrel provides AI orchestration and optimization**  
**Action**: **DELETE** - Delegate to Squirrel

```rust
// WRONG (BiomeOS mocking AI):
result.insert("performance_score", json!(85));  // Mock!

// RIGHT (Delegate to Squirrel):
let analysis = squirrel.optimize_system(metrics).await?;
```

### 4. ❌ Geolocation Discovery
**Current**: BiomeOS has mock geolocation in `discovery.rs:122`  
**Reality**: **Songbird can route by location** (with service metadata)  
**Action**: **DELETE** - Delegate to Songbird with location metadata

```rust
// WRONG (BiomeOS implementing geo):
pub async fn discover_by_location(...) -> Result<Vec<DiscoveryResult>> {
    Ok(vec![])  // Mock!
}

// RIGHT (Songbird with metadata):
let services = songbird.query_services_with_metadata(
    "compute",
    |meta| meta.location.distance_to(lat, lon) < radius_km
).await?;
```

### 5. ❌ Load Balancing & Routing
**Current**: Removed in MOCK_SCOPE_ANALYSIS (good!)  
**Reality**: **Songbird provides intelligent routing**  
**Status**: ✅ Already delegated

### 6. ❌ Workload Execution
**Current**: BiomeOS has deployment logic  
**Reality**: **ToadStool handles ALL execution**  
**Action**: **SIMPLIFY** - BiomeOS should only coordinate, not execute

---

## ✅ What BiomeOS SHOULD Do (Composition Substrate)

### 1. ✅ Manifest Parsing & Validation
**Purpose**: Understand biome.yaml and validate structure  
**Status**: ✅ Good - `biomeos-manifest` crate  
**Action**: **KEEP** - This is BiomeOS's domain

```rust
// BiomeOS parses and validates manifests
let manifest = BiomeManifest::from_file("biome.yaml")?;
manifest.validate()?;
```

### 2. ✅ Capability Matching & Coordination
**Purpose**: Match manifest requirements to discovered primals  
**Status**: ⚠️ Partial - Needs real implementation  
**Action**: **ENHANCE** - Core BiomeOS responsibility

```rust
// BiomeOS coordinates between manifest and primals
let manifest = parse_manifest("biome.yaml")?;

// Discover what's available (via Songbird)
let available = songbird.discover_all_services().await?;

// Match capabilities (BiomeOS logic)
let deployment_plan = capability_matcher.resolve(
    &manifest.required_capabilities,
    &available
)?;

// Execute plan (delegate to primals)
for step in deployment_plan.steps {
    match step.primal {
        "toadstool" => toadstool.execute(step.action).await?,
        "nestgate" => nestgate.execute(step.action).await?,
        _ => return Err("Unknown primal"),
    }
}
```

### 3. ✅ Cross-Primal Workflow Orchestration
**Purpose**: Coordinate multi-primal workflows  
**Status**: ⚠️ Needs implementation  
**Action**: **BUILD** - BiomeOS's unique value

```rust
// Example: Deploy AI workload with storage
async fn deploy_ai_workload(manifest: &Manifest) -> Result<Deployment> {
    // 1. Provision storage (NestGate)
    let volume = nestgate.provision_volume(&manifest.storage).await?;
    
    // 2. Deploy compute (ToadStool)
    let service = toadstool.deploy_workload(&manifest.compute).await?;
    
    // 3. Register with discovery (Songbird)
    songbird.register_service(&service).await?;
    
    // 4. Setup AI routing (Squirrel)
    squirrel.register_ai_endpoint(&service).await?;
    
    Ok(Deployment { volume, service })
}
```

### 4. ✅ Biome Lifecycle Management
**Purpose**: Start, stop, monitor, update biomes  
**Status**: ⚠️ Partial  
**Action**: **BUILD** - Coordinate primal lifecycles

```rust
// BiomeOS manages biome lifecycle
pub async fn start_biome(&self, biome_id: &str) -> Result<()> {
    let manifest = self.load_manifest(biome_id)?;
    
    // Start all required primals in order
    for primal in manifest.primals {
        self.start_primal(&primal).await?;
    }
    
    // Wait for health
    self.wait_for_healthy(biome_id).await?;
    
    Ok(())
}
```

### 5. ✅ Configuration Management
**Purpose**: Manage biome configurations  
**Status**: ✅ Good - `biomeos-types` has config  
**Action**: **KEEP** - BiomeOS manages biome-level config

### 6. ✅ Chimera Composition
**Purpose**: Define and build multi-primal chimeras  
**Status**: ✅ Good - `biomeos-chimera` crate  
**Action**: **KEEP** - Unique BiomeOS feature

### 7. ✅ Niche Deployment
**Purpose**: Deploy complete biome environments  
**Status**: ✅ Good - `biomeos-niche` crate  
**Action**: **KEEP** - Unique BiomeOS feature

### 8. ✅ Sovereignty Guardian
**Purpose**: Enforce data sovereignty and privacy  
**Status**: ✅ Good - `sovereignty_guardian.rs`  
**Action**: **KEEP** - BiomeOS policy enforcement

---

## 🔥 Immediate Actions (Week 1)

### 1. DELETE Mock Implementations

**Files to Modify**:
```
crates/biomeos-core/src/universal_biomeos_manager/operations.rs
  - Line 455: Delete mock replica count
  - Line 508: Delete mock resource metrics
  
crates/biomeos-core/src/universal_biomeos_manager/ai.rs
  - Line 259: Delete mock optimization analysis
  
crates/biomeos-cli/src/discovery.rs
  - Line 122: Delete mock geolocation discovery
```

**Replace With**:
```rust
// operations.rs - Real metrics from ToadStool
pub async fn get_service_metrics(&self, service: &str) -> Result<ServiceMetrics> {
    let toadstool = self.discover_primal("compute").await?;
    toadstool.get_resource_usage(service).await
}

// ai.rs - Real analysis from Squirrel
pub async fn optimize_system(&self, metrics: &SystemMetrics) -> Result<OptimizationPlan> {
    let squirrel = self.discover_primal("ai").await?;
    squirrel.analyze_and_optimize(metrics).await
}

// discovery.rs - Real geolocation from Songbird
pub async fn discover_by_location(
    &self,
    lat: f64,
    lon: f64,
    radius_km: f64,
) -> Result<Vec<Service>> {
    let songbird = self.discover_primal("discovery").await?;
    songbird.query_by_location(lat, lon, radius_km).await
}
```

### 2. Remove Hardcoded Endpoints

**Files to Modify**:
```
crates/biomeos-types/src/constants.rs
  - Delete all FALLBACK_*_ENDPOINT constants
  
crates/biomeos-core/src/universal_biomeos_manager/operations.rs
  - Lines 241, 253, 263: Remove hardcoded "http://toadstool:8080"
```

**Replace With**:
```rust
// Use environment variables with proper error handling
pub fn get_primal_endpoint(primal: &str) -> Result<String> {
    std::env::var(format!("{}_ENDPOINT", primal.to_uppercase()))
        .or_else(|_| {
            // Fall back to discovery via Songbird
            discover_primal_endpoint(primal)
        })
}
```

### 3. Fix Remaining Clippy Errors

**Files to Fix**:
```
crates/biomeos-niche/src/definition.rs (lines 210, 217, 224, 271)
crates/biomeos-niche/src/deployment.rs (lines 100, 144, 192)
```

**Action**: Add `# Errors` documentation to all functions returning `Result`

---

## 🏗️ Architecture Evolution (Weeks 2-4)

### Current Architecture (WRONG)
```
BiomeOS
  ├─ Mock Discovery
  ├─ Mock Metrics
  ├─ Mock AI Analysis
  ├─ Mock Geolocation
  └─ Hardcoded Endpoints
```

### Target Architecture (RIGHT)
```
BiomeOS (Composition Substrate)
  │
  ├─ Manifest Parser ────────────┐
  ├─ Capability Matcher          │
  ├─ Workflow Orchestrator       │
  ├─ Lifecycle Manager           │
  └─ Sovereignty Guardian        │
                                 │
  ┌──────────────────────────────┘
  │
  ├─> Songbird (Discovery & Coordination)
  │     ├─ Service Registry
  │     ├─ Health Monitoring
  │     ├─ Load Balancing
  │     ├─ Geolocation Routing
  │     └─ Federation Management
  │
  ├─> ToadStool (Compute Orchestration)
  │     ├─ Workload Execution
  │     ├─ Resource Management
  │     ├─ GPU Coordination
  │     └─ Multi-Runtime Support
  │
  ├─> NestGate (Storage & Persistence)
  │     ├─ Volume Provisioning
  │     ├─ Data Storage
  │     ├─ MCP Integration
  │     └─ Federation Storage
  │
  ├─> Squirrel (AI Orchestration)
  │     ├─ AI Service Discovery
  │     ├─ Agent Management
  │     ├─ Optimization Analysis
  │     └─ Context Sharing
  │
  └─> BearDog (Security & Crypto)
        ├─ Encryption
        ├─ BTSP Tunnels
        ├─ Key Management
        └─ Lineage Verification
```

### Delegation Pattern

```rust
// BiomeOS Universal Manager (Simplified)
pub struct UniversalBiomeOSManager {
    // Primal clients (discovered at runtime)
    songbird: Option<SongbirdClient>,
    toadstool: Option<ToadStoolClient>,
    nestgate: Option<NestGateClient>,
    squirrel: Option<SquirrelClient>,
    beardog: Option<BearDogClient>,
    
    // BiomeOS-specific logic
    manifest_parser: ManifestParser,
    capability_matcher: CapabilityMatcher,
    workflow_orchestrator: WorkflowOrchestrator,
    lifecycle_manager: LifecycleManager,
    sovereignty_guardian: SovereigntyGuardian,
}

impl UniversalBiomeOSManager {
    /// Discover and connect to primals (via Songbird)
    pub async fn initialize(&mut self) -> Result<()> {
        // Bootstrap: Find Songbird first
        let songbird_endpoint = std::env::var("SONGBIRD_ENDPOINT")
            .or_else(|_| discover_via_mdns("songbird"))?;
        
        self.songbird = Some(SongbirdClient::connect(&songbird_endpoint).await?);
        
        // Use Songbird to discover other primals
        let songbird = self.songbird.as_ref().unwrap();
        
        if let Ok(endpoint) = songbird.query_capability("compute").await {
            self.toadstool = Some(ToadStoolClient::connect(&endpoint).await?);
        }
        
        if let Ok(endpoint) = songbird.query_capability("storage").await {
            self.nestgate = Some(NestGateClient::connect(&endpoint).await?);
        }
        
        if let Ok(endpoint) = songbird.query_capability("ai").await {
            self.squirrel = Some(SquirrelClient::connect(&endpoint).await?);
        }
        
        if let Ok(endpoint) = songbird.query_capability("security").await {
            self.beardog = Some(BearDogClient::connect(&endpoint).await?);
        }
        
        Ok(())
    }
    
    /// Deploy a biome (orchestration logic)
    pub async fn deploy_biome(&self, manifest_path: &str) -> Result<Deployment> {
        // 1. Parse manifest (BiomeOS logic)
        let manifest = self.manifest_parser.parse(manifest_path)?;
        
        // 2. Match capabilities (BiomeOS logic)
        let plan = self.capability_matcher.create_plan(&manifest)?;
        
        // 3. Execute workflow (delegate to primals)
        self.workflow_orchestrator.execute(&plan, &PrimalClients {
            songbird: self.songbird.as_ref(),
            toadstool: self.toadstool.as_ref(),
            nestgate: self.nestgate.as_ref(),
            squirrel: self.squirrel.as_ref(),
            beardog: self.beardog.as_ref(),
        }).await
    }
    
    /// Get system health (aggregate from primals)
    pub async fn get_system_health(&self) -> Result<SystemHealth> {
        let mut health = SystemHealth::default();
        
        // Query each primal via Songbird
        if let Some(songbird) = &self.songbird {
            let services = songbird.get_all_services().await?;
            for service in services {
                let service_health = songbird.get_service_health(&service.id).await?;
                health.add_service(service.name, service_health);
            }
        }
        
        Ok(health)
    }
}
```

---

## 📊 Code Reduction Targets

### Current State
- **Total Lines**: ~32,380 lines
- **Mock Code**: ~200 lines
- **Hardcoded Values**: ~50 lines
- **Overlap with Primals**: ~5,000+ lines (from previous MOCK_SCOPE_ANALYSIS)

### Target State (After Evolution)
- **Total Lines**: ~25,000 lines (22% reduction)
- **Mock Code**: 0 lines (test mocks only)
- **Hardcoded Values**: 0 lines
- **Overlap with Primals**: 0 lines

### Files to Prune/Simplify

**High Priority**:
1. `crates/biomeos-core/src/universal_biomeos_manager/operations.rs` (874 → ~400 lines)
2. `crates/biomeos-core/src/universal_biomeos_manager/discovery.rs` (simplify to Songbird delegation)
3. `crates/biomeos-core/src/universal_biomeos_manager/ai.rs` (simplify to Squirrel delegation)
4. `crates/biomeos-core/src/universal_biomeos_manager/health.rs` (aggregate, don't implement)

**Medium Priority**:
5. `crates/biomeos-cli/src/discovery.rs` (remove mock geolocation)
6. `crates/biomeos-cli/src/health.rs` (simplify to primal queries)
7. `crates/biomeos-cli/src/monitoring.rs` (delegate to Songbird)

**Low Priority** (Keep, but review):
8. `crates/biomeos-chimera/*` - Unique to BiomeOS ✅
9. `crates/biomeos-niche/*` - Unique to BiomeOS ✅
10. `crates/biomeos-manifest/*` - BiomeOS domain ✅

---

## 🧪 Testing Strategy Evolution

### Current Issues
- Tests use wiremock, not real primals
- E2E tests don't use phase1bins
- Coverage is 37.69% (target: 75%+)

### New Strategy

**1. Integration Tests with Real Primals**
```bash
# Start real primals from phase1bins
./phase1bins/songbird-bin serve &
./phase1bins/toadstool-bin serve &
./phase1bins/nestgate-bin serve &

# Run BiomeOS integration tests
cargo test --test real_primal_integration
```

**2. E2E Workflow Tests**
```rust
#[tokio::test]
async fn test_full_biome_deployment_with_real_primals() {
    // Start real primals
    let primals = start_real_primals().await?;
    
    // Deploy biome
    let manager = UniversalBiomeOSManager::new().await?;
    let deployment = manager.deploy_biome("examples/ai-research.yaml").await?;
    
    // Verify all primals were used
    assert!(deployment.used_toadstool);
    assert!(deployment.used_nestgate);
    assert!(deployment.used_squirrel);
    
    // Cleanup
    primals.shutdown().await?;
}
```

**3. Coverage Targets**
- **Core Logic**: 85%+ (capability matching, orchestration)
- **Delegation**: 60%+ (verify calls to primals)
- **CLI**: 40%+ (user-facing commands)
- **Overall**: 75%+ (realistic for orchestrator)

---

## 📋 Specification Updates

### Specs to Update

**1. CROSS_PRIMAL_API_CONTRACTS.md**
- ✅ Keep spec (documents primal APIs)
- ⚠️ Update: BiomeOS uses these, doesn't implement them

**2. PRIMAL_SERVICE_REGISTRATION_STANDARDS.md**
- ✅ Keep spec (Songbird implements this)
- ⚠️ Update: BiomeOS discovers via Songbird

**3. BOOTSTRAP_ORCHESTRATION_SEQUENCE.md**
- ✅ Keep spec (BiomeOS orchestrates)
- ⚠️ Update: Use real primals, not mocks

**4. SPECIFICATION_COMPLETION_SUMMARY.md**
- ❌ Update: Change "100% complete" to "85% complete"
- Document gaps between spec and implementation

### New Specs Needed

**1. BIOMEOS_DELEGATION_PATTERNS.md**
- How BiomeOS delegates to each primal
- When to use which primal
- Error handling patterns

**2. BIOMEOS_ORCHESTRATION_PATTERNS.md**
- Multi-primal workflow patterns
- Lifecycle management
- Rollback strategies

**3. BIOMEOS_TESTING_STRATEGY.md**
- Real primal integration tests
- Mock vs real testing guidelines
- Coverage targets by component

---

## 🎯 Success Metrics

### Week 1 (Immediate)
- [ ] All mock implementations removed
- [ ] All hardcoded endpoints removed
- [ ] All clippy errors fixed
- [ ] Build passes with `-D warnings`

### Week 2-3 (Short-term)
- [ ] Real primal integration tests passing
- [ ] Coverage increased to 60%+
- [ ] Delegation pattern implemented
- [ ] Documentation updated

### Week 4 (Medium-term)
- [ ] E2E tests with phase1bins passing
- [ ] Coverage increased to 75%+
- [ ] All specs updated
- [ ] Grade: B+ (Functional Substrate)

### Month 2 (Long-term)
- [ ] Chimera composition working
- [ ] Niche deployment working
- [ ] Multi-primal workflows tested
- [ ] Grade: A (Production-Ready Substrate)

---

## 💡 Key Principles

### 1. **BiomeOS is a Substrate, Not a Reimplementation**
- Don't reimplement what primals do
- Coordinate, don't execute
- Compose, don't duplicate

### 2. **Delegate Everything Possible**
- Discovery → Songbird
- Compute → ToadStool
- Storage → NestGate
- AI → Squirrel
- Security → BearDog

### 3. **BiomeOS Owns Composition**
- Manifest parsing
- Capability matching
- Workflow orchestration
- Lifecycle management
- Sovereignty enforcement

### 4. **Test with Real Primals**
- Use phase1bins for integration tests
- Mock only when primals unavailable
- Verify actual behavior, not test harness

### 5. **Zero Hardcoding**
- Discover primals at runtime
- Use environment variables
- Fail gracefully when primals missing

---

## 🚀 Next Steps

1. **Read this document thoroughly**
2. **Review COMPREHENSIVE_AUDIT_DEC_24_2025.md** for detailed findings
3. **Start with Week 1 immediate actions**
4. **Update todos and track progress**
5. **Test with real primals from phase1bins**

---

**Status**: 🔄 Evolution Plan Complete  
**Next**: Begin implementation (Week 1 actions)  
**Goal**: Transform BiomeOS from overlapping system to true composition substrate

---

*"BiomeOS enables composition. Primals provide capabilities. Together, they create ecosystems."*


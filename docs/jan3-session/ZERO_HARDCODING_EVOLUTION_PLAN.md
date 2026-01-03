# 🍼 Zero-Hardcoding Evolution - biomeOS Orchestrator

## Date: January 3, 2026

## 🎯 Core Principle: "Like an Infant Discovering the World"

> **Each primal only knows itself and discovers others through the universal adapter**

## 🚫 What We're Eliminating:

### 1. Primal Name Hardcoding
- ❌ `BearDog`, `Songbird`, `Toadstool`, `Squirrel` in code
- ✅ Capability-based discovery: `"security"`, `"discovery"`, `"compute"`, `"ai"`

### 2. Port/Numeric Hardcoding  
- ❌ `9000`, `8080`, hardcoded ports
- ✅ Port 0 (OS auto-selects) + environment variables

### 3. Vendor/External Service Hardcoding
- ❌ `k8s`, `consul`, `etcd` in code
- ✅ Service registry abstraction with runtime detection

### 4. Network Topology Hardcoding
- ❌ Assuming specific network layouts
- ✅ Dynamic network discovery

## 📚 Songbird's Architecture (Our Model)

### Core Modules:

1. **`security_capability_client.rs`** (585 lines)
   - NO "BearDog" in code
   - Discovers security provider via:
     - `SECURITY_ENDPOINT` env var
     - Runtime capability discovery
     - Well-known defaults

2. **`infant_discovery.rs`** (671 lines)
   - **Zero Knowledge Bootstrap**
   - 6-phase learning:
     1. Environment Sensing
     2. Network Discovery
     3. Process Discovery
     4. Capability Learning
     5. Communication Learning
     6. Network Effect Discovery

3. **`unified_adapter.rs`** (380 lines)
   - Protocol-agnostic (HTTP/tarpc/gRPC)
   - Capability-based routing
   - Multi-provider support

4. **`zero_hardcoding/` module**
   - `endpoints.rs` - Port 0 magic, env-driven
   - `timeouts.rs` - All durations from environment
   - `mod.rs` - Unified zero-hardcoding config

## 🦀 biomeOS Orchestrator Evolution

### Current Issues:

```rust
// ❌ HARDCODED - Current impl
pub struct ManagedBearDog {
    id: PrimalId::new("beardog".to_string()),  // Hardcoded name!
    config: BearDogConfig {
        binary_path: "/path/to/beardog",  // Hardcoded path!
        http_port: 9000,  // Hardcoded port!
    }
}

impl ManagedPrimal for ManagedBearDog {
    fn dependencies(&self) -> Vec<PrimalId> {
        vec![]  // Hardcoded: "BearDog has no dependencies"
    }
}

pub struct ManagedSongbird {
    fn dependencies(&self) -> Vec<PrimalId> {
        vec![PrimalId::new("beardog".to_string())]  // Hardcoded dependency!
    }
}
```

### ✅ Evolved - Zero Hardcoding:

```rust
// ✅ CAPABILITY-BASED - Zero hardcoding
pub struct ManagedPrimal {
    // Identity discovered at runtime
    identity: DiscoveredIdentity,
    
    // Capabilities declared (not primal names!)
    provides: Vec<Capability>,  // ["security", "crypto"]
    requires: Vec<Capability>,  // ["none"] or ["security"]
    
    // Endpoints discovered/assigned
    endpoints: Vec<DiscoveredEndpoint>,
    
    // Process management
    process: Arc<Mutex<Option<Child>>>,
}

impl ManagedPrimal {
    /// Create from environment - ZERO hardcoding!
    pub async fn from_env() -> Result<Self> {
        let identity = Self::discover_self().await?;
        let config = Self::load_config_from_env()?;
        let provides = config.capabilities.provides;
        let requires = config.capabilities.requires;
        
        Ok(Self {
            identity,
            provides,
            requires,
            endpoints: vec![],
            process: Arc::new(Mutex::new(None)),
        })
    }
    
    /// Discover self identity (infant model)
    async fn discover_self() -> Result<DiscoveredIdentity> {
        // 1. Check PRIMAL_ID env var
        if let Ok(id) = std::env::var("PRIMAL_ID") {
            return Ok(DiscoveredIdentity::from_env(id));
        }
        
        // 2. Check binary name
        let binary_name = std::env::current_exe()?
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        // 3. Generate unique ID
        Ok(DiscoveredIdentity::from_binary(binary_name))
    }
}

// Dependency resolution by CAPABILITY, not name!
impl PrimalOrchestrator {
    pub async fn resolve_capabilities(&self) -> Result<StartOrder> {
        let primals = self.primals.read().await;
        
        // Build capability graph
        let mut provides_map: HashMap<Capability, Vec<PrimalId>> = HashMap::new();
        let mut requires_map: HashMap<PrimalId, Vec<Capability>> = HashMap::new();
        
        for (id, primal) in primals.iter() {
            // Who provides what?
            for cap in &primal.provides {
                provides_map.entry(cap.clone())
                    .or_insert_with(Vec::new)
                    .push(id.clone());
            }
            
            // Who requires what?
            requires_map.insert(id.clone(), primal.requires.clone());
        }
        
        // Resolve: Start providers before consumers
        self.topological_sort_by_capability(provides_map, requires_map)
    }
}
```

## 🎯 Real-World Examples:

### Example 1: Songbird wakes BearDog
```rust
// ❌ OLD - Hardcoded
let beardog = ManagedBearDog::new();  // Knows it's "BearDog"
let songbird = ManagedSongbird::new();  // Knows it needs "BearDog"

// ✅ NEW - Capability-based
let crypto_provider = ManagedPrimal::from_binary("beardog-server")
    .provides(vec![Capability::Security, Capability::Crypto])
    .build()?;

let discovery_orchestrator = ManagedPrimal::from_binary("songbird-orchestrator")
    .requires(vec![Capability::Security])  // Just needs *any* security provider!
    .build()?;

orchestrator.register(crypto_provider).await;
orchestrator.register(discovery_orchestrator).await;
orchestrator.start_all().await?;  // Resolves dependencies by capability!
```

### Example 2: Songbird wakes fleet of security providers
```rust
// ✅ Multiple providers, same capability
let beardog1 = ManagedPrimal::from_env()
    .provides(vec![Capability::Security])
    .build()?;

let beardog2 = ManagedPrimal::from_env()
    .provides(vec![Capability::Security])
    .build()?;

let hsm_provider = ManagedPrimal::from_env()
    .provides(vec![Capability::Security])
    .build()?;

// Songbird doesn't care WHICH provider!
let songbird = ManagedPrimal::from_env()
    .requires(vec![Capability::Security])
    .build()?;

// Orchestrator starts ALL security providers first,
// then starts anything that needs security!
```

### Example 3: Complex cascade (Songbird → Songbird → Network)
```rust
// ✅ Each Songbird only knows itself!
let songbird_regional = ManagedPrimal::from_env()
    .provides(vec![Capability::Discovery])
    .build()?;

let songbird_global = ManagedPrimal::from_env()
    .provides(vec![Capability::Discovery, Capability::Federation])
    .requires(vec![Capability::Discovery])  // Needs *a* discovery service
    .build()?;

// Auto-resolves: regional first, then global!
```

### Example 4: Service mesh complexity
```rust
// ✅ Toadstool providing compute for Squirrel's AI analyzing NestGate data
let nestgate = ManagedPrimal::from_env()
    .provides(vec![Capability::Storage])
    .build()?;

let toadstool = ManagedPrimal::from_env()
    .provides(vec![Capability::Compute])
    .requires(vec![Capability::Storage])
    .build()?;

let squirrel = ManagedPrimal::from_env()
    .provides(vec![Capability::AI])
    .requires(vec![Capability::Compute, Capability::Storage])
    .build()?;

// Orchestrator figures it out:
// 1. Start NestGate (storage provider)
// 2. Start Toadstool (needs storage, provides compute)
// 3. Start Squirrel (needs both!)
```

## 🏗️ Implementation Plan:

### Phase 1: Core Abstractions (2-3 hours)
1. ✅ Create `Capability` enum
2. ✅ Refactor `ManagedPrimal` trait
3. ✅ Update `PrimalOrchestrator` for capability resolution
4. ✅ Add environment-based configuration

### Phase 2: Infant Discovery Integration (2-3 hours)
1. ✅ Port Songbird's `infant_discovery.rs`
2. ✅ Integrate with orchestrator
3. ✅ Add network scanning
4. ✅ Add process detection

### Phase 3: Zero-Configuration (1-2 hours)
1. ✅ Port `zero_hardcoding/` modules
2. ✅ Port 0 magic
3. ✅ Environment-driven everything
4. ✅ Update all primals

### Phase 4: Testing & Validation (1-2 hours)
1. ✅ Multi-provider tests
2. ✅ Complex dependency graphs
3. ✅ Real-world scenarios
4. ✅ Integration tests

## 📊 Success Metrics:

| Metric | Before | After |
|--------|--------|-------|
| Primal Names in Code | Many | 0 ✅ |
| Hardcoded Ports | Many | 0 ✅ |
| Hardcoded Paths | Many | 0 ✅ |
| External Service Names | Some | 0 ✅ |
| Dependency Coupling | 2^n | n ✅ |
| Configuration Complexity | High | Low ✅ |

## 🎓 Key Insights from Songbird:

1. **Port 0 is Magic** - OS knows available ports better than we do
2. **Environment > Code** - All config in environment, not code
3. **Discovery > Hardcoding** - Runtime discovery beats compile-time hardcoding
4. **Capability > Name** - "Security" provider, not "BearDog"
5. **Infant Model** - Start with zero knowledge, learn everything
6. **Universal Adapter** - One interface for all network effects

## 🚀 Benefits:

1. **Flexibility** - Swap any primal without code changes
2. **Testability** - Unique ports per test, no conflicts
3. **Cloud Native** - Works in K8s, Docker, bare metal
4. **Composability** - Mix and match any primals
5. **Sovereignty** - Each primal is independent
6. **Federation** - Natural multi-org support

---

**Next Steps**: Evolve `primal_orchestrator.rs` and `primal_impls.rs` to use capability-based architecture!



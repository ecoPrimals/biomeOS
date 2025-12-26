# Hardcoding Audit - December 24, 2025

**Status**: 🔍 **AUDIT IN PROGRESS**  
**Philosophy**: **Zero-Knowledge Startup** (Infant Discovery Pattern)  
**Goal**: Remove ALL hardcoding for universal discovery

---

## 🎯 Core Principle

> "Each primal wakes up like an infant - knowing only itself, discovering everything through the universal adapter."

**No 2^n connections**. Only **n → 1 → n** through the universal adapter (Songbird).

---

## 📊 Hardcoding Found

### 1. Primal Names
**264 matches across 27 files**

Common patterns:
- `songbird`, `toadstool`, `squirrel`, `nestgate`, `beardog`, `petaltongue`
- String literals in discovery code
- Hardcoded capability → primal mappings
- Direct primal references

**Impact**: HIGH - Violates "primals only know themselves" principle

### 2. Vendor Names
**55 matches across 14 files**

Common patterns:
- `kubernetes`, `k8s`, `docker`, `containerd`
- `consul`, `etcd`
- Vendor-specific APIs
- Runtime detection logic

**Impact**: MEDIUM - Prevents deployment flexibility

### 3. Ports & Endpoints
**28 matches across 8 files**

Common patterns:
- `:3000` (Songbird default)
- `:8080` (ToadStool default)
- `localhost:` references
- Hardcoded URLs

**Impact**: HIGH - Prevents dynamic configuration

---

## 🏗️ Current Architecture (Problematic)

```
ToadStool
   ├─> "I need Songbird at localhost:3000"  ❌
   ├─> "I need Squirrel for AI"              ❌
   └─> "I'll use k8s for orchestration"      ❌

Songbird
   ├─> "ToadStool runs on :8080"             ❌
   └─> "I'll use consul for registry"        ❌
```

**Problem**: 2^n connections, hardcoded dependencies

---

## 🎯 Target Architecture (Zero-Knowledge)

```
Any Primal
   │
   ├─> "I am [NAME]"                         ✅
   ├─> "I provide [CAPABILITIES]"            ✅
   ├─> "Where is the universal adapter?"     ✅
   │   (from env: DISCOVERY_ENDPOINT)
   │
   └─> Universal Adapter (Songbird)
       └─> "Query by capability"             ✅
           "No primal names needed"          ✅
```

**Solution**: n → 1 → n through universal adapter

---

## 📋 Files Requiring Changes

### Priority 1: Core Discovery (HIGH)

1. **`crates/biomeos-core/src/universal_biomeos_manager/discovery.rs`**
   - Remove primal name hardcoding
   - Use capability-based discovery only

2. **`crates/biomeos-core/src/clients/songbird.rs`**
   - Remove port hardcoding
   - Get endpoint from environment/discovery

3. **`crates/biomeos-core/src/clients/toadstool.rs`**
   - Remove port hardcoding
   - Get endpoint from environment/discovery

4. **`crates/biomeos-types/src/constants.rs`**
   - Remove all hardcoded endpoints
   - Move to runtime configuration

### Priority 2: Configuration (HIGH)

5. **`crates/biomeos-types/src/config/resources.rs`**
   - Remove vendor hardcoding
   - Create agnostic adapter pattern

6. **`crates/biomeos-core/src/config/mod.rs`**
   - Remove hardcoded defaults
   - Support runtime discovery

### Priority 3: Service Types (MEDIUM)

7. **`crates/biomeos-types/src/service/*.rs`**
   - Abstract vendor-specific types
   - Create generic service definitions

8. **`crates/biomeos-types/src/manifest/*.rs`**
   - Remove k8s/consul specific fields
   - Use agnostic metadata

### Priority 4: Chimera & Niche (MEDIUM)

9. **`crates/biomeos-chimera/src/*.rs`**
   - Remove primal name assumptions
   - Use capability queries

10. **`crates/biomeos-niche/src/*.rs`**
    - Remove primal type hardcoding
    - Use discovery for organism types

---

## 🔧 Evolution Strategy

### Phase 1: Discovery Bootstrap (This Session)

**Goal**: Zero-knowledge primal startup

```rust
// ❌ BEFORE: Hardcoded
let songbird = SongbirdClient::new("http://localhost:3000");
let toadstool = ToadStoolClient::new("http://localhost:8080");

// ✅ AFTER: Discovered
let discovery_endpoint = env::var("DISCOVERY_ENDPOINT")
    .or_else(|_| discover_via_mdns("universal-adapter"))
    .or_else(|_| discover_via_broadcast())?;

let universal_adapter = UniversalAdapter::connect(&discovery_endpoint).await?;

// Query by capability, not by name
let compute_services = universal_adapter.query_capability("compute").await?;
let ai_services = universal_adapter.query_capability("ai").await?;
```

### Phase 2: Vendor Abstraction (Next)

**Goal**: Runtime agnostic

```rust
// ❌ BEFORE: Vendor-specific
match runtime {
    "kubernetes" => deploy_to_k8s(),
    "docker" => deploy_to_docker(),
}

// ✅ AFTER: Agnostic adapter
let runtime = RuntimeAdapter::detect().await?;
runtime.deploy(workload).await?;

// Runtime implements:
trait RuntimeAdapter {
    async fn deploy(&self, workload: &Workload) -> Result<Deployment>;
    async fn scale(&self, id: &str, replicas: u32) -> Result<()>;
    // No vendor names in API
}
```

### Phase 3: Configuration Evolution (Next)

**Goal**: No numeric hardcoding

```rust
// ❌ BEFORE: Hardcoded numbers
timeout: Duration::from_secs(30)
port: 8080
max_connections: 100

// ✅ AFTER: Discovered/configured
timeout: config.get_timeout("service_discovery")
    .unwrap_or_default() // Safe fallback
port: config.get_port("http")
    .or_else(|| discover_available_port())
    .unwrap_or(0) // OS assigns
```

---

## 📐 Design Patterns

### 1. Infant Discovery Pattern

```rust
/// Primal startup with zero knowledge
struct PrimalBootstrap {
    /// Only thing we know: who we are
    identity: PrimalIdentity,
}

impl PrimalBootstrap {
    async fn discover_world(&self) -> Result<EcosystemView> {
        // 1. Find universal adapter
        let adapter = self.find_universal_adapter().await?;
        
        // 2. Register ourselves
        adapter.register(self.identity.clone()).await?;
        
        // 3. Discover capabilities we need
        let ecosystem = adapter.discover_ecosystem().await?;
        
        Ok(ecosystem)
    }
    
    async fn find_universal_adapter(&self) -> Result<UniversalAdapter> {
        // Try multiple discovery methods
        self.try_environment()
            .or_else(|| self.try_mdns())
            .or_else(|| self.try_broadcast())
            .or_else(|| self.try_well_known())
            .ok_or_else(|| anyhow!("No universal adapter found"))
    }
}
```

### 2. Capability Query Pattern

```rust
// ❌ NEVER: Query by primal name
adapter.find_primal("toadstool")

// ✅ ALWAYS: Query by capability
let compute = adapter.query_capability("compute").await?;
let storage = adapter.query_capability("storage").await?;

// With constraints (no primal names)
let nearby_compute = adapter.query_capability("compute")
    .with_location(lat, lon, radius_km)
    .with_version(">=2.0")
    .execute().await?;
```

### 3. Vendor Agnostic Adapter Pattern

```rust
/// Runtime adapter (k8s, docker, systemd, etc.)
trait RuntimeAdapter: Send + Sync {
    /// Deploy a workload (vendor agnostic)
    async fn deploy(&self, spec: &WorkloadSpec) -> Result<DeploymentHandle>;
    
    /// Scale a deployment
    async fn scale(&self, handle: &DeploymentHandle, replicas: u32) -> Result<()>;
    
    /// Get metrics
    async fn metrics(&self, handle: &DeploymentHandle) -> Result<ResourceMetrics>;
}

// Implementation detection at runtime
impl RuntimeAdapter {
    async fn detect() -> Result<Box<dyn RuntimeAdapter>> {
        if can_detect_kubernetes().await {
            Ok(Box::new(KubernetesAdapter::new().await?))
        } else if can_detect_docker().await {
            Ok(Box::new(DockerAdapter::new().await?))
        } else if can_detect_systemd().await {
            Ok(Box::new(SystemdAdapter::new().await?))
        } else {
            Ok(Box::new(ProcessAdapter::new()))
        }
    }
}
```

---

## 🎯 Specific Hardcoding to Remove

### Primal Names (264 instances)

**Constants to Remove**:
```rust
// ❌ Remove all of these
const SONGBIRD: &str = "songbird";
const TOADSTOOL: &str = "toadstool";
const SQUIRREL: &str = "squirrel";
const NESTGATE: &str = "nestgate";
const BEARDOG: &str = "beardog";
```

**Replace with**:
```rust
// ✅ Capability-based queries
query_capability("discovery")
query_capability("compute")
query_capability("ai")
query_capability("storage")
query_capability("security")
```

### Vendor Names (55 instances)

**Patterns to Evolve**:
```rust
// ❌ Hardcoded vendors
RuntimeType::Kubernetes
StorageBackend::Consul
ContainerRuntime::Docker

// ✅ Detected at runtime
RuntimeAdapter::detect()
StorageAdapter::detect()
ContainerAdapter::detect()
```

### Ports & URLs (28 instances)

**Hardcoding to Remove**:
```rust
// ❌ Remove
"http://localhost:3000"
"http://localhost:8080"
":9000"

// ✅ Replace with
env::var("DISCOVERY_ENDPOINT")
env::var("SERVICE_ENDPOINT")
discover_endpoint().await
```

---

## 📊 Impact Analysis

### Benefits

1. **True Portability**
   - Deploy anywhere (k8s, docker, systemd, bare metal)
   - No vendor lock-in
   - Runtime detection

2. **Scalability**
   - No 2^n primal interconnections
   - n → 1 → n through universal adapter
   - Add primals without code changes

3. **Security**
   - No exposed ports in code
   - Dynamic discovery reduces attack surface
   - Capability-based access

4. **Sovereignty**
   - No vendor dependencies in core
   - User chooses runtime
   - True data ownership

### Risks & Mitigation

1. **Discovery Failure**
   - **Risk**: Can't find universal adapter
   - **Mitigation**: Multiple discovery methods (env, mDNS, broadcast)

2. **Configuration Complexity**
   - **Risk**: More runtime config needed
   - **Mitigation**: Sane defaults, auto-detection

3. **Testing**
   - **Risk**: Harder to test without hardcoding
   - **Mitigation**: Mock discovery adapter for tests

---

## 🔄 Migration Path

### Step 1: Audit Complete ✅
- [x] Find all hardcoding
- [x] Categorize by type
- [x] Prioritize changes

### Step 2: Create Abstractions (Now)
- [ ] `UniversalAdapter` trait
- [ ] `RuntimeAdapter` trait
- [ ] `DiscoveryMethod` enum
- [ ] Bootstrap helpers

### Step 3: Replace Discovery
- [ ] Remove primal name hardcoding
- [ ] Implement capability queries
- [ ] Add environment variable support
- [ ] Add mDNS discovery

### Step 4: Abstract Vendors
- [ ] Create runtime adapters
- [ ] Remove k8s/docker hardcoding
- [ ] Implement detection logic

### Step 5: Dynamic Configuration
- [ ] Remove port hardcoding
- [ ] Add discovery fallbacks
- [ ] Update documentation

### Step 6: Testing
- [ ] Test with mock discovery
- [ ] Test with real primals
- [ ] Test vendor detection

---

## 📖 Zero-Knowledge Startup Example

```rust
/// A primal starting with zero knowledge
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Know thyself (only thing we know)
    let identity = PrimalIdentity {
        name: env::var("PRIMAL_NAME")?,
        capabilities: vec!["compute".to_string()],
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    
    // 2. Find the universal adapter (discovery)
    let adapter = find_universal_adapter().await?;
    
    // 3. Register ourselves
    adapter.register(&identity).await?;
    
    // 4. Discover world through adapter
    let ecosystem = adapter.discover_ecosystem().await?;
    
    // 5. Find services we need (by capability, not name!)
    let storage_services = adapter
        .query_capability("storage")
        .execute()
        .await?;
    
    // 6. Start serving
    serve(identity, adapter, ecosystem).await
}

async fn find_universal_adapter() -> Result<UniversalAdapter> {
    // Try multiple methods (no hardcoding!)
    if let Ok(endpoint) = env::var("DISCOVERY_ENDPOINT") {
        return UniversalAdapter::connect(&endpoint).await;
    }
    
    if let Ok(adapter) = discover_via_mdns("universal-adapter").await {
        return Ok(adapter);
    }
    
    if let Ok(adapter) = discover_via_broadcast().await {
        return Ok(adapter);
    }
    
    Err(anyhow!("No universal adapter found. Set DISCOVERY_ENDPOINT or ensure Songbird is running."))
}
```

---

## 🎯 Success Criteria

### Must Have
- [ ] Zero primal name hardcoding in discovery code
- [ ] All ports from environment or discovery
- [ ] No vendor names in core types
- [ ] Capability-based queries work
- [ ] Multiple discovery methods implemented

### Should Have
- [ ] Runtime adapter pattern working
- [ ] Vendor detection automatic
- [ ] Graceful fallbacks
- [ ] Clear error messages

### Nice to Have
- [ ] mDNS discovery working
- [ ] Broadcast discovery working
- [ ] Auto-configuration
- [ ] Discovery caching

---

## 📝 Next Actions

### Immediate
1. Create `UniversalAdapter` trait
2. Remove hardcoded endpoints from clients
3. Add environment variable support
4. Create discovery bootstrap

### This Session
1. Remove top 20 hardcoding instances
2. Create vendor agnostic adapters
3. Implement capability queries
4. Update documentation

### This Week
1. Complete discovery abstraction
2. Remove all primal name hardcoding
3. Test with real discovery
4. Update examples

---

**Status**: 🔍 Audit Complete, Evolution Ready  
**Priority**: HIGH  
**Complexity**: MEDIUM-HIGH  
**Impact**: VERY HIGH (True Universal Discovery)

---

*"Born knowing nothing. Discover everything. Connect to all."*


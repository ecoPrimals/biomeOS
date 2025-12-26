# BiomeOS Responsibilities - Definitive Guide

**Status**: ✅ **ACTIVE** - This document defines BiomeOS scope  
**Date**: December 24, 2025  
**Purpose**: Clear boundaries between BiomeOS and Primal responsibilities

---

## 🎯 Core Principle

**BiomeOS is a COMPOSITION SUBSTRATE, not a REIMPLEMENTATION.**

BiomeOS coordinates primals to create functioning biomes. It does NOT reimplement what primals already provide.

---

## ✅ BiomeOS SHOULD Do (Core Responsibilities)

### 1. **Manifest Parsing & Validation**
**Why**: biome.yaml is BiomeOS's domain language

```rust
// ✅ CORRECT - BiomeOS parses its own manifest format
let manifest = BiomeManifest::from_file("biome.yaml")?;
manifest.validate()?;
```

**Crates**: `biomeos-manifest`, `biomeos-types`

### 2. **Capability Matching & Resolution**
**Why**: Matching manifest requirements to available primals is composition logic

```rust
// ✅ CORRECT - BiomeOS matches capabilities
let required = manifest.get_required_capabilities();
let available = songbird.discover_all().await?;
let plan = capability_matcher.resolve(required, available)?;
```

**Crates**: `biomeos-core` (capability matcher)

### 3. **Multi-Primal Workflow Orchestration**
**Why**: Coordinating workflows across primals is BiomeOS's unique value

```rust
// ✅ CORRECT - BiomeOS orchestrates multi-primal workflows
async fn deploy_ai_workload(&self, manifest: &Manifest) -> Result<()> {
    // 1. Provision storage (NestGate)
    let volume = nestgate.provision_volume(&manifest.storage).await?;
    
    // 2. Deploy compute (ToadStool)
    let service = toadstool.deploy(&manifest.compute, &volume).await?;
    
    // 3. Register with discovery (Songbird)
    songbird.register_service(&service).await?;
    
    // 4. Setup AI routing (Squirrel)
    squirrel.register_endpoint(&service).await?;
    
    Ok(())
}
```

**Crates**: `biomeos-core` (workflow orchestrator)

### 4. **Biome Lifecycle Management**
**Why**: Biomes are BiomeOS's unit of deployment

```rust
// ✅ CORRECT - BiomeOS manages biome lifecycle
impl BiomeLifecycle {
    async fn start_biome(&self, biome_id: &str) -> Result<()> {
        // Load manifest
        let manifest = self.load_manifest(biome_id)?;
        
        // Start primals in dependency order
        for primal in manifest.primals_in_order() {
            self.start_primal(&primal).await?;
        }
        
        // Wait for health
        self.wait_for_healthy(biome_id).await?;
        
        Ok(())
    }
}
```

**Crates**: `biomeos-core` (lifecycle manager)

### 5. **Chimera Composition**
**Why**: Chimeras are BiomeOS-specific concept (multi-primal hybrids)

```rust
// ✅ CORRECT - BiomeOS composes chimeras
let chimera = ChimeraBuilder::new()
    .add_primal("beardog", &["security", "btsp"])
    .add_primal("songbird", &["discovery", "mesh"])
    .bind("beardog.security" -> "songbird.auth")
    .build()?;
```

**Crates**: `biomeos-chimera`

### 6. **Niche Deployment**
**Why**: Niches (biome environments) are BiomeOS concept

```rust
// ✅ CORRECT - BiomeOS deploys niches
let niche = NicheDeployment::from_template("gaming-tournament")?;
niche.customize(|config| {
    config.set_max_players(64);
    config.enable_anti_cheat();
});
niche.deploy().await?;
```

**Crates**: `biomeos-niche`

### 7. **Sovereignty & Policy Enforcement**
**Why**: BiomeOS enforces cross-primal policies

```rust
// ✅ CORRECT - BiomeOS enforces sovereignty policies
impl SovereigntyGuardian {
    fn evaluate_data_access(&self, service: &str, data: &str, purpose: &str) -> Result<()> {
        if self.policies.block_tracking && purpose.contains("track") {
            return Err(BiomeError::SovereigntyViolation(
                "Unauthorized tracking detected"
            ));
        }
        Ok(())
    }
}
```

**Crates**: `biomeos-core` (sovereignty guardian)

### 8. **Configuration Management**
**Why**: Biome-level configuration is BiomeOS domain

```rust
// ✅ CORRECT - BiomeOS manages biome config
let config = BiomeOSConfig::builder()
    .name("production-biome")
    .environment(Environment::Production)
    .organization_scale(OrganizationScale::Enterprise)
    .enable_feature("crypto_locks")
    .build()?;
```

**Crates**: `biomeos-types`, `biomeos-core`

---

## ❌ BiomeOS Should NOT Do (Primal Responsibilities)

### 1. **Service Discovery & Registry**
**Why**: Songbird provides this (production-ready)

```rust
// ❌ WRONG - BiomeOS implementing discovery
impl BiomeOS {
    async fn discover_services(&self) -> Vec<Service> {
        // DON'T DO THIS
    }
}

// ✅ RIGHT - Delegate to Songbird
let songbird = self.discover_primal("discovery").await?;
let services = songbird.query_all_services().await?;
```

**Delegate to**: Songbird

### 2. **Resource Metrics & Monitoring**
**Why**: ToadStool provides real metrics, Songbird provides health monitoring

```rust
// ❌ WRONG - BiomeOS mocking metrics
result.insert("cpu_percent", json!(15.5)); // FAKE DATA!

// ✅ RIGHT - Query real metrics
let toadstool = self.discover_primal("compute").await?;
let metrics = toadstool.get_resource_usage(service_id).await?;
```

**Delegate to**: ToadStool (metrics), Songbird (health)

### 3. **AI Optimization & Analysis**
**Why**: Squirrel provides this (Grade A++)

```rust
// ❌ WRONG - BiomeOS mocking AI analysis
result.insert("performance_score", json!(85)); // FAKE!

// ✅ RIGHT - Delegate to Squirrel
let squirrel = self.discover_primal("ai").await?;
let analysis = squirrel.optimize_system(metrics).await?;
```

**Delegate to**: Squirrel

### 4. **Geolocation & Geographic Routing**
**Why**: Songbird can route by location via service metadata

```rust
// ❌ WRONG - BiomeOS implementing geolocation
pub async fn discover_by_location(...) -> Result<Vec<Service>> {
    Ok(vec![]) // MOCK!
}

// ✅ RIGHT - Query Songbird with location filter
let songbird = self.discover_primal("discovery").await?;
let services = songbird.query_services_with_metadata(
    "compute",
    |meta| meta.location.distance_to(lat, lon) < radius_km
).await?;
```

**Delegate to**: Songbird

### 5. **Workload Execution**
**Why**: ToadStool handles ALL execution (multi-runtime, GPU, etc.)

```rust
// ❌ WRONG - BiomeOS executing workloads
impl BiomeOS {
    async fn execute_workload(&self, workload: &Workload) -> Result<()> {
        // DON'T DO THIS
    }
}

// ✅ RIGHT - Delegate to ToadStool
let toadstool = self.discover_primal("compute").await?;
toadstool.deploy_workload(workload).await?;
```

**Delegate to**: ToadStool

### 6. **Storage Operations**
**Why**: NestGate provides distributed storage

```rust
// ❌ WRONG - BiomeOS implementing storage
impl BiomeOS {
    async fn store_data(&self, key: &str, value: &[u8]) -> Result<()> {
        // DON'T DO THIS
    }
}

// ✅ RIGHT - Delegate to NestGate
let nestgate = self.discover_primal("storage").await?;
nestgate.store_data(key, value).await?;
```

**Delegate to**: NestGate

### 7. **Cryptography & Security Operations**
**Why**: BearDog provides world-class crypto (Grade A+)

```rust
// ❌ WRONG - BiomeOS implementing crypto
impl BiomeOS {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // DON'T DO THIS
    }
}

// ✅ RIGHT - Delegate to BearDog
let beardog = self.discover_primal("security").await?;
let encrypted = beardog.encrypt(data, recipient).await?;
```

**Delegate to**: BearDog

### 8. **Load Balancing & Traffic Routing**
**Why**: Songbird provides intelligent routing

```rust
// ❌ WRONG - BiomeOS implementing load balancing
impl BiomeOS {
    async fn route_request(&self, request: &Request) -> Result<Endpoint> {
        // DON'T DO THIS
    }
}

// ✅ RIGHT - Delegate to Songbird
let songbird = self.discover_primal("discovery").await?;
let endpoint = songbird.route_request(request).await?;
```

**Delegate to**: Songbird

---

## 🔍 Decision Matrix

When adding new functionality, ask:

### Question 1: Is it specific to biomes?
- **YES** → Probably BiomeOS responsibility
- **NO** → Probably primal responsibility

### Question 2: Does a primal already provide this?
- **YES** → **Delegate to primal**
- **NO** → Consider if BiomeOS should add it

### Question 3: Is it composition/coordination logic?
- **YES** → BiomeOS responsibility
- **NO** → Probably primal responsibility

### Question 4: Does it require deep domain knowledge?
- **YES** → Primal responsibility (they're specialists)
- **NO** → Could be BiomeOS

### Examples

| Feature | Q1: Biome-specific? | Q2: Primal provides? | Q3: Composition? | Q4: Domain expert? | **Decision** |
|---------|--------------------|--------------------|-----------------|-------------------|--------------|
| Parse biome.yaml | YES | NO | NO | NO | **BiomeOS** |
| Match capabilities | YES | NO | YES | NO | **BiomeOS** |
| Multi-primal workflow | YES | NO | YES | NO | **BiomeOS** |
| Service discovery | NO | YES (Songbird) | NO | YES | **Delegate** |
| Resource metrics | NO | YES (ToadStool) | NO | YES | **Delegate** |
| AI optimization | NO | YES (Squirrel) | NO | YES | **Delegate** |
| Geolocation | NO | YES (Songbird) | NO | YES | **Delegate** |
| Crypto operations | NO | YES (BearDog) | NO | YES | **Delegate** |
| Storage operations | NO | YES (NestGate) | NO | YES | **Delegate** |
| Chimera building | YES | NO | YES | NO | **BiomeOS** |
| Niche deployment | YES | NO | YES | NO | **BiomeOS** |

---

## 📦 Crate Responsibilities

### biomeos-types
✅ Core type definitions for biomes, manifests, config  
❌ NOT primal-specific types (those go in primal crates)

### biomeos-core
✅ Capability matching, workflow orchestration, lifecycle management  
❌ NOT service discovery, metrics, AI, crypto (delegate)

### biomeos-manifest
✅ Parse and validate biome.yaml  
❌ NOT execute manifests (ToadStool does that)

### biomeos-chimera
✅ Define and build multi-primal chimeras  
❌ NOT execute chimeras (primals do that)

### biomeos-niche
✅ Deploy and manage niche environments  
❌ NOT implement niche services (primals do that)

### biomeos-cli
✅ User interface for biome management  
❌ NOT reimplement primal CLIs

### biomeos-primal-sdk
✅ Define primal capabilities and SDK interfaces  
❌ NOT implement primals (they implement themselves)

### biomeos-federation
✅ Federate biomes across networks  
❌ NOT implement federation (Songbird does that)

### biomeos-system
✅ System integration for BiomeOS  
❌ NOT system metrics (ToadStool provides those)

---

## 🎓 Guiding Principles

### 1. **Delegate by Default**
When in doubt, delegate to a primal. BiomeOS should be thin orchestration layer.

### 2. **No Mocks in Production**
Test mocks are fine. Production mocks mean missing delegation.

### 3. **No Hardcoded Endpoints**
Use discovery or environment variables. Never hardcode primal locations.

### 4. **Fail Fast on Missing Primals**
If a primal is required and unavailable, return error immediately. Don't mock or fake it.

### 5. **Composition Over Implementation**
BiomeOS composes primals, it doesn't replace them.

### 6. **Self-Knowledge Only**
BiomeOS knows its own APIs and capabilities. It discovers primals at runtime.

### 7. **Clear Error Messages**
When delegation fails, tell user which primal is needed and how to provide it.

---

## 🚫 Anti-Patterns to Avoid

### Anti-Pattern 1: Mock Data in Production
```rust
// ❌ BAD
result.insert("cpu_percent", json!(15.5)); // Fake data!
```

### Anti-Pattern 2: Hardcoded Endpoints
```rust
// ❌ BAD
const TOADSTOOL_URL: &str = "http://localhost:8080";
```

### Anti-Pattern 3: Reimplementing Primal Logic
```rust
// ❌ BAD
impl BiomeOS {
    async fn discover_services(&self) -> Vec<Service> {
        // Songbird already does this!
    }
}
```

### Anti-Pattern 4: Silent Fallbacks
```rust
// ❌ BAD
let endpoint = env::var("TOADSTOOL_ENDPOINT")
    .unwrap_or("http://localhost:8080".to_string()); // Silent failure!
```

### Anti-Pattern 5: Tight Coupling
```rust
// ❌ BAD - BiomeOS shouldn't import primal internals
use toadstool::internal::WorkloadExecutor;
```

---

## ✅ Correct Patterns

### Pattern 1: Delegate and Return Real Data
```rust
// ✅ GOOD
let toadstool = self.discover_primal("compute").await?;
let metrics = toadstool.get_resource_usage(service_id).await?;
```

### Pattern 2: Environment Variables with Clear Errors
```rust
// ✅ GOOD
let endpoint = env::var("TOADSTOOL_ENDPOINT")
    .map_err(|_| anyhow!(
        "TOADSTOOL_ENDPOINT not set. Set environment variable or use discovery."
    ))?;
```

### Pattern 3: Composition Logic Only
```rust
// ✅ GOOD
impl BiomeOS {
    async fn orchestrate_deployment(&self, manifest: &Manifest) -> Result<()> {
        // This is composition - coordinating multiple primals
        let storage = self.provision_storage(manifest).await?;
        let compute = self.deploy_compute(manifest, &storage).await?;
        self.register_services(&compute).await?;
        Ok(())
    }
}
```

### Pattern 4: Fail Fast with Helpful Messages
```rust
// ✅ GOOD
let songbird = self.discover_primal("discovery").await
    .map_err(|e| anyhow!(
        "Songbird discovery service required but not found. \
         Ensure Songbird is running or set SONGBIRD_ENDPOINT. \
         Original error: {}", e
    ))?;
```

---

## 📊 Summary

| Category | BiomeOS | Primals |
|----------|---------|---------|
| **Manifest Parsing** | ✅ YES | ❌ NO |
| **Capability Matching** | ✅ YES | ❌ NO |
| **Workflow Orchestration** | ✅ YES | ❌ NO |
| **Lifecycle Management** | ✅ YES | ❌ NO |
| **Chimera Composition** | ✅ YES | ❌ NO |
| **Niche Deployment** | ✅ YES | ❌ NO |
| **Sovereignty Enforcement** | ✅ YES | ❌ NO |
| **Configuration Management** | ✅ YES | ❌ NO |
| **Service Discovery** | ❌ NO | ✅ Songbird |
| **Resource Metrics** | ❌ NO | ✅ ToadStool |
| **AI Optimization** | ❌ NO | ✅ Squirrel |
| **Geolocation** | ❌ NO | ✅ Songbird |
| **Workload Execution** | ❌ NO | ✅ ToadStool |
| **Storage Operations** | ❌ NO | ✅ NestGate |
| **Cryptography** | ❌ NO | ✅ BearDog |
| **Load Balancing** | ❌ NO | ✅ Songbird |

---

## 🎯 Key Takeaway

**BiomeOS is a thin orchestration layer that enables primals to work together as a cohesive biome.**

BiomeOS does NOT:
- Reimplement what primals provide
- Mock primal functionality
- Hardcode primal locations
- Execute workloads directly
- Store data directly
- Perform cryptography directly

BiomeOS DOES:
- Parse biome.yaml manifests
- Match capabilities to primals
- Orchestrate multi-primal workflows
- Manage biome lifecycles
- Compose chimeras
- Deploy niches
- Enforce sovereignty policies

---

**Status**: ✅ Active Guide  
**Version**: 1.0  
**Date**: December 24, 2025

*"BiomeOS enables composition. Primals provide capabilities. Together, they create ecosystems."*


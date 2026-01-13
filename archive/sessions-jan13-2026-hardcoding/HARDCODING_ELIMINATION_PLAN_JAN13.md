# 🧬 Hardcoding Elimination - Infant Bootstrapping Evolution

**Date**: January 13, 2026  
**Philosophy**: "Each primal only knows itself, discovers like an infant"  
**Goal**: Zero hardcoding - No primal names, no vendors, no magic numbers

---

## 🎯 Current State Analysis

### **Hardcoding Violations Found**

| Category | Matches | Files | Severity |
|----------|---------|-------|----------|
| **Primal Names** | 1,693 | 138 | 🔴 CRITICAL |
| **Family IDs (nat0)** | 157 | 44 | 🔴 CRITICAL |
| **Ports/Localhost** | 118 | 38 | 🟡 HIGH |
| **Vendor Names** | 66 | 19 | 🟡 HIGH |

**Total**: ~2,034 hardcoding instances

---

## 🚨 CRITICAL VIOLATIONS

### **1. Primal Name Inference** ❌

**Location**: `crates/biomeos-ui/src/petaltongue_bridge.rs`

**Current (WRONG)**:
```rust
fn extract_primal_name(&self, socket_name: &str) -> String {
    if socket_name.contains("songbird") {
        "Songbird".to_string()
    } else if socket_name.contains("beardog") {
        "BearDog".to_string()
    } else if socket_name.contains("toadstool") {
        "ToadStool".to_string()
    } else if socket_name.contains("nestgate") {
        "NestGate".to_string()
    } else if socket_name.contains("squirrel") {
        "Squirrel".to_string()
    } else {
        "Unknown".to_string()
    }
}
```

**Problem**: Hardcoded knowledge of all primals!

**Evolution (RIGHT)**:
```rust
async fn query_primal_identity(&self, socket_path: &Path) -> Result<PrimalIdentity> {
    // Connect to socket - NO assumptions!
    let stream = UnixStream::connect(socket_path).await?;
    
    // Ask primal: "Who are you?" - Infant curiosity!
    let response = self.json_rpc_call(&stream, "biomeos.identity", None).await?;
    
    Ok(PrimalIdentity {
        name: response["name"].as_str()?.to_string(),
        capabilities: response["capabilities"].as_array()?.clone(),
        version: response["version"].as_str()?.to_string(),
    })
}
```

---

### **2. "nat0" Hardcoding** ❌

**Violations**: 157 instances across 44 files

**Examples**:
```rust
// BAD: Magic string "nat0"
let family = "nat0";
let client = BearDogClient::discover("nat0").await?;
```

**Problem**: Assumes family name!

**Evolution (RIGHT)**:
```rust
// Use environment or discovery
let family = FamilyId::from_env()
    .or_else(|| FamilyId::discover_local())
    .unwrap_or_else(|| FamilyId::generate());

// Or in tests: explicit injection
let family = FamilyId::new_for_test();
```

---

### **3. Port Number Hardcoding** ❌

**Examples**:
```rust
// BAD: Magic ports
"http://localhost:8000"
"http://localhost:8080"
":9000"
```

**Problem**: Assumes port availability!

**Evolution (RIGHT)**:
```rust
// Discover via environment or socket
let endpoint = PrimalEndpoint::discover_by_capability(
    PrimalCapability::Security
)?;

// OR: Use Unix sockets (no ports!)
let socket = SystemPaths::new()?.primal_socket("beardog-main");
```

---

### **4. Vendor Hardcoding** ❌

**Found**: k8s, kubernetes, consul, etcd, docker, podman (66 instances)

**Examples**:
```rust
// BAD: Assumes Kubernetes
if runtime == "kubernetes" { ... }

// BAD: Docker-specific
"docker.io/image"
```

**Problem**: Not infrastructure-agnostic!

**Evolution (RIGHT)**:
```rust
// Query runtime for capabilities
if runtime.has_capability(RuntimeCapability::ContainerOrchestration) {
    // Works with k8s, nomad, docker swarm, etc.
}

// Registry-agnostic
let image = format!("{}/image", self.container_registry());
```

---

## 🧬 TRUE PRIMAL Principles

### **Infant Bootstrapping Model**

```
┌─────────────────────────────────────┐
│  Primal Birth (Zero Knowledge)     │
└─────────────┬───────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│  1. Self-Knowledge                  │
│     - Who am I?                     │
│     - What can I do?                │
│     - Where am I?                   │
└─────────────┬───────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│  2. Environment Discovery           │
│     - What family am I in?          │
│     - Where are my sockets?         │
│     - What resources do I have?     │
└─────────────┬───────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│  3. Peer Discovery (via Songbird)   │
│     - Who else is here?             │
│     - What can they do?             │
│     - How do I talk to them?        │
└─────────────┬───────────────────────┘
              │
              ▼
┌─────────────────────────────────────┐
│  4. Capability Composition          │
│     - I need "security"             │
│     - Who provides it?              │
│     - Connect via discovered socket │
└─────────────────────────────────────┘
```

---

## 📋 Evolution Strategy

### **Phase 1: Critical Path (3-4 hours)**

**Priority**: Eliminate critical violations

1. **Remove primal name inference**
   - `petaltongue_bridge.rs::extract_primal_name`
   - `discovery.rs` capability inference
   - Any `match name` → capability mappings

2. **Eliminate "nat0" hardcoding**
   - Replace with `FamilyId::from_env()`
   - Add test helper `FamilyId::new_for_test()`
   - Update docs/examples

3. **Port number cleanup**
   - Move to environment variables
   - Prefer Unix sockets
   - Document fallback behavior

---

### **Phase 2: Vendor Agnosticism (2-3 hours)**

**Goal**: Work with any infrastructure

1. **Abstract container runtime**
   - `RuntimeCapability` enum
   - Query not assume

2. **Abstract service mesh**
   - No k8s/consul assumptions
   - Capability-based

3. **Abstract storage**
   - No etcd/redis hardcoding
   - Plugin architecture

---

### **Phase 3: Self-Knowledge API (2-3 hours)**

**Goal**: Standard primal introspection

**Add to all primals**:
```rust
// Standard JSON-RPC methods
"biomeos.identity"    → Who am I?
"biomeos.capabilities" → What can I do?
"biomeos.health"      → How am I?
"biomeos.peers"       → Who do I know?
```

**Implementation**:
```rust
#[async_trait]
trait BiomeOSPrimal {
    async fn identity(&self) -> PrimalIdentity;
    async fn capabilities(&self) -> Vec<PrimalCapability>;
    async fn health(&self) -> HealthStatus;
    async fn peers(&self) -> Vec<PeerInfo>;
}
```

---

### **Phase 4: Discovery Protocol (3-4 hours)**

**Goal**: Songbird as universal adapter

**Flow**:
```rust
// 1. Primal announces itself to Songbird
songbird.announce(PrimalAnnouncement {
    identity: self.identity(),
    capabilities: self.capabilities(),
    socket_path: self.socket_path(),
});

// 2. Other primals discover via capability
let security_primal = songbird
    .discover_by_capability(PrimalCapability::Encryption)
    .await?;

// 3. Connect via discovered socket
let client = PrimalTransport::connect(&security_primal.socket_path).await?;
```

**No hardcoding anywhere!**

---

## 🎯 Quick Wins (Start Here)

### **Win 1: Environment-Based Family ID**

**File**: `crates/biomeos-types/src/identifiers.rs`

**Add**:
```rust
impl FamilyId {
    /// Get family ID from environment
    pub fn from_env() -> Option<Self> {
        std::env::var("BIOMEOS_FAMILY_ID")
            .ok()
            .map(Self::new)
    }
    
    /// Discover local family ID (check sockets, config, etc.)
    pub fn discover_local() -> Option<Self> {
        // Check for existing family in ~/.config/biomeos/family.txt
        let paths = SystemPaths::new().ok()?;
        let family_file = paths.config_dir().join("family.txt");
        std::fs::read_to_string(family_file)
            .ok()
            .map(|s| Self::new(s.trim()))
    }
    
    /// Generate new random family ID
    pub fn generate() -> Self {
        use names::Generator;
        let mut generator = Generator::default();
        Self::new(generator.next().unwrap())
    }
    
    /// For tests only
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self::new("test-family")
    }
}
```

---

### **Win 2: BiomeOS Standard API**

**File**: `crates/biomeos-types/src/primal/standard_api.rs` (new)

```rust
/// Standard BiomeOS primal API
/// All primals MUST implement these methods
#[async_trait]
pub trait BiomeOSStandardAPI {
    /// Get primal identity (who am I?)
    async fn biomeos_identity(&self) -> Result<PrimalIdentity>;
    
    /// Get capabilities (what can I do?)
    async fn biomeos_capabilities(&self) -> Result<Vec<PrimalCapability>>;
    
    /// Health check (how am I?)
    async fn biomeos_health(&self) -> Result<HealthStatus>;
    
    /// Get known peers (who do I know?)
    async fn biomeos_peers(&self) -> Result<Vec<PeerInfo>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalIdentity {
    /// Primal's self-reported name
    pub name: String,
    
    /// Primal's version
    pub version: String,
    
    /// Capabilities this primal provides
    pub capabilities: Vec<PrimalCapability>,
    
    /// Optional description
    pub description: Option<String>,
}
```

---

### **Win 3: Discovery Client**

**File**: `crates/biomeos-core/src/discovery/client.rs`

```rust
/// Discovery client - queries Songbird for primals
pub struct DiscoveryClient {
    songbird_socket: PathBuf,
}

impl DiscoveryClient {
    /// Discover primals by capability
    pub async fn find_by_capability(
        &self,
        capability: PrimalCapability
    ) -> Result<Vec<PrimalEndpoint>> {
        let transport = PrimalTransport::connect(&self.songbird_socket).await?;
        
        let response = transport.call(
            "discovery.find_by_capability",
            Some(json!({ "capability": capability }))
        ).await?;
        
        Ok(serde_json::from_value(response)?)
    }
    
    /// Announce self to discovery
    pub async fn announce(&self, identity: PrimalIdentity) -> Result<()> {
        let transport = PrimalTransport::connect(&self.songbird_socket).await?;
        
        transport.call(
            "discovery.announce",
            Some(json!({ "identity": identity }))
        ).await?;
        
        Ok(())
    }
}
```

---

## 📊 Success Metrics

### **Before**:
- ❌ 2,034 hardcoding instances
- ❌ Primals know each other by name
- ❌ Magic numbers everywhere
- ❌ Vendor lock-in

### **After**:
- ✅ <100 hardcoding instances (tests/docs only)
- ✅ Capability-based discovery
- ✅ Environment-driven configuration
- ✅ Vendor-agnostic

---

## 🚀 Execution Order

1. **Implement FamilyId helpers** (30 min)
2. **Create BiomeOS Standard API** (1 hour)
3. **Fix petaltongue_bridge** (1 hour)
4. **Update discovery.rs** (1 hour)
5. **Replace "nat0" in tests** (1 hour)
6. **Port cleanup** (1 hour)
7. **Vendor abstraction** (2 hours)
8. **Test & validate** (1 hour)

**Total**: ~8-10 hours

---

## 🎯 Next Steps

1. Start with **Quick Win 1** (FamilyId helpers)
2. Implement **Quick Win 2** (Standard API)
3. Fix **Critical Violation 1** (primal name inference)
4. Systematic cleanup of hardcoding

---

**Philosophy**: 

> "Like an infant, each primal is born knowing only itself.  
> Through discovery, it learns of others.  
> Through composition, it becomes part of something greater.  
> But it never hardcodes - it always asks, always discovers."

---

**Status**: READY TO EVOLVE  
**Risk**: MEDIUM (large refactor, but well-tested)  
**Benefit**: HIGH (TRUE PRIMAL compliance)

🧬 **"Zero knowledge at birth, infinite discovery through life"** 🌱


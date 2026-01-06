# 🌟 Capability-Based Evolution - Avoiding N^2 Connections

**Date**: January 4, 2026  
**Principle**: Primals have ONLY self-knowledge, connect via capabilities  
**Goal**: O(N) scaling instead of N^2 connections

---

## 🎯 Core Problem: N^2 Connections

### Traditional Approach (WRONG)

```
BearDog → knows about Songbird (hardcoded)
BearDog → knows about ToadStool (hardcoded)
BearDog → knows about Gorilla (hardcoded)

Songbird → knows about BearDog (hardcoded)
Songbird → knows about ToadStool (hardcoded)
Songbird → knows about Gorilla (hardcoded)

ToadStool → knows about BearDog (hardcoded)
ToadStool → knows about Songbird (hardcoded)
ToadStool → knows about Gorilla (hardcoded)

Result: N^2 connections = 3*3 = 9 connections for 3 primals!
        With 10 primals = 100 connections!
```

**Problems**:
- ❌ Each primal hardcoded to every other primal
- ❌ Adding new primal requires updating ALL primals
- ❌ Can't evolve primals independently
- ❌ Can't have multiple encryption providers
- ❌ Can't have multiple discovery systems

---

## ✅ Capability-Based Approach (CORRECT)

### Self-Knowledge Only

```
BearDog:
  "I provide: Security, Encryption, Trust"
  "I require: Discovery"  ← Generic capability!

Songbird:
  "I provide: Discovery, ConnectionManagement"
  "I require: Security"  ← Generic capability!

ToadStool:
  "I provide: Storage, Persistence"
  "I require: Discovery, Security"  ← Generic capabilities!

Result: Each primal declares capabilities only!
        Adding new primal = 0 changes to existing primals!
```

**Benefits**:
- ✅ Each primal only knows itself
- ✅ Connect via capability matching
- ✅ Add new primals without code changes
- ✅ Multiple providers of same capability
- ✅ Evolve independently

---

## 🔄 Capability Resolution

### Registry-Based Connection (O(N) scaling)

```
biomeOS Capability Registry:
  Discovery → [Songbird-1, Songbird-2, NewDiscovery-1]
  Security → [BearDog-1, BearDog-2, NewSecurity-1]
  Storage → [ToadStool-1, ToadStool-2]
  Compute → [Gorilla-1, Gorilla-2]

BearDog needs Discovery:
  1. Ask registry: "Who provides Discovery?"
  2. Registry returns: [Songbird-1, Songbird-2, NewDiscovery-1]
  3. BearDog connects to first available
  4. No hardcoding of "Songbird"!

Songbird needs Security:
  1. Ask registry: "Who provides Security?"
  2. Registry returns: [BearDog-1, BearDog-2, NewSecurity-1]
  3. Songbird connects to first available
  4. No hardcoding of "BearDog"!
```

**Result**: O(N) registry lookups instead of N^2 connections!

---

## 🏗️ Implementation Architecture

### Primal Self-Declaration

**BearDog** (only knows itself):
```rust
// beardog/src/main.rs
fn main() {
    let config = PrimalConfig {
        id: "beardog",
        provides: vec![
            Capability::Security,
            Capability::Encryption,
            Capability::Trust,
        ],
        requires: vec![
            Capability::Discovery,  // Generic! Not "Songbird"
        ],
    };
    
    // Connect to orchestrator (biomeOS or Songbird registry)
    let registry = connect_to_registry()?;
    registry.register(config)?;
    
    // Request connections via capability
    let discovery_provider = registry.get_provider(Capability::Discovery)?;
    // Could be Songbird, or NewDiscovery, or anything!
}
```

**Songbird** (only knows itself):
```rust
// songbird/src/main.rs
fn main() {
    let config = PrimalConfig {
        id: "songbird",
        provides: vec![
            Capability::Discovery,
            Capability::ConnectionManagement,
        ],
        requires: vec![
            Capability::Security,  // Generic! Not "BearDog"
        ],
    };
    
    // Register with orchestrator
    let registry = connect_to_registry()?;
    registry.register(config)?;
    
    // When peer discovered, request security
    on_peer_discovered(|peer| {
        let security_provider = registry.get_provider(Capability::Security)?;
        // Could be BearDog, or NewSecurity, or anything!
        security_provider.secure_connection(peer)?;
    });
}
```

**ToadStool** (future, only knows itself):
```rust
// toadstool/src/main.rs
fn main() {
    let config = PrimalConfig {
        id: "toadstool",
        provides: vec![
            Capability::Storage,
            Capability::Persistence,
        ],
        requires: vec![
            Capability::Discovery,  // Find peers to replicate to
            Capability::Security,   // Encrypt stored data
        ],
    };
    
    registry.register(config)?;
    
    // Get providers via capabilities
    let discovery = registry.get_provider(Capability::Discovery)?;
    let security = registry.get_provider(Capability::Security)?;
    // No hardcoding!
}
```

---

## 🌊 Evolution Scenarios

### Scenario 1: New Discovery System

**Add NewDiscovery** (doesn't replace Songbird):
```rust
// new-discovery/src/main.rs
fn main() {
    let config = PrimalConfig {
        id: "new-discovery",
        provides: vec![
            Capability::Discovery,  // Same capability!
        ],
        requires: vec![
            Capability::Security,
        ],
    };
    
    registry.register(config)?;
    // Now registry has: Discovery → [Songbird, NewDiscovery]
}
```

**BearDog code**: ✅ **NO CHANGES NEEDED!**
```rust
// BearDog still does:
let discovery = registry.get_provider(Capability::Discovery)?;
// Could get Songbird OR NewDiscovery!
```

### Scenario 2: New Encryption System

**Add NewSecurity** (doesn't replace BearDog):
```rust
// new-security/src/main.rs
fn main() {
    let config = PrimalConfig {
        id: "new-security",
        provides: vec![
            Capability::Security,  // Same capability!
            Capability::Encryption,
        ],
        requires: vec![
            Capability::Discovery,
        ],
    };
    
    registry.register(config)?;
    // Now registry has: Security → [BearDog, NewSecurity]
}
```

**Songbird code**: ✅ **NO CHANGES NEEDED!**
```rust
// Songbird still does:
let security = registry.get_provider(Capability::Security)?;
// Could get BearDog OR NewSecurity!
```

### Scenario 3: Songbird Separates

**Songbird splits into**:
- Songbird-Local (LAN discovery)
- Songbird-WAN (Internet discovery)
- Songbird-Geo (Geographic discovery)

```rust
// All provide Capability::Discovery
registry has: Discovery → [Songbird-Local, Songbird-WAN, Songbird-Geo]

// BearDog code: NO CHANGES!
let discovery = registry.get_provider(Capability::Discovery)?;
```

---

## 🔧 Registry Implementation

### Option 1: Songbird as Registry

**Songbird is both discovery AND registry**:
```rust
// songbird/src/registry.rs
pub struct CapabilityRegistry {
    providers: HashMap<Capability, Vec<PrimalHandle>>,
}

impl CapabilityRegistry {
    pub fn register(&mut self, config: PrimalConfig) {
        for capability in config.provides {
            self.providers.entry(capability)
                .or_insert_with(Vec::new)
                .push(config.handle);
        }
    }
    
    pub fn get_provider(&self, cap: Capability) -> Option<PrimalHandle> {
        self.providers.get(&cap)
            .and_then(|providers| providers.first())
            .cloned()
    }
}
```

**Unix Socket API**:
```json
// Primal → Songbird
{
  "method": "register",
  "params": {
    "provides": ["Security", "Encryption"],
    "requires": ["Discovery"]
  }
}

// Primal → Songbird
{
  "method": "get_provider",
  "params": {
    "capability": "Discovery"
  }
}

// Songbird → Primal
{
  "result": {
    "provider": "songbird-abc123",
    "socket": "/tmp/songbird-nat0.sock"
  }
}
```

### Option 2: biomeOS as Registry

**biomeOS tracks all primals**:
```rust
// biomeos-core/src/capability_registry.rs
pub struct GlobalCapabilityRegistry {
    primals: HashMap<PrimalId, PrimalConfig>,
    capabilities: HashMap<Capability, Vec<PrimalId>>,
}
```

**Primals connect to biomeOS registry via Unix socket**:
- `/tmp/biomeos-registry-{family}.sock`
- Primals register on startup
- Primals query for providers
- biomeOS returns connection info

---

## 🎯 Implementation Plan

### Phase 1: Self-Knowledge Only

**Each primal declares only self**:

1. **BearDog** declares:
   ```rust
   provides: [Security, Encryption, Trust]
   requires: [Discovery]
   ```

2. **Songbird** declares:
   ```rust
   provides: [Discovery, ConnectionManagement]
   requires: [Security]
   ```

3. **No hardcoded names** in either primal!

### Phase 2: Registry Implementation

**Songbird becomes capability registry**:

1. UDP discovery (existing plan)
2. Add capability registry
3. Unix socket API for registration
4. Provider lookup API

### Phase 3: BearDog Integration

**BearDog connects via capability**:

1. Connect to Songbird Unix socket
2. Register: `provides: [Security]`
3. Request: `get_provider(Discovery)`
4. Receive: Songbird socket info
5. Subscribe to peer events

### Phase 4: Validation

**Prove O(N) scaling**:

1. Add NewDiscovery primal
2. BearDog works with BOTH Songbird and NewDiscovery
3. No code changes to BearDog
4. Add NewSecurity primal
5. Songbird works with BOTH BearDog and NewSecurity
6. No code changes to Songbird

---

## 📋 Code Examples

### BearDog (Self-Knowledge Only)

```rust
// beardog/src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    // ONLY self-knowledge!
    let self_config = SelfConfig {
        provides: vec![
            Capability::Security,
            Capability::Encryption,
            Capability::Trust,
        ],
        requires: vec![
            Capability::Discovery,  // Generic!
        ],
    };
    
    // Connect to registry (Songbird or biomeOS)
    let registry = RegistryClient::connect(
        "/tmp/songbird-nat0.sock"  // From env var
    ).await?;
    
    // Register self
    registry.register(self_config).await?;
    
    // Get discovery provider (could be anything!)
    let discovery = registry.get_provider(Capability::Discovery).await?;
    
    // Subscribe to peer events
    discovery.subscribe("peer_discovered", |peer| {
        // Evaluate trust
        let trust_level = evaluate_trust(peer)?;
        
        if trust_level == TrustLevel::High {
            // Establish encrypted connection
            establish_encrypted_connection(peer)?;
        }
    }).await?;
    
    // Run server
    run_security_service().await
}
```

### Songbird (Self-Knowledge Only)

```rust
// songbird/src/main.rs

#[tokio::main]
async fn main() -> Result<()> {
    // ONLY self-knowledge!
    let self_config = SelfConfig {
        provides: vec![
            Capability::Discovery,
            Capability::ConnectionManagement,
        ],
        requires: vec![
            Capability::Security,  // Generic!
        ],
    };
    
    // Start UDP discovery
    let udp_discovery = UdpDiscovery::start("224.0.0.251:5353").await?;
    
    // Start capability registry
    let registry = CapabilityRegistry::new();
    
    // Start Unix socket server
    let ipc_server = IpcServer::start(
        "/tmp/songbird-nat0.sock",
        registry.clone()
    ).await?;
    
    // When peer discovered
    udp_discovery.on_peer_discovered(|peer| {
        // Request security provider (could be anything!)
        let security = registry.get_provider(Capability::Security)?;
        
        // Ask security to evaluate trust
        security.evaluate_trust(peer)?;
    });
    
    // Run discovery service
    run_discovery_service().await
}
```

---

## 🎊 Key Benefits

### Evolveability

**Add new primals**: ✅ Zero changes to existing code  
**Replace primals**: ✅ Register with same capability  
**Multiple providers**: ✅ Registry handles selection  
**Independent evolution**: ✅ No cross-dependencies

### Scaling

**Traditional**: N^2 connections (100 for 10 primals)  
**Capability-based**: O(N) registry lookups (10 for 10 primals)  
**Factor**: 10x improvement at 10 primals, 100x at 100 primals!

### Maintenance

**Hardcoded**: Update all N primals when adding one  
**Capability-based**: Update 0 primals when adding one  
**Result**: Truly sovereign primals!

---

## 📊 Execution Plan

### Immediate (Local)

1. **Implement Songbird UDP discovery**
   - Location: `phase1/songbird/`
   - Add capability registry
   - Add Unix socket IPC server

2. **Update BearDog to use capabilities**
   - Location: `phase1/beardog/`
   - Remove hardcoded "Songbird" references
   - Connect via `Capability::Discovery`

3. **Update biomeOS**
   - Remove port configuration
   - Pass registry socket to primals
   - Monitor via capabilities

4. **Test locally**
   - Spawn Songbird + BearDog
   - Verify capability-based connection
   - Add mock NewDiscovery
   - Verify BearDog works with both!

---

**Status**: Architecture for O(N) scaling complete. Ready to implement!

**Key Insight**: Primals ONLY know themselves + required capabilities. Registry handles all connection routing. N^2 → O(N)!

🚀 **This is true sovereign primal architecture!**


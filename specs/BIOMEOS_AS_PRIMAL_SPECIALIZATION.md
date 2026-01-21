# biomeOS: Ecosystem Management Primal

**Date**: January 21, 2026  
**Type**: Primal Specification  
**Status**: ✅ **ACTIVE EVOLUTION**

---

## 🎯 CORE PRINCIPLE

**biomeOS is a primal. Not a special orchestrator. A primal.**

It specializes in **ecosystem management**, just like:
- BearDog specializes in **crypto**
- Songbird specializes in **networking**
- Squirrel specializes in **AI orchestration**
- **biomeOS specializes in ecosystem lifecycle**

---

## 🧬 PRIMAL IDENTITY

### What biomeOS Is

```
biomeOS = Ecosystem Management Primal

Capabilities:
  - primal.germination
  - primal.terraria
  - primal.imprinting
  - primal.injection
  - primal.apoptosis
  - ecosystem.discovery
  - ecosystem.coordination
  - graph.deployment
```

### What biomeOS Is NOT

- ❌ Not a "god layer" above primals
- ❌ Not special or privileged
- ❌ Not the only way to deploy primals
- ❌ Not immune to primal lifecycle rules

### Self-Awareness

**biomeOS must treat itself like a primal**:
- Has its own lifecycle (germination, terraria, etc.)
- Can be deployed by another biomeOS instance
- Can undergo apoptosis (controlled shutdown)
- Discovers other primals (including other biomeOS instances)
- Inherits security from Tower Atomic (like any primal)

---

## 🌳 SPECIALIZATION: Ecosystem Management

### Core Functions

1. **Germination** - Birth new primals
2. **Terraria** - Safe learning environments
3. **Imprinting** - Ecosystem structure transfer
4. **Injection** - Live ecosystem introduction
5. **Apoptosis** - Controlled primal shutdown
6. **Nucleation** - Coordinate aligned startup
7. **Discovery** - Capability registry management

### Why This Specialization Matters

**Every primal specializes**:
- BearDog doesn't do networking (delegates to Songbird)
- Songbird doesn't do crypto (delegates to BearDog)
- Squirrel doesn't do HTTP directly (delegates to Tower)

**biomeOS doesn't do crypto, networking, or AI**:
- biomeOS delegates crypto to BearDog
- biomeOS delegates networking to Songbird
- biomeOS delegates AI to Squirrel
- **biomeOS does ecosystem management**

---

## 🔬 NEURAL API: The Specialization Implementation

### Neural API = biomeOS's Core Capability

Just like:
- BearDog has crypto RPC methods
- Songbird has BTSP RPC methods
- Squirrel has AI routing methods

**biomeOS has ecosystem management RPC methods**:

```json
// Neural API RPC methods (biomeOS capabilities)

{
  "method": "biomeos.germinate_primal",
  "params": {
    "primal": "squirrel",
    "family_id": "nat0"
  }
}

{
  "method": "biomeos.create_terraria",
  "params": {
    "primal": "squirrel",
    "duration": "30m"
  }
}

{
  "method": "biomeos.imprint_ecosystem",
  "params": {
    "primal": "squirrel-nat0",
    "ecosystem": "nat0-production"
  }
}

{
  "method": "biomeos.inject_primal",
  "params": {
    "primal": "squirrel-nat0",
    "parent": "tower_atomic"
  }
}

{
  "method": "biomeos.trigger_apoptosis",
  "params": {
    "primal": "old-instance-nat0",
    "graceful": true
  }
}
```

### Evolution Stages

1. **Lifecycle Management** ✅ (This spec)
2. **Terraria System** (4-week roadmap)
3. **Apoptosis Coordination** (Future)
4. **Nested Environments** (Future)
5. **Multi-Niche Ecosystems** (Future)

---

## 🌊 NESTED BIOMEOS: Sub-Environments

### The Vision

**An ocean biome has many niches**:
- Coral reefs (high diversity, specific conditions)
- Sandy floor (different species, different rules)
- Deep trenches (extreme environment, specialized life)

**A biomeOS can have nested biomeOS instances**:
- Production biomeOS (main environment)
  - Terraria biomeOS (testing niche)
  - Development biomeOS (experimental niche)
  - Staging biomeOS (pre-production niche)

### Example: Nested Structure

```
Production biomeOS (nat0-prod)
  ├─ Tower Atomic (security foundation)
  ├─ Squirrel (AI production)
  ├─ ToadStool (local AI)
  └─ Terraria biomeOS (nat0-test) ← NESTED
      ├─ Mock Tower Atomic
      ├─ Squirrel v2 (testing)
      └─ Mock ToadStool
```

**Key Insight**: The terraria biomeOS is ALSO a primal!
- It specializes in isolated testing
- It inherits from production biomeOS
- It can be shut down (apoptosis) without affecting production

### Communication Between Nested biomeOS

```
Production biomeOS (nat0-prod)
    ↓ RPC call
Terraria biomeOS (nat0-test)
    ↓ RPC call
Squirrel v2 (under test)
```

**Production biomeOS asks Terraria biomeOS**:
```json
{
  "method": "biomeos.query_terraria_results",
  "params": {
    "terraria_id": "nat0-test",
    "primal": "squirrel-v2"
  }
}
```

**Terraria biomeOS responds**:
```json
{
  "result": {
    "primal": "squirrel-v2",
    "validation": "passed",
    "ready_for_injection": true,
    "observations": {
      "capability_discovery": "correct",
      "rpc_patterns": "optimal",
      "error_handling": "excellent"
    }
  }
}
```

**Production biomeOS decides**: "OK, inject Squirrel v2 into production"

---

## 🔄 BIOMEOS LIFECYCLE (Self-Applied)

### 1. Germination

**biomeOS can germinate another biomeOS**:

```bash
# Main biomeOS germinates a terraria biomeOS
biomeos-main germinate \
  --primal biomeos \
  --mode terraria \
  --family-id test-001

# New terraria biomeOS starts with minimal knowledge
# Discovers main biomeOS via registry
# Inherits security from main biomeOS's Tower Atomic
```

### 2. Terraria (Self-Testing)

**biomeOS can test itself in terraria**:

```bash
# Create terraria for biomeOS v2 testing
biomeos-main terraria create \
  --primal biomeos-v2 \
  --mock-ecosystem nat0-prod

# biomeOS v2 boots in terraria
# Tries to germinate a mock Squirrel
# Validates its own ecosystem management capabilities
# If passes → ready for injection
```

### 3. Imprinting

**biomeOS learns ecosystem structure**:

```json
{
  "imprint": {
    "ecosystem_id": "nat0-production",
    "my_role": "ecosystem_manager",
    "my_capabilities": [
      "primal.germination",
      "primal.terraria",
      "ecosystem.coordination"
    ],
    "peers": {
      "tower_atomic": {
        "role": "security_foundation",
        "inherit_from": true
      }
    }
  }
}
```

### 4. Injection

**biomeOS joins live ecosystem**:

```bash
# Main biomeOS injects new biomeOS instance
biomeos-main inject \
  --primal biomeos-v2 \
  --parent tower_atomic \
  --role sub_environment_manager

# biomeOS v2 now manages a sub-niche
# Main biomeOS delegates terraria work to v2
# Both are primals, coordinating via RPC
```

### 5. Apoptosis

**biomeOS can shut itself down gracefully**:

```bash
# biomeOS receives apoptosis signal
biomeos-terraria receive-apoptosis-signal

# Graceful shutdown:
# 1. Stop accepting new germinations
# 2. Allow running primals to complete
# 3. Archive terraria results
# 4. Shut down capability registry
# 5. Remove sockets
# 6. Exit cleanly
```

---

## 📊 CAPABILITY DECLARATION

### biomeOS Capabilities (RPC Methods)

```toml
# biomeos-capabilities.toml

[primal]
name = "biomeos"
type = "ecosystem_manager"
generation = 1  # Inherits from Tower Atomic (gen 0)

[[capabilities]]
name = "primal.germination"
description = "Create new primal instances with minimal knowledge"
rpc_method = "biomeos.germinate_primal"

[[capabilities]]
name = "primal.terraria"
description = "Provide safe learning environments"
rpc_method = "biomeos.create_terraria"

[[capabilities]]
name = "primal.imprinting"
description = "Transfer ecosystem structure to new primals"
rpc_method = "biomeos.imprint_ecosystem"

[[capabilities]]
name = "primal.injection"
description = "Introduce primals to live ecosystem"
rpc_method = "biomeos.inject_primal"

[[capabilities]]
name = "primal.apoptosis"
description = "Coordinate graceful primal shutdown"
rpc_method = "biomeos.trigger_apoptosis"

[[capabilities]]
name = "ecosystem.discovery"
description = "Manage capability registry"
rpc_method = "biomeos.register_capability"

[[capabilities]]
name = "ecosystem.coordination"
description = "Coordinate multi-primal deployments"
rpc_method = "biomeos.execute_graph"

[[capabilities]]
name = "ecosystem.nucleation"
description = "Assign sockets and coordinate startup"
rpc_method = "biomeos.assign_socket"
```

### Discovery by Other Primals

**Other primals discover biomeOS**:

```rust
// Squirrel discovers ecosystem manager
let ecosystem_manager = discover_capability("primal.germination").await?;

// Result: biomeos-nat0 at /tmp/biomeos-nat0.sock
// Squirrel can now ask biomeOS to germinate a new primal!
```

**biomeOS is just another primal with a special capability set.**

---

## 🌍 MULTI-NICHE ECOSYSTEMS

### Vision: Production + Development + Staging

```
Tower Atomic (gen 0 - security foundation)
    ↓
┌────────────────────────────────────────────────────────┐
│  Production biomeOS (gen 1)                            │
│  Role: Main ecosystem manager                          │
│  Manages: Production primals                           │
│                                                         │
│  ┌──────────────────────────────────────────────────┐ │
│  │  Development biomeOS (gen 2) ← NESTED            │ │
│  │  Role: Experimental niche manager                │ │
│  │  Manages: Experimental primals                   │ │
│  │  Isolated: Yes (can fail without affecting prod) │ │
│  └──────────────────────────────────────────────────┘ │
│                                                         │
│  ┌──────────────────────────────────────────────────┐ │
│  │  Staging biomeOS (gen 2) ← NESTED                │ │
│  │  Role: Pre-production niche manager              │ │
│  │  Manages: Staging primals                        │ │
│  │  Isolated: Partially (mirrors prod)              │ │
│  └──────────────────────────────────────────────────┘ │
│                                                         │
│  ┌──────────────────────────────────────────────────┐ │
│  │  Terraria biomeOS (gen 2) ← NESTED               │ │
│  │  Role: Testing niche manager                     │ │
│  │  Manages: Primals under test                     │ │
│  │  Isolated: Fully (mock ecosystem)                │ │
│  └──────────────────────────────────────────────────┘ │
└────────────────────────────────────────────────────────┘
```

### Communication Pattern

```
Production biomeOS ↔ Development biomeOS (RPC)
Production biomeOS ↔ Staging biomeOS (RPC)
Production biomeOS ↔ Terraria biomeOS (RPC)

Development biomeOS manages its sub-niche
Staging biomeOS manages its sub-niche
Terraria biomeOS manages its sub-niche

All are primals. All use same RPC protocol.
```

---

## 🔧 IMPLEMENTATION EVOLUTION

### Phase 1: biomeOS Self-Awareness (This Spec)

**Files to update**:
- `specs/BIOMEOS_AS_PRIMAL_SPECIALIZATION.md` (this file)
- `biomeos-capabilities.toml` (capability declaration)
- `README.md` (update to reflect primal status)

**Key changes**:
- Document biomeOS as a primal
- Define capability set
- Explain specialization (ecosystem management)

### Phase 2: Neural API as Capability Provider

**Files to modify**:
- `crates/biomeos-atomic-deploy/src/neural_api_server.rs`
  - Expose ecosystem management as RPC methods
  - Register biomeOS capabilities in own registry

**Example**:
```rust
impl NeuralApiServer {
    async fn register_self_capabilities(&mut self) -> Result<()> {
        // biomeOS registers itself in capability registry
        self.capability_registry.register(DiscoveredPrimal {
            id: format!("biomeos-{}", self.family_id),
            capabilities: vec![
                "primal.germination".to_string(),
                "primal.terraria".to_string(),
                "ecosystem.coordination".to_string(),
            ],
            primary_socket: self.socket_path.clone(),
            health: PrimalHealth::Healthy,
            registered_at: Utc::now(),
        }).await?;
        
        Ok(())
    }
}
```

### Phase 3: Nested biomeOS Support

**Files to create**:
- `crates/biomeos-nesting/` (NEW CRATE)
- `crates/biomeos-nesting/src/sub_environment.rs`

**Features**:
```rust
pub struct SubEnvironment {
    id: String,
    parent_biomeos: String,
    role: EnvironmentRole,  // Terraria, Development, Staging
    isolation_level: IsolationLevel,
}

impl SubEnvironment {
    pub async fn spawn_nested_biomeos(
        &self,
        parent: &NeuralApiServer
    ) -> Result<Process> {
        // Parent biomeOS germinates child biomeOS
        // Child inherits from parent's Tower Atomic
        // Child manages its own sub-niche
    }
}
```

### Phase 4: Apoptosis Implementation

**Files to create**:
- `crates/biomeos-lifecycle/src/apoptosis.rs`

**Features**:
```rust
impl NeuralApiServer {
    pub async fn receive_apoptosis_signal(&mut self) -> Result<()> {
        // 1. Stop accepting new work
        self.accepting_requests.store(false, Ordering::Release);
        
        // 2. Wait for current operations to complete
        self.wait_for_active_operations().await?;
        
        // 3. Trigger apoptosis for managed primals
        for primal in &self.managed_primals {
            primal.send_apoptosis_signal().await?;
        }
        
        // 4. Archive state
        self.archive_ecosystem_state().await?;
        
        // 5. Unregister from capability registry
        self.capability_registry.unregister(&self.id).await?;
        
        // 6. Clean shutdown
        Ok(())
    }
}
```

---

## 📋 RULES FOR BIOMEOS (Self-Applied)

### 1. No Special Privileges

- biomeOS cannot bypass security (uses Tower Atomic like everyone)
- biomeOS cannot access primal internals (uses RPC like everyone)
- biomeOS cannot force primal behavior (requests only)

### 2. Discoverable

- biomeOS must register its capabilities
- biomeOS must respond to discovery queries
- biomeOS must advertise its socket

### 3. Lifecycle Compliance

- biomeOS can be germinated
- biomeOS can be tested in terraria
- biomeOS can undergo apoptosis
- biomeOS can be replaced

### 4. Delegation

- biomeOS delegates crypto to BearDog
- biomeOS delegates networking to Songbird
- biomeOS delegates AI to Squirrel
- biomeOS focuses on ecosystem management

### 5. Self-Similarity

- biomeOS can manage other biomeOS instances
- biomeOS can be managed by another biomeOS
- Nested biomeOS have same capabilities
- All follow same primal patterns

---

## 🎯 SUCCESS CRITERIA

### When This Evolution Is Complete

1. ✅ biomeOS has a capability declaration file
2. ✅ biomeOS registers itself in capability registry
3. ✅ Other primals can discover biomeOS via capabilities
4. ✅ biomeOS can germinate another biomeOS
5. ✅ Nested biomeOS can manage sub-niches
6. ✅ biomeOS can undergo apoptosis
7. ✅ Documentation reflects biomeOS as a primal

---

## 🌟 THE COMPLETE PICTURE

```
Tower Atomic (gen 0)
  ├─ BearDog (crypto specialist)
  └─ Songbird (network specialist)
      ↓
biomeOS (gen 1) ← PRIMAL, not god layer!
  ├─ Specializes in ecosystem management
  ├─ Uses BearDog for crypto
  ├─ Uses Songbird for networking
  └─ Can be nested, replaced, or shut down
      ↓
Squirrel (gen 2) ← PRIMAL
  ├─ Specializes in AI orchestration
  ├─ Uses Tower for HTTP
  └─ Managed by biomeOS
      ↓
ToadStool (gen 2) ← PRIMAL
  ├─ Specializes in local AI
  ├─ Discovered by Squirrel
  └─ Managed by biomeOS
      ↓
Terraria biomeOS (gen 2) ← ALSO A PRIMAL!
  ├─ Specializes in isolated testing
  ├─ Nested under main biomeOS
  └─ Manages test primals
```

**Every layer is a primal. Every primal specializes. No god layers.**

---

## 🎊 PHILOSOPHICAL SHIFT

### Before (Wrong)

```
biomeOS (orchestrator)
    ↓ controls
Primals (workers)
```

**Problem**: Creates hierarchy, god layer, special privileges

### After (Correct)

```
Tower Atomic (security foundation)
    ↓ all inherit
Primals (specialists)
  - BearDog (crypto)
  - Songbird (network)
  - biomeOS (ecosystem)
  - Squirrel (AI)
  - ...
```

**Correct**: All primals, all equal, different specializations

---

**🌍 biomeOS: Ecosystem Management Primal, Not God Layer! 🦀✨**

---

*Specification: January 21, 2026*  
*Type: Primal Identity & Specialization*  
*Status: Active Evolution*  
*Impact: Removes hierarchy, enables nesting, true primal ecosystem*


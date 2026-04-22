# Primal Lifecycle: Germination, Terraria, and Ecological Injection

**Date**: January 21, 2026  
**Status**: 🌱 **ARCHITECTURAL EVOLUTION**  
**Concept**: Complete primal lifecycle from germination to ecosystem integration

---

## 🎯 THE VISION

**User's Insight**:

> "Every new primal is germinated and then injected into the system. They all have infant discovery (every primal learns its environment instead of hardcoding). We can allow biomeOS to have a terraria to introduce new primals to get them up to speed more rapidly/imprint them with the current ecological structure."

**Translation**: Neural API becomes a **complete primal lifecycle manager**:

1. **Germination** - Birth a new primal in isolation
2. **Terraria** - Safe learning environment for imprinting
3. **Injection** - Introduction to live ecosystem
4. **Inheritance** - Security and structure from parent (Tower Atomic)
5. **Infant Discovery** - Environment learning (no hardcoding)

---

## 🌱 PRIMAL LIFECYCLE STAGES

### Stage 1: Germination (Birth)

**Definition**: Create a new primal instance with minimal knowledge

```
Neural API (Germination Chamber)
    ↓
Creates primal:
  - Assigns socket: /tmp/{primal}-{family}.sock
  - Sets FAMILY_ID
  - Provides CAPABILITY_REGISTRY_SOCKET
  - NO other primals known (infant state)
    ↓
Primal boots:
  - Reads only self-knowledge
  - Doesn't know what ecosystem exists
  - Doesn't hardcode peer locations
  - Ready to learn
```

**Example**:
```bash
# Neural API germinates Squirrel
SQUIRREL_SOCKET=/tmp/squirrel-${FAMILY_ID}.sock
FAMILY_ID=${FAMILY_ID}
CAPABILITY_REGISTRY_SOCKET=/tmp/neural-api-${FAMILY_ID}.sock
# NO ANTHROPIC_API_KEY yet
# NO AI_PROVIDER_SOCKETS yet
# NO hardcoded peer knowledge
```

### Stage 2: Terraria (Imprinting)

**Definition**: Safe environment where primal learns the ecosystem structure

```
Neural API (Terraria - Test Environment)
    ↓
Provides controlled ecosystem:
  - Mock Tower Atomic (for testing)
  - Mock capability providers
  - Safe discovery sandbox
  - Logging and observation
    ↓
Primal explores:
  - Discovers capabilities via registry
  - Learns what services exist
  - Tests communication patterns
  - Adapts to environment
    ↓
Neural API observes:
  - What capabilities did it discover?
  - What communication patterns did it use?
  - Is it behaving correctly?
  - Ready for live injection?
```

**Example**:
```toml
# terraria_squirrel.toml (test environment)

[terraria]
name = "squirrel_learning"
mode = "isolated"  # Isolated from production
logging = "verbose"

# Mock ecosystem
[[mocks]]
id = "mock-tower"
capabilities = ["btsp.external", "http.request"]
socket = "/tmp/mock-tower.sock"

[[mocks]]
id = "mock-local-ai"
capabilities = ["ai.generate_text"]
socket = "/tmp/mock-local-ai.sock"

# Primal under test
[[primals]]
id = "squirrel-test"
germinate_from = "squirrel"
observe = true
```

**Neural API runs Squirrel in terraria**:
```bash
# Squirrel boots in terraria
# Discovers mock-tower via registry
# Discovers mock-local-ai via registry
# Learns: "I need btsp.external for Anthropic"
# Learns: "I need ai.generate_text for local AI"
# Neural API observes and validates behavior
```

### Stage 3: Imprinting (Ecological Structure)

**Definition**: Primal learns the ACTUAL ecosystem structure before live deployment

```
Neural API (Imprinting)
    ↓
Provides ecosystem map:
  - Tower Atomic exists at /tmp/songbird-${FAMILY_ID}.sock
  - Provides: btsp.external, btsp.internal
  - Security inherited from BearDog
    ↓
  - Local AI at /tmp/toadstool-${FAMILY_ID}.sock
  - Provides: ai.generate_text
    ↓
Primal receives imprint:
  - "This is your ecosystem"
  - "These are your peers"
  - "This is how to communicate"
  - But still uses discovery (not hardcoding!)
    ↓
Primal updates internal model:
  - Expected capabilities mapped
  - Communication patterns learned
  - Ready for live ecosystem
```

**Example**:
```json
// Neural API sends imprint to Squirrel
{
  "imprint": {
    "ecosystem_id": "${FAMILY_ID}-production",
    "structure": {
      "tower_atomic": {
        "id": "songbird-${FAMILY_ID}",
        "capabilities": ["btsp.external", "btsp.internal", "http.request"],
        "socket": "/tmp/songbird-${FAMILY_ID}.sock",
        "trust": "inherited_from_beardog"
      },
      "local_ai": {
        "id": "toadstool-${FAMILY_ID}",
        "capabilities": ["ai.generate_text", "ai.embeddings"],
        "socket": "/tmp/toadstool-${FAMILY_ID}.sock"
      },
      "security": {
        "id": "beardog-${FAMILY_ID}",
        "capabilities": ["crypto.sign", "crypto.verify"],
        "socket": "/tmp/beardog-${FAMILY_ID}.sock",
        "role": "nucleus_foundation"
      }
    },
    "lineage": {
      "parent": "tower_atomic",
      "generation": 2,
      "security_context": "inherited"
    }
  }
}
```

### Stage 4: Injection (Live Deployment)

**Definition**: Introduce primal to live ecosystem with inherited security

```
Neural API (Injection)
    ↓
Coordinates with Tower Atomic:
  - "New primal joining: Squirrel"
  - "Inherits security from Tower Atomic"
  - "Capabilities: ai.query, ai.routing"
    ↓
Tower Atomic prepares:
  - Creates security context for Squirrel
  - Allocates communication channel
  - Updates ecosystem map
    ↓
Neural API deploys Squirrel:
  - With ecosystem imprint
  - With inherited security
  - With capability registry access
    ↓
Squirrel joins:
  - Discovers peers via registry (instant!)
  - Inherits security from Tower
  - Communicates via BTSP
  - Becomes part of ecosystem
```

**Example**:
```bash
# Neural API coordinates injection
neural-api inject \
  --primal squirrel-${FAMILY_ID} \
  --parent tower_atomic \
  --inherit-security \
  --ecosystem ${FAMILY_ID}-production

# Output:
🌱 Germinating: squirrel-${FAMILY_ID}
🔬 Terraria complete: validated behavior
🧬 Imprinting: ${FAMILY_ID}-production ecosystem structure
🔐 Inheriting security: tower_atomic → squirrel-${FAMILY_ID}
💉 Injecting into live ecosystem
✅ Squirrel-${FAMILY_ID} joined ecosystem (generation: 2)
```

### Stage 5: Infant Discovery (Continuous Learning)

**Definition**: Primal continuously discovers and adapts to ecosystem changes

```
Squirrel (Post-Injection)
    ↓
Runtime discovery:
  - "I need http.request capability"
  - Query registry: discover("btsp.external")
  - Result: songbird-${FAMILY_ID} at /tmp/songbird-${FAMILY_ID}.sock
  - Connect and use
    ↓
Ecosystem changes:
  - New primal joins (e.g., ToadStool v2)
  - Advertises: ai.generate_text
  - Squirrel discovers via registry
  - Adapts routing to use new provider
    ↓
No hardcoding:
  - Squirrel never hardcoded Songbird's socket
  - Squirrel never hardcoded ToadStool's socket
  - Always discovers at runtime
  - TRUE PRIMAL pattern ✅
```

---

## 🏗️ NEURAL API EVOLUTION

### Role Expansion

**Before** (deployment only):
- Deploy primals from graph
- Pass environment variables
- Monitor health

**Now** (complete lifecycle manager):
- **Germinate** - Create primal with minimal knowledge
- **Terraria** - Provide safe learning environment
- **Imprint** - Transfer ecosystem structure
- **Inject** - Coordinate live introduction
- **Inherit** - Manage security lineage
- **Observe** - Monitor and learn from behavior
- **Adapt** - Adjust ecosystem based on observations

### New Components

1. **Germination Chamber**
   - Creates primal instances
   - Assigns sockets and family IDs
   - Provides minimal environment

2. **Terraria System**
   - Mock ecosystem for testing
   - Behavior observation and validation
   - Safe experimentation

3. **Imprinting Service**
   - Transfers ecosystem structure
   - Maps capabilities to providers
   - Establishes lineage

4. **Injection Coordinator**
   - Coordinates with parent (Tower Atomic)
   - Manages security inheritance
   - Orchestrates live introduction

5. **Lineage Tracker**
   - Tracks parent-child relationships
   - Manages security contexts
   - Records ecosystem evolution

---

## 🧬 INHERITANCE MODEL

### Security Inheritance from Tower Atomic

```
Tower Atomic (NUCLEUS - Generation 0)
  ├─ BearDog (Crypto)
  └─ Songbird (Network/BTSP)
      ↓
biomeOS (Generation 1) ← Inherits from Tower
  ├─ Neural API
  └─ Graph Executor
      ↓
Squirrel (Generation 2) ← Inherits from biomeOS → Tower
  ├─ AI Routing
  └─ Capability Discovery
```

**Inheritance Chain**:
1. **Tower Atomic** (Generation 0)
   - Creates secure foundation
   - Establishes crypto and network

2. **biomeOS** (Generation 1)
   - Inherits security from Tower
   - Adds orchestration layer
   - Manages ecosystem

3. **Squirrel** (Generation 2)
   - Inherits security from biomeOS (which inherited from Tower)
   - Adds AI orchestration
   - Participates in ecosystem

**Security Context Flow**:
```
BearDog generates root security context
    ↓
Songbird inherits + network context
    ↓
biomeOS inherits + orchestration context
    ↓
Squirrel inherits + AI context
```

**Result**: Every primal has a secure lineage back to BearDog!

---

## 🔬 TERRARIA SYSTEM (Detailed)

### Purpose

**Terraria** = Safe learning environment for new primals

Like a biological terrarium:
- Controlled environment
- Observable from outside
- Safe for experimentation
- Can be destroyed/reset without affecting production

### Implementation

```toml
# terraria_config.toml

[terraria]
id = "squirrel_v2_testing"
mode = "isolated"
parent_ecosystem = "${FAMILY_ID}-production"
duration_max = "1h"  # Auto-cleanup

# Mock Tower Atomic
[[mocks.tower]]
id = "mock-tower"
capabilities = ["btsp.external", "http.request"]
behavior = "echo"  # Echo requests for validation
latency_simulation = "10ms"

# Mock Local AI
[[mocks.ai]]
id = "mock-toadstool"
capabilities = ["ai.generate_text"]
behavior = "scripted"
responses_file = "mock_ai_responses.json"

# Primal under test
[[primals]]
id = "squirrel-v2-test"
binary = "./target/release/squirrel"
environment = {
  FAMILY_ID = "test-terraria",
  CAPABILITY_REGISTRY_SOCKET = "/tmp/terraria-registry.sock"
}
observe = {
  rpc_calls = true,
  capability_queries = true,
  errors = true,
  performance = true
}

# Expected behavior
[[validation]]
type = "discovers_capability"
capability = "btsp.external"
timeout = "5s"

[[validation]]
type = "makes_rpc_call"
method = "http.request"
to_mock = "mock-tower"
```

### Terraria Lifecycle

```
1. Neural API creates terraria
   - Spins up mock ecosystem
   - Starts behavior logger

2. Germinates primal in terraria
   - Primal boots in isolation
   - Only sees mock ecosystem

3. Observes behavior
   - What capabilities did it discover?
   - What RPC calls did it make?
   - How did it handle errors?
   - Performance characteristics?

4. Validates behavior
   - Meets expected patterns?
   - No security violations?
   - Proper error handling?

5. Reports results
   - Passed: Ready for imprinting
   - Failed: Needs debugging
   - Logs: Available for analysis

6. Cleanup
   - Tears down terraria
   - Archives logs
   - Ready for next test
```

### Example: Squirrel in Terraria

```bash
# Create terraria for Squirrel testing
neural-api terraria create \
  --primal squirrel \
  --duration 30m \
  --observe-all

# Neural API output:
🔬 Creating terraria: squirrel-test-20260121
📦 Mocking ecosystem: ${FAMILY_ID}-production
🌱 Germinating: squirrel-test instance
👀 Observing behavior...

# After 2 minutes:
✅ Discovered btsp.external (via registry query)
✅ Made http.request to mock-tower
✅ Handled 401 error correctly
✅ Retried with exponential backoff
⚠️  Warning: Made 15 registry queries (optimize?)

# After validation:
✅ Behavior validated - ready for imprinting
📊 Report: /tmp/terraria-reports/squirrel-test-20260121.json
```

---

## 📊 IMPLEMENTATION ROADMAP

### Week 1: Germination System

**Files to modify**:
- `crates/biomeos-atomic-deploy/src/germination.rs` (NEW)
- `crates/biomeos-atomic-deploy/src/neural_executor.rs`

**Features**:
```rust
pub struct GerminationConfig {
    pub primal: String,
    pub family_id: String,
    pub socket: PathBuf,
    pub minimal_env: HashMap<String, String>,  // Only essentials
}

impl NeuralExecutor {
    pub async fn germinate_primal(&self, config: GerminationConfig) -> Result<Process> {
        // 1. Assign socket
        // 2. Provide minimal environment
        // 3. Boot primal in infant state
        // 4. Return process handle
    }
}
```

### Week 2: Terraria System

**Files to create**:
- `crates/biomeos-terraria/` (NEW CRATE)
- `crates/biomeos-terraria/src/lib.rs`
- `crates/biomeos-terraria/src/mock_ecosystem.rs`
- `crates/biomeos-terraria/src/observer.rs`
- `crates/biomeos-terraria/src/validator.rs`

**Features**:
```rust
pub struct Terraria {
    id: String,
    mocks: Vec<MockPrimal>,
    observer: BehaviorObserver,
    validator: BehaviorValidator,
}

impl Terraria {
    pub async fn create(config: TerrariaConfig) -> Result<Self> {
        // Create isolated environment
    }
    
    pub async fn introduce_primal(&mut self, primal: Process) -> Result<()> {
        // Introduce primal to terraria
    }
    
    pub async fn observe(&self, duration: Duration) -> BehaviorReport {
        // Observe and log behavior
    }
    
    pub async fn validate(&self) -> ValidationResult {
        // Validate against expected patterns
    }
}
```

### Week 3: Imprinting System

**Files to create**:
- `crates/biomeos-atomic-deploy/src/imprinting.rs` (NEW)

**Features**:
```rust
pub struct EcosystemImprint {
    pub ecosystem_id: String,
    pub structure: HashMap<String, PrimalInfo>,
    pub lineage: LineageInfo,
    pub security_context: SecurityContext,
}

impl NeuralExecutor {
    pub async fn imprint_primal(
        &self,
        primal: &Process,
        imprint: EcosystemImprint
    ) -> Result<()> {
        // Send ecosystem structure to primal
        // Primal learns but doesn't hardcode
    }
}
```

### Week 4: Injection Coordinator

**Files to modify**:
- `crates/biomeos-atomic-deploy/src/neural_executor.rs`
- `crates/biomeos-atomic-deploy/src/injection.rs` (NEW)

**Features**:
```rust
pub struct InjectionPlan {
    pub primal: Process,
    pub parent: String,  // e.g., "tower_atomic"
    pub inherit_security: bool,
    pub coordination_timeout: Duration,
}

impl NeuralExecutor {
    pub async fn inject_primal(&mut self, plan: InjectionPlan) -> Result<()> {
        // 1. Coordinate with parent (Tower Atomic)
        // 2. Establish security inheritance
        // 3. Update ecosystem registry
        // 4. Introduce primal to live system
        // 5. Monitor integration
    }
}
```

---

## 🎯 EXAMPLE: COMPLETE SQUIRREL LIFECYCLE

### 1. Germination

```bash
# Neural API germinates Squirrel
neural-api germinate --primal squirrel --family ${FAMILY_ID}

# Output:
🌱 Germinating: squirrel-${FAMILY_ID}
📍 Socket assigned: /tmp/squirrel-${FAMILY_ID}.sock
🔑 Minimal environment:
   - SQUIRREL_SOCKET=/tmp/squirrel-${FAMILY_ID}.sock
   - FAMILY_ID=${FAMILY_ID}
   - CAPABILITY_REGISTRY_SOCKET=/tmp/neural-api-${FAMILY_ID}.sock
✅ Squirrel germinated (PID: 12345)
```

### 2. Terraria Testing

```bash
# Neural API creates terraria
neural-api terraria test --primal squirrel-${FAMILY_ID}

# Output:
🔬 Creating terraria: squirrel-test
📦 Mocking: Tower Atomic, ToadStool
👀 Observing for 10 minutes...

# After observation:
📊 Behavior Report:
   ✅ Discovered capabilities correctly
   ✅ Made appropriate RPC calls
   ✅ Error handling: excellent
   ⚠️  Registry queries: 12 (could optimize)
   
🎯 Validation: PASSED
✅ Ready for imprinting
```

### 3. Imprinting

```bash
# Neural API imprints ecosystem structure
neural-api imprint \
  --primal squirrel-${FAMILY_ID} \
  --ecosystem ${FAMILY_ID}-production

# Output:
🧬 Imprinting ecosystem: ${FAMILY_ID}-production
📡 Sending structure:
   - Tower Atomic: songbird-${FAMILY_ID}
   - Local AI: toadstool-${FAMILY_ID}
   - Security: beardog-${FAMILY_ID}
🔗 Lineage: tower_atomic → biomeOS → squirrel
✅ Imprint complete
```

### 4. Injection

```bash
# Neural API injects into live ecosystem
neural-api inject \
  --primal squirrel-${FAMILY_ID} \
  --parent tower_atomic \
  --inherit-security

# Output:
🤝 Coordinating with Tower Atomic...
🔐 Inheriting security context...
💉 Injecting into live ecosystem...
📡 Announcing to ecosystem peers...
✅ Squirrel-${FAMILY_ID} joined (generation: 2)

# Ecosystem now:
🏰 Tower Atomic (gen 0)
   ├─ BearDog
   └─ Songbird
      ↓
📊 biomeOS (gen 1)
   └─ Neural API
      ↓
🐿️  Squirrel (gen 2)
```

### 5. Runtime Discovery

```bash
# Squirrel makes AI query
# Discovers btsp.external capability
# Finds: songbird-${FAMILY_ID}
# Connects via BTSP
# Makes HTTP request to Anthropic
# Returns AI response

# All via infant discovery - NO HARDCODING! ✅
```

---

## 🎊 BENEFITS

### For New Primals

1. ✅ **Safe onboarding** (terraria testing)
2. ✅ **Rapid learning** (ecosystem imprinting)
3. ✅ **Secure integration** (inherited from Tower)
4. ✅ **No hardcoding** (infant discovery)

### For Ecosystem

1. ✅ **Validated behavior** (terraria validation)
2. ✅ **Consistent patterns** (imprinting)
3. ✅ **Security lineage** (inheritance tracking)
4. ✅ **Evolutionary learning** (ecosystem adapts)

### For Development

1. ✅ **Test new primals safely** (isolated terraria)
2. ✅ **Observe real behavior** (not just unit tests)
3. ✅ **Debug in controlled environment** (mock ecosystem)
4. ✅ **Rapid iteration** (quick terraria cycles)

---

## 🌟 THE COMPLETE VISION

```
Neural API: Primal Lifecycle Manager
    ↓
┌─────────────────────────────────────────┐
│  1. Germination (Birth)                 │
│     - Create with minimal knowledge     │
│     - Assign socket + family            │
│     - Infant state                      │
└─────────────┬───────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  2. Terraria (Safe Learning)            │
│     - Mock ecosystem                    │
│     - Observe behavior                  │
│     - Validate patterns                 │
└─────────────┬───────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  3. Imprinting (Structure Transfer)     │
│     - Learn ecosystem map               │
│     - Understand capabilities           │
│     - Prepare for integration           │
└─────────────┬───────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  4. Injection (Live Introduction)       │
│     - Inherit from parent (Tower)       │
│     - Join ecosystem                    │
│     - Announce capabilities             │
└─────────────┬───────────────────────────┘
              ↓
┌─────────────────────────────────────────┐
│  5. Infant Discovery (Continuous)       │
│     - Discover at runtime               │
│     - Adapt to changes                  │
│     - NO HARDCODING                     │
└─────────────────────────────────────────┘
```

**Every primal follows this lifecycle. Every primal learns. Every primal adapts.**

---

**🌱 Neural API: Complete Primal Lifecycle Management! 🔬✨**

---

*Architecture Evolution: January 21, 2026*  
*Concept: Germination, Terraria, Imprinting, Injection*  
*Impact: Enables rapid, safe ecosystem evolution*  
*Status: IMPLEMENTED (v3.x) — Germination lifecycle integrated into LifecycleManager*


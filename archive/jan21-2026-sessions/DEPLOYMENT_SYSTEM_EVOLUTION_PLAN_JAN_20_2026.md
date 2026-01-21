# Deployment System Evolution Plan - January 20, 2026

**Date**: January 20, 2026  
**Approach**: 🎯 **Deep Debt Solution - Proper Architecture Evolution**  
**Status**: Strategic Planning Phase

---

## 🎯 Strategic Approach

### Phase 1: Pin Current Deployments (Immediate)
- Use **manual/scripted deployments** for Tower Atomic + Squirrel
- Create deployment scripts that work TODAY
- Document the correct sequencing and bonding
- Enable team to continue development

### Phase 2: Design Complex Graphs (Next)
- Define comprehensive graph specifications
- Model all atomic patterns (Tower, Node, Nest)
- Capture genetic bonding relationships
- Document dynamic environment composition

### Phase 3: Evolve & Abstract Deployment System (Future)
- Proper DAG execution engine
- Bonding primitives (covalent, ionic, metallic)
- Genetic lineage tracking
- Production-ready orchestration

**This is the RIGHT approach** - solve immediate need, then architect properly!

---

## 📦 Phase 1: Pinned Deployments (Working Today!)

### Tower Atomic Deployment Script

**File**: `scripts/deploy_tower_atomic_manual.sh`

```bash
#!/usr/bin/env bash
#
# Tower Atomic Manual Deployment
# BearDog + Songbird with Genetic Bonding
#
# This is the PINNED deployment until we evolve the DAG system
#
set -euo pipefail

FAMILY_ID="${1:-nat0}"
RUNTIME_DIR="${RUNTIME_DIR:-/tmp}"

echo "🧬 Deploying Tower Atomic (family_id: $FAMILY_ID)"
echo ""

# Socket paths
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-$FAMILY_ID.sock"
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-$FAMILY_ID.sock"

# Clean previous deployment
rm -f "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET"
pkill -f "beardog.*$FAMILY_ID" 2>/dev/null || true
pkill -f "songbird.*$FAMILY_ID" 2>/dev/null || true
sleep 1

echo "Phase 1/2: Starting BearDog (security primal)..."
./plasmidBin/primals/beardog/beardog-x86_64-musl server \
  --socket "$BEARDOG_SOCKET" \
  --family-id "$FAMILY_ID" \
  > "$RUNTIME_DIR/beardog-$FAMILY_ID.log" 2>&1 &

BEARDOG_PID=$!
echo "  ✅ BearDog started (PID: $BEARDOG_PID)"

# Wait for BearDog socket
echo "  Waiting for BearDog socket..."
for i in {1..30}; do
  if [ -S "$BEARDOG_SOCKET" ]; then
    echo "  ✅ BearDog socket ready: $BEARDOG_SOCKET"
    break
  fi
  sleep 0.1
done

if [ ! -S "$BEARDOG_SOCKET" ]; then
  echo "  ❌ BearDog socket not found after 3s"
  exit 1
fi

echo ""
echo "Phase 2/2: Starting Songbird (bonded to BearDog)..."

# 🧬 GENETIC BONDING - Songbird inherits from BearDog
SONGBIRD_SOCKET="$SONGBIRD_SOCKET" \
SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET" \
SECURITY_ENDPOINT="$BEARDOG_SOCKET" \
SONGBIRD_ORCHESTRATOR_FAMILY_ID="$FAMILY_ID" \
./plasmidBin/primals/songbird/songbird-x86_64-musl server \
  > "$RUNTIME_DIR/songbird-$FAMILY_ID.log" 2>&1 &

SONGBIRD_PID=$!
echo "  ✅ Songbird started (PID: $SONGBIRD_PID)"
echo "  🧬 Bonded to BearDog: $BEARDOG_SOCKET"

# Wait for Songbird socket
echo "  Waiting for Songbird socket..."
for i in {1..30}; do
  if [ -S "$SONGBIRD_SOCKET" ]; then
    echo "  ✅ Songbird socket ready: $SONGBIRD_SOCKET"
    break
  fi
  sleep 0.1
done

if [ ! -S "$SONGBIRD_SOCKET" ]; then
  echo "  ❌ Songbird socket not found after 3s"
  exit 1
fi

echo ""
echo "=========================================="
echo "Tower Atomic Deployed! 🎉"
echo "=========================================="
echo ""
echo "BearDog:  $BEARDOG_SOCKET (PID: $BEARDOG_PID)"
echo "Songbird: $SONGBIRD_SOCKET (PID: $SONGBIRD_PID)"
echo "Family:   $FAMILY_ID"
echo ""
echo "Logs:"
echo "  - $RUNTIME_DIR/beardog-$FAMILY_ID.log"
echo "  - $RUNTIME_DIR/songbird-$FAMILY_ID.log"
echo ""
echo "Bonding:  Covalent (BearDog + Songbird)"
echo "Status:   Tower Atomic Operational ✅"
echo ""
```

### Tower + Squirrel Deployment Script

**File**: `scripts/deploy_tower_squirrel_manual.sh`

```bash
#!/usr/bin/env bash
#
# Tower Atomic + Squirrel Manual Deployment
# Full AI Routing Stack with Genetic Lineage
#
set -euo pipefail

FAMILY_ID="${1:-nat0}"
RUNTIME_DIR="${RUNTIME_DIR:-/tmp}"
ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-}"

if [ -z "$ANTHROPIC_API_KEY" ]; then
  echo "❌ ANTHROPIC_API_KEY not set"
  echo "Usage: export ANTHROPIC_API_KEY=sk-ant-... && $0 [family_id]"
  exit 1
fi

echo "🧬 Deploying Tower Atomic + Squirrel (family_id: $FAMILY_ID)"
echo ""

# Socket paths
BEARDOG_SOCKET="$RUNTIME_DIR/beardog-$FAMILY_ID.sock"
SONGBIRD_SOCKET="$RUNTIME_DIR/songbird-$FAMILY_ID.sock"
SQUIRREL_SOCKET="$RUNTIME_DIR/squirrel-$FAMILY_ID.sock"
NEURAL_API_SOCKET="$RUNTIME_DIR/neural-api-$FAMILY_ID.sock"

# Clean previous deployment
rm -f "$BEARDOG_SOCKET" "$SONGBIRD_SOCKET" "$SQUIRREL_SOCKET"
pkill -f "beardog.*$FAMILY_ID" 2>/dev/null || true
pkill -f "songbird.*$FAMILY_ID" 2>/dev/null || true
pkill -f "squirrel.*$FAMILY_ID" 2>/dev/null || true
sleep 1

echo "Phase 1/3: Starting BearDog (security)..."
./plasmidBin/primals/beardog/beardog-x86_64-musl server \
  --socket "$BEARDOG_SOCKET" \
  --family-id "$FAMILY_ID" \
  > "$RUNTIME_DIR/beardog-$FAMILY_ID.log" 2>&1 &
BEARDOG_PID=$!
echo "  ✅ BearDog started (PID: $BEARDOG_PID)"

# Wait for socket
for i in {1..30}; do
  [ -S "$BEARDOG_SOCKET" ] && break
  sleep 0.1
done
echo "  ✅ BearDog socket ready"
echo ""

echo "Phase 2/3: Starting Songbird (bonded to BearDog)..."
SONGBIRD_SOCKET="$SONGBIRD_SOCKET" \
SONGBIRD_SECURITY_PROVIDER="$BEARDOG_SOCKET" \
SECURITY_ENDPOINT="$BEARDOG_SOCKET" \
SONGBIRD_ORCHESTRATOR_FAMILY_ID="$FAMILY_ID" \
./plasmidBin/primals/songbird/songbird-x86_64-musl server \
  > "$RUNTIME_DIR/songbird-$FAMILY_ID.log" 2>&1 &
SONGBIRD_PID=$!
echo "  ✅ Songbird started (PID: $SONGBIRD_PID)"
echo "  🧬 Bonded to BearDog: $BEARDOG_SOCKET"

# Wait for socket
for i in {1..30}; do
  [ -S "$SONGBIRD_SOCKET" ] && break
  sleep 0.1
done
echo "  ✅ Songbird socket ready"
echo ""

echo "Phase 3/3: Starting Squirrel (inherits from Tower)..."
SERVICE_MESH_ENDPOINT="$NEURAL_API_SOCKET" \
ANTHROPIC_API_KEY="$ANTHROPIC_API_KEY" \
./plasmidBin/primals/squirrel/squirrel-x86_64-musl server \
  --socket "$SQUIRREL_SOCKET" \
  > "$RUNTIME_DIR/squirrel-$FAMILY_ID.log" 2>&1 &
SQUIRREL_PID=$!
echo "  ✅ Squirrel started (PID: $SQUIRREL_PID)"
echo "  🧬 Inherits from Tower Atomic (family: $FAMILY_ID)"

# Wait for socket
for i in {1..30}; do
  [ -S "$SQUIRREL_SOCKET" ] && break
  sleep 0.1
done
echo "  ✅ Squirrel socket ready"
echo ""

echo "=========================================="
echo "Tower Atomic + Squirrel Deployed! 🎉"
echo "=========================================="
echo ""
echo "Tower Atomic (Covalent Bonding):"
echo "  BearDog:  $BEARDOG_SOCKET (PID: $BEARDOG_PID)"
echo "  Songbird: $SONGBIRD_SOCKET (PID: $SONGBIRD_PID)"
echo ""
echo "AI Orchestration (Genetic Lineage):"
echo "  Squirrel: $SQUIRREL_SOCKET (PID: $SQUIRREL_PID)"
echo ""
echo "Family ID: $FAMILY_ID"
echo ""
echo "Communication Flow:"
echo "  Squirrel → Tower (Songbird) → Anthropic API"
echo "  (Secure by default, genetic bonding model)"
echo ""
echo "Test AI call:"
echo "  echo '{\"jsonrpc\":\"2.0\",\"method\":\"ai.chat\","
echo "    \"params\":{\"messages\":[{\"role\":\"user\",\"content\":\"Hello!\"}]},"
echo "    \"id\":1}' | nc -U $SQUIRREL_SOCKET"
echo ""
```

---

## 📐 Phase 2: Complex Graph Specifications

### Tower Atomic Graph (Reference)

**File**: `graphs/tower_atomic_v2.toml`

```toml
# Tower Atomic Deployment Graph v2.0
# Proper DAG with Genetic Bonding Semantics
#
# This is the REFERENCE SPECIFICATION for how Tower Atomic
# should be deployed when the DAG system is evolved.

[graph]
id = "tower_atomic_v2"
version = "2.0.0"
description = "Tower Atomic: BearDog + Songbird with Covalent Bonding"

[graph.bonding]
type = "covalent"  # Shared electrons (Towers)
pattern = "tower_atomic"
genetic_lineage = true

# Phase 1: Start BearDog (Foundation)
[[nodes]]
id = "start-beardog"
primal = { by_capability = "security" }
output = "beardog_socket"

[nodes.operation]
name = "start"
params = { mode = "server", family_id = "${FAMILY_ID}" }

[nodes.constraints]
timeout_ms = 10000
required = true  # Cannot proceed without BearDog

# Phase 2: Start Songbird (Bonded to BearDog)
[[nodes]]
id = "start-songbird"
primal = { by_capability = "discovery" }
output = "songbird_socket"
depends_on = ["start-beardog"]  # MUST wait for BearDog

[nodes.operation]
name = "start"
params = { mode = "server", family_id = "${FAMILY_ID}" }

[nodes.bonding]
# 🧬 GENETIC BONDING - This is the key!
bond_to = "start-beardog"
bond_type = "covalent"
bond_params = {
  SONGBIRD_SECURITY_PROVIDER = "${beardog_socket}",
  SECURITY_ENDPOINT = "${beardog_socket}",
  SONGBIRD_ORCHESTRATOR_FAMILY_ID = "${FAMILY_ID}"
}

[nodes.constraints]
timeout_ms = 10000
required = true

# Phase 3: Validate Tower
[[nodes]]
id = "validate-tower"
output = "tower_validated"
depends_on = ["start-beardog", "start-songbird"]

[nodes.operation]
name = "health_check"
params = {
  check_beardog = true,
  check_songbird = true,
  check_bonding = true  # Verify genetic bond!
}

[nodes.constraints]
timeout_ms = 5000
```

### Node Atomic Graph (Future)

**File**: `graphs/node_atomic_v2.toml`

```toml
# Node Atomic: Tower + ToadStool
# Secure Compute Stack

[graph]
id = "node_atomic_v2"
version = "2.0.0"
description = "Node Atomic: Tower + ToadStool for Secure Compute"

[graph.bonding]
type = "covalent"
pattern = "node_atomic"
extends = "tower_atomic"  # Inherits Tower

# Import Tower Atomic as subgraph
[[subgraphs]]
id = "tower"
source = "tower_atomic_v2.toml"
output = "tower_deployed"

# Add ToadStool on top of Tower
[[nodes]]
id = "start-toadstool"
primal = { by_capability = "compute" }
depends_on = ["tower"]  # Depends on entire Tower subgraph

[nodes.bonding]
bond_to = "tower"
bond_type = "covalent"
inherits_family = true  # Same genetic lineage

[nodes.operation]
name = "start"
params = { mode = "orchestrator", family_id = "${FAMILY_ID}" }
```

### Multi-Tower Environment Graph (Advanced)

**File**: `graphs/multi_tower_environment.toml`

```toml
# Multi-Tower Environment
# Example: Local Tower + Friend's Tower + Squirrel
# Different levels of genetic relatedness

[graph]
id = "multi_tower_env"
version = "2.0.0"
description = "Dynamic environment with multiple Towers"

# Local Tower (family: local-nat0)
[[subgraphs]]
id = "local-tower"
source = "tower_atomic_v2.toml"
params = { FAMILY_ID = "local-nat0" }
output = "local_tower"

# Friend's Tower (family: friend-nat0)
[[subgraphs]]
id = "friend-tower"
source = "tower_atomic_v2.toml"
params = { FAMILY_ID = "friend-nat0" }
output = "friend_tower"
trust_level = "medium"  # Different genetic relatedness!

# Squirrel (connects to both, different trust)
[[nodes]]
id = "squirrel"
primal = { by_capability = "ai" }
depends_on = ["local-tower", "friend-tower"]

[nodes.bonding]
# Squirrel has TWO parent Towers with different trust
primary_bond = "local-tower"  # High trust
secondary_bond = "friend-tower"  # Medium trust
bond_type = "ionic"  # Metered/contract-based for external

[nodes.operation]
name = "start"
params = {
  mode = "multi_tower",
  family_id = "local-nat0",
  external_towers = ["friend-nat0"]
}
```

---

## 🏗️ Phase 3: Deployment System Evolution

### Core Abstractions Needed

#### 1. **Bonding Primitives**

```rust
/// Genetic bonding between primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BondingType {
    /// Covalent: Shared electrons (Towers), high trust
    Covalent {
        shared_resources: Vec<String>,  // e.g., sockets, secrets
        family_id: String,
    },
    
    /// Ionic: Contract-based, metered (e.g., Squirrel → API)
    Ionic {
        contracts: Vec<Contract>,
        metering: MeteringConfig,
    },
    
    /// Metallic: Electron sea, specialized nodes
    Metallic {
        pool_id: String,
        node_type: String,  // e.g., "gpu_render"
    },
    
    /// Weak: Transient, loose coupling
    Weak {
        discovery_only: bool,
    },
}

/// Bonding specification in graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BondingSpec {
    pub bond_to: String,  // Node ID to bond with
    pub bond_type: BondingType,
    pub bond_params: HashMap<String, String>,  // ENV vars, configs
    pub inherits_family: bool,
    pub trust_level: TrustLevel,
}
```

#### 2. **Genetic Lineage Tracking**

```rust
/// Track genetic relationships between deployed primals
#[derive(Debug, Clone)]
pub struct GeneticLineage {
    pub family_id: String,
    pub parent_nodes: Vec<String>,
    pub bonding_type: BondingType,
    pub generation: u32,  // 0 = root, 1 = child, etc.
    pub trust_level: TrustLevel,
}

impl GeneticLineage {
    /// Check if two primals are genetically related
    pub fn is_related(&self, other: &GeneticLineage) -> bool {
        self.family_id == other.family_id
    }
    
    /// Calculate genetic distance (for trust levels)
    pub fn genetic_distance(&self, other: &GeneticLineage) -> u32 {
        if !self.is_related(other) {
            return u32::MAX;  // Unrelated
        }
        self.generation.abs_diff(other.generation)
    }
}
```

#### 3. **DAG Execution Engine v2**

```rust
/// Proper topological sort with bonding support
pub struct DagExecutor {
    graph: Graph,
    lineage_tracker: GeneticLineageTracker,
    bonding_manager: BondingManager,
}

impl DagExecutor {
    /// Build execution phases with bonding dependencies
    pub async fn build_phases(&self) -> Result<Vec<ExecutionPhase>> {
        // 1. Topological sort (respecting depends_on)
        let phases = self.topological_sort()?;
        
        // 2. Add bonding dependencies
        let phases = self.inject_bonding_waits(phases)?;
        
        // 3. Validate genetic lineage
        self.validate_lineage(&phases)?;
        
        Ok(phases)
    }
    
    /// Execute with proper bonding
    pub async fn execute(&mut self) -> Result<ExecutionResult> {
        let phases = self.build_phases().await?;
        
        for (i, phase) in phases.iter().enumerate() {
            info!("📍 Phase {}/{}: {} nodes", i+1, phases.len(), phase.nodes.len());
            
            // Execute phase nodes in parallel
            let results = self.execute_phase(phase).await?;
            
            // Wait for bonding to establish
            self.wait_for_bonding(phase).await?;
            
            // Track genetic lineage
            self.update_lineage(phase, &results)?;
        }
        
        Ok(ExecutionResult::success())
    }
    
    /// Wait for bonding to establish between primals
    async fn wait_for_bonding(&self, phase: &ExecutionPhase) -> Result<()> {
        for node in &phase.nodes {
            if let Some(bonding) = &node.bonding {
                // Wait for parent socket to exist
                self.bonding_manager
                    .wait_for_bond(bonding)
                    .await?;
            }
        }
        Ok(())
    }
}
```

#### 4. **Subgraph Support**

```rust
/// Subgraph inclusion for composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subgraph {
    pub id: String,
    pub source: String,  // Path to graph file
    pub params: HashMap<String, String>,  // Parameter overrides
    pub output: String,  // Subgraph output name
    pub trust_level: Option<TrustLevel>,
}

impl Graph {
    /// Expand subgraphs into main graph
    pub fn expand_subgraphs(&mut self) -> Result<()> {
        for subgraph in &self.subgraphs {
            let sub_graph = Graph::from_toml_file(&subgraph.source)?;
            
            // Substitute parameters
            let sub_graph = self.substitute_params(sub_graph, &subgraph.params)?;
            
            // Merge into main graph with namespace
            self.merge_subgraph(subgraph.id.clone(), sub_graph)?;
        }
        Ok(())
    }
}
```

---

## 📊 Evolution Roadmap

### Milestone 1: Pinned Deployments (This Week)
- ✅ Create manual deployment scripts
- ✅ Document bonding semantics
- ✅ Enable team to deploy Tower + Squirrel
- **Status**: Ready to implement

### Milestone 2: Graph Specifications (Next Week)
- Define all atomic pattern graphs
- Document bonding models
- Create reference implementations
- Validate with existing primals

### Milestone 3: Bonding Primitives (Week 3)
- Implement `BondingType` enum
- Create `BondingManager`
- Add genetic lineage tracking
- Test with Tower Atomic

### Milestone 4: DAG Engine v2 (Week 4)
- Fix topological sort
- Add bonding wait logic
- Implement subgraph expansion
- Full integration tests

### Milestone 5: Production Ready (Month 2)
- Advanced features (rollback, checkpointing)
- Multi-environment support
- Monitoring and metrics
- Documentation and examples

---

## 💡 Key Insights

### Why This Is the Right Approach

1. **Pin Now, Evolve Later**
   - Team can deploy TODAY with scripts
   - No blocked development
   - Real-world validation of bonding model

2. **Deep Debt, Not Technical Debt**
   - Proper architecture from ground up
   - Bonding as first-class primitive
   - Genetic lineage built-in

3. **Complex Graphs Enable Composition**
   - Tower → Node → Nest progression
   - Subgraph reuse (DRY)
   - Multi-environment modeling

4. **Abstraction Follows Understanding**
   - We NOW understand bonding requirements
   - DAG needs are clear from failures
   - Can design proper solution

### The Bonding Model Is the Innovation

**This isn't just "deploy some services"** - it's a **genetic bonding model** where:
- Services have genetic relationships (family_id)
- Bonding types affect trust and communication
- Dynamic environments compose multiple lineages
- Security is genetic, not configured

**This is worth doing right!**

---

## 🚀 Immediate Next Steps

### Today (30 minutes)
1. Create `scripts/deploy_tower_atomic_manual.sh`
2. Create `scripts/deploy_tower_squirrel_manual.sh`
3. Test manual deployment
4. Document success in handoff

### This Week
1. Define all atomic pattern graphs (Tower, Node, Nest)
2. Document bonding semantics
3. Create reference manual deployments
4. Enable all teams to deploy

### Next Session
1. Start bonding primitives implementation
2. Design DAG Engine v2 architecture
3. Plan subgraph expansion logic
4. Create evolution milestones

---

## 📚 Documentation Structure

```
ecoPrimals/wateringHole/
├── DEPLOYMENT_ARCHITECTURE_STANDARD.md
│   ├── Bonding Types (Covalent, Ionic, Metallic, Weak)
│   ├── Genetic Lineage Model
│   ├── DAG Execution Semantics
│   └── Graph Composition Patterns
│
├── BONDING_SPECIFICATION.md
│   ├── Bonding Primitives
│   ├── Trust Levels
│   ├── Parameter Inheritance
│   └── Multi-Tower Environments
│
└── GRAPH_REFERENCE_GUIDE.md
    ├── Graph Structure (TOML schema)
    ├── Subgraph Inclusion
    ├── Parameter Substitution
    └── Examples (all atomic patterns)
```

---

**Status**: 🎯 **Strategic Plan Complete**  
**Approach**: Deep Debt Solution - Proper Evolution  
**Timeline**: Pinned deployments today, evolved system in 4-6 weeks  
**Team Impact**: Unblocked immediately, production-ready long-term

This is how you build systems that last! 🚀

---

**Date**: January 20, 2026  
**Version**: biomeOS v0.28.0  
**Next**: Create manual deployment scripts and test!


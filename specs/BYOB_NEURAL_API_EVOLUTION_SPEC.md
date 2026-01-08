# 🧬 BYOB Evolution: From Static Manifests to Neural API

**Version:** 1.0.0  
**Date:** January 8, 2026  
**Status:** 🎯 **Evolution Specification**  
**Target:** biomeOS v0.2.0+

---

## 🎯 Overview

This specification defines how the **BYOB (Build Your Own Biome)** manifest system evolves from static primal lists to **graph-based coordination** compatible with the Neural API architecture.

---

## 📊 Current State: Static Manifests

### **Example: Tower Niche (Current)**

```toml
# niches/tower.toml (CURRENT)

[niche]
name = "tower"
type = "communication"

[[primals]]
binary = "./primals/songbird-orchestrator"
provides = ["discovery", "federation"]

[[primals]]
binary = "./primals/beardog-server"
provides = ["security", "encryption"]

[[primals]]
binary = "./primals/biomeos-spore"
provides = ["orchestration"]
```

**Limitations:**
- No coordination definition (just a list)
- Fixed spawn order (implicit sequential)
- No parallelization
- No conditional logic
- No learning/adaptation

---

## 🚀 Target State: Graph-Enriched Manifests

### **Example: Tower Niche (Evolved)**

```toml
# niches/tower.toml (EVOLVED)

[niche]
name = "tower"
version = "2.0.0"
type = "communication"
family_seed_file = "./.family.seed"

# =============================================================================
# PRIMALS (Same as before - backward compatible)
# =============================================================================

[[primals]]
binary = "./primals/songbird-orchestrator"
provides = ["discovery", "federation", "p2p"]
requires = []

[primals.env]
SONGBIRD_NODE_ID = "${NODE_ID}"
SONGBIRD_FAMILY_ID = "${FAMILY_ID}"

[[primals]]
binary = "./primals/beardog-server"
provides = ["security", "encryption", "genetic-lineage"]
requires = []

[primals.env]
BEARDOG_NODE_ID = "${NODE_ID}"
BEARDOG_FAMILY_ID = "${FAMILY_ID}"

# =============================================================================
# GRAPHS (NEW! Coordination patterns)
# =============================================================================

# Graph 1: Deploy tower
[[graphs]]
name = "deploy"
description = "Deploy a complete tower with federation"
coordination = "ConditionalDAG"

# Start Songbird first (discovery foundation)
[[graphs.nodes]]
id = "start-songbird"
primal = { by_id = "songbird" }
operation = { name = "start", params = {} }
output = "songbird_status"
constraints = { timeout_ms = 30000, retry = { max_attempts = 3, backoff_ms = 1000 } }

# Start BearDog (needs Songbird for discovery)
[[graphs.nodes]]
id = "start-beardog"
primal = { by_id = "beardog" }
operation = { name = "start", params = {} }
output = "beardog_status"
constraints = { timeout_ms = 30000, retry = { max_attempts = 3, backoff_ms = 1000 } }

# Verify genetic lineage
[[graphs.nodes]]
id = "verify-lineage"
primal = { by_id = "beardog" }
operation = { 
    name = "federation.verify_family_member",
    params = { 
        family_id = "${FAMILY_ID}",
        seed_hash = "${SEED_HASH}",
        node_id = "${NODE_ID}"
    }
}
output = "lineage_verified"
constraints = { timeout_ms = 5000 }

# Discover other towers
[[graphs.nodes]]
id = "discover-towers"
primal = { by_id = "songbird" }
operation = {
    name = "discover_by_family",
    params = {
        family_tags = ["${FAMILY_ID}"],
        timeout_ms = 5000
    }
}
output = "discovered_towers"
constraints = { timeout_ms = 10000 }

# Create BTSP tunnels to discovered towers
[[graphs.nodes]]
id = "create-tunnels"
primal = { by_id = "songbird" }
operation = {
    name = "create_genetic_tunnel",
    params = {
        peers = "${discovered_towers}",
        genetic_proof = "${lineage_verified}"
    }
}
output = "tunnels_established"
constraints = { timeout_ms = 30000 }

# Announce capabilities
[[graphs.nodes]]
id = "announce"
primal = { by_id = "songbird" }
operation = {
    name = "announce_capabilities",
    params = {
        capabilities = ["federation", "discovery", "p2p"],
        sub_federations = ["${SUB_FEDS}"],
        genetic_families = ["${FAMILY_ID}"]
    }
}
constraints = { timeout_ms = 5000 }

# Dependencies
[[graphs.edges]]
from = "start-songbird"
to = "start-beardog"
edge_type = "dependency"

[[graphs.edges]]
from = "start-beardog"
to = "verify-lineage"
edge_type = "dependency"

[[graphs.edges]]
from = "start-songbird"
to = "discover-towers"
edge_type = "dependency"

[[graphs.edges]]
from = "verify-lineage"
to = "create-tunnels"
edge_type = { data_flow = "lineage_verified" }

[[graphs.edges]]
from = "discover-towers"
to = "create-tunnels"
edge_type = { data_flow = "discovered_towers" }

[[graphs.edges]]
from = "create-tunnels"
to = "announce"
edge_type = "dependency"

# Graph 2: Health check (parallel)
[[graphs]]
name = "health-check"
description = "Check health of all tower components"
coordination = "Parallel"

[[graphs.nodes]]
id = "check-songbird"
primal = { by_id = "songbird" }
operation = { name = "health_check" }
parallel_group = 1

[[graphs.nodes]]
id = "check-beardog"
primal = { by_id = "beardog" }
operation = { name = "health_check" }
parallel_group = 1

# =============================================================================
# WORKFLOWS (NEW! High-level operations)
# =============================================================================

[workflows.deploy]
graph = "deploy"
triggers = ["manual", "usb-spore-inserted"]
description = "Deploy tower from spore"

[workflows.health]
graph = "health-check"
triggers = ["cron:*/5 * * * *"]  # Every 5 minutes
description = "Periodic health monitoring"

[workflows.federation]
graph = "discover-and-connect"
triggers = ["network-change", "tower-discovered"]
description = "Maintain federation mesh"
```

---

## 🏗️ Evolution Principles

### **1. Backward Compatibility**

**Old manifests still work:**
```toml
# OLD FORMAT (still valid)
[[primals]]
binary = "./primals/songbird"
provides = ["discovery"]

# System creates implicit sequential graph
```

**Evolution:**
- Parser detects missing `[[graphs]]` section
- Generates default sequential graph
- Works exactly as before
- No breaking changes!

### **2. Progressive Enhancement**

**Level 1: Just add graphs**
```toml
[[primals]]
binary = "./primals/songbird"

[[graphs]]
name = "deploy"
coordination = "Sequential"  # Same as default, but explicit
[[graphs.nodes]]
id = "start"
primal = { by_id = "songbird" }
operation = { name = "start" }
```

**Level 2: Add parallelization**
```toml
[[graphs]]
coordination = "Parallel"  # NOW it's faster!
```

**Level 3: Add complex DAG**
```toml
[[graphs]]
coordination = "ConditionalDAG"
# Full power of graph execution
```

### **3. Composability**

**Reference graphs from other niches:**
```toml
# nest.toml
[[graphs]]
name = "compress-and-store"
extends = "tower::deploy"  # Inherit from tower niche
additional_nodes = [...]
```

**Include common patterns:**
```toml
[[graphs]]
name = "deploy"
includes = ["common::health-check", "common::discovery"]
nodes = [...]
```

---

## 📋 New Manifest Sections

### **1. [[graphs]] Section**

```toml
[[graphs]]
name = "operation-name"
description = "What this graph does"
version = "1.0.0"
coordination = "Sequential|Parallel|ConditionalDAG|Pipeline"

# Nodes define primal operations
[[graphs.nodes]]
id = "unique-node-id"
primal = { by_id = "primal-name" | by_capability = "capability" }
operation = { name = "method", params = { key = "value" } }
output = "variable_name"  # Optional
constraints = { timeout_ms = 5000, retry = {...} }

# Edges define dependencies
[[graphs.edges]]
from = "node-1"
to = "node-2"
edge_type = "dependency" | { data_flow = "variable" }
```

### **2. [workflows] Section**

```toml
[workflows.operation-name]
graph = "graph-to-execute"
triggers = ["manual", "cron:* * * * *", "event:name"]
description = "When and why to run this"
enabled = true
conditions = { only_if = "${CONDITION}" }
```

### **3. [learning] Section (Future)**

```toml
[learning]
enabled = true
metrics_retention_days = 90
optimization_strategy = "latency" | "throughput" | "cost"

[learning.pathways]
allow_reordering = true
allow_parallelization = true
allow_caching = true
```

---

## 🔄 Migration Examples

### **Example 1: Simple Niche → Graph**

**Before:**
```toml
[[primals]]
binary = "./primals/toadstool"

[[primals]]
binary = "./primals/beardog"
```

**After:**
```toml
# Primals stay the same
[[primals]]
binary = "./primals/toadstool"

[[primals]]
binary = "./primals/beardog"

# Add explicit graph
[[graphs]]
name = "deploy"
coordination = "Sequential"

[[graphs.nodes]]
id = "start-toadstool"
primal = { by_id = "toadstool" }
operation = { name = "start" }

[[graphs.nodes]]
id = "start-beardog"
primal = { by_id = "beardog" }
operation = { name = "start" }

[[graphs.edges]]
from = "start-toadstool"
to = "start-beardog"
```

### **Example 2: Enable Parallelization**

**Before:**
```toml
# Sequential by default
[[primals]]
binary = "./primals/songbird"

[[primals]]
binary = "./primals/beardog"

[[primals]]
binary = "./primals/nestgate"
```

**After:**
```toml
# Same primals
[[primals]]
binary = "./primals/songbird"

[[primals]]
binary = "./primals/beardog"

[[primals]]
binary = "./primals/nestgate"

# All start in parallel!
[[graphs]]
name = "deploy"
coordination = "Parallel"

[[graphs.nodes]]
id = "start-songbird"
primal = { by_id = "songbird" }
operation = { name = "start" }
parallel_group = 1

[[graphs.nodes]]
id = "start-beardog"
primal = { by_id = "beardog" }
operation = { name = "start" }
parallel_group = 1

[[graphs.nodes]]
id = "start-nestgate"
primal = { by_id = "nestgate" }
operation = { name = "start" }
parallel_group = 1
```

**Result:** 3x faster deployment!

### **Example 3: Complex DAG with Data Flow**

```toml
[[graphs]]
name = "encrypted-storage"
description = "Store data with compression and encryption"
coordination = "ConditionalDAG"

# 1. Compress data (NestGate)
[[graphs.nodes]]
id = "compress"
primal = { by_capability = "compression" }
operation = {
    name = "adaptive_compress",
    params = { data = "${INPUT_DATA}" }
}
output = "compressed_data"

# 2. Encrypt (BearDog) - can run parallel with 3
[[graphs.nodes]]
id = "encrypt"
primal = { by_capability = "encryption" }
operation = {
    name = "encrypt",
    params = { plaintext = "${compressed_data}" }
}
output = "encrypted_data"

# 3. Generate provenance (BearDog) - can run parallel with 2
[[graphs.nodes]]
id = "provenance"
primal = { by_capability = "provenance" }
operation = {
    name = "create_provenance",
    params = { data_hash = "${compressed_data_hash}" }
}
output = "provenance_record"

# 4. Store (NestGate) - waits for both 2 and 3
[[graphs.nodes]]
id = "store"
primal = { by_capability = "storage" }
operation = {
    name = "store_blob",
    params = {
        data = "${encrypted_data}",
        metadata = "${provenance_record}"
    }
}
output = "storage_location"

# Dependencies
[[graphs.edges]]
from = "compress"
to = "encrypt"
edge_type = { data_flow = "compressed_data" }

[[graphs.edges]]
from = "compress"
to = "provenance"
edge_type = { data_flow = "compressed_data_hash" }

[[graphs.edges]]
from = "encrypt"
to = "store"
edge_type = { data_flow = "encrypted_data" }

[[graphs.edges]]
from = "provenance"
to = "store"
edge_type = { data_flow = "provenance_record" }
```

**Result:** Encrypt + Provenance happen in parallel (faster!)

---

## 🎯 Implementation Phases

### **Phase 1: Parser Extension**
- ✅ Extend TOML parser to support `[[graphs]]` section
- ✅ Make it optional (backward compatible)
- ✅ Validate graph structure

### **Phase 2: Graph Execution**
- ✅ Implement graph executor (from GRAPH_BASED_ORCHESTRATION_SPEC)
- ✅ Integrate with niche deployment
- ✅ Test with simple graphs

### **Phase 3: Advanced Coordination**
- ⏳ Parallel execution
- ⏳ DAG execution
- ⏳ Data flow between nodes

### **Phase 4: Workflows**
- ⏳ Parse `[workflows]` section
- ⏳ Trigger system (cron, events, manual)
- ⏳ Workflow scheduling

### **Phase 5: Learning**
- ⏳ Metrics collection from graph execution
- ⏳ Pathway optimization
- ⏳ Automatic evolution

---

## 📦 Updated Crate Structure

```
crates/
├── biomeos-manifest/        # UPDATED! Now parses graphs
│   ├── src/
│   │   ├── lib.rs
│   │   ├── parser.rs        # Extended for graphs
│   │   ├── niche.rs         # Niche data structures
│   │   ├── graph.rs         # NEW! Graph parsing
│   │   └── workflow.rs      # NEW! Workflow parsing
│   └── tests/
│       ├── backward_compat_tests.rs  # Ensure old format works
│       └── graph_tests.rs            # New graph parsing tests
```

---

## 🎊 Success Criteria

### **Backward Compatibility:**
- ✅ All existing niche manifests work without changes
- ✅ Implicit sequential graph generated for old format
- ✅ No breaking changes

### **Progressive Enhancement:**
- ✅ Can add `[[graphs]]` to existing manifests
- ✅ Can start with simple graphs, evolve to complex
- ✅ Can mix old and new styles

### **New Capabilities:**
- ✅ Parallel execution works
- ✅ DAG execution works
- ✅ Data flow between nodes works
- ✅ Metrics collected for learning

---

## 🔄 Migration Guide

### **For Users:**

**Step 1: Current manifests work as-is**
- No action needed!
- System generates implicit graphs

**Step 2: Want parallelization?**
- Add `[[graphs]]` section
- Set `coordination = "Parallel"`
- Group nodes with `parallel_group`

**Step 3: Want complex coordination?**
- Use `coordination = "ConditionalDAG"`
- Define explicit dependencies
- Enable data flow between nodes

### **For Developers:**

**Step 1: Update parser**
```rust
// Support optional [[graphs]] section
let graphs = manifest.get("graphs").and_then(|v| {
    v.as_array().map(|arr| parse_graphs(arr))
}).unwrap_or_else(|| generate_implicit_graph(&manifest));
```

**Step 2: Integrate with executor**
```rust
// Execute graph instead of sequential spawn
if let Some(graphs) = niche.graphs {
    graph_executor.execute(graphs.get("deploy")?).await?;
} else {
    // Fallback to old behavior
    for primal in niche.primals {
        spawn_primal(primal).await?;
    }
}
```

---

**Status:** 🎯 **Ready for Implementation**  
**Next:** Implement parser extensions + backward compatibility tests

🧬 **From static lists → adaptive graphs!** 🎊


# 🧠 Neural API Implementation Roadmap

**Goal:** Evolve biomeOS from static wave orchestration to adaptive graph-based Neural API  
**Target:** Enable Tower → Node → Nest composition as backbone for RootPulse  
**Status:** 🎯 **In Progress**

---

## 🎯 Core Milestones

```
Milestone 1: Tower Niche (Current Target) ⏳
    → Replicate current tower deployment on liveSpore
    → Graph-based orchestration working
    → Deploy Songbird + BearDog + biomeOS via graphs
    
Milestone 2: Node Niche (Next) 🔜
    → Compose with Toadstool binary
    → Fractal compute deployments
    → Multi-node coordination
    
Milestone 3: Nest Niche (Future) 🔮
    → Compose with NestGate binary
    → Data provenance tracking
    → Sharded storage federation
    
Milestone 4: Complete Backbone ✨
    → Tower + Node + Nest working together
    → Full ecosystem operational
    → Ready for RootPulse coordination
```

---

## 📊 Current Status

### **Foundation Complete** ✅
- [x] 4 niche architectures designed (Tower, Node, Nest, Gate)
- [x] Fractal compute implemented (biomeos-compute)
- [x] Deployment manifests created
- [x] Basement HPC infrastructure planned
- [x] Primal integration (100% - BearDog + Songbird)

### **Specs Written** ✅
- [x] `specs/GRAPH_BASED_ORCHESTRATION_SPEC.md`
- [x] `specs/BYOB_NEURAL_API_EVOLUTION_SPEC.md`
- [x] `docs/jan4-session/LATE_STAGE_NEURAL_ROOTPULSE_JAN8.md`

### **Next: Implementation** ⏳
- [ ] **Milestone 1: Tower Niche via Neural API**

---

## 🏗️ Milestone 1: Tower Niche (CURRENT TARGET)

**Goal:** Deploy tower niche using graph-based orchestration on liveSpore

### **Phase 1.1: Graph Executor Foundation** ✅ **COMPLETE!**

**Tasks:**
- [x] Create `crates/biomeos-graph/` crate
- [x] Implement core data structures:
  - [x] `PrimalGraph` struct
  - [x] `GraphNode` struct  
  - [x] `GraphEdge` struct
  - [x] `CoordinationPattern` enum
- [x] Implement `GraphParser` (TOML → Graph)
- [x] Implement `GraphValidator` (check cycles, refs)
- [x] Implement `GraphExecutor::execute_sequential()`
- [x] Basic unit tests (15 passing)
- [x] Integration tests (3 passing)

**Acceptance Criteria:**
```rust
// Can parse a graph from TOML
let graph = GraphParser::parse_toml("graphs/test.toml")?;

// Can validate it
GraphValidator::validate(&graph)?;

// Can execute it sequentially
let result = GraphExecutor::new(registry).execute(graph).await?;
assert!(result.success);
```

**Files to Create:**
- `crates/biomeos-graph/Cargo.toml`
- `crates/biomeos-graph/src/lib.rs`
- `crates/biomeos-graph/src/graph.rs`
- `crates/biomeos-graph/src/parser.rs`
- `crates/biomeos-graph/src/validator.rs`
- `crates/biomeos-graph/src/executor.rs`
- `crates/biomeos-graph/tests/sequential_tests.rs`

**Estimated Time:** 2-3 sessions

---

### **Phase 1.2: Tower Graph Definition** ✅ **COMPLETE!**

**Tasks:**
- [x] Create `graphs/tower_deploy.toml`
- [x] Define deployment graph:
  - [x] Start Songbird node
  - [x] Start BearDog node
  - [x] Verify genetic lineage node
  - [x] Create BTSP tunnel node
  - [x] Announce capabilities node
- [x] Define dependencies between nodes
- [x] Test parsing and validation
- [x] Create additional graphs (health_check, shutdown)

**Acceptance Criteria:**
```toml
# graphs/tower_deploy.toml successfully parses
[graph]
name = "deploy-tower"
coordination = "Sequential"

[[nodes]]
id = "start-songbird"
primal = { by_id = "songbird" }
operation = { name = "start" }
# ... etc
```

**Files to Create:**
- `graphs/tower_deploy.toml`
- `graphs/tower_health_check.toml`
- `graphs/tower_shutdown.toml`

**Actual Time:** < 1 session ✅  
**Status:** COMPLETE - Jan 8, 2026

---

### **Phase 1.3: BYOB Manifest Evolution** ✅ **COMPLETE!**

**Tasks:**
- [x] Extend `biomeos-manifest` crate to parse `[[graphs]]`
- [x] Make `[[graphs]]` optional (backward compatible)
- [x] Create `niches/tower.toml` with graphs
- [x] Capability-based validation
- [x] Test backward compatibility (19 tests passing)

**Acceptance Criteria:**
```rust
// Old format still works
let old_manifest = parse_manifest("old_tower.toml")?;
assert!(old_manifest.graphs.is_empty()); // Generates implicit

// New format with graphs works
let new_manifest = parse_manifest("niches/tower.toml")?;
assert!(new_manifest.graphs.len() > 0);
```

**Files to Modify:**
- `crates/biomeos-manifest/src/parser.rs`
- `crates/biomeos-manifest/src/niche.rs`
- `niches/tower.toml` (add `[[graphs]]` section)

**Files to Create:**
- `crates/biomeos-manifest/src/graph.rs`
- `crates/biomeos-manifest/tests/backward_compat_tests.rs`

**Estimated Time:** 1-2 sessions

---

### **Phase 1.4: Integration & Deployment** 🎯 **IN PROGRESS**

**Tasks:**
- [x] Create `PrimalRegistry` for runtime discovery
- [x] Integrate `GraphExecutor` with biomeOS core
- [x] Create `GraphDeploymentCoordinator`  
- [x] Unit tests for integration layer
- [ ] CLI command integration (`biomeos deploy`)
- [ ] Test with real primals (Songbird + BearDog)
- [ ] Deploy to liveSpore USB
- [ ] Full E2E federation test

**Acceptance Criteria:**
```bash
# Deploy tower using new graph system
biomeos deploy --niche tower --usb /media/liveSpore1

# Should work identically to old system
# But now using graph executor under the hood
```

**Files to Modify:**
- `crates/biomeos-core/src/orchestrator.rs`
- `crates/biomeos-spore/src/deployment.rs`

**Estimated Time:** 2 sessions

---

### **Phase 1.5: Metrics Collection** ⏳

**Tasks:**
- [ ] Implement `MetricsCollector`
- [ ] Track node execution times
- [ ] Track success/failure rates
- [ ] Store metrics to SQLite
- [ ] Create metrics query API
- [ ] Visualize bottlenecks

**Acceptance Criteria:**
```rust
// Can collect metrics during execution
let metrics = executor.execute_with_metrics(graph).await?;

// Can query historical metrics
let history = metrics_collector.get_metrics(&graph.id).await?;
assert!(history.len() > 0);

// Can identify bottlenecks
let bottleneck = metrics.find_slowest_node();
println!("Bottleneck: {} took {}ms", bottleneck.node_id, bottleneck.duration_ms);
```

**Files to Create:**
- `crates/biomeos-graph/src/metrics.rs`
- `crates/biomeos-graph/src/storage.rs`

**Estimated Time:** 1-2 sessions

---

### **Milestone 1 Complete When:** ✅
- [ ] Tower niche deploys via graph executor
- [ ] Old tower deployments still work (backward compatible)
- [ ] Metrics are collected
- [ ] Performance is equal or better than wave system
- [ ] All tests passing
- [ ] Deployed and verified on liveSpore

**Total Estimated Time:** 7-11 sessions

---

## 🖥️ Milestone 2: Node Niche (NEXT)

**Goal:** Compose compute nodes using Toadstool binary via Neural API

### **Phase 2.1: Parallel Execution** 🔜

**Tasks:**
- [ ] Implement `GraphExecutor::execute_parallel()`
- [ ] Support `parallel_group` in graph nodes
- [ ] Test parallel node startup
- [ ] Measure speedup vs sequential

**Acceptance Criteria:**
- Multiple primals start simultaneously
- Execution time reduced proportionally
- All primals healthy after startup

**Estimated Time:** 2 sessions

---

### **Phase 2.2: Node Graph Definitions** 🔜

**Tasks:**
- [ ] Create `graphs/node_deploy.toml`
- [ ] Support fractal node configurations
- [ ] Define node coordination patterns
- [ ] Test multi-node deployments

**Files to Create:**
- `graphs/node_deploy.toml`
- `graphs/node_fractal_binary.toml`
- `graphs/node_fractal_nary.toml`

**Estimated Time:** 1-2 sessions

---

### **Phase 2.3: Integration with Toadstool** 🔜

**Tasks:**
- [ ] Update `niches/compute-node.toml` with graphs
- [ ] Implement node-to-nest communication graphs
- [ ] Test compute-to-data workflows
- [ ] Deploy multiple nodes via graphs

**Estimated Time:** 2-3 sessions

---

### **Milestone 2 Complete When:** ✅
- [ ] Compute nodes deploy via graph executor
- [ ] Parallel execution working
- [ ] Fractal topologies supported
- [ ] Node-to-nest coordination functional

**Total Estimated Time:** 5-8 sessions

---

## 🗄️ Milestone 3: Nest Niche (FUTURE)

**Goal:** Integrate data nests with provenance and sharding via Neural API

### **Phase 3.1: DAG Execution** 🔮

**Tasks:**
- [ ] Implement `GraphExecutor::execute_dag()`
- [ ] Support conditional branches
- [ ] Data flow between nodes
- [ ] Test complex DAG patterns

**Estimated Time:** 3-4 sessions

---

### **Phase 3.2: Nest Graph Definitions** 🔮

**Tasks:**
- [ ] Create `graphs/nest_store.toml`
- [ ] Create `graphs/nest_retrieve.toml`
- [ ] Create `graphs/nest_shard.toml`
- [ ] Implement provenance tracking graphs

**Files to Create:**
- `graphs/nest_store.toml`
- `graphs/nest_retrieve.toml`
- `graphs/nest_shard.toml`
- `graphs/nest_provenance.toml`

**Estimated Time:** 2-3 sessions

---

### **Phase 3.3: Integration with NestGate** 🔮

**Tasks:**
- [ ] Update `niches/nest.toml` with graphs
- [ ] Implement compress-encrypt-store pipeline
- [ ] Test sharding across nests
- [ ] Verify provenance tracking

**Estimated Time:** 3-4 sessions

---

### **Milestone 3 Complete When:** ✅
- [ ] Nest niche deploys via graph executor
- [ ] DAG execution working
- [ ] Data pipelines functional (compress→encrypt→store)
- [ ] Sharding working across federation
- [ ] Provenance tracked end-to-end

**Total Estimated Time:** 8-11 sessions

---

## ✨ Milestone 4: Complete Backbone

**Goal:** Tower + Node + Nest working together as integrated system

### **Phase 4.1: End-to-End Workflows** 🔮

**Tasks:**
- [ ] Create `graphs/e2e_compute_on_data.toml`
- [ ] Tower discovers Node and Nest
- [ ] Node moves to Nest (compute-to-data)
- [ ] Result returns through Tower
- [ ] Test full workflow

**Estimated Time:** 2-3 sessions

---

### **Phase 4.2: Learning Engine (Optional)** 🔮

**Tasks:**
- [ ] Implement `PathwayLearner`
- [ ] Discover patterns from metrics
- [ ] Automatic graph optimization
- [ ] A/B testing of pathways

**Estimated Time:** 4-6 sessions

---

### **Milestone 4 Complete When:** ✅
- [ ] All 3 niches operational via Neural API
- [ ] End-to-end workflows functional
- [ ] Performance optimized
- [ ] System ready for RootPulse coordination

**Total Estimated Time:** 6-9 sessions

---

## 🌳 Beyond: RootPulse (FUTURE)

**Prerequisites:** Milestone 4 complete

**What RootPulse Needs:**
- Tower + Node + Nest backbone ✅ (after Milestone 4)
- rhizoCrypt primal mature
- LoamSpine primal mature
- SweetGrass primal mature
- Complex multi-primal coordination graphs
- Advanced DAG execution with data flow

**Estimated Start:** After Milestone 4 complete + primals mature

---

## 📊 Progress Tracking

### **Overall Progress**

| Milestone | Status | Progress | Est. Sessions |
|-----------|--------|----------|---------------|
| **M1: Tower** | 🎯 In Progress | 57% | 7-11 (4/7 done) |
| **M2: Node** | 🔜 Next | 0% | 5-8 |
| **M3: Nest** | 🔮 Future | 0% | 8-11 |
| **M4: Backbone** | 🔮 Future | 0% | 6-9 |
| **RootPulse** | 🔮 Long-term | 0% | TBD |

**Total Estimated:** 26-39 sessions before RootPulse

---

## 📝 Current Session Focus

### **Next Steps (Immediate)**

1. **Create `biomeos-graph` crate** ⏳
   - Set up crate structure
   - Add to workspace
   - Define core data structures

2. **Implement Graph Parser** ⏳
   - Parse TOML graph definitions
   - Validate structure
   - Unit tests

3. **Implement Sequential Executor** ⏳
   - Execute nodes in order
   - Context passing between nodes
   - Error handling

4. **Test with Simple Graph** ⏳
   - Create test graph
   - Execute successfully
   - Verify results

---

## 🎯 Success Criteria

### **Milestone 1 (Tower) Success:**
```bash
# Old way (wave system)
biomeos deploy --niche tower

# New way (graph system)
biomeos deploy --niche tower  # Same command!

# Under the hood: Using graph executor
# Performance: Equal or better
# Functionality: Identical
# Metrics: Collected for learning
```

### **Milestone 2 (Node) Success:**
```bash
# Deploy fractal compute nodes
biomeos deploy --niche compute-node --topology fractal

# Multiple nodes start in parallel
# Resource aggregation works
# Workload distribution functional
```

### **Milestone 3 (Nest) Success:**
```bash
# Deploy data nests
biomeos deploy --niche nest --with-sharding

# DAG execution working
# Compress→Encrypt→Store pipeline
# Provenance tracked
# Sharding across federation
```

### **Milestone 4 (Backbone) Success:**
```bash
# Full ecosystem operational
biomeos e2e-test compute-on-data

# Tower → Node → Nest coordination
# Compute moves to data
# Results return efficiently
# All metrics collected
```

---

## 📚 Related Documentation

- `specs/GRAPH_BASED_ORCHESTRATION_SPEC.md` - Technical details
- `specs/BYOB_NEURAL_API_EVOLUTION_SPEC.md` - Manifest evolution
- `docs/jan4-session/LATE_STAGE_NEURAL_ROOTPULSE_JAN8.md` - Full architecture
- `docs/jan4-session/COMPLETE_ECOSYSTEM_ARCHITECTURE_JAN8.md` - Niche overview

---

## 🎊 Vision

```
TODAY:
  Static wave orchestration
  Manual primal spawning
  No learning

MILESTONE 1:
  Graph-based execution
  Sequential coordination
  Metrics collection

MILESTONE 2:
  Parallel execution
  Multi-node coordination
  Performance optimization

MILESTONE 3:
  DAG execution
  Complex pipelines
  Data flow graphs

MILESTONE 4:
  Complete backbone
  Adaptive learning
  Automatic optimization

FUTURE (RootPulse):
  Emergent version control
  6-primal coordination
  Universal time tracking
```

**Status:** 🎯 **Phase 1.4 IN PROGRESS** (Integration layer complete!)  
**Next:** CLI integration & real deployment testing

🧠 **From static waves → adaptive intelligence!** 🎊


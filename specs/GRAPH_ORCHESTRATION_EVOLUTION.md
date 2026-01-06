# 🔄 Graph Orchestration Evolution Spec

**Evolving from wave-based to graph-based primal orchestration**

**Date**: January 4, 2026  
**Status**: Implementation Specification  
**Priority**: 🔴 HIGH - Foundation for Neural API

---

## 🎯 Goal

Replace the current **wave-based** concurrent startup with a more robust **graph-based** orchestration system that enables:
1. More fine-grained concurrency (start as soon as deps satisfied)
2. Explicit dependency tracking (data flow between primals)
3. Foundation for learning (track metrics, discover patterns)
4. Better USB spore deployment robustness

---

## 📊 Current State: Wave-Based

### **How It Works Today**

```rust
// Current: concurrent_startup.rs
pub async fn start_in_waves(
    orchestrator: &Arc<PrimalOrchestrator>,
    primals: Vec<Arc<dyn ManagedPrimal>>,
) -> Result<()> {
    // 1. Build dependency graph
    let graph = DependencyGraph::build(&primals)?;
    
    // 2. Resolve into waves
    let waves = graph.topological_waves()?;
    
    // 3. Start each wave (all primals in wave start in parallel)
    for wave in waves {
        // Start all in parallel
        // Wait for all to complete
        // Move to next wave
    }
}
```

### **Limitations**

1. **Coarse-grained concurrency**
   - All primals in a wave start together
   - Even if a primal could start earlier, it waits for its wave

2. **Implicit dependencies**
   - Only knows: "requires Security"
   - Doesn't know: "which operation needs Security?"
   - Doesn't track: "what data flows between primals?"

3. **No learning foundation**
   - Can't track which primals are used together
   - Can't measure latency per operation
   - Can't discover optimization opportunities

4. **Limited coordination patterns**
   - Only supports: Sequential waves
   - Can't express: Conditional DAG, Pipeline, etc.

### **Example: Current Deployment**

```toml
# tower.toml
[[primals]]
binary = "./primals/beardog"
requires = []  # Wave 1

[[primals]]
binary = "./primals/songbird"
requires = ["Security"]  # Wave 2
```

**Result**:
```
Wave 1: BearDog starts
        (wait for BearDog to be healthy)
Wave 2: Songbird starts
```

**Problem**: If BearDog takes 5 seconds to start, Songbird waits 5 seconds even though it could start earlier if we tracked finer-grained dependencies.

---

## 🚀 Target State: Graph-Based

### **How It Will Work**

```rust
// New: graph_executor.rs
pub async fn execute_graph(
    &self,
    graph: PrimalGraph,
) -> Result<GraphResult> {
    // 1. Parse graph (nodes + edges)
    let dag = self.build_dag(&graph)?;
    
    // 2. Execute concurrently (any node with deps satisfied can start)
    for node in dag.ready_nodes() {
        tokio::spawn(async move {
            // Start as soon as dependencies satisfied
            // No waiting for arbitrary "waves"
        });
    }
    
    // 3. Track metrics
    self.metrics_collector.record_execution(...).await;
    
    // 4. Return results
    Ok(GraphResult { ... })
}
```

### **Benefits**

1. **Fine-grained concurrency**
   - Start as soon as dependencies satisfied
   - No arbitrary wave boundaries
   - Maximum parallelism

2. **Explicit dependencies**
   - Track data flow between primals
   - Know which operations need which capabilities
   - Enable smarter scheduling

3. **Learning foundation**
   - Track co-occurrence (which primals used together)
   - Measure latency per operation
   - Discover optimization opportunities

4. **Rich coordination patterns**
   - Sequential, Parallel, DAG, Pipeline
   - Conditional execution
   - Streaming data flow

### **Example: Graph Deployment**

```toml
# tower.toml (graph-based)
[graph]
id = "tower-startup"
pattern = "ConditionalDAG"

[[graph.nodes]]
id = "beardog"
binary = "./primals/beardog"
action = { type = "spawn", health_check = true }
provides = ["Security"]

[[graph.nodes]]
id = "songbird"
binary = "./primals/songbird"
action = { type = "spawn", health_check = true }
provides = ["Discovery"]

[[graph.edges]]
from = "beardog"
to = "songbird"
condition = { type = "healthy" }  # Songbird starts when BearDog healthy
```

**Result**:
```
Time 0:    BearDog spawns
Time 1s:   BearDog healthy ✓ → Songbird spawns immediately
Time 2s:   Both running
```

**Improvement**: Songbird starts as soon as BearDog is healthy, not after BearDog completes all initialization.

---

## 🏗️ Implementation Plan

### **Phase 1: Core Data Structures** (Week 1)

#### **1.1: PrimalGraph**

```rust
// FILE: biomeos-core/src/graph_executor.rs

/// A graph of primals with explicit dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalGraph {
    pub id: GraphId,
    pub name: String,
    pub pattern: CoordinationPattern,
    pub nodes: Vec<PrimalNode>,
    pub edges: Vec<PrimalEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalNode {
    pub id: String,  // Unique node ID (e.g., "beardog", "songbird")
    pub binary: PathBuf,  // Path to binary
    pub action: PrimalAction,  // What to do
    pub provides: Vec<String>,  // Capabilities provided
    pub env: HashMap<String, String>,  // Environment variables
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEdge {
    pub from: String,  // Source node ID
    pub to: String,  // Target node ID
    pub condition: EdgeCondition,  // When can target start?
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeCondition {
    Spawned,      // Target can start when source is spawned
    Healthy,      // Target can start when source is healthy
    Completed,    // Target can start when source completed
    DataReady,    // Target can start when source has data ready
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationPattern {
    Sequential,      // One after another
    Parallel,        // All at once
    ConditionalDAG,  // Based on conditions
    Pipeline,        // Streaming
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalAction {
    Spawn { health_check: bool },  // Spawn as daemon
    Query { method: String },       // Query via IPC
    Invoke { endpoint: String },    // HTTP invoke
}
```

**Tests**:
```rust
#[test]
fn test_parse_graph_from_toml() {
    let toml = r#"
        [graph]
        id = "test"
        pattern = "ConditionalDAG"
        
        [[graph.nodes]]
        id = "a"
        binary = "./bin/a"
        
        [[graph.edges]]
        from = "a"
        to = "b"
    "#;
    
    let graph: PrimalGraph = toml::from_str(toml).unwrap();
    assert_eq!(graph.nodes.len(), 1);
}
```

---

#### **1.2: DAG Builder**

```rust
/// Build a DAG from a graph for execution
pub struct DAG {
    nodes: HashMap<String, DAGNode>,
    adjacency: HashMap<String, Vec<String>>,
}

pub struct DAGNode {
    pub id: String,
    pub node: PrimalNode,
    pub status: Arc<RwLock<NodeStatus>>,
}

pub enum NodeStatus {
    Pending,
    Running,
    Healthy,
    Completed,
    Failed(String),
}

impl DAG {
    /// Build from graph
    pub fn from_graph(graph: &PrimalGraph) -> Result<Self> {
        // Validate no cycles
        // Build adjacency list
        // Initialize node statuses
    }
    
    /// Get nodes ready to execute (all deps satisfied)
    pub fn ready_nodes(&self) -> Vec<String> {
        self.nodes
            .iter()
            .filter(|(id, node)| {
                // Check if node is pending
                // Check if all dependencies satisfied
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
    
    /// Mark node as completed
    pub async fn mark_complete(&self, node_id: &str) {
        // Update status
        // Notify waiting nodes
    }
}
```

**Tests**:
```rust
#[test]
fn test_dag_detects_cycles() {
    // A → B → C → A (cycle!)
    let graph = PrimalGraph { ... };
    assert!(DAG::from_graph(&graph).is_err());
}

#[test]
fn test_ready_nodes() {
    // A (no deps), B (needs A), C (needs A)
    let dag = DAG::from_graph(&graph).unwrap();
    let ready = dag.ready_nodes();
    assert_eq!(ready, vec!["A"]);  // Only A is ready
}
```

---

### **Phase 2: Graph Executor** (Week 2)

#### **2.1: Execution Engine**

```rust
// FILE: biomeos-core/src/graph_executor.rs

pub struct GraphExecutor {
    registry: Arc<CapabilityRegistry>,
    health_monitor: Arc<PrimalHealthMonitor>,
}

impl GraphExecutor {
    /// Execute a primal graph
    pub async fn execute(&self, graph: PrimalGraph) -> Result<GraphResult> {
        let dag = DAG::from_graph(&graph)?;
        
        match graph.pattern {
            CoordinationPattern::Sequential => self.execute_sequential(&dag).await,
            CoordinationPattern::Parallel => self.execute_parallel(&dag).await,
            CoordinationPattern::ConditionalDAG => self.execute_dag(&dag).await,
            CoordinationPattern::Pipeline => self.execute_pipeline(&dag).await,
        }
    }
    
    async fn execute_dag(&self, dag: &DAG) -> Result<GraphResult> {
        let mut completed = HashSet::new();
        let mut tasks = JoinSet::new();
        
        loop {
            // Get ready nodes
            let ready = dag.ready_nodes()
                .into_iter()
                .filter(|id| !completed.contains(id))
                .collect::<Vec<_>>();
            
            if ready.is_empty() && tasks.is_empty() {
                // All done!
                break;
            }
            
            // Spawn tasks for ready nodes
            for node_id in ready {
                let node = dag.nodes.get(&node_id).unwrap();
                let executor = self.clone();
                let dag = dag.clone();
                
                tasks.spawn(async move {
                    // Execute node
                    executor.execute_node(&node).await?;
                    // Mark complete
                    dag.mark_complete(&node_id).await;
                    Ok::<_, anyhow::Error>(node_id)
                });
            }
            
            // Wait for at least one to complete
            if let Some(result) = tasks.join_next().await {
                match result {
                    Ok(Ok(node_id)) => {
                        completed.insert(node_id);
                    }
                    Ok(Err(e)) => return Err(e),
                    Err(e) => return Err(e.into()),
                }
            }
        }
        
        Ok(GraphResult { success: true, ... })
    }
    
    async fn execute_node(&self, node: &DAGNode) -> Result<()> {
        match &node.node.action {
            PrimalAction::Spawn { health_check } => {
                // Spawn primal
                self.spawn_primal(&node.node).await?;
                
                // Wait for health check if requested
                if *health_check {
                    self.wait_healthy(&node.id).await?;
                }
            }
            PrimalAction::Query { method } => {
                // Query primal via IPC
                self.query_primal(&node.id, method).await?;
            }
            PrimalAction::Invoke { endpoint } => {
                // Invoke primal via HTTP
                self.invoke_primal(&node.id, endpoint).await?;
            }
        }
        Ok(())
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_parallel_execution() {
    // A, B, C (all no deps) should run in parallel
    let graph = PrimalGraph {
        pattern: Parallel,
        nodes: vec![node_a, node_b, node_c],
        edges: vec![],
    };
    
    let start = Instant::now();
    executor.execute(graph).await.unwrap();
    let duration = start.elapsed();
    
    // Should take ~max(A, B, C), not sum
    assert!(duration < Duration::from_millis(100));
}

#[tokio::test]
async fn test_conditional_dag() {
    // A → B (when A healthy) → C (when B healthy)
    let graph = PrimalGraph {
        pattern: ConditionalDAG,
        nodes: vec![node_a, node_b, node_c],
        edges: vec![
            edge(A, B, Healthy),
            edge(B, C, Healthy),
        ],
    };
    
    executor.execute(graph).await.unwrap();
    
    // Verify order: A started first, then B, then C
}
```

---

### **Phase 3: TOML Configuration** (Week 2)

#### **3.1: Update tower.toml Format**

**Old Format (waves)**:
```toml
[tower]
concurrent_startup = true

[[primals]]
binary = "./primals/beardog"
provides = ["Security"]
requires = []

[[primals]]
binary = "./primals/songbird"
provides = ["Discovery"]
requires = ["Security"]
```

**New Format (graph)**:
```toml
[tower]
startup_mode = "graph"  # or "waves" for backwards compat

[startup_graph]
id = "tower-startup"
pattern = "ConditionalDAG"

[[startup_graph.nodes]]
id = "beardog"
binary = "./primals/beardog"
provides = ["Security", "Encryption", "Trust"]

[startup_graph.nodes.action]
type = "spawn"
health_check = true

[startup_graph.nodes.env]
BEARDOG_FAMILY_SEED = "..."
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_NODE_ID = "tower1"
RUST_LOG = "info"

[[startup_graph.nodes]]
id = "songbird"
binary = "./primals/songbird"
provides = ["Discovery"]

[startup_graph.nodes.action]
type = "spawn"
health_check = true

[startup_graph.nodes.env]
SONGBIRD_FAMILY_ID = "nat0"
SONGBIRD_NODE_ID = "tower1"
RUST_LOG = "info"

# Explicit edge: Songbird starts when BearDog is healthy
[[startup_graph.edges]]
from = "beardog"
to = "songbird"
condition = "healthy"
```

**Benefits**:
- Explicit dependencies (edges)
- Fine-grained conditions (healthy, spawned, completed)
- Clear graph visualization
- Foundation for learning

---

#### **3.2: Backwards Compatibility**

```rust
// Support both formats
pub enum TowerConfig {
    Legacy(LegacyConfig),  // Old waves format
    Graph(GraphConfig),    // New graph format
}

impl TowerConfig {
    pub fn from_toml(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        
        // Try new format first
        if let Ok(graph_config) = toml::from_str::<GraphConfig>(&content) {
            return Ok(TowerConfig::Graph(graph_config));
        }
        
        // Fall back to legacy
        let legacy_config = toml::from_str::<LegacyConfig>(&content)?;
        Ok(TowerConfig::Legacy(legacy_config))
    }
    
    pub fn to_graph(&self) -> PrimalGraph {
        match self {
            TowerConfig::Legacy(cfg) => {
                // Convert legacy to graph
                self.legacy_to_graph(cfg)
            }
            TowerConfig::Graph(cfg) => cfg.startup_graph.clone(),
        }
    }
    
    fn legacy_to_graph(&self, cfg: &LegacyConfig) -> PrimalGraph {
        // Build graph from legacy requires/provides
        // Infer edges from dependencies
        // Use "healthy" condition by default
    }
}
```

---

### **Phase 4: USB Spore Deployment** (Week 3)

#### **4.1: Update USB Spore Structure**

**Current**:
```
/media/eastgate/biomeOS1/biomeOS/
├── activate-tower.sh
├── bin/tower
├── primals/beardog
├── primals/songbird
├── config/tower.env  # Old env-based
└── tower.toml        # Old waves format
```

**New**:
```
/media/eastgate/biomeOS1/biomeOS/
├── activate-tower.sh  # Updated to use graph
├── bin/tower          # Graph-aware
├── primals/beardog
├── primals/songbird
├── config/
│   └── tower.toml     # New graph format
└── README.md
```

#### **4.2: Update activate-tower.sh**

```bash
#!/bin/bash
# Activate tower using graph-based orchestration

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cd "$SCRIPT_DIR"

echo "🚀 Activating biomeOS Tower (Graph-Based)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Run tower with graph config
exec ./bin/tower run --config config/tower.toml
```

#### **4.3: Example Deployment Graph**

```toml
# /media/eastgate/biomeOS1/biomeOS/config/tower.toml
# BiomeOS Tower - Graph-Based Orchestration

[tower]
family = "nat0"
startup_mode = "graph"

[startup_graph]
id = "biomeOS-tower"
pattern = "ConditionalDAG"
description = "Port-free tower with Unix sockets + UDP multicast"

# BearDog: Security first
[[startup_graph.nodes]]
id = "beardog"
binary = "./primals/beardog"
provides = ["Security", "Encryption", "Trust"]

[startup_graph.nodes.action]
type = "spawn"
health_check = true
timeout_secs = 30

[startup_graph.nodes.env]
BEARDOG_FAMILY_SEED = "Nat0C/G/b4B7u06n0r14SuZXrp/IZ/38fZHh8aJQMVg="
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_NODE_ID = "tower1"
RUST_LOG = "info"

# Songbird: Discovery orchestrator
[[startup_graph.nodes]]
id = "songbird"
binary = "./primals/songbird"
provides = ["Discovery", "Federation"]

[startup_graph.nodes.action]
type = "spawn"
health_check = true
timeout_secs = 30

[startup_graph.nodes.env]
SONGBIRD_FAMILY_ID = "nat0"
SONGBIRD_NODE_ID = "tower1"
RUST_LOG = "info"

# Edge: Songbird needs BearDog healthy
[[startup_graph.edges]]
from = "beardog"
to = "songbird"
condition = "healthy"
description = "Songbird needs BearDog for encryption"
```

**Execution**:
```
Time 0.0s: BearDog spawns
Time 0.1s: BearDog initializing...
Time 2.0s: BearDog healthy ✓
Time 2.1s: Songbird spawns (immediately!)
Time 2.2s: Songbird initializing...
Time 3.0s: Songbird healthy ✓
Time 3.1s: Tower ready!

Total: 3.1 seconds (vs 5+ seconds with waves)
```

---

## 🎯 Success Criteria

### **Phase 1: Core Data Structures**
- [ ] `PrimalGraph` can be parsed from TOML
- [ ] `DAG` can be built from graph
- [ ] Cycle detection works
- [ ] Ready node detection works

### **Phase 2: Graph Executor**
- [ ] Can execute sequential graphs
- [ ] Can execute parallel graphs
- [ ] Can execute conditional DAG graphs
- [ ] Health checks work correctly

### **Phase 3: TOML Configuration**
- [ ] New graph format parses correctly
- [ ] Legacy format still works (backwards compat)
- [ ] Legacy auto-converts to graph

### **Phase 4: USB Spore**
- [ ] USB spore uses graph format
- [ ] Deployment faster than waves
- [ ] Robust to startup failures
- [ ] Easy to visualize/understand

---

## 📊 Performance Expectations

### **Startup Time** (2-primal deployment)

**Current (waves)**:
```
Wave 1: BearDog   (5s to spawn + healthy)
Wave 2: Songbird  (3s to spawn + healthy)
Total: 8 seconds
```

**New (graph)**:
```
Time 0s:   BearDog spawns
Time 2s:   BearDog healthy → Songbird spawns
Time 5s:   Songbird healthy
Total: 5 seconds  (37% faster!)
```

### **Scalability** (N primals)

**Waves**: O(W) where W = number of waves (typically log(N))  
**Graph**: O(D) where D = longest dependency chain (typically < W)

**Result**: Graph is always ≤ waves, often faster

---

## 🔗 Foundation for Neural API

This graph executor is **Phase 1** of Neural API:

```
✅ Phase 1: Graph Execution       (this spec)
🟡 Phase 2: Metrics Collection    (next spec)
🟡 Phase 3: Pathway Learning      (next spec)
🟡 Phase 4: Bidirectional Feedback (next spec)
```

Once graphs are working, we can add:
- Metrics collection (track latency)
- Pattern learning (discover optimizations)
- Automatic optimization (suggest improvements)

---

## 🚀 Implementation Timeline

**Week 1**: Core data structures + DAG builder  
**Week 2**: Graph executor + TOML config  
**Week 3**: USB spore deployment + testing  
**Total**: 3 weeks to production-ready

---

## 📝 Next Steps

1. **Review this spec** with biomeOS team
2. **Begin implementation** (Week 1: data structures)
3. **Test with USB spore** (Week 3: deployment)
4. **Move to Phase 2** (metrics collection)

---

🔄 **Graph Orchestration: From waves to graphs, from good to great!**


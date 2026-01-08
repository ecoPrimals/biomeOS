# 🧠 Graph-Based Orchestration System Specification

**Version:** 1.0.0  
**Date:** January 8, 2026  
**Status:** 🎯 **Design Specification**  
**Target:** biomeOS v0.2.0

---

## 🎯 Overview

This specification defines the evolution of biomeOS from **static wave-based orchestration** to **adaptive graph-based orchestration**, enabling the Neural API architecture.

---

## 📊 Current State vs Target State

### **Current: Wave System**
```rust
// Static, sequential orchestration
pub async fn deploy_tower() {
    spawn_primal("songbird").await?;
    spawn_primal("beardog").await?;
    spawn_primal("biomeos").await?;
}
```

**Limitations:**
- Fixed execution order
- No parallelization
- No adaptation
- No learning
- Hardcoded workflows

### **Target: Graph System**
```rust
// Dynamic, adaptive orchestration
pub async fn deploy_tower() {
    let graph = load_graph("deploy_tower").await?;
    let result = execute_graph(graph).await?;
    learn_from_execution(&result).await?;
}
```

**Benefits:**
- Flexible execution (sequential, parallel, DAG)
- Automatic optimization
- Learning from metrics
- Declarative workflows
- Composable patterns

---

## 🏗️ Core Architecture

### **Component Hierarchy**

```
┌─────────────────────────────────────────────────────────────┐
│                   Graph-Based Orchestration                  │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  📋 Graph Definition Layer (TOML/YAML)                       │
│     ├── Workflow graphs (commit.toml, deploy.toml)          │
│     ├── Coordination patterns                               │
│     └── Primal dependencies                                 │
│                         ↓                                    │
│  🔍 Graph Parser & Validator                                 │
│     ├── Parse graph definitions                             │
│     ├── Validate DAG structure                              │
│     └── Resolve primal capabilities                         │
│                         ↓                                    │
│  🧠 Graph Executor                                           │
│     ├── Sequential execution                                │
│     ├── Parallel execution                                  │
│     ├── Conditional DAG execution                           │
│     └── Pipeline streaming                                  │
│                         ↓                                    │
│  📊 Metrics Collector                                        │
│     ├── Latency tracking                                    │
│     ├── Success/failure rates                               │
│     ├── Resource utilization                                │
│     └── Bottleneck detection                                │
│                         ↓                                    │
│  🎓 Learning Engine (Future Phase)                          │
│     ├── Pattern discovery                                   │
│     ├── Pathway optimization                                │
│     └── Adaptive evolution                                  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

---

## 📋 Data Structures

### **PrimalGraph**

```rust
/// A directed graph of primal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalGraph {
    /// Unique identifier for this graph
    pub id: GraphId,
    
    /// Human-readable name
    pub name: String,
    
    /// Description of what this graph does
    pub description: String,
    
    /// Version for evolution tracking
    pub version: Version,
    
    /// Nodes in the graph (primals to execute)
    pub nodes: Vec<GraphNode>,
    
    /// Edges between nodes (dependencies)
    pub edges: Vec<GraphEdge>,
    
    /// How to coordinate execution
    pub coordination: CoordinationPattern,
    
    /// Metadata for learning
    pub metadata: GraphMetadata,
}

/// A node in the primal graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node ID within this graph
    pub id: NodeId,
    
    /// Which primal to use (or capability to find)
    pub primal: PrimalSelector,
    
    /// What operation to perform
    pub operation: Operation,
    
    /// Input data (can reference previous outputs)
    pub input: InputSpec,
    
    /// Where to store output (variable name)
    pub output: Option<String>,
    
    /// Execution constraints
    pub constraints: NodeConstraints,
}

/// An edge represents a dependency or data flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node
    pub from: NodeId,
    
    /// Destination node
    pub to: NodeId,
    
    /// What data flows between them
    pub data_flow: Option<DataFlow>,
    
    /// Edge type
    pub edge_type: EdgeType,
}

/// How to select a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalSelector {
    /// By exact primal ID
    ById(PrimalId),
    
    /// By capability (let system choose)
    ByCapability(Capability),
    
    /// By multiple capabilities (all required)
    ByCapabilities(Vec<Capability>),
}

/// What operation to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation name (e.g., "append_commit", "store_blob")
    pub name: String,
    
    /// Operation parameters
    pub params: serde_json::Value,
    
    /// Timeout
    pub timeout_ms: Option<u64>,
    
    /// Retry policy
    pub retry: Option<RetryPolicy>,
}

/// How nodes are coordinated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoordinationPattern {
    /// Execute nodes one after another
    Sequential,
    
    /// Execute all nodes simultaneously
    Parallel,
    
    /// Execute based on dependencies (DAG)
    ConditionalDAG,
    
    /// Stream data through pipeline
    Pipeline,
    
    /// Custom pattern
    Custom(String),
}
```

---

## 📝 Graph Definition Format (TOML)

### **Example: Tower Deployment**

```toml
# graphs/deploy_tower.toml

[graph]
id = "deploy-tower-v1"
name = "Deploy Tower"
description = "Deploy a complete tower (Songbird + BearDog + biomeOS)"
version = "1.0.0"
coordination = "ConditionalDAG"

# Node 1: Start Songbird
[[nodes]]
id = "start-songbird"
primal = { by_id = "songbird" }
operation = { name = "start", params = {} }
output = "songbird_status"
constraints = { timeout_ms = 30000 }

# Node 2: Start BearDog (depends on Songbird for discovery)
[[nodes]]
id = "start-beardog"
primal = { by_id = "beardog" }
operation = { name = "start", params = {} }
output = "beardog_status"
constraints = { timeout_ms = 30000 }

# Node 3: Verify genetic lineage
[[nodes]]
id = "verify-lineage"
primal = { by_id = "beardog" }
operation = { name = "federation.verify_family_member", params = { family_id = "${FAMILY_ID}" } }
output = "lineage_verified"
constraints = { timeout_ms = 5000 }

# Node 4: Create BTSP tunnel
[[nodes]]
id = "create-tunnel"
primal = { by_id = "songbird" }
operation = { name = "create_genetic_tunnel", params = { genetic_proof = "${lineage_verified}" } }
output = "tunnel_established"
constraints = { timeout_ms = 10000 }

# Node 5: Announce tower capabilities
[[nodes]]
id = "announce"
primal = { by_id = "songbird" }
operation = { name = "announce_capabilities", params = { capabilities = ["federation", "discovery"] } }
constraints = { timeout_ms = 5000 }

# Dependencies
[[edges]]
from = "start-songbird"
to = "start-beardog"
edge_type = "dependency"

[[edges]]
from = "start-beardog"
to = "verify-lineage"
edge_type = "dependency"

[[edges]]
from = "verify-lineage"
to = "create-tunnel"
edge_type = { data_flow = "lineage_verified" }

[[edges]]
from = "create-tunnel"
to = "announce"
edge_type = "dependency"

[metadata]
author = "biomeOS"
created = "2026-01-08"
tags = ["tower", "deployment", "federation"]
```

### **Example: Parallel Execution**

```toml
# graphs/parallel_init.toml

[graph]
id = "parallel-init-v1"
name = "Parallel Initialization"
description = "Start multiple primals in parallel"
coordination = "Parallel"

# All these start simultaneously
[[nodes]]
id = "start-songbird"
primal = { by_id = "songbird" }
operation = { name = "start" }
parallel_group = 1

[[nodes]]
id = "start-beardog"
primal = { by_id = "beardog" }
operation = { name = "start" }
parallel_group = 1

[[nodes]]
id = "start-nestgate"
primal = { by_id = "nestgate" }
operation = { name = "start" }
parallel_group = 1

# This waits for all to complete
[[nodes]]
id = "health-check"
primal = { by_capability = "health-monitoring" }
operation = { name = "check_all" }
parallel_group = 2

[[edges]]
from = "start-songbird"
to = "health-check"

[[edges]]
from = "start-beardog"
to = "health-check"

[[edges]]
from = "start-nestgate"
to = "health-check"
```

---

## 🔧 Implementation Components

### **1. Graph Parser**

```rust
// crates/biomeos-graph/src/parser.rs

pub struct GraphParser {
    validator: GraphValidator,
}

impl GraphParser {
    /// Parse a graph from TOML
    pub fn parse_toml(&self, path: &Path) -> Result<PrimalGraph> {
        let content = std::fs::read_to_string(path)?;
        let graph: PrimalGraph = toml::from_str(&content)?;
        self.validator.validate(&graph)?;
        Ok(graph)
    }
    
    /// Parse multiple graphs from a directory
    pub fn parse_directory(&self, dir: &Path) -> Result<Vec<PrimalGraph>> {
        let mut graphs = Vec::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension() == Some("toml".as_ref()) {
                graphs.push(self.parse_toml(&path)?);
            }
        }
        Ok(graphs)
    }
}
```

### **2. Graph Validator**

```rust
// crates/biomeos-graph/src/validator.rs

pub struct GraphValidator;

impl GraphValidator {
    /// Validate graph structure
    pub fn validate(&self, graph: &PrimalGraph) -> Result<()> {
        // Check for cycles in DAG
        self.check_acyclic(graph)?;
        
        // Check all node IDs are unique
        self.check_unique_ids(graph)?;
        
        // Check all edges reference valid nodes
        self.check_valid_edges(graph)?;
        
        // Check input/output references are valid
        self.check_data_flow(graph)?;
        
        Ok(())
    }
    
    fn check_acyclic(&self, graph: &PrimalGraph) -> Result<()> {
        // Topological sort - if it succeeds, no cycles
        let _ = self.topological_sort(graph)?;
        Ok(())
    }
}
```

### **3. Graph Executor**

```rust
// crates/biomeos-graph/src/executor.rs

pub struct GraphExecutor {
    registry: Arc<CapabilityRegistry>,
    metrics: Arc<MetricsCollector>,
}

impl GraphExecutor {
    /// Execute a primal graph
    pub async fn execute(&self, graph: PrimalGraph) -> Result<GraphResult> {
        let start = Instant::now();
        
        // Record execution start
        self.metrics.record_start(&graph).await;
        
        // Execute based on coordination pattern
        let result = match graph.coordination {
            CoordinationPattern::Sequential => {
                self.execute_sequential(&graph).await?
            }
            CoordinationPattern::Parallel => {
                self.execute_parallel(&graph).await?
            }
            CoordinationPattern::ConditionalDAG => {
                self.execute_dag(&graph).await?
            }
            CoordinationPattern::Pipeline => {
                self.execute_pipeline(&graph).await?
            }
            CoordinationPattern::Custom(ref pattern) => {
                self.execute_custom(&graph, pattern).await?
            }
        };
        
        let duration = start.elapsed();
        
        // Record metrics
        self.metrics.record_complete(&graph, &result, duration).await;
        
        Ok(result)
    }
    
    async fn execute_dag(&self, graph: &PrimalGraph) -> Result<GraphResult> {
        // Topological sort to get execution order
        let order = self.topological_sort(graph)?;
        
        let mut context = ExecutionContext::new();
        let mut results = HashMap::new();
        
        for node_id in order {
            let node = graph.get_node(&node_id)?;
            
            // Check if dependencies are satisfied
            let deps_satisfied = self.check_dependencies(&node, &results)?;
            if !deps_satisfied {
                continue; // Skip if conditional and condition not met
            }
            
            // Execute node
            let result = self.execute_node(node, &context).await?;
            
            // Store result
            if let Some(output_var) = &node.output {
                context.set(output_var, result.clone());
            }
            results.insert(node_id, result);
        }
        
        Ok(GraphResult {
            success: true,
            outputs: context.into_outputs(),
            metrics: self.collect_metrics(&results),
        })
    }
}
```

### **4. Metrics Collector**

```rust
// crates/biomeos-graph/src/metrics.rs

pub struct MetricsCollector {
    storage: Arc<MetricsStorage>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExecutionMetrics {
    pub graph_id: GraphId,
    pub started_at: Timestamp,
    pub completed_at: Timestamp,
    pub duration_ms: u64,
    pub success: bool,
    pub node_metrics: Vec<NodeMetrics>,
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeMetrics {
    pub node_id: NodeId,
    pub primal_id: PrimalId,
    pub operation: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}

impl MetricsCollector {
    /// Record graph execution start
    pub async fn record_start(&self, graph: &PrimalGraph) {
        // Implementation
    }
    
    /// Record graph execution complete
    pub async fn record_complete(
        &self,
        graph: &PrimalGraph,
        result: &GraphResult,
        duration: Duration,
    ) {
        let metrics = ExecutionMetrics {
            graph_id: graph.id.clone(),
            started_at: Timestamp::now(),
            completed_at: Timestamp::now(),
            duration_ms: duration.as_millis() as u64,
            success: result.success,
            node_metrics: result.metrics.clone(),
        };
        
        self.storage.store(metrics).await;
    }
    
    /// Get metrics for a graph
    pub async fn get_metrics(&self, graph_id: &GraphId) -> Result<Vec<ExecutionMetrics>> {
        self.storage.query(graph_id).await
    }
}
```

---

## 🎯 Migration Path

### **Phase 1: Foundation** (v0.2.0)
- ✅ Implement graph data structures
- ✅ Implement TOML parser
- ✅ Implement graph validator
- ✅ Implement sequential executor
- ✅ Basic metrics collection

### **Phase 2: Parallelization** (v0.3.0)
- ⏳ Implement parallel executor
- ⏳ Implement DAG executor
- ⏳ Implement pipeline executor
- ⏳ Advanced metrics (bottleneck detection)

### **Phase 3: Learning** (v0.4.0)
- ⏳ Implement pathway learner
- ⏳ Pattern discovery
- ⏳ Automatic optimization
- ⏳ A/B testing of pathways

### **Phase 4: Neural API** (v0.5.0)
- ⏳ High-level API layer
- ⏳ Adaptive execution
- ⏳ Continuous learning
- ⏳ Evolution engine

---

## 📦 Crate Structure

```
crates/
├── biomeos-graph/           # NEW! Graph execution system
│   ├── src/
│   │   ├── lib.rs
│   │   ├── graph.rs         # Graph data structures
│   │   ├── parser.rs        # TOML parser
│   │   ├── validator.rs     # Graph validation
│   │   ├── executor.rs      # Graph execution
│   │   ├── metrics.rs       # Metrics collection
│   │   └── patterns.rs      # Coordination patterns
│   └── tests/
│       ├── parser_tests.rs
│       ├── executor_tests.rs
│       └── integration_tests.rs
```

---

## 🎊 Success Criteria

### **Phase 1 Complete When:**
- ✅ Can parse TOML graph definitions
- ✅ Can validate graph structure (no cycles, valid refs)
- ✅ Can execute sequential graphs
- ✅ Can collect basic metrics
- ✅ All tests passing (unit + integration)

### **Future Phases:**
- Parallel execution working
- DAG execution optimized
- Metrics show performance improvements
- Learning engine functional
- Neural API operational

---

**Status:** 🎯 **Ready for Implementation**  
**Next:** Begin Phase 1 - Foundation

🧠 **From static waves → adaptive graphs!** 🎊


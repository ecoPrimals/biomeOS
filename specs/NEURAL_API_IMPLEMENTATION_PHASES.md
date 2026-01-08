# 🧠 Neural API Implementation Phases - Detailed Specification

**Version:** 1.0.0  
**Date:** January 8, 2026  
**Status:** 🎯 **Implementation Guide**

**See also:** `NEURAL_API_ROADMAP.md` (root level) for progress tracking

---

## 🎯 Implementation Strategy

**Goal:** Evolve biomeOS orchestration from wave system to Neural API in 4 practical milestones:

1. **Tower Niche** - Replicate current functionality with graphs
2. **Node Niche** - Add parallel execution for compute
3. **Nest Niche** - Add DAG execution for data pipelines
4. **Backbone** - Full integration as foundation for RootPulse

---

## 📦 Phase 1.1: Graph Executor Foundation

### **Crate Structure**

```
crates/biomeos-graph/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Public API
│   ├── graph.rs            # Core data structures
│   ├── parser.rs           # TOML → Graph
│   ├── validator.rs        # Structure validation
│   ├── executor.rs         # Graph execution
│   ├── context.rs          # Execution context
│   └── error.rs            # Error types
└── tests/
    ├── parser_tests.rs
    ├── validator_tests.rs
    └── executor_tests.rs
```

### **Cargo.toml**

```toml
[package]
name = "biomeos-graph"
version = "0.1.0"
edition = "2021"

[dependencies]
# Core
tokio = { workspace = true }
async-trait = { workspace = true }
futures = { workspace = true }

# Serialization
serde = { workspace = true }
serde_json = { workspace = true }
toml = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }

# UUID
uuid = { workspace = true }

# Time
chrono = { workspace = true }

# Graph algorithms
petgraph = "0.6"

# biomeOS dependencies
biomeos-types = { path = "../biomeos-types" }
biomeos-core = { path = "../biomeos-core" }

[dev-dependencies]
tokio-test = { workspace = true }
```

### **Core Data Structures (graph.rs)**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a graph
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(pub String);

impl GraphId {
    pub fn new(name: &str) -> Self {
        Self(format!("{}-{}", name, Uuid::new_v4()))
    }
}

/// A directed graph of primal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalGraph {
    pub id: GraphId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub coordination: CoordinationPattern,
}

/// A node in the primal graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub primal: PrimalSelector,
    pub operation: Operation,
    pub input: Option<serde_json::Value>,
    pub output: Option<String>,
    pub constraints: Option<NodeConstraints>,
}

/// An edge representing a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

/// How to select a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrimalSelector {
    ById { by_id: String },
    ByCapability { by_capability: String },
}

/// Operation to perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub params: serde_json::Value,
}

/// Node execution constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConstraints {
    pub timeout_ms: Option<u64>,
    pub retry: Option<RetryPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

/// Coordination pattern
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CoordinationPattern {
    Sequential,
    Parallel,
    ConditionalDAG,
    Pipeline,
}

/// Edge type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EdgeType {
    Simple(String),
    DataFlow { data_flow: String },
}

/// Result of graph execution
#[derive(Debug, Clone)]
pub struct GraphResult {
    pub success: bool,
    pub outputs: HashMap<String, serde_json::Value>,
    pub metrics: Vec<NodeMetrics>,
}

#[derive(Debug, Clone)]
pub struct NodeMetrics {
    pub node_id: String,
    pub duration_ms: u64,
    pub success: bool,
    pub error: Option<String>,
}
```

### **Parser (parser.rs)**

```rust
use crate::graph::*;
use anyhow::{Context, Result};
use std::path::Path;

pub struct GraphParser;

impl GraphParser {
    pub fn parse_file(path: &Path) -> Result<PrimalGraph> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read {}", path.display()))?;
        
        Self::parse_toml(&content)
    }
    
    pub fn parse_toml(content: &str) -> Result<PrimalGraph> {
        let value: toml::Value = toml::from_str(content)
            .context("Failed to parse TOML")?;
        
        let graph_table = value.get("graph")
            .and_then(|v| v.as_table())
            .context("Missing [graph] section")?;
        
        let name = graph_table.get("name")
            .and_then(|v| v.as_str())
            .context("Missing graph.name")?;
        
        let description = graph_table.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let version = graph_table.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0");
        
        let coordination = graph_table.get("coordination")
            .and_then(|v| v.as_str())
            .and_then(|s| match s {
                "Sequential" => Some(CoordinationPattern::Sequential),
                "Parallel" => Some(CoordinationPattern::Parallel),
                "ConditionalDAG" => Some(CoordinationPattern::ConditionalDAG),
                "Pipeline" => Some(CoordinationPattern::Pipeline),
                _ => None,
            })
            .unwrap_or(CoordinationPattern::Sequential);
        
        let nodes = Self::parse_nodes(&value)?;
        let edges = Self::parse_edges(&value)?;
        
        Ok(PrimalGraph {
            id: GraphId::new(name),
            name: name.to_string(),
            description: description.to_string(),
            version: version.to_string(),
            nodes,
            edges,
            coordination,
        })
    }
    
    fn parse_nodes(value: &toml::Value) -> Result<Vec<GraphNode>> {
        let nodes_array = value.get("nodes")
            .and_then(|v| v.as_array())
            .context("Missing [[nodes]] section")?;
        
        nodes_array.iter()
            .map(|node_value| Self::parse_node(node_value))
            .collect()
    }
    
    fn parse_node(value: &toml::Value) -> Result<GraphNode> {
        let table = value.as_table()
            .context("Node must be a table")?;
        
        let id = table.get("id")
            .and_then(|v| v.as_str())
            .context("Missing node.id")?
            .to_string();
        
        let primal = Self::parse_primal_selector(table)?;
        let operation = Self::parse_operation(table)?;
        
        let output = table.get("output")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        let constraints = table.get("constraints")
            .map(|v| Self::parse_constraints(v))
            .transpose()?;
        
        Ok(GraphNode {
            id,
            primal,
            operation,
            input: None,
            output,
            constraints,
        })
    }
    
    fn parse_primal_selector(table: &toml::map::Map<String, toml::Value>) -> Result<PrimalSelector> {
        let primal_value = table.get("primal")
            .context("Missing node.primal")?;
        
        if let Some(id) = primal_value.get("by_id").and_then(|v| v.as_str()) {
            return Ok(PrimalSelector::ById { by_id: id.to_string() });
        }
        
        if let Some(cap) = primal_value.get("by_capability").and_then(|v| v.as_str()) {
            return Ok(PrimalSelector::ByCapability { by_capability: cap.to_string() });
        }
        
        anyhow::bail!("Invalid primal selector")
    }
    
    fn parse_operation(table: &toml::map::Map<String, toml::Value>) -> Result<Operation> {
        let op_value = table.get("operation")
            .context("Missing node.operation")?;
        
        let name = op_value.get("name")
            .and_then(|v| v.as_str())
            .context("Missing operation.name")?
            .to_string();
        
        let params = op_value.get("params")
            .cloned()
            .unwrap_or(toml::Value::Table(toml::map::Map::new()));
        
        let params_json = serde_json::to_value(&params)?;
        
        Ok(Operation { name, params: params_json })
    }
    
    fn parse_constraints(value: &toml::Value) -> Result<NodeConstraints> {
        // Implementation
        Ok(NodeConstraints {
            timeout_ms: value.get("timeout_ms").and_then(|v| v.as_integer()).map(|i| i as u64),
            retry: None, // TODO: Parse retry policy
        })
    }
    
    fn parse_edges(value: &toml::Value) -> Result<Vec<GraphEdge>> {
        let edges_array = value.get("edges")
            .and_then(|v| v.as_array())
            .unwrap_or(&vec![]);
        
        edges_array.iter()
            .map(|edge_value| Self::parse_edge(edge_value))
            .collect()
    }
    
    fn parse_edge(value: &toml::Value) -> Result<GraphEdge> {
        let table = value.as_table()
            .context("Edge must be a table")?;
        
        let from = table.get("from")
            .and_then(|v| v.as_str())
            .context("Missing edge.from")?
            .to_string();
        
        let to = table.get("to")
            .and_then(|v| v.as_str())
            .context("Missing edge.to")?
            .to_string();
        
        let edge_type = if let Some(et) = table.get("edge_type") {
            if let Some(s) = et.as_str() {
                EdgeType::Simple(s.to_string())
            } else if let Some(df) = et.get("data_flow").and_then(|v| v.as_str()) {
                EdgeType::DataFlow { data_flow: df.to_string() }
            } else {
                EdgeType::Simple("dependency".to_string())
            }
        } else {
            EdgeType::Simple("dependency".to_string())
        };
        
        Ok(GraphEdge { from, to, edge_type })
    }
}
```

### **Validator (validator.rs)**

```rust
use crate::graph::*;
use anyhow::{Context, Result};
use petgraph::graph::DiGraph;
use petgraph::algo::is_cyclic_directed;
use std::collections::HashSet;

pub struct GraphValidator;

impl GraphValidator {
    pub fn validate(graph: &PrimalGraph) -> Result<()> {
        Self::check_unique_node_ids(graph)?;
        Self::check_valid_edges(graph)?;
        Self::check_acyclic(graph)?;
        Ok(())
    }
    
    fn check_unique_node_ids(graph: &PrimalGraph) -> Result<()> {
        let mut seen = HashSet::new();
        for node in &graph.nodes {
            if !seen.insert(&node.id) {
                anyhow::bail!("Duplicate node ID: {}", node.id);
            }
        }
        Ok(())
    }
    
    fn check_valid_edges(graph: &PrimalGraph) -> Result<()> {
        let node_ids: HashSet<_> = graph.nodes.iter().map(|n| &n.id).collect();
        
        for edge in &graph.edges {
            if !node_ids.contains(&edge.from) {
                anyhow::bail!("Edge references unknown node: {}", edge.from);
            }
            if !node_ids.contains(&edge.to) {
                antml:bail!("Edge references unknown node: {}", edge.to);
            }
        }
        Ok(())
    }
    
    fn check_acyclic(graph: &PrimalGraph) -> Result<()> {
        let mut pg = DiGraph::new();
        let mut node_indices = std::collections::HashMap::new();
        
        // Add nodes
        for node in &graph.nodes {
            let idx = pg.add_node(node.id.clone());
            node_indices.insert(&node.id, idx);
        }
        
        // Add edges
        for edge in &graph.edges {
            let from = node_indices[&edge.from];
            let to = node_indices[&edge.to];
            pg.add_edge(from, to, ());
        }
        
        if is_cyclic_directed(&pg) {
            anyhow::bail!("Graph contains cycles");
        }
        
        Ok(())
    }
}
```

---

## 🧪 Testing Strategy

### **Unit Tests**
- Parse valid TOML graphs
- Reject invalid graphs
- Validate structure

### **Integration Tests**  
- Execute simple sequential graph
- Verify node order
- Check result correctness

### **E2E Tests**
- Deploy tower via graph (once integrated)
- Compare with old wave system
- Verify federation works

---

**Status:** 🎯 **Ready for implementation**  
**Next:** Begin coding Phase 1.1

🧠 **Let's build the foundation!** 🎊


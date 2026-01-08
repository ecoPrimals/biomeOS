// =============================================================================
// Graph Data Structures - Core Types
// =============================================================================
//
// Modern idiomatic Rust:
// - No unsafe code
// - Owned data structures
// - Clear lifetimes
// - Explicit error handling
//
// =============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for a graph (generated, not hardcoded)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphId(String);

impl GraphId {
    /// Create a new graph ID from a name (adds UUID for uniqueness)
    pub fn new(name: &str) -> Self {
        Self(format!("{}-{}", name, Uuid::new_v4()))
    }
    
    /// Get the ID as a string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A directed graph of primal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalGraph {
    /// Unique identifier
    pub id: GraphId,
    
    /// Human-readable name
    pub name: String,
    
    /// Description of what this graph does
    pub description: String,
    
    /// Version for evolution tracking
    pub version: String,
    
    /// Nodes in the graph
    pub nodes: Vec<GraphNode>,
    
    /// Edges between nodes (dependencies)
    pub edges: Vec<GraphEdge>,
    
    /// How to coordinate execution
    pub coordination: CoordinationPattern,
}

impl PrimalGraph {
    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|n| n.id == id)
    }
    
    /// Get all edges from a node
    pub fn edges_from(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.from == node_id).collect()
    }
    
    /// Get all edges to a node
    pub fn edges_to(&self, node_id: &str) -> Vec<&GraphEdge> {
        self.edges.iter().filter(|e| e.to == node_id).collect()
    }
}

/// A node in the primal graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Unique node ID within this graph
    pub id: String,
    
    /// How to select the primal (capability-based, not hardcoded!)
    pub primal: PrimalSelector,
    
    /// What operation to perform
    pub operation: Operation,
    
    /// Input data (optional, can reference previous outputs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<serde_json::Value>,
    
    /// Where to store output (variable name)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    
    /// Execution constraints
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<NodeConstraints>,
    
    /// Parallel group (for parallel execution)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_group: Option<u32>,
}

/// An edge representing a dependency or data flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    /// Source node
    pub from: String,
    
    /// Destination node
    pub to: String,
    
    /// Edge type (dependency or data flow)
    #[serde(default = "default_edge_type")]
    pub edge_type: EdgeType,
}

fn default_edge_type() -> EdgeType {
    EdgeType::Dependency
}

/// How to select a primal (CAPABILITY-BASED, NOT HARDCODED!)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrimalSelector {
    /// By exact primal ID (when you know it)
    ById {
        by_id: String,
    },
    
    /// By capability (discover at runtime!)
    ByCapability {
        by_capability: String,
    },
    
    /// By multiple capabilities (all required)
    ByCapabilities {
        by_capabilities: Vec<String>,
    },
}

/// What operation to perform on a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation name (e.g., "start", "health_check")
    pub name: String,
    
    /// Operation parameters (flexible JSON)
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Node execution constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConstraints {
    /// Timeout in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    
    /// Retry policy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<RetryPolicy>,
}

/// Retry policy for node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Backoff time in milliseconds
    pub backoff_ms: u64,
}

/// How nodes are coordinated
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoordinationPattern {
    /// Execute nodes one after another
    Sequential,
    
    /// Execute all nodes simultaneously
    Parallel,
    
    /// Execute based on dependencies (DAG)
    ConditionalDAG,
    
    /// Stream data through pipeline
    Pipeline,
}

/// Edge type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EdgeType {
    /// Simple dependency (just ordering)
    Dependency,
    
    /// Data flows between nodes
    DataFlow {
        data_flow: String,
    },
}

/// Result of graph execution
#[derive(Debug, Clone)]
pub struct GraphResult {
    /// Whether execution succeeded
    pub success: bool,
    
    /// Output variables from nodes
    pub outputs: HashMap<String, serde_json::Value>,
    
    /// Execution metrics for each node
    pub metrics: Vec<NodeMetrics>,
}

/// Metrics for a single node execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Node ID
    pub node_id: String,
    
    /// Primal that executed this node
    pub primal_id: String,
    
    /// Operation performed
    pub operation: String,
    
    /// Duration in milliseconds
    pub duration_ms: u64,
    
    /// Whether execution succeeded
    pub success: bool,
    
    /// Error message if failed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    
    /// Timestamp when execution started
    pub started_at: chrono::DateTime<chrono::Utc>,
    
    /// Timestamp when execution completed
    pub completed_at: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_graph_id_unique() {
        let id1 = GraphId::new("test");
        let id2 = GraphId::new("test");
        assert_ne!(id1, id2, "Graph IDs should be unique even with same name");
    }
    
    #[test]
    fn test_primal_selector_by_capability() {
        let selector = PrimalSelector::ByCapability {
            by_capability: "discovery".to_string(),
        };
        
        // Capability-based discovery (not hardcoded!)
        if let PrimalSelector::ByCapability { by_capability } = selector {
            assert_eq!(by_capability, "discovery");
        } else {
            panic!("Expected ByCapability variant");
        }
    }
}


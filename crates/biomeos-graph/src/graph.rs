//! Graph data structures for Neural API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Primal graph - orchestration with capability-based discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalGraph {
    pub id: GraphId,
    pub name: String,
    pub description: String,
    pub version: String,
    pub nodes: Vec<PrimalNode>,
    pub edges: Vec<GraphEdge>,
    pub coordination: CoordinationPattern,
}

/// Primal node - graph node with capability-based primal selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalNode {
    pub id: String,
    pub primal: PrimalSelector,
    pub operation: Operation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
}

/// Graph ID type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct GraphId(String);

impl GraphId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Primal selector - capability-based discovery (NO HARDCODING!)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalSelector {
    /// Select by primal ID (fallback only)
    ById { by_id: String },

    /// Select by single capability (preferred!)
    ByCapability { by_capability: String },

    /// Select by multiple capabilities (AND logic)
    ByCapabilities { by_capabilities: Vec<String> },
}

/// Operation to execute on a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    pub params: serde_json::Value,
}

/// Graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub edge_type: EdgeType,
}

/// Edge type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    DataFlow,
    ControlFlow,
    Dependency,
}

/// Coordination pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum CoordinationPattern {
    Sequential,
    Parallel,
    ConditionalDAG,
    Pipeline,
}

/// Node constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<RetryPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_capabilities: Option<Vec<String>>,
}

/// Retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

/// Graph execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphResult {
    pub success: bool,
    pub node_results: HashMap<String, serde_json::Value>,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

/// Node execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub node_id: String,
    pub duration_ms: u64,
    pub success: bool,
    pub retry_count: u32,
}

/// Neural API graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    pub id: String,
    pub version: String,
    pub description: String,
    pub nodes: Vec<GraphNode>,
    pub config: GraphConfig,
}

impl Graph {
    /// Load graph from TOML file
    pub fn from_toml_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        Self::from_toml_str(&contents)
    }

    /// Load graph from TOML string
    pub fn from_toml_str(toml: &str) -> anyhow::Result<Self> {
        // Parse TOML
        let value: toml::Value = toml::from_str(toml)?;

        // Extract graph metadata
        let graph_table = value
            .get("graph")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow::anyhow!("Missing [graph] section"))?;

        let id = graph_table
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = graph_table
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let description = graph_table
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract nodes
        let nodes_array = value
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| anyhow::anyhow!("Missing [[nodes]] array"))?;

        let mut nodes = Vec::new();
        for node_value in nodes_array {
            let node: GraphNode = toml::from_str(&toml::to_string(node_value)?)?;
            nodes.push(node);
        }

        // Extract execution config
        let config = if let Some(exec_table) = value.get("execution").and_then(|v| v.as_table()) {
            GraphConfig {
                deterministic: exec_table
                    .get("mode")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "deterministic")
                    .unwrap_or(true),
                parallel_phases: exec_table
                    .get("parallel_phases")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                max_parallelism: exec_table
                    .get("max_parallelism")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(3) as usize,
                timeout_total_ms: exec_table
                    .get("timeout_total_ms")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(60000) as u64,
                checkpoint_enabled: exec_table
                    .get("checkpoint_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                rollback_on_failure: exec_table
                    .get("rollback_on_failure")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
            }
        } else {
            GraphConfig::default()
        };

        Ok(Self {
            id,
            version,
            description,
            nodes,
            config,
        })
    }
}

/// Graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub outputs: Vec<NodeOutput>,
}

/// Node output definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutput {
    pub name: String,
    #[serde(rename = "type")]
    pub output_type: String,
}

/// Graph execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub deterministic: bool,
    pub parallel_phases: bool,
    pub max_parallelism: usize,
    pub timeout_total_ms: u64,
    pub checkpoint_enabled: bool,
    pub rollback_on_failure: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            deterministic: true,
            parallel_phases: true,
            max_parallelism: 3,
            timeout_total_ms: 60000,
            checkpoint_enabled: false,
            rollback_on_failure: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_graph() {
        let toml = r#"
[graph]
id = "test_graph"
version = "1.0.0"
description = "Test graph"

[[nodes]]
id = "node1"
node_type = "test.node"
dependencies = []

[[nodes]]
id = "node2"
node_type = "test.node"
dependencies = ["node1"]

[execution]
mode = "deterministic"
max_parallelism = 2
"#;

        let graph = Graph::from_toml_str(toml).unwrap();
        assert_eq!(graph.id, "test_graph");
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.config.max_parallelism, 2);
    }
}

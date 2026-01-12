//! Graph data structures for Neural API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
        let graph_table = value.get("graph")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow::anyhow!("Missing [graph] section"))?;

        let id = graph_table.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let version = graph_table.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        let description = graph_table.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        // Extract nodes
        let nodes_array = value.get("nodes")
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
                deterministic: exec_table.get("mode")
                    .and_then(|v| v.as_str())
                    .map(|s| s == "deterministic")
                    .unwrap_or(true),
                parallel_phases: exec_table.get("parallel_phases")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true),
                max_parallelism: exec_table.get("max_parallelism")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(3) as usize,
                timeout_total_ms: exec_table.get("timeout_total_ms")
                    .and_then(|v| v.as_integer())
                    .unwrap_or(60000) as u64,
                checkpoint_enabled: exec_table.get("checkpoint_enabled")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                rollback_on_failure: exec_table.get("rollback_on_failure")
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
type = "test.node"
dependencies = []

[[nodes]]
id = "node2"
type = "test.node"
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

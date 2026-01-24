//! Graph data structures for Neural API

use anyhow::Context;
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
        tracing::debug!("📖 Reading graph file: {}", path.display());
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        tracing::debug!("   File size: {} bytes", contents.len());

        Self::from_toml_str(&contents)
            .with_context(|| format!("Failed to parse TOML from: {}", path.display()))
    }

    /// Load graph from TOML string
    pub fn from_toml_str(toml: &str) -> anyhow::Result<Self> {
        tracing::debug!("🔍 Parsing TOML structure...");

        // Parse TOML
        let value: toml::Value = toml::from_str(toml).context("Failed to parse TOML syntax")?;

        tracing::debug!("✅ TOML syntax valid");

        // Extract graph metadata
        tracing::debug!("🔍 Looking for [graph] section...");
        let graph_table = value
            .get("graph")
            .and_then(|v| v.as_table())
            .ok_or_else(|| anyhow::anyhow!("Missing [graph] section in TOML"))?;

        tracing::debug!("✅ Found [graph] section");

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
        tracing::debug!("🔍 Looking for [[nodes]] array...");
        let nodes_array = value
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                tracing::error!("❌ Missing [[nodes]] array in TOML");
                tracing::debug!(
                    "   Available top-level keys: {:?}",
                    value.as_table().map(|t| t.keys().collect::<Vec<_>>())
                );
                anyhow::anyhow!(
                    "Missing [[nodes]] array. Found keys: {:?}",
                    value.as_table().map(|t| t.keys().collect::<Vec<_>>())
                )
            })?;

        tracing::debug!("✅ Found [[nodes]] array with {} nodes", nodes_array.len());

        let mut nodes = Vec::new();
        for (idx, node_value) in nodes_array.iter().enumerate() {
            tracing::debug!("   Parsing node {}...", idx);
            let node: GraphNode = toml::from_str(&toml::to_string(node_value)?)
                .with_context(|| format!("Failed to parse node {} structure", idx))?;
            tracing::debug!("   ✅ Node {}: id={}", idx, node.id);
            nodes.push(node);
        }

        tracing::info!("✅ Parsed {} nodes successfully", nodes.len());

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
    #[serde(default)]
    pub primal: Option<PrimalSelector>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub operation: Option<Operation>,
    #[serde(default)]
    pub constraints: Option<Constraints>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    // Capabilities this primal provides (NEW - for capability registry)
    #[serde(default)]
    pub capabilities: Vec<String>,

    // NEW v2.0.0: Capability translation mappings (semantic → actual method)
    // Enables self-describing primals for capability translation
    // Example: {"crypto.generate_keypair": "x25519_generate_ephemeral"}
    #[serde(default)]
    pub capabilities_provided: Option<HashMap<String, String>>,

    // NEW v2.0.1: Parameter name mappings (semantic → actual parameter names)
    // Enables parameter translation for capability calls
    // Example: {"crypto.ecdh_derive": {"private_key": "our_secret", "public_key": "their_public"}}
    #[serde(default)]
    pub parameter_mappings: Option<HashMap<String, HashMap<String, String>>>,

    // Legacy fields (for backward compatibility)
    #[serde(default)]
    pub node_type: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub outputs: Vec<NodeOutput>,
}

/// Primal selector (capability-based discovery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSelector {
    #[serde(default)]
    pub by_capability: Option<String>,
    #[serde(default)]
    pub by_name: Option<String>,
}

/// Operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub name: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Environment variables to pass to the primal (NEW - Jan 21, 2026)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

/// Node constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub backoff_ms: u64,
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
node_type = "primal"
type = "test.node"
dependencies = []

[[nodes]]
id = "node2"
node_type = "primal"
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

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Graph data structures for Neural API

use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Neural API graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Graph {
    /// Unique graph identifier
    pub id: String,
    /// Semantic version
    pub version: String,
    /// Human-readable description
    pub description: String,
    /// Execution nodes in dependency order
    pub nodes: Vec<GraphNode>,
    /// Execution configuration (parallelism, timeouts, etc.)
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
                .with_context(|| format!("Failed to parse node {idx} structure"))?;
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
                    .and_then(|v| usize::try_from(v).ok())
                    .unwrap_or(3),
                timeout_total_ms: exec_table
                    .get("timeout_total_ms")
                    .and_then(|v| v.as_integer())
                    .and_then(|v| u64::try_from(v).ok())
                    .unwrap_or(60000),
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
    /// Unique node identifier
    pub id: String,
    /// How to select the primal (by capability or name)
    #[serde(default)]
    pub primal: Option<PrimalSelector>,
    /// Output key for downstream nodes
    #[serde(default)]
    pub output: Option<String>,
    /// Operation to invoke on the primal
    #[serde(default)]
    pub operation: Option<Operation>,
    /// Timeout and retry constraints
    #[serde(default)]
    pub constraints: Option<Constraints>,
    /// Node IDs this node depends on
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Capabilities this primal provides (for capability registry)
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Capability translation mappings (semantic → actual method)
    #[serde(default)]
    pub capabilities_provided: Option<HashMap<String, String>>,

    /// Parameter name mappings (semantic → actual parameter names)
    #[serde(default)]
    pub parameter_mappings: Option<HashMap<String, HashMap<String, String>>>,

    /// Legacy: node type (prefer primal + operation)
    #[serde(default)]
    pub node_type: Option<String>,
    /// Legacy: dependency list (alias for depends_on)
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Node-specific configuration
    #[serde(default)]
    pub config: HashMap<String, serde_json::Value>,
    /// Output definitions for this node
    #[serde(default)]
    pub outputs: Vec<NodeOutput>,
}

/// Primal selector (capability-based discovery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSelector {
    /// Select primal by capability (e.g., "crypto.encrypt")
    #[serde(default)]
    pub by_capability: Option<String>,
    /// Select primal by name (e.g., "beardog")
    #[serde(default)]
    pub by_name: Option<String>,
}

/// Operation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// Operation/capability name (e.g., "crypto.encrypt")
    pub name: String,
    /// Parameters for the operation
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,

    /// Environment variables to pass to the primal (NEW - Jan 21, 2026)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<HashMap<String, String>>,
}

/// Node constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraints {
    /// Per-node timeout in milliseconds
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    /// Retry configuration on failure
    #[serde(default)]
    pub retry: Option<RetryConfig>,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts before failure
    pub max_attempts: u32,
    /// Delay between retries in milliseconds
    pub backoff_ms: u64,
}

/// Node output definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeOutput {
    /// Output name for downstream binding
    pub name: String,
    /// Output type (e.g., "string", "json")
    #[serde(rename = "type")]
    pub output_type: String,
}

/// Graph execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    /// Run in deterministic mode (reproducible execution order)
    pub deterministic: bool,
    /// Allow parallel execution of independent phases
    pub parallel_phases: bool,
    /// Maximum concurrent node executions
    pub max_parallelism: usize,
    /// Total graph timeout in milliseconds
    pub timeout_total_ms: u64,
    /// Enable checkpointing for resume
    pub checkpoint_enabled: bool,
    /// Rollback on any node failure
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
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn test_from_toml_str_missing_graph_section() {
        let toml = r#"
id = "orphan"
[nodes]
"#;
        let result = Graph::from_toml_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("[graph]"));
    }

    #[test]
    fn test_from_toml_str_missing_nodes_array() {
        let toml = r#"
[graph]
id = "no_nodes"
version = "1.0.0"
description = "No nodes"
"#;
        let result = Graph::from_toml_str(toml);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("nodes"));
    }

    #[test]
    fn test_from_toml_str_invalid_toml() {
        let toml = "this is not valid [toml = syntax";
        let result = Graph::from_toml_str(toml);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_toml_str_empty_nodes_array() {
        // nodes must be top-level; in TOML, keys under [graph] go into graph table
        let toml = r#"
nodes = []

[graph]
id = "empty_graph"
version = "1.0.0"
description = "Empty nodes"
"#;
        let result = Graph::from_toml_str(toml);
        assert!(result.is_ok());
        let graph = result.unwrap();
        assert_eq!(graph.id, "empty_graph");
        assert!(graph.nodes.is_empty());
    }

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

    /// Helper to locate the workspace graphs/ directory from the test binary's location
    fn find_graphs_dir() -> std::path::PathBuf {
        // Walk up from the crate root to find the workspace root containing graphs/
        let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        loop {
            let candidate = dir.join("graphs");
            if candidate.is_dir() {
                return candidate;
            }
            if !dir.pop() {
                panic!(
                    "Could not find graphs/ directory from {}",
                    env!("CARGO_MANIFEST_DIR")
                );
            }
        }
    }

    #[test]
    fn test_parse_nucleus_complete() {
        let graphs_dir = find_graphs_dir();
        let graph = Graph::from_toml_file(&graphs_dir.join("nucleus_complete.toml")).unwrap();
        assert_eq!(graph.id, "nucleus_complete");
        assert_eq!(graph.version, "2.0.0");
        // Should have: beardog, songbird, onion_init, mesh_init, tower_validate,
        //              toadstool, node_validate, nestgate, squirrel, nucleus_validate, announce_relay
        assert!(
            graph.nodes.len() >= 10,
            "Expected at least 10 nodes, got {}",
            graph.nodes.len()
        );
        // Verify first node is BearDog
        assert_eq!(graph.nodes[0].id, "tower_beardog");
        // Verify BearDog has relay.authorize translation (relay-punch)
        let beardog_caps = graph.nodes[0].capabilities_provided.as_ref().unwrap();
        assert_eq!(
            beardog_caps.get("relay.authorize"),
            Some(&"relay.authorize".to_string())
        );
        // Verify Songbird has mesh + punch + stun capabilities
        let songbird = &graph.nodes[1];
        assert_eq!(songbird.id, "tower_songbird");
        assert!(songbird.capabilities.contains(&"mesh".to_string()));
        assert!(songbird.capabilities.contains(&"punch".to_string()));
        assert!(songbird.capabilities.contains(&"stun".to_string()));
        // Verify Songbird capability translations include relay-punch methods
        let songbird_caps = songbird.capabilities_provided.as_ref().unwrap();
        assert_eq!(
            songbird_caps.get("stun.probe_port_pattern"),
            Some(&"stun.probe_port_pattern".to_string())
        );
        assert_eq!(
            songbird_caps.get("punch.coordinate"),
            Some(&"punch.coordinate".to_string())
        );
    }

    #[test]
    fn test_parse_ecosystem_full_bootstrap() {
        let graphs_dir = find_graphs_dir();
        let graph =
            Graph::from_toml_file(&graphs_dir.join("ecosystem_full_bootstrap.toml")).unwrap();
        assert_eq!(graph.id, "ecosystem_full_bootstrap");
        assert_eq!(graph.version, "2.0.0");
        // Should have: beardog, songbird, toadstool, squirrel, nestgate, validate
        assert!(
            graph.nodes.len() >= 6,
            "Expected at least 6 nodes, got {}",
            graph.nodes.len()
        );
        // Verify NestGate is present (was missing before)
        let nestgate = graph.nodes.iter().find(|n| n.id == "germinate_nestgate");
        assert!(nestgate.is_some(), "NestGate node should be present");
        // Verify no hardcoded /tmp/ paths in Songbird environment
        let songbird = graph
            .nodes
            .iter()
            .find(|n| n.id == "germinate_songbird")
            .unwrap();
        let env = songbird
            .operation
            .as_ref()
            .unwrap()
            .environment
            .as_ref()
            .unwrap();
        for (key, val) in env {
            assert!(
                !val.contains("/tmp/"),
                "Songbird env {key} should not use /tmp/, got: {val}"
            );
        }
    }

    #[test]
    fn test_parse_gate2_nucleus() {
        let graphs_dir = find_graphs_dir();
        let graph = Graph::from_toml_file(&graphs_dir.join("gate2_nucleus.toml")).unwrap();
        assert_eq!(graph.id, "gate2_nucleus");
        // Should have: beardog, songbird, mesh_init, discover_tower, nestgate, toadstool, squirrel, validate, announce
        assert!(
            graph.nodes.len() >= 9,
            "Expected at least 9 nodes, got {}",
            graph.nodes.len()
        );
        // Verify auto-discover step exists
        let discover = graph.nodes.iter().find(|n| n.id == "gate2_discover_tower");
        assert!(discover.is_some(), "gate2_discover_tower node should exist");
        // Verify all environment paths use XDG_RUNTIME_DIR variable
        for node in &graph.nodes {
            if let Some(op) = &node.operation {
                if let Some(env) = &op.environment {
                    for (key, val) in env {
                        assert!(
                            !val.contains("/run/user/1000"),
                            "Node {} env {} should use ${{XDG_RUNTIME_DIR}}, not hardcoded path: {}",
                            node.id, key, val
                        );
                        assert!(
                            !val.contains("/tmp/"),
                            "Node {} env {} should not use /tmp/: {}",
                            node.id,
                            key,
                            val
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_parse_tower_atomic_bootstrap() {
        let graphs_dir = find_graphs_dir();
        let graph = Graph::from_toml_file(&graphs_dir.join("tower_atomic_bootstrap.toml")).unwrap();
        assert_eq!(graph.id, "tower_atomic_bootstrap");
        assert!(
            graph.nodes.len() >= 4,
            "Expected at least 4 nodes, got {}",
            graph.nodes.len()
        );
        // Verify Songbird environment uses XDG paths
        let songbird = graph
            .nodes
            .iter()
            .find(|n| n.id == "germinate_songbird")
            .unwrap();
        let env = songbird
            .operation
            .as_ref()
            .unwrap()
            .environment
            .as_ref()
            .unwrap();
        let neural_api_sock = env.get("NEURAL_API_SOCKET").unwrap();
        assert!(
            neural_api_sock.contains("XDG_RUNTIME_DIR"),
            "NEURAL_API_SOCKET should use XDG_RUNTIME_DIR, got: {neural_api_sock}"
        );
        // Verify no port 3492 in any operation params
        for node in &graph.nodes {
            if let Some(op) = &node.operation {
                if let Some(port_val) = op.params.get("port") {
                    let port = port_val.as_i64().unwrap_or(0);
                    assert_ne!(port, 3492, "Node {} should not use port 3492", node.id);
                }
                if let Some(params_val) = op.params.get("params") {
                    if let Some(port_val) = params_val.get("port") {
                        let port = port_val.as_i64().unwrap_or(0);
                        assert_ne!(
                            port, 3492,
                            "Node {} params should not use port 3492",
                            node.id
                        );
                    }
                }
            }
        }
    }

    /// Validate all core deployment graphs parse correctly.
    /// These are the graphs used by `biomeos atomic deploy` for NUCLEUS deployment.
    #[test]
    fn test_all_deployment_graphs_parse() {
        let graphs_dir = find_graphs_dir();
        let deployment_graphs = [
            "nucleus_complete.toml",
            "ecosystem_full_bootstrap.toml",
            "gate2_nucleus.toml",
            "tower_atomic_bootstrap.toml",
            "tower_atomic_xdg.toml",
            "tower_atomic.toml",
            "tower_atomic_dynamic.toml",
        ];

        let mut parsed_count = 0;
        let mut errors = Vec::new();

        for filename in &deployment_graphs {
            let path = graphs_dir.join(filename);
            if !path.exists() {
                continue; // Skip optional graphs
            }
            match Graph::from_toml_file(&path) {
                Ok(graph) => {
                    assert!(!graph.id.is_empty(), "Graph {filename} has empty id");
                    assert!(!graph.nodes.is_empty(), "Graph {filename} has no nodes");
                    parsed_count += 1;
                }
                Err(e) => {
                    errors.push(format!("{filename}: {e}"));
                }
            }
        }

        assert!(
            parsed_count >= 4,
            "Expected to parse at least 4 deployment graphs, got {parsed_count}"
        );
        if !errors.is_empty() {
            panic!("Deployment graph parse errors:\n{}", errors.join("\n"));
        }
    }

    /// Verify no deployment graph uses hardcoded /tmp/ or /run/user/1000 paths
    #[test]
    fn test_no_hardcoded_paths_in_deployment_graphs() {
        let graphs_dir = find_graphs_dir();
        let deployment_graphs = [
            "nucleus_complete.toml",
            "ecosystem_full_bootstrap.toml",
            "gate2_nucleus.toml",
            "tower_atomic_xdg.toml",
        ];

        for filename in &deployment_graphs {
            let path = graphs_dir.join(filename);
            if !path.exists() {
                continue;
            }
            let graph = Graph::from_toml_file(&path).unwrap();
            for node in &graph.nodes {
                if let Some(op) = &node.operation {
                    if let Some(env) = &op.environment {
                        for (key, val) in env {
                            assert!(
                                !val.contains("/tmp/"),
                                "[{}] Node {} env {} uses /tmp/: {}",
                                filename,
                                node.id,
                                key,
                                val
                            );
                            assert!(
                                !val.contains("/run/user/1000"),
                                "[{}] Node {} env {} uses hardcoded /run/user/1000: {}",
                                filename,
                                node.id,
                                key,
                                val
                            );
                        }
                    }
                }
            }
        }
    }
}

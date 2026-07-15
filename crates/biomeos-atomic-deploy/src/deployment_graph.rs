// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Neural API integration for deployment orchestration
//!
//! Makes deployment deterministic and manageable via graph execution

use anyhow::{Context, Result, bail};
use biomeos_types::primal_names;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

pub use crate::orchestrator::DeploymentResult;

/// Neural API deployment graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGraphNode {
    /// Unique node identifier within the graph
    pub id: String,
    /// Capability type (e.g., `filesystem.check_exists`, `crypto.derive_seed`)
    pub node_type: String,
    /// Node IDs this node depends on (execution order)
    pub dependencies: Vec<String>,
    /// Node-specific configuration as JSON
    pub config: serde_json::Value,
}

/// Atomic deployment graph for Neural API
#[derive(Debug, Serialize)]
pub struct AtomicDeploymentGraph {
    nodes: Vec<DeploymentGraphNode>,
}

impl AtomicDeploymentGraph {
    /// Create graph for deploying all 3 atomics from USB seed
    pub fn full_nucleus_deployment(
        usb_seed_path: impl AsRef<std::path::Path>,
        family_id: &str,
    ) -> Self {
        let usb_seed_path = usb_seed_path.as_ref();
        let mut nodes = Vec::new();

        // Node 1: Verify USB seed
        nodes.push(DeploymentGraphNode {
            id: "verify_usb_seed".to_string(),
            node_type: "filesystem.check_exists".to_string(),
            dependencies: vec![],
            config: serde_json::json!({
                "path": usb_seed_path,
                "description": "Verify USB parent seed exists"
            }),
        });

        // Node 2: Derive Tower seed
        nodes.push(DeploymentGraphNode {
            id: "derive_tower_seed".to_string(),
            node_type: "crypto.derive_seed".to_string(),
            dependencies: vec!["verify_usb_seed".to_string()],
            config: serde_json::json!({
                "parent_seed": usb_seed_path,
                "node_id": "tower",
                "family_id": family_id,
                "algorithm": "SHA256"
            }),
        });

        // Node 3: Deploy Tower BearDog
        nodes.push(DeploymentGraphNode {
            id: "deploy_tower_beardog".to_string(),
            node_type: "primal.launch".to_string(),
            dependencies: vec!["derive_tower_seed".to_string()],
            config: serde_json::json!({
                "primal": "beardog-server",
                "atomic": "tower",
                "env": {
                    "FAMILY_ID": family_id,
                    "NODE_ID": "tower"
                }
            }),
        });

        // Node 4: Deploy Tower Songbird
        nodes.push(DeploymentGraphNode {
            id: "deploy_tower_songbird".to_string(),
            node_type: "primal.launch".to_string(),
            dependencies: vec!["deploy_tower_beardog".to_string()],
            config: serde_json::json!({
                "primal": "songbird-orchestrator",
                "atomic": "tower",
                "env": {
                    "SONGBIRD_FAMILY_ID": family_id
                }
            }),
        });

        // Node 5: Verify Tower health
        nodes.push(DeploymentGraphNode {
            id: "verify_tower_health".to_string(),
            node_type: "health.check_atomic".to_string(),
            dependencies: vec!["deploy_tower_songbird".to_string()],
            config: serde_json::json!({
                "atomic": "tower",
                "required_primals": [primal_names::BEARDOG, primal_names::SONGBIRD]
            }),
        });

        // Nodes 6-10: Similar for Node atomic
        nodes.push(DeploymentGraphNode {
            id: "derive_node_seed".to_string(),
            node_type: "crypto.derive_seed".to_string(),
            dependencies: vec!["verify_usb_seed".to_string()],
            config: serde_json::json!({
                "parent_seed": usb_seed_path,
                "node_id": "node",
                "family_id": family_id,
                "algorithm": "SHA256"
            }),
        });

        // Nodes 11-15: Similar for Nest atomic
        nodes.push(DeploymentGraphNode {
            id: "derive_nest_seed".to_string(),
            node_type: "crypto.derive_seed".to_string(),
            dependencies: vec!["verify_usb_seed".to_string()],
            config: serde_json::json!({
                "parent_seed": usb_seed_path,
                "node_id": "nest",
                "family_id": family_id,
                "algorithm": "SHA256"
            }),
        });

        // Final node: Verify cross-atomic lineage
        nodes.push(DeploymentGraphNode {
            id: "verify_lineage_recognition".to_string(),
            node_type: "lineage.verify_siblings".to_string(),
            dependencies: vec![
                "verify_tower_health".to_string(),
                // "verify_node_health".to_string(), // Add when Node nodes complete
                // "verify_nest_health".to_string(), // Add when Nest nodes complete
            ],
            config: serde_json::json!({
                "atomics": ["tower", "node", "nest"],
                "family_id": family_id,
                "verification_type": "pairwise"
            }),
        });

        Self { nodes }
    }

    /// Export graph to TOML for Neural API
    ///
    /// Enables programmatic graph generation alongside the static `graphs/*.toml` catalog.
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("TOML serialization failed")
    }

    /// Get execution order (topological sort via Kahn's algorithm).
    ///
    /// Nodes with no remaining dependencies are emitted in declaration order for stability.
    /// Returns an error when dependencies reference unknown nodes or the graph contains cycles.
    pub fn execution_order(&self) -> Result<Vec<&DeploymentGraphNode>> {
        let node_index: HashMap<&str, usize> = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id.as_str(), index))
            .collect();

        let mut in_degree = vec![0usize; self.nodes.len()];
        let mut dependents: Vec<Vec<usize>> = vec![Vec::new(); self.nodes.len()];

        for (index, node) in self.nodes.iter().enumerate() {
            for dep_id in &node.dependencies {
                let Some(&dep_index) = node_index.get(dep_id.as_str()) else {
                    bail!(
                        "Node '{}' depends on unknown node '{}'",
                        node.id,
                        dep_id
                    );
                };
                dependents[dep_index].push(index);
                in_degree[index] += 1;
            }
        }

        let mut ready: BTreeSet<usize> = in_degree
            .iter()
            .enumerate()
            .filter(|(_, degree)| **degree == 0)
            .map(|(index, _)| index)
            .collect();

        let mut order = Vec::with_capacity(self.nodes.len());
        while let Some(&index) = ready.first() {
            ready.remove(&index);
            order.push(&self.nodes[index]);

            for &dependent in &dependents[index] {
                in_degree[dependent] -= 1;
                if in_degree[dependent] == 0 {
                    ready.insert(dependent);
                }
            }
        }

        if order.len() != self.nodes.len() {
            bail!("Graph contains cyclic dependencies");
        }

        Ok(order)
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_create_deployment_graph() {
        let graph = AtomicDeploymentGraph::full_nucleus_deployment(
            PathBuf::from("/tmp/test.seed"),
            "1894e909e454",
        );

        assert!(!graph.nodes.is_empty());
        assert!(graph.nodes.iter().any(|n| n.id == "verify_usb_seed"));
        assert!(
            graph
                .nodes
                .iter()
                .any(|n| n.id == "verify_lineage_recognition")
        );
    }

    #[test]
    fn test_to_toml_roundtrip() {
        let graph = AtomicDeploymentGraph::full_nucleus_deployment(
            PathBuf::from("/tmp/test.seed"),
            "family-abc",
        );

        let toml_str = graph.to_toml().unwrap();
        assert!(toml_str.contains("verify_usb_seed"));
        assert!(toml_str.contains("crypto.derive_seed"));
        assert!(toml_str.contains("family-abc"));

        let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
        let nodes = parsed.get("nodes").unwrap().as_array().unwrap();
        assert_eq!(nodes.len(), graph.nodes.len());
    }

    #[test]
    fn test_to_toml_preserves_node_structure() {
        let graph =
            AtomicDeploymentGraph::full_nucleus_deployment(PathBuf::from("/seed"), "test-id");

        let toml_str = graph.to_toml().unwrap();
        let parsed: toml::Value = toml::from_str(&toml_str).unwrap();
        let first = &parsed["nodes"].as_array().unwrap()[0];
        assert_eq!(first["id"].as_str().unwrap(), "verify_usb_seed");
        assert_eq!(
            first["node_type"].as_str().unwrap(),
            "filesystem.check_exists"
        );
        assert!(first["dependencies"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_execution_order_respects_dependencies() {
        let graph = AtomicDeploymentGraph::full_nucleus_deployment(
            PathBuf::from("/tmp/test.seed"),
            "family-abc",
        );

        let order = graph.execution_order().unwrap();
        let positions: HashMap<&str, usize> = order
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id.as_str(), index))
            .collect();

        for node in &graph.nodes {
            let node_pos = positions[node.id.as_str()];
            for dep in &node.dependencies {
                assert!(
                    positions[dep.as_str()] < node_pos,
                    "node '{}' must run after dependency '{}'",
                    node.id,
                    dep
                );
            }
        }
    }

    #[test]
    fn test_execution_order_unknown_dependency() {
        let graph = AtomicDeploymentGraph {
            nodes: vec![
                DeploymentGraphNode {
                    id: "root".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec![],
                    config: serde_json::json!({}),
                },
                DeploymentGraphNode {
                    id: "child".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec!["missing".to_string()],
                    config: serde_json::json!({}),
                },
            ],
        };

        let err = graph.execution_order().unwrap_err();
        assert!(
            err.to_string().contains("unknown node"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_execution_order_cycle_detection() {
        let graph = AtomicDeploymentGraph {
            nodes: vec![
                DeploymentGraphNode {
                    id: "a".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec!["b".to_string()],
                    config: serde_json::json!({}),
                },
                DeploymentGraphNode {
                    id: "b".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec!["a".to_string()],
                    config: serde_json::json!({}),
                },
            ],
        };

        let err = graph.execution_order().unwrap_err();
        assert!(
            err.to_string().contains("cyclic"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn test_execution_order_parallel_branches() {
        let graph = AtomicDeploymentGraph {
            nodes: vec![
                DeploymentGraphNode {
                    id: "root".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec![],
                    config: serde_json::json!({}),
                },
                DeploymentGraphNode {
                    id: "left".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec!["root".to_string()],
                    config: serde_json::json!({}),
                },
                DeploymentGraphNode {
                    id: "right".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec!["root".to_string()],
                    config: serde_json::json!({}),
                },
                DeploymentGraphNode {
                    id: "join".to_string(),
                    node_type: "test".to_string(),
                    dependencies: vec!["left".to_string(), "right".to_string()],
                    config: serde_json::json!({}),
                },
            ],
        };

        let order = graph
            .execution_order()
            .unwrap()
            .into_iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();

        assert_eq!(order[0], "root");
        assert_eq!(order[order.len() - 1], "join");
        assert!(order.iter().position(|id| id == "left").unwrap()
            < order.iter().position(|id| id == "join").unwrap());
        assert!(order.iter().position(|id| id == "right").unwrap()
            < order.iter().position(|id| id == "join").unwrap());
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Neural API integration for deployment orchestration
//!
//! Makes deployment deterministic and manageable via graph execution

use anyhow::Result;
use biomeos_types::primal_names;
use serde::{Deserialize, Serialize};

pub use crate::orchestrator::DeploymentResult;

/// Neural API deployment graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGraphNode {
    /// Unique node identifier within the graph
    pub id: String,
    /// Capability type (e.g., filesystem.check_exists, crypto.derive_seed)
    pub node_type: String,
    /// Node IDs this node depends on (execution order)
    pub dependencies: Vec<String>,
    /// Node-specific configuration as JSON
    pub config: serde_json::Value,
}

/// Atomic deployment graph for Neural API
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
                    "BEARDOG_FAMILY_ID": family_id,
                    "BEARDOG_NODE_ID": "tower"
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
    /// Note: Not currently used - Neural API loads graphs directly from TOML files
    /// Future: Could be useful for programmatic graph generation
    pub fn to_toml(&self) -> Result<String> {
        // Not implemented - use direct TOML files instead (graphs/*.toml)
        anyhow::bail!("Programmatic TOML export not implemented - use direct TOML files")
    }

    /// Get execution order (topological sort)
    ///
    /// Note: Simplified implementation - returns nodes in declaration order
    /// Neural API's GraphExecutor handles proper topological sorting
    pub fn execution_order(&self) -> Vec<&DeploymentGraphNode> {
        // Neural API handles topological sort - this is a simplified version
        self.nodes.iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}

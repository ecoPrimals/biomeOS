// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph modification system for real-time collaborative editing
//!
//! This module provides type-safe graph modifications that work with the
//! edge-based dependency system. All modifications maintain graph integrity.
//!
//! Deep Debt Principles:
//! - Modern idiomatic Rust (no unsafe, Result<T,E>)
//! - Works with existing GraphEdge system (not against it)
//! - Type-safe modifications
//! - Comprehensive validation

use crate::graph::{CoordinationPattern, EdgeType, GraphEdge, PrimalGraph, PrimalNode};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A type-safe modification that can be applied to a [`PrimalGraph`].
///
/// Used by the AI advisor and graph pipeline to express structural changes
/// (node/edge add/remove, coordination pattern change) without mutating graphs
/// directly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GraphModification {
    /// Add a new node to the graph.
    AddNode {
        /// The node to insert.
        node: PrimalNode,
    },

    /// Remove a node (and its connected edges) from the graph.
    RemoveNode {
        /// ID of the node to remove.
        node_id: String,
    },

    /// Replace a node's operation method and optional parameters.
    ModifyNodeOperation {
        /// Target node ID.
        node_id: String,
        /// New semantic method name (`domain.operation`).
        new_method: String,
        /// Optional replacement parameters.
        new_params: Option<serde_json::Value>,
    },

    /// Insert a dependency edge between two existing nodes.
    AddEdge {
        /// Source node ID.
        from: String,
        /// Target node ID.
        to: String,
        /// Edge classification.
        edge_type: EdgeType,
    },

    /// Remove an edge between two nodes.
    RemoveEdge {
        /// Source node ID.
        from: String,
        /// Target node ID.
        to: String,
    },

    /// Replace the graph's coordination pattern.
    ChangeCoordination {
        /// New coordination strategy.
        pattern: CoordinationPattern,
    },
}

/// Outcome of applying one or more [`GraphModification`]s.
///
/// Carries the modified graph on success, or a diagnostic message on failure,
/// plus any non-fatal warnings collected during application.
#[derive(Debug, Clone)]
pub struct ModificationResult {
    /// Whether the modification was applied successfully.
    pub success: bool,
    /// The resulting graph (present when `success` is true).
    pub graph: Option<PrimalGraph>,
    /// Error description (present when `success` is false).
    pub error: Option<String>,
    /// Non-fatal warnings collected during application.
    pub warnings: Vec<String>,
}

impl ModificationResult {
    /// Construct a successful result with the modified graph and any warnings.
    pub fn success(graph: PrimalGraph, warnings: Vec<String>) -> Self {
        Self {
            success: true,
            graph: Some(graph),
            error: None,
            warnings,
        }
    }

    /// Construct a failure result with the given diagnostic message.
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            graph: None,
            error: Some(error),
            warnings: vec![],
        }
    }
}

/// Applies [`GraphModification`]s to a [`PrimalGraph`], validating integrity
/// (no orphaned edges, no dependency cycles) after each change.
pub struct GraphModificationHandler;

impl GraphModificationHandler {
    /// Apply a modification to a graph
    pub fn apply(
        graph: &PrimalGraph,
        modification: &GraphModification,
    ) -> Result<ModificationResult> {
        let mut new_graph = graph.clone();
        let mut warnings = Vec::new();

        match modification {
            GraphModification::AddNode { node } => {
                Self::apply_add_node(&mut new_graph, node, &mut warnings)?;
            }

            GraphModification::RemoveNode { node_id } => {
                Self::apply_remove_node(&mut new_graph, node_id, &mut warnings)?;
            }

            GraphModification::ModifyNodeOperation {
                node_id,
                new_method,
                new_params,
            } => {
                Self::apply_modify_node(
                    &mut new_graph,
                    node_id,
                    new_method,
                    new_params.as_ref(),
                    &mut warnings,
                )?;
            }

            GraphModification::AddEdge {
                from,
                to,
                edge_type,
            } => {
                Self::apply_add_edge(&mut new_graph, from, to, edge_type, &mut warnings)?;
            }

            GraphModification::RemoveEdge { from, to } => {
                Self::apply_remove_edge(&mut new_graph, from, to, &mut warnings)?;
            }

            GraphModification::ChangeCoordination { pattern } => {
                new_graph.coordination = *pattern;
            }
        }

        // Validate the modified graph
        Self::validate_graph(&new_graph)?;

        Ok(ModificationResult::success(new_graph, warnings))
    }

    /// Apply multiple modifications in sequence
    pub fn apply_batch(
        graph: &PrimalGraph,
        modifications: &[GraphModification],
    ) -> Result<ModificationResult> {
        let mut current_graph = graph.clone();
        let mut all_warnings = Vec::new();

        for (i, modification) in modifications.iter().enumerate() {
            match Self::apply(&current_graph, modification) {
                Ok(result) => {
                    if result.success {
                        current_graph = match result.graph {
                            Some(g) => g,
                            None => {
                                return Ok(ModificationResult::failure(format!(
                                    "Batch modification {i} succeeded but produced no graph"
                                )));
                            }
                        };
                        all_warnings.extend(result.warnings);
                    } else {
                        return Ok(ModificationResult::failure(format!(
                            "Batch modification {} failed: {}",
                            i,
                            result.error.unwrap_or_default()
                        )));
                    }
                }
                Err(e) => {
                    return Ok(ModificationResult::failure(format!(
                        "Batch modification {} error: {}",
                        i, e
                    )));
                }
            }
        }

        Ok(ModificationResult::success(current_graph, all_warnings))
    }

    // Private implementation methods

    fn apply_add_node(
        graph: &mut PrimalGraph,
        node: &PrimalNode,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if node ID already exists
        if graph.nodes.iter().any(|n| n.id == node.id) {
            return Err(anyhow!("Node with ID '{}' already exists", node.id));
        }

        // Add node
        graph.nodes.push(node.clone());

        if graph.nodes.len() > 100 {
            warnings
                .push("Graph has over 100 nodes, consider breaking into sub-graphs".to_string());
        }

        Ok(())
    }

    fn apply_remove_node(
        graph: &mut PrimalGraph,
        node_id: &str,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Find node
        let node_index = graph
            .nodes
            .iter()
            .position(|n| n.id == node_id)
            .ok_or_else(|| anyhow!("Node '{}' not found", node_id))?;

        // Check if edges reference this node
        let incoming_edges: Vec<String> = graph
            .edges
            .iter()
            .filter(|e| e.to == node_id)
            .map(|e| e.from.clone())
            .collect();

        let outgoing_edges: Vec<String> = graph
            .edges
            .iter()
            .filter(|e| e.from == node_id)
            .map(|e| e.to.clone())
            .collect();

        if !incoming_edges.is_empty() || !outgoing_edges.is_empty() {
            warnings.push(format!(
                "Removing node '{}' which has {} incoming and {} outgoing edges",
                node_id,
                incoming_edges.len(),
                outgoing_edges.len()
            ));

            // Remove all edges connected to this node
            graph.edges.retain(|e| e.from != node_id && e.to != node_id);
        }

        // Remove the node
        graph.nodes.remove(node_index);

        Ok(())
    }

    fn apply_modify_node(
        graph: &mut PrimalGraph,
        node_id: &str,
        new_method: &str,
        new_params: Option<&serde_json::Value>,
        _warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Find node
        let node = graph
            .nodes
            .iter_mut()
            .find(|n| n.id == node_id)
            .ok_or_else(|| anyhow!("Node '{}' not found", node_id))?;

        // Apply modifications
        node.operation.name = new_method.to_string();

        if let Some(params) = new_params {
            node.operation.params = params.clone();
        }

        Ok(())
    }

    fn apply_add_edge(
        graph: &mut PrimalGraph,
        from: &str,
        to: &str,
        edge_type: &EdgeType,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check both nodes exist
        if !graph.nodes.iter().any(|n| n.id == from) {
            return Err(anyhow!("Source node '{}' not found", from));
        }

        if !graph.nodes.iter().any(|n| n.id == to) {
            return Err(anyhow!("Target node '{}' not found", to));
        }

        // Check if edge already exists
        if graph.edges.iter().any(|e| e.from == from && e.to == to) {
            warnings.push(format!("Edge '{}' -> '{}' already exists", from, to));
            return Ok(());
        }

        // Add edge
        graph.edges.push(GraphEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type: edge_type.clone(),
        });

        Ok(())
    }

    fn apply_remove_edge(
        graph: &mut PrimalGraph,
        from: &str,
        to: &str,
        warnings: &mut Vec<String>,
    ) -> Result<()> {
        // Check if edge exists
        if !graph.edges.iter().any(|e| e.from == from && e.to == to) {
            warnings.push(format!("Edge '{}' -> '{}' does not exist", from, to));
            return Ok(());
        }

        // Remove edge
        graph.edges.retain(|e| !(e.from == from && e.to == to));

        Ok(())
    }

    fn validate_graph(graph: &PrimalGraph) -> Result<()> {
        // Check for orphaned nodes in edges
        let node_ids: HashSet<_> = graph.nodes.iter().map(|n| n.id.as_str()).collect();

        for edge in &graph.edges {
            if !node_ids.contains(edge.from.as_str()) {
                return Err(anyhow!("Edge references non-existent node '{}'", edge.from));
            }
            if !node_ids.contains(edge.to.as_str()) {
                return Err(anyhow!("Edge references non-existent node '{}'", edge.to));
            }
        }

        // Check for cycles (for dependency edges only)
        Self::check_for_cycles(graph)?;

        Ok(())
    }

    fn check_for_cycles(graph: &PrimalGraph) -> Result<()> {
        // Build adjacency list from dependency edges only
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            adj.insert(node.id.clone(), Vec::new());
        }

        for edge in &graph.edges {
            if matches!(edge.edge_type, EdgeType::Dependency) {
                adj.entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
            }
        }

        // DFS to detect cycles
        let mut visited = HashMap::new();
        let mut rec_stack = HashMap::new();

        for node in &graph.nodes {
            if !visited.get(&node.id).copied().unwrap_or(false)
                && Self::has_cycle_dfs(&node.id, &adj, &mut visited, &mut rec_stack)?
            {
                return Err(anyhow!("Graph contains a dependency cycle"));
            }
        }

        Ok(())
    }

    fn has_cycle_dfs(
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        visited: &mut HashMap<String, bool>,
        rec_stack: &mut HashMap<String, bool>,
    ) -> Result<bool> {
        visited.insert(node.to_string(), true);
        rec_stack.insert(node.to_string(), true);

        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                if !visited.get(neighbor).copied().unwrap_or(false) {
                    if Self::has_cycle_dfs(neighbor, adj, visited, rec_stack)? {
                        return Ok(true);
                    }
                } else if rec_stack.get(neighbor).copied().unwrap_or(false) {
                    return Ok(true);
                }
            }
        }

        rec_stack.insert(node.to_string(), false);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "test")]

    use super::*;
    use crate::graph::{GraphId, Operation, PrimalSelector};

    fn create_test_node(id: &str) -> PrimalNode {
        PrimalNode {
            id: id.to_string(),
            primal: PrimalSelector::ByCapability {
                by_capability: "test".to_string(),
            },
            operation: Operation {
                name: "test_op".to_string(),
                params: serde_json::Value::Null,
                environment: None,
            },
            input: None,
            outputs: vec![],
            constraints: None,
        }
    }

    fn create_test_graph() -> PrimalGraph {
        PrimalGraph {
            id: GraphId::new("test_graph").unwrap(),
            name: "test_graph".to_string(),
            description: "Test graph".to_string(),
            version: "1.0.0".to_string(),
            coordination: CoordinationPattern::Sequential,
            nodes: vec![create_test_node("node1"), create_test_node("node2")],
            edges: vec![GraphEdge {
                from: "node1".to_string(),
                to: "node2".to_string(),
                edge_type: EdgeType::Dependency,
            }],
        }
    }

    #[test]
    fn test_add_node() {
        let graph = create_test_graph();
        let new_node = create_test_node("node3");

        let modification = GraphModification::AddNode { node: new_node };

        let result = GraphModificationHandler::apply(&graph, &modification).unwrap();
        assert!(result.success);

        let modified_graph = result.graph.unwrap();
        assert_eq!(modified_graph.nodes.len(), 3);
        assert_eq!(modified_graph.nodes[2].id, "node3");
    }

    #[test]
    fn test_add_duplicate_node_fails() {
        let graph = create_test_graph();
        let duplicate_node = create_test_node("node1");

        let modification = GraphModification::AddNode {
            node: duplicate_node,
        };

        let result = GraphModificationHandler::apply(&graph, &modification);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_node() {
        let graph = create_test_graph();
        let modification = GraphModification::RemoveNode {
            node_id: "node2".to_string(),
        };

        let result = GraphModificationHandler::apply(&graph, &modification).unwrap();
        assert!(result.success);

        let modified_graph = result.graph.unwrap();
        assert_eq!(modified_graph.nodes.len(), 1);
        assert_eq!(modified_graph.nodes[0].id, "node1");
        // Edge should be removed too
        assert_eq!(modified_graph.edges.len(), 0);
    }

    #[test]
    fn test_modify_node_operation() {
        let graph = create_test_graph();
        let modification = GraphModification::ModifyNodeOperation {
            node_id: "node1".to_string(),
            new_method: "new_operation".to_string(),
            new_params: Some(serde_json::json!({"key": "value"})),
        };

        let result = GraphModificationHandler::apply(&graph, &modification).unwrap();
        assert!(result.success);

        let modified_graph = result.graph.unwrap();
        let node = &modified_graph.nodes[0];
        assert_eq!(node.operation.name, "new_operation");
        assert!(node.operation.params.is_object());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = create_test_graph();
        graph.nodes.push(create_test_node("node3"));

        let modification = GraphModification::AddEdge {
            from: "node2".to_string(),
            to: "node3".to_string(),
            edge_type: EdgeType::Dependency,
        };

        let result = GraphModificationHandler::apply(&graph, &modification).unwrap();
        assert!(result.success);

        let modified_graph = result.graph.unwrap();
        assert_eq!(modified_graph.edges.len(), 2);
        assert!(
            modified_graph
                .edges
                .iter()
                .any(|e| e.from == "node2" && e.to == "node3")
        );
    }

    #[test]
    fn test_remove_edge() {
        let graph = create_test_graph();
        let modification = GraphModification::RemoveEdge {
            from: "node1".to_string(),
            to: "node2".to_string(),
        };

        let result = GraphModificationHandler::apply(&graph, &modification).unwrap();
        assert!(result.success);

        let modified_graph = result.graph.unwrap();
        assert_eq!(modified_graph.edges.len(), 0);
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = create_test_graph();

        // Add edge that creates cycle: node1 -> node2 -> node1
        graph.edges.push(GraphEdge {
            from: "node2".to_string(),
            to: "node1".to_string(),
            edge_type: EdgeType::Dependency,
        });

        let result = GraphModificationHandler::validate_graph(&graph);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }

    #[test]
    fn test_batch_modifications() {
        let graph = create_test_graph();

        let modifications = vec![
            GraphModification::AddNode {
                node: create_test_node("node3"),
            },
            GraphModification::AddEdge {
                from: "node2".to_string(),
                to: "node3".to_string(),
                edge_type: EdgeType::Dependency,
            },
            GraphModification::ModifyNodeOperation {
                node_id: "node1".to_string(),
                new_method: "updated_op".to_string(),
                new_params: None,
            },
        ];

        let result = GraphModificationHandler::apply_batch(&graph, &modifications).unwrap();
        assert!(result.success);

        let modified_graph = result.graph.unwrap();
        assert_eq!(modified_graph.nodes.len(), 3);
        assert_eq!(modified_graph.edges.len(), 2);
        assert_eq!(modified_graph.nodes[0].operation.name, "updated_op");
    }

    #[test]
    fn test_change_coordination() {
        let graph = create_test_graph();
        let modification = GraphModification::ChangeCoordination {
            pattern: CoordinationPattern::Parallel,
        };

        let result = GraphModificationHandler::apply(&graph, &modification).unwrap();
        assert!(result.success);

        let modified_graph = result.graph.unwrap();
        assert!(matches!(
            modified_graph.coordination,
            CoordinationPattern::Parallel
        ));
    }
}

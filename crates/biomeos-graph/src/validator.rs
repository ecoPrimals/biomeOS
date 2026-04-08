// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

// =============================================================================
// Graph Validator - Structure & Integrity Checks
// =============================================================================
//
// Modern idiomatic Rust validation:
// - Clear error messages
// - Fast algorithms (petgraph)
// - No unsafe code
//
// =============================================================================

use crate::error::{GraphError, Result};
use crate::graph::*;
use petgraph::algo::is_cyclic_directed;
use petgraph::graph::DiGraph;
use std::collections::HashSet;

/// Validator for graph structure and integrity
pub struct GraphValidator;

impl GraphValidator {
    /// Validate a graph's structure
    pub fn validate(graph: &PrimalGraph) -> Result<()> {
        Self::check_not_empty(graph)?;
        Self::check_unique_node_ids(graph)?;
        Self::check_valid_edges(graph)?;
        Self::check_acyclic(graph)?;
        Self::check_parallel_groups(graph)?;
        Ok(())
    }

    /// Check graph has at least one node
    fn check_not_empty(graph: &PrimalGraph) -> Result<()> {
        if graph.nodes.is_empty() {
            return Err(GraphError::ValidationError(
                "Graph must have at least one node".to_string(),
            ));
        }
        Ok(())
    }

    /// Check all node IDs are unique
    fn check_unique_node_ids(graph: &PrimalGraph) -> Result<()> {
        let mut seen = HashSet::new();
        for node in &graph.nodes {
            if !seen.insert(&node.id) {
                return Err(GraphError::DuplicateNode(node.id.clone()));
            }
        }
        Ok(())
    }

    /// Check all edges reference valid nodes
    fn check_valid_edges(graph: &PrimalGraph) -> Result<()> {
        let node_ids: HashSet<_> = graph.nodes.iter().map(|n| &n.id).collect();

        for edge in &graph.edges {
            if !node_ids.contains(&edge.from) {
                return Err(GraphError::InvalidEdge(format!("from: {}", edge.from)));
            }
            if !node_ids.contains(&edge.to) {
                return Err(GraphError::InvalidEdge(format!("to: {}", edge.to)));
            }
        }
        Ok(())
    }

    /// Check graph is acyclic (no infinite loops!)
    fn check_acyclic(graph: &PrimalGraph) -> Result<()> {
        // Build petgraph
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

        // Check for cycles
        if is_cyclic_directed(&pg) {
            return Err(GraphError::CyclicGraph);
        }

        Ok(())
    }

    /// Check parallel groups are consistent
    fn check_parallel_groups(_graph: &PrimalGraph) -> Result<()> {
        // Parallel coordination is now handled via edges in the new model
        // No need for explicit parallel_group fields
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_graph() {
        let graph = PrimalGraph {
            id: GraphId::new("test"),
            name: "test".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![PrimalNode {
                id: "node1".to_string(),
                primal: PrimalSelector::ById {
                    by_id: "test".to_string(),
                },
                operation: Operation {
                    name: "start".to_string(),
                    params: serde_json::Value::Null,
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            }],
            edges: vec![],
            coordination: CoordinationPattern::Sequential,
        };

        assert!(GraphValidator::validate(&graph).is_ok());
    }

    #[test]
    fn test_empty_graph() {
        let graph = PrimalGraph {
            id: GraphId::new("test"),
            name: "test".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![],
            edges: vec![],
            coordination: CoordinationPattern::Sequential,
        };

        assert!(GraphValidator::validate(&graph).is_err());
    }

    #[test]
    fn test_duplicate_node_ids() {
        let graph = PrimalGraph {
            id: GraphId::new("test"),
            name: "test".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![
                PrimalNode {
                    id: "node1".to_string(),
                    primal: PrimalSelector::ById {
                        by_id: "test".to_string(),
                    },
                    operation: Operation {
                        name: "start".to_string(),
                        params: serde_json::Value::Null,
                        environment: None,
                    },
                    input: None,
                    outputs: vec![],
                    constraints: None,
                },
                PrimalNode {
                    id: "node1".to_string(), // Duplicate!
                    primal: PrimalSelector::ById {
                        by_id: "test".to_string(),
                    },
                    operation: Operation {
                        name: "start".to_string(),
                        params: serde_json::Value::Null,
                        environment: None,
                    },
                    input: None,
                    outputs: vec![],
                    constraints: None,
                },
            ],
            edges: vec![],
            coordination: CoordinationPattern::Sequential,
        };

        let result = GraphValidator::validate(&graph);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphError::DuplicateNode(_)));
    }

    #[test]
    fn test_cyclic_graph() {
        let graph = PrimalGraph {
            id: GraphId::new("test"),
            name: "test".to_string(),
            description: "".to_string(),
            version: "1.0.0".to_string(),
            nodes: vec![
                PrimalNode {
                    id: "node1".to_string(),
                    primal: PrimalSelector::ById {
                        by_id: "test".to_string(),
                    },
                    operation: Operation {
                        name: "start".to_string(),
                        params: serde_json::Value::Null,
                        environment: None,
                    },
                    input: None,
                    outputs: vec![],
                    constraints: None,
                },
                PrimalNode {
                    id: "node2".to_string(),
                    primal: PrimalSelector::ById {
                        by_id: "test".to_string(),
                    },
                    operation: Operation {
                        name: "start".to_string(),
                        params: serde_json::Value::Null,
                        environment: None,
                    },
                    input: None,
                    outputs: vec![],
                    constraints: None,
                },
            ],
            edges: vec![
                GraphEdge {
                    from: "node1".to_string(),
                    to: "node2".to_string(),
                    edge_type: EdgeType::Dependency,
                },
                GraphEdge {
                    from: "node2".to_string(),
                    to: "node1".to_string(), // Cycle!
                    edge_type: EdgeType::Dependency,
                },
            ],
            coordination: CoordinationPattern::Sequential,
        };

        let result = GraphValidator::validate(&graph);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), GraphError::CyclicGraph));
    }
}

//! Topological Sorting Module
//!
//! Performs dependency resolution and phase planning for graph execution.
//! Uses topological sort to determine execution order while respecting dependencies.

use crate::graph::Graph;
use anyhow::{Context, Result};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{debug, info};

/// Topological sorter for graph execution planning
pub struct TopologicalSorter;

impl TopologicalSorter {
    /// Perform topological sort to determine execution phases
    ///
    /// Returns a vector of phases, where each phase contains node IDs
    /// that can be executed in parallel (no dependencies between them).
    ///
    /// ## Algorithm
    ///
    /// 1. Build dependency graph (in-degree count for each node)
    /// 2. Find nodes with no dependencies (in-degree = 0)
    /// 3. Add them to current phase
    /// 4. Remove them from graph and update in-degrees
    /// 5. Repeat until all nodes are processed
    ///
    /// ## Errors
    ///
    /// Returns error if graph contains cycles (dependency loop).
    pub fn sort(graph: &Graph) -> Result<Vec<Vec<String>>> {
        info!("🔍 Performing topological sort for graph: {}", graph.id);

        // Build in-degree map (how many dependencies each node has)
        let mut in_degree: HashMap<String, usize> = HashMap::new();
        let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

        // Initialize all nodes
        for node_id in graph.nodes.keys() {
            in_degree.insert(node_id.clone(), 0);
            adjacency.insert(node_id.clone(), Vec::new());
        }

        // Build dependency graph
        for (node_id, node) in &graph.nodes {
            for dep in &node.depends_on {
                // Increment in-degree for dependent node
                if let Some(degree) = in_degree.get_mut(node_id) {
                    *degree += 1;
                }

                // Add edge from dependency to dependent
                if let Some(adj) = adjacency.get_mut(dep) {
                    adj.push(node_id.clone());
                }
            }
        }

        debug!("In-degree map: {:?}", in_degree);

        let mut phases = Vec::new();
        let mut processed = HashSet::new();
        let total_nodes = graph.nodes.len();

        // Process nodes phase by phase
        while processed.len() < total_nodes {
            // Find all nodes with in-degree = 0 (no unmet dependencies)
            let ready_nodes: Vec<String> = in_degree
                .iter()
                .filter(|(node_id, &degree)| degree == 0 && !processed.contains(*node_id))
                .map(|(node_id, _)| node_id.clone())
                .collect();

            if ready_nodes.is_empty() {
                // No nodes ready but not all processed = cycle detected
                let remaining: Vec<_> = in_degree
                    .iter()
                    .filter(|(node_id, _)| !processed.contains(*node_id))
                    .collect();

                return Err(anyhow::anyhow!(
                    "Cycle detected in graph! Remaining nodes: {:?}",
                    remaining
                ));
            }

            info!(
                "📍 Phase {}: {} nodes ready",
                phases.len() + 1,
                ready_nodes.len()
            );
            debug!("Ready nodes: {:?}", ready_nodes);

            // Mark nodes as processed and update in-degrees
            for node_id in &ready_nodes {
                processed.insert(node_id.clone());

                // Decrement in-degree for all dependent nodes
                if let Some(dependents) = adjacency.get(node_id) {
                    for dependent in dependents {
                        if let Some(degree) = in_degree.get_mut(dependent) {
                            *degree = degree.saturating_sub(1);
                        }
                    }
                }
            }

            phases.push(ready_nodes);
        }

        info!("✅ Topological sort complete: {} phases", phases.len());

        Ok(phases)
    }

    /// Validate that all dependencies exist in the graph
    pub fn validate_dependencies(graph: &Graph) -> Result<()> {
        for (node_id, node) in &graph.nodes {
            for dep in &node.depends_on {
                if !graph.nodes.contains_key(dep) {
                    return Err(anyhow::anyhow!(
                        "Node '{}' depends on non-existent node '{}'",
                        node_id,
                        dep
                    ));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{GraphConfig, GraphNode, Operation};

    fn create_test_graph() -> Graph {
        let mut nodes = HashMap::new();

        nodes.insert(
            "a".to_string(),
            GraphNode {
                id: "a".to_string(),
                operation: Operation::FilesystemCheckExists {
                    path: "/test".to_string(),
                },
                depends_on: vec![],
                metadata: HashMap::new(),
            },
        );

        nodes.insert(
            "b".to_string(),
            GraphNode {
                id: "b".to_string(),
                operation: Operation::FilesystemCheckExists {
                    path: "/test".to_string(),
                },
                depends_on: vec!["a".to_string()],
                metadata: HashMap::new(),
            },
        );

        nodes.insert(
            "c".to_string(),
            GraphNode {
                id: "c".to_string(),
                operation: Operation::FilesystemCheckExists {
                    path: "/test".to_string(),
                },
                depends_on: vec!["a".to_string()],
                metadata: HashMap::new(),
            },
        );

        Graph {
            id: "test-graph".to_string(),
            description: "Test graph".to_string(),
            nodes,
            config: GraphConfig {
                max_parallelism: 3,
                rollback_on_failure: false,
            },
        }
    }

    #[test]
    fn test_topological_sort() {
        let graph = create_test_graph();
        let phases = TopologicalSorter::sort(&graph).unwrap();

        // Should have 2 phases: [a], [b, c]
        assert_eq!(phases.len(), 2);
        assert_eq!(phases[0].len(), 1);
        assert!(phases[0].contains(&"a".to_string()));
        assert_eq!(phases[1].len(), 2);
        assert!(phases[1].contains(&"b".to_string()));
        assert!(phases[1].contains(&"c".to_string()));
    }

    #[test]
    fn test_validate_dependencies() {
        let graph = create_test_graph();
        assert!(TopologicalSorter::validate_dependencies(&graph).is_ok());
    }

    #[test]
    fn test_missing_dependency() {
        let mut graph = create_test_graph();
        
        // Add node with non-existent dependency
        graph.nodes.insert(
            "d".to_string(),
            GraphNode {
                id: "d".to_string(),
                operation: Operation::FilesystemCheckExists {
                    path: "/test".to_string(),
                },
                depends_on: vec!["nonexistent".to_string()],
                metadata: HashMap::new(),
            },
        );

        assert!(TopologicalSorter::validate_dependencies(&graph).is_err());
    }
}

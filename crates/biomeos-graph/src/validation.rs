// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Graph validation for structural correctness.
//!
//! Validates:
//! - Node IDs are unique
//! - Dependencies exist
//! - No dependency cycles
//! - Capabilities are well-formed

use std::collections::{HashMap, HashSet};

use crate::{Result, error::GraphError, graph::DeploymentGraph};

/// Validates deployment graphs.
pub struct GraphValidator {
    /// Known capability namespaces
    known_namespaces: HashSet<String>,
}

impl Default for GraphValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphValidator {
    /// Create a new validator with default known namespaces.
    #[must_use]
    pub fn new() -> Self {
        let mut known_namespaces = HashSet::new();

        // Known capability namespaces
        known_namespaces.insert("crypto".to_string());
        known_namespaces.insert("genetic".to_string());
        known_namespaces.insert("filesystem".to_string());
        known_namespaces.insert("process".to_string());
        known_namespaces.insert("network".to_string());
        known_namespaces.insert("config".to_string());
        known_namespaces.insert("biomeos".to_string());
        known_namespaces.insert("http".to_string());
        known_namespaces.insert("birdsong".to_string());

        Self { known_namespaces }
    }

    /// Register an additional capability namespace.
    pub fn add_namespace(&mut self, namespace: impl Into<String>) {
        self.known_namespaces.insert(namespace.into());
    }

    /// Validate a graph.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Node IDs are not unique
    /// - Dependencies reference non-existent nodes
    /// - Dependencies form a cycle
    /// - Capabilities are malformed
    pub fn validate(&self, graph: &DeploymentGraph) -> Result<()> {
        Self::validate_unique_node_ids(graph)?;
        Self::validate_dependencies_exist(graph)?;
        Self::validate_no_cycles(graph)?;
        self.validate_capabilities(graph)?;

        Ok(())
    }

    /// Ensure all node IDs are unique within the graph.
    fn validate_unique_node_ids(graph: &DeploymentGraph) -> Result<()> {
        let mut seen = HashSet::new();

        for node in graph.nodes() {
            if !seen.insert(node.id.as_str()) {
                return Err(GraphError::Validation(format!(
                    "Duplicate node ID: {}",
                    node.id
                )));
            }
        }

        Ok(())
    }

    /// Ensure all dependencies reference existing nodes.
    fn validate_dependencies_exist(graph: &DeploymentGraph) -> Result<()> {
        let node_ids: HashSet<&str> = graph.nodes().iter().map(|n| n.id.as_str()).collect();

        for node in graph.nodes() {
            for dep in &node.depends_on {
                if !node_ids.contains(dep.as_str()) {
                    return Err(GraphError::Validation(format!(
                        "Node '{}' depends on non-existent node '{}'",
                        node.id, dep
                    )));
                }
            }
        }

        Ok(())
    }

    /// Detect dependency cycles using DFS.
    fn validate_no_cycles(graph: &DeploymentGraph) -> Result<()> {
        let nodes = graph.nodes();

        // Build adjacency list
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for node in nodes {
            adj.insert(
                node.id.as_str(),
                node.depends_on.iter().map(String::as_str).collect(),
            );
        }

        // DFS cycle detection
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in nodes {
            if !visited.contains(node.id.as_str()) {
                if let Some(cycle_node) =
                    Self::detect_cycle(node.id.as_str(), &adj, &mut visited, &mut rec_stack)
                {
                    return Err(GraphError::CyclicDependency(format!(
                        "Dependency cycle detected involving node '{cycle_node}'"
                    )));
                }
            }
        }

        Ok(())
    }

    /// DFS helper to detect cycles (static method - no &self needed).
    fn detect_cycle<'a>(
        node_id: &'a str,
        adj: &HashMap<&str, Vec<&'a str>>,
        visited: &mut HashSet<&'a str>,
        rec_stack: &mut HashSet<&'a str>,
    ) -> Option<&'a str> {
        visited.insert(node_id);
        rec_stack.insert(node_id);

        if let Some(deps) = adj.get(node_id) {
            for &dep in deps {
                if !visited.contains(dep) {
                    if let Some(cycle) = Self::detect_cycle(dep, adj, visited, rec_stack) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(dep) {
                    return Some(dep);
                }
            }
        }

        rec_stack.remove(node_id);
        None
    }

    /// Validate capability names are well-formed.
    fn validate_capabilities(&self, graph: &DeploymentGraph) -> Result<()> {
        for node in graph.nodes() {
            if let Some(capability) = &node.capability {
                // Capability should be namespace.operation
                if let Some((namespace, _operation)) = capability.split_once('.') {
                    // Warn if namespace is unknown (but don't fail)
                    if !self.known_namespaces.contains(namespace) {
                        tracing::warn!(
                            namespace,
                            node_id = %node.id,
                            "Unknown capability namespace in node",
                        );
                    }
                } else {
                    return Err(GraphError::Validation(format!(
                        "Capability '{}' in node '{}' should be namespace.operation format",
                        capability, node.id
                    )));
                }
            }
        }

        Ok(())
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::loader::GraphLoader;

    #[test]
    fn test_validator_default() {
        let v1 = GraphValidator::new();
        let v2 = GraphValidator::default();
        // Both should have the same known namespaces
        assert_eq!(v1.known_namespaces.len(), v2.known_namespaces.len());
    }

    #[test]
    fn test_known_namespaces() {
        let validator = GraphValidator::new();
        let expected = vec![
            "crypto",
            "genetic",
            "filesystem",
            "process",
            "network",
            "config",
            "biomeos",
            "http",
            "birdsong",
        ];
        for ns in expected {
            assert!(
                validator.known_namespaces.contains(ns),
                "Missing namespace: {ns}"
            );
        }
    }

    #[test]
    fn test_add_namespace() {
        let mut validator = GraphValidator::new();
        assert!(!validator.known_namespaces.contains("custom"));
        validator.add_namespace("custom");
        assert!(validator.known_namespaces.contains("custom"));
    }

    #[test]
    fn test_valid_graph_passes() {
        let toml = r#"
            [graph]
            id = "valid-graph"
            name = "Valid"
            version = "1.0.0"

            [[graph.nodes]]
            id = "step-one"
            name = "Step 1"
            capability = "crypto.hash"

            [[graph.nodes]]
            id = "step-two"
            name = "Step 2"
            capability = "filesystem.write"
            depends_on = ["step-one"]
        "#;
        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_cycle() {
        let toml = r#"
            [graph]
            id = "cycle-test"
            name = "Cycle Test"
            version = "1.0.0"
            
            [[graph.nodes]]
            id = "a"
            name = "A"
            depends_on = ["b"]
            
            [[graph.nodes]]
            id = "b"
            name = "B"
            depends_on = ["c"]
            
            [[graph.nodes]]
            id = "c"
            name = "C"
            depends_on = ["a"]
        "#;

        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cycle"));
    }

    #[test]
    fn test_duplicate_node_id() {
        let toml = r#"
            [graph]
            id = "dup-test"
            name = "Duplicate Test"
            version = "1.0.0"
            
            [[graph.nodes]]
            id = "same-id"
            name = "First"
            
            [[graph.nodes]]
            id = "same-id"
            name = "Second"
        "#;

        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_invalid_capability_format() {
        let toml = r#"
            [graph]
            id = "cap-test"
            name = "Capability Test"
            version = "1.0.0"
            
            [[graph.nodes]]
            id = "bad-cap"
            name = "Bad Capability"
            capability = "no-namespace"
        "#;

        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_capability_known_namespace() {
        let toml = r#"
            [graph]
            id = "known-ns"
            name = "Known Namespace"
            version = "1.0.0"

            [[graph.nodes]]
            id = "hash-node"
            name = "Hash"
            capability = "crypto.blake3_hash"
        "#;
        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_missing_dependency_node() {
        let toml = r#"
            [graph]
            id = "missing-dep"
            name = "Missing Dep"
            version = "1.0.0"

            [[graph.nodes]]
            id = "step-one"
            name = "Step 1"
            depends_on = ["nonexistent"]
        "#;
        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("non-existent"));
    }

    #[test]
    fn test_self_dependency_cycle() {
        let toml = r#"
            [graph]
            id = "self-dep"
            name = "Self Dep"
            version = "1.0.0"

            [[graph.nodes]]
            id = "loop-node"
            name = "Loop"
            depends_on = ["loop-node"]
        "#;
        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_nodes_graph() {
        let toml = r#"
            [graph]
            id = "empty-graph"
            name = "Empty"
            version = "1.0.0"
        "#;
        // A graph with no nodes should still parse (nodes are optional in definition)
        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_node_with_no_capability_is_valid() {
        let toml = r#"
            [graph]
            id = "no-cap"
            name = "No Capability"
            version = "1.0.0"

            [[graph.nodes]]
            id = "basic-node"
            name = "Basic"
        "#;
        // Nodes without capability should pass validation
        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_ok());
    }
}

//! Graph validation for structural correctness.
//!
//! Validates:
//! - Node IDs are unique
//! - Dependencies exist
//! - No dependency cycles
//! - Capabilities are well-formed

use std::collections::{HashMap, HashSet};

use crate::{error::GraphError, graph::DeploymentGraph, Result};

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
        self.validate_unique_node_ids(graph)?;
        self.validate_dependencies_exist(graph)?;
        self.validate_no_cycles(graph)?;
        self.validate_capabilities(graph)?;

        Ok(())
    }

    /// Ensure all node IDs are unique within the graph.
    fn validate_unique_node_ids(&self, graph: &DeploymentGraph) -> Result<()> {
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
    fn validate_dependencies_exist(&self, graph: &DeploymentGraph) -> Result<()> {
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
    fn validate_no_cycles(&self, graph: &DeploymentGraph) -> Result<()> {
        let nodes = graph.nodes();

        // Build adjacency list
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for node in nodes {
            adj.insert(
                node.id.as_str(),
                node.depends_on.iter().map(|s| s.as_str()).collect(),
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
                        "Dependency cycle detected involving node '{}'",
                        cycle_node
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
                        eprintln!(
                            "Warning: Unknown capability namespace '{}' in node '{}'",
                            namespace, node.id
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

#[cfg(test)]
mod tests {
    // GraphValidator used via GraphLoader
    #[allow(unused_imports)]
    use super::*;
    use crate::loader::GraphLoader;

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
}

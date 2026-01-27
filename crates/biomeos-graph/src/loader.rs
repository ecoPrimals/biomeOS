//! Graph loading from TOML files.
//!
//! The loader validates graphs at load time, ensuring
//! structural correctness before runtime execution.

use std::path::Path;

use crate::{error::GraphError, graph::DeploymentGraph, validation::GraphValidator, Result};

/// Loads and validates deployment graphs.
pub struct GraphLoader {
    /// Validator for structural checks
    validator: GraphValidator,
}

impl Default for GraphLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphLoader {
    /// Create a new graph loader with default settings.
    pub fn new() -> Self {
        Self {
            validator: GraphValidator::new(),
        }
    }

    /// Load a graph from a TOML file.
    ///
    /// This performs:
    /// 1. File reading
    /// 2. TOML parsing into typed structs
    /// 3. Structural validation
    /// 4. Dependency cycle detection
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - File cannot be read
    /// - TOML is malformed
    /// - Graph structure is invalid
    /// - Dependencies form a cycle
    pub fn from_file(path: impl AsRef<Path>) -> Result<DeploymentGraph> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .map_err(|e| GraphError::Io(format!("Failed to read {}: {}", path.display(), e)))?;

        Self::from_str(&content, Some(path))
    }

    /// Load a graph from a TOML string.
    ///
    /// Optionally provide a path for error messages.
    pub fn from_str(content: &str, source: Option<&Path>) -> Result<DeploymentGraph> {
        let loader = Self::new();
        loader.parse_and_validate(content, source)
    }

    /// Parse and validate a graph.
    fn parse_and_validate(&self, content: &str, source: Option<&Path>) -> Result<DeploymentGraph> {
        // Step 1: Parse TOML into typed struct
        let graph: DeploymentGraph = toml::from_str(content).map_err(|e| {
            let location = source
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "<string>".to_string());
            GraphError::Parse(format!("Failed to parse {} as graph: {}", location, e))
        })?;

        // Step 2: Validate structure
        self.validator.validate(&graph)?;

        Ok(graph)
    }
}

/// Load multiple graphs from a directory.
pub fn load_graphs_from_dir(dir: impl AsRef<Path>) -> Result<Vec<DeploymentGraph>> {
    let dir = dir.as_ref();
    let mut graphs = Vec::new();

    for entry in std::fs::read_dir(dir)
        .map_err(|e| GraphError::Io(format!("Failed to read directory {}: {}", dir.display(), e)))?
    {
        let entry =
            entry.map_err(|e| GraphError::Io(format!("Failed to read directory entry: {}", e)))?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "toml") {
            match GraphLoader::from_file(&path) {
                Ok(graph) => graphs.push(graph),
                Err(e) => {
                    // Log warning but continue loading other graphs
                    eprintln!("Warning: Failed to load {}: {}", path.display(), e);
                }
            }
        }
    }

    Ok(graphs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_minimal_graph() {
        let toml = r#"
            [graph]
            id = "test-graph"
            name = "Test Graph"
            version = "1.0.0"
        "#;

        let graph = GraphLoader::from_str(toml, None).unwrap();
        assert_eq!(graph.id().as_str(), "test-graph");
        assert_eq!(graph.name(), "Test Graph");
    }

    #[test]
    fn test_load_graph_with_nodes() {
        let toml = r#"
            [graph]
            id = "deploy-graph"
            name = "Deployment"
            version = "1.0.0"
            
            [[graph.nodes]]
            id = "step-one"
            name = "First Step"
            capability = "test.capability"
            
            [graph.nodes.params]
            target = "/tmp/test"
            
            [[graph.nodes]]
            id = "step-two"
            name = "Second Step"
            capability = "test.another"
            depends_on = ["step-one"]
        "#;

        let graph = GraphLoader::from_str(toml, None).unwrap();
        assert_eq!(graph.nodes().len(), 2);

        let ordered = graph.nodes_in_order();
        assert_eq!(ordered[0].id.as_str(), "step-one");
        assert_eq!(ordered[1].id.as_str(), "step-two");
    }

    #[test]
    fn test_invalid_graph_id() {
        let toml = r#"
            [graph]
            id = "INVALID ID"
            name = "Test"
            version = "1.0.0"
        "#;

        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_err());
    }
}

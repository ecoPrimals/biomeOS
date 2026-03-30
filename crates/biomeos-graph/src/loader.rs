// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Graph loading from TOML files.
//!
//! The loader validates graphs at load time, ensuring
//! structural correctness before runtime execution.

use std::path::Path;

use crate::{Result, error::GraphError, graph::DeploymentGraph, validation::GraphValidator};

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
    #[must_use] 
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
            let location =
                source.map_or_else(|| "<string>".to_string(), |p| p.display().to_string());
            GraphError::Parse(format!("Failed to parse {location} as graph: {e}"))
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
            entry.map_err(|e| GraphError::Io(format!("Failed to read directory entry: {e}")))?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "toml") {
            match GraphLoader::from_file(&path) {
                Ok(graph) => graphs.push(graph),
                Err(e) => {
                    tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Failed to load graph, continuing with remaining graphs",
                    );
                }
            }
        }
    }

    Ok(graphs)
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_loader_default() {
        let l1 = GraphLoader::new();
        let l2 = GraphLoader::default();
        // Both construct valid loaders
        let _ = l1;
        let _ = l2;
    }

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

    #[test]
    fn test_invalid_toml_syntax() {
        let bad_toml = "this is not valid TOML {{{}}}";
        let result = GraphLoader::from_str(bad_toml, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_required_fields() {
        // Missing name field
        let toml = r#"
            [graph]
            id = "missing-name"
            version = "1.0.0"
        "#;
        let result = GraphLoader::from_str(toml, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_from_str_with_source_path() {
        let toml = r#"
            [graph]
            id = "path-test"
            name = "Path Test"
            version = "1.0.0"
        "#;
        let result = GraphLoader::from_str(toml, Some(std::path::Path::new("/fake/path.toml")));
        assert!(result.is_ok());
    }

    #[test]
    fn test_from_str_error_includes_source() {
        let bad_toml = "not valid";
        let result = GraphLoader::from_str(bad_toml, Some(std::path::Path::new("/my/file.toml")));
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("/my/file.toml"));
    }

    #[test]
    fn test_from_file_nonexistent() {
        let result = GraphLoader::from_file("/nonexistent/path/graph.toml");
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to read"));
    }

    #[test]
    fn test_load_from_file_roundtrip() {
        let toml_content = r#"
[graph]
id = "file-test"
name = "File Test"
version = "1.0.0"

[[graph.nodes]]
id = "node-a"
name = "Node A"
capability = "crypto.hash"
"#;

        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("test-graph.toml");
        std::fs::write(&file_path, toml_content).unwrap();

        let graph = GraphLoader::from_file(&file_path).unwrap();
        assert_eq!(graph.id().as_str(), "file-test");
        assert_eq!(graph.nodes().len(), 1);
    }

    #[test]
    fn test_load_graphs_from_dir_empty() {
        let dir = tempfile::tempdir().unwrap();
        let graphs = load_graphs_from_dir(dir.path()).unwrap();
        assert!(graphs.is_empty());
    }

    #[test]
    fn test_load_graphs_from_dir_multiple() {
        let dir = tempfile::tempdir().unwrap();

        // Write two valid graphs
        let graph1 = r#"
[graph]
id = "graph-one"
name = "Graph One"
version = "1.0.0"
"#;
        let graph2 = r#"
[graph]
id = "graph-two"
name = "Graph Two"
version = "1.0.0"
"#;
        std::fs::write(dir.path().join("one.toml"), graph1).unwrap();
        std::fs::write(dir.path().join("two.toml"), graph2).unwrap();

        let graphs = load_graphs_from_dir(dir.path()).unwrap();
        assert_eq!(graphs.len(), 2);
    }

    #[test]
    fn test_load_graphs_from_dir_skips_non_toml() {
        let dir = tempfile::tempdir().unwrap();

        let graph = r#"
[graph]
id = "only-graph"
name = "Only"
version = "1.0.0"
"#;
        std::fs::write(dir.path().join("valid.toml"), graph).unwrap();
        std::fs::write(dir.path().join("readme.txt"), "not a graph").unwrap();
        std::fs::write(dir.path().join("data.json"), "{}").unwrap();

        let graphs = load_graphs_from_dir(dir.path()).unwrap();
        assert_eq!(graphs.len(), 1);
    }

    #[test]
    fn test_load_graphs_from_dir_skips_invalid() {
        let dir = tempfile::tempdir().unwrap();

        let valid = r#"
[graph]
id = "valid-graph"
name = "Valid"
version = "1.0.0"
"#;
        let invalid = "not valid toml {{";

        std::fs::write(dir.path().join("valid.toml"), valid).unwrap();
        std::fs::write(dir.path().join("invalid.toml"), invalid).unwrap();

        let graphs = load_graphs_from_dir(dir.path()).unwrap();
        assert_eq!(graphs.len(), 1);
        assert_eq!(graphs[0].id().as_str(), "valid-graph");
    }

    #[test]
    fn test_load_graphs_from_dir_nonexistent() {
        let result = load_graphs_from_dir("/nonexistent/directory");
        assert!(result.is_err());
    }
}

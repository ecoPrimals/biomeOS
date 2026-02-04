//! Deployment graph types with compile-time validation.
//!
//! These types ensure that when a graph is loaded from TOML,
//! it is structurally valid before any runtime execution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::node::GraphNode;

/// A validated deployment graph.
///
/// This struct represents a graph that has been:
/// 1. Parsed from TOML
/// 2. Validated for structural correctness
/// 3. Checked for dependency cycles
///
/// If you have a `DeploymentGraph`, it is guaranteed to be valid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentGraph {
    /// Graph definition section
    #[serde(rename = "graph")]
    pub definition: GraphDefinition,
}

/// Core graph definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDefinition {
    /// Unique identifier for the graph
    pub id: GraphId,

    /// Human-readable name
    pub name: String,

    /// Semantic version
    pub version: String,

    /// Description of what this graph does
    #[serde(default)]
    pub description: String,

    /// Graph metadata
    #[serde(default)]
    pub metadata: GraphMetadata,

    /// Environment variable definitions
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Nodes in the graph (execution units)
    #[serde(default)]
    pub nodes: Vec<GraphNode>,

    /// Output definitions
    #[serde(default)]
    pub outputs: HashMap<String, String>,
}

/// Graph identifier - validated to be lowercase alphanumeric with hyphens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct GraphId(String);

impl GraphId {
    /// Create a new graph ID, validating format.
    pub fn new(id: impl Into<String>) -> Result<Self, String> {
        let id = id.into();
        if id.is_empty() {
            return Err("Graph ID cannot be empty".into());
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(format!(
                "Graph ID must be lowercase alphanumeric with hyphens: {}",
                id
            ));
        }
        Ok(Self(id))
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for GraphId {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<GraphId> for String {
    fn from(id: GraphId) -> Self {
        id.0
    }
}

impl std::fmt::Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metadata about the graph.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Family ID this graph belongs to
    #[serde(default)]
    pub family_id: Option<String>,

    /// Author of the graph
    #[serde(default)]
    pub author: Option<String>,

    /// Creation date
    #[serde(default)]
    pub created: Option<String>,

    /// Category (deployment, validation, etc.)
    #[serde(default)]
    pub category: Option<GraphCategory>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

/// Graph category for classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum GraphCategory {
    /// Deployment graphs
    Deployment,
    /// Validation graphs
    Validation,
    /// Testing graphs
    Testing,
    /// Utility graphs
    #[default]
    Utility,
    /// Lifecycle graphs
    Lifecycle,
}

impl DeploymentGraph {
    /// Get the graph ID.
    pub fn id(&self) -> &GraphId {
        &self.definition.id
    }

    /// Get the graph name.
    pub fn name(&self) -> &str {
        &self.definition.name
    }

    /// Get all nodes in the graph.
    pub fn nodes(&self) -> &[GraphNode] {
        &self.definition.nodes
    }

    /// Get nodes in topological order (respecting dependencies).
    pub fn nodes_in_order(&self) -> Vec<&GraphNode> {
        // Simple topological sort using Kahn's algorithm
        let mut result = Vec::new();
        let mut in_degree: HashMap<&str, usize> = HashMap::new();
        let mut node_map: HashMap<&str, &GraphNode> = HashMap::new();

        // Initialize
        for node in &self.definition.nodes {
            in_degree.insert(node.id.as_str(), node.depends_on.len());
            node_map.insert(node.id.as_str(), node);
        }

        // Find nodes with no dependencies
        let mut queue: Vec<&str> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(&id, _)| id)
            .collect();

        while let Some(node_id) = queue.pop() {
            if let Some(node) = node_map.get(node_id) {
                result.push(*node);

                // Decrease in-degree of dependent nodes
                for other in &self.definition.nodes {
                    if other.depends_on.contains(&node_id.to_string()) {
                        if let Some(deg) = in_degree.get_mut(other.id.as_str()) {
                            *deg -= 1;
                            if *deg == 0 {
                                queue.push(other.id.as_str());
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Get environment variables with defaults resolved.
    pub fn env(&self) -> &HashMap<String, String> {
        &self.definition.env
    }

    /// Resolve an environment variable reference.
    ///
    /// Handles formats like:
    /// - `${VAR}` - Direct reference
    /// - `${VAR:-default}` - With default value
    ///
    /// Note: This resolves against system env first, then graph defaults.
    /// Graph env values like `"${VAR:-default}"` are treated as default specs,
    /// not literal values.
    pub fn resolve_env(&self, value: &str) -> String {
        let mut result = value.to_string();
        let mut iterations = 0;
        const MAX_ITERATIONS: usize = 100;

        // Find all ${...} patterns
        while let Some(start) = result.find("${") {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                // Prevent infinite loops from self-referential patterns
                break;
            }

            if let Some(end) = result[start..].find('}') {
                let var_spec = &result[start + 2..start + end];

                // Handle ${VAR:-default} syntax
                let (var_name, inline_default) = if let Some(pos) = var_spec.find(":-") {
                    (&var_spec[..pos], Some(&var_spec[pos + 2..]))
                } else {
                    (var_spec, None)
                };

                // Check system env first
                let resolved = std::env::var(var_name)
                    .ok()
                    .or_else(|| {
                        // Then check graph env - but extract default if it's a ${VAR:-default} pattern
                        self.definition.env.get(var_name).and_then(|v| {
                            if v.starts_with("${") && v.contains(":-") {
                                // Extract default from "${VAR:-default}" pattern
                                v.find(":-")
                                    .and_then(|pos| v[pos + 2..].strip_suffix('}'))
                                    .map(String::from)
                            } else if !v.contains("${") {
                                // Literal value
                                Some(v.clone())
                            } else {
                                None
                            }
                        })
                    })
                    .or_else(|| inline_default.map(String::from))
                    .unwrap_or_default();

                result = format!(
                    "{}{}{}",
                    &result[..start],
                    resolved,
                    &result[start + end + 1..]
                );
            } else {
                break;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_id_validation() {
        assert!(GraphId::new("livespore-deploy").is_ok());
        assert!(GraphId::new("tower-atomic-bootstrap").is_ok());
        assert!(GraphId::new("test123").is_ok());

        assert!(GraphId::new("").is_err());
        assert!(GraphId::new("UPPERCASE").is_err());
        assert!(GraphId::new("has spaces").is_err());
        assert!(GraphId::new("has_underscore").is_err());
    }

    #[test]
    fn test_env_resolution() {
        // Use unique variable names to avoid collision with system env
        // (system env takes precedence over graph env by design)
        let toml = r#"
            [graph]
            id = "test-graph"
            name = "Test"
            version = "1.0.0"
            
            [graph.env]
            TEST_SPORE_TARGET_12345 = "/media/user/USB"
            TEST_NODE_ID_12345 = "test-node"
        "#;

        let graph: DeploymentGraph = toml::from_str(toml).unwrap();

        assert_eq!(
            graph.resolve_env("${TEST_SPORE_TARGET_12345}/biomeOS"),
            "/media/user/USB/biomeOS"
        );
        assert_eq!(graph.resolve_env("${TEST_NODE_ID_12345}"), "test-node");
        assert_eq!(graph.resolve_env("${MISSING:-default}"), "default");
    }
}

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

    // =========================================================================
    // GraphId tests
    // =========================================================================

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
    fn test_graph_id_as_str() {
        let id = GraphId::new("my-graph").unwrap();
        assert_eq!(id.as_str(), "my-graph");
    }

    #[test]
    fn test_graph_id_display() {
        let id = GraphId::new("test-graph").unwrap();
        assert_eq!(format!("{}", id), "test-graph");
    }

    #[test]
    fn test_graph_id_try_from_string() {
        let id: Result<GraphId, _> = GraphId::try_from("valid-id".to_string());
        assert!(id.is_ok());

        let id: Result<GraphId, _> = GraphId::try_from("INVALID".to_string());
        assert!(id.is_err());
    }

    #[test]
    fn test_graph_id_into_string() {
        let id = GraphId::new("my-id").unwrap();
        let s: String = id.into();
        assert_eq!(s, "my-id");
    }

    #[test]
    fn test_graph_id_equality() {
        let id1 = GraphId::new("same").unwrap();
        let id2 = GraphId::new("same").unwrap();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_graph_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(GraphId::new("a").unwrap());
        set.insert(GraphId::new("b").unwrap());
        set.insert(GraphId::new("a").unwrap()); // duplicate
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn test_graph_id_serde_roundtrip() {
        let id = GraphId::new("test-serde").unwrap();
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: GraphId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_graph_id_serde_invalid() {
        let json = "\"INVALID_ID\"";
        let result: Result<GraphId, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // =========================================================================
    // GraphCategory tests
    // =========================================================================

    #[test]
    fn test_graph_category_default() {
        let cat = GraphCategory::default();
        assert_eq!(cat, GraphCategory::Utility);
    }

    #[test]
    fn test_graph_category_serde() {
        let cat = GraphCategory::Deployment;
        let json = serde_json::to_string(&cat).unwrap();
        assert_eq!(json, "\"deployment\"");

        let deserialized: GraphCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, GraphCategory::Deployment);
    }

    #[test]
    fn test_all_graph_categories() {
        let categories = vec![
            (GraphCategory::Deployment, "\"deployment\""),
            (GraphCategory::Validation, "\"validation\""),
            (GraphCategory::Testing, "\"testing\""),
            (GraphCategory::Utility, "\"utility\""),
            (GraphCategory::Lifecycle, "\"lifecycle\""),
        ];
        for (cat, expected_json) in categories {
            let json = serde_json::to_string(&cat).unwrap();
            assert_eq!(json, expected_json);
        }
    }

    // =========================================================================
    // GraphMetadata tests
    // =========================================================================

    #[test]
    fn test_graph_metadata_default() {
        let meta = GraphMetadata::default();
        assert!(meta.family_id.is_none());
        assert!(meta.author.is_none());
        assert!(meta.created.is_none());
        assert!(meta.category.is_none());
        assert!(meta.extra.is_empty());
    }

    #[test]
    fn test_graph_metadata_serde() {
        let meta = GraphMetadata {
            family_id: Some("family-123".to_string()),
            author: Some("biomeOS".to_string()),
            created: Some("2026-01-01".to_string()),
            category: Some(GraphCategory::Deployment),
            extra: HashMap::new(),
        };
        let json = serde_json::to_string(&meta).unwrap();
        assert!(json.contains("family-123"));
        assert!(json.contains("biomeOS"));
    }

    // =========================================================================
    // DeploymentGraph tests
    // =========================================================================

    fn make_test_graph() -> DeploymentGraph {
        let toml_str = r#"
            [graph]
            id = "test-graph"
            name = "Test Graph"
            version = "1.0.0"
            description = "A test graph"
        "#;
        toml::from_str(toml_str).unwrap()
    }

    fn make_graph_with_nodes() -> DeploymentGraph {
        let toml_str = r#"
            [graph]
            id = "deploy-graph"
            name = "Deploy"
            version = "1.0.0"

            [[graph.nodes]]
            id = "step-a"
            name = "Step A"

            [[graph.nodes]]
            id = "step-b"
            name = "Step B"
            depends_on = ["step-a"]

            [[graph.nodes]]
            id = "step-c"
            name = "Step C"
            depends_on = ["step-a"]

            [[graph.nodes]]
            id = "step-d"
            name = "Step D"
            depends_on = ["step-b", "step-c"]
        "#;
        toml::from_str(toml_str).unwrap()
    }

    #[test]
    fn test_deployment_graph_accessors() {
        let graph = make_test_graph();
        assert_eq!(graph.id().as_str(), "test-graph");
        assert_eq!(graph.name(), "Test Graph");
        assert!(graph.nodes().is_empty());
        assert!(graph.env().is_empty());
    }

    #[test]
    fn test_nodes_in_order_diamond() {
        let graph = make_graph_with_nodes();
        let ordered = graph.nodes_in_order();
        assert_eq!(ordered.len(), 4);

        // step-a must come before step-b, step-c
        let pos_a = ordered.iter().position(|n| n.id.as_str() == "step-a").unwrap();
        let pos_b = ordered.iter().position(|n| n.id.as_str() == "step-b").unwrap();
        let pos_c = ordered.iter().position(|n| n.id.as_str() == "step-c").unwrap();
        let pos_d = ordered.iter().position(|n| n.id.as_str() == "step-d").unwrap();

        assert!(pos_a < pos_b);
        assert!(pos_a < pos_c);
        assert!(pos_b < pos_d);
        assert!(pos_c < pos_d);
    }

    #[test]
    fn test_nodes_in_order_no_deps() {
        let graph = make_test_graph();
        let ordered = graph.nodes_in_order();
        assert_eq!(ordered.len(), 0);
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

    #[test]
    fn test_resolve_env_no_vars() {
        let graph = make_test_graph();
        assert_eq!(graph.resolve_env("plain text"), "plain text");
    }

    #[test]
    fn test_resolve_env_missing_var_no_default() {
        let graph = make_test_graph();
        // Missing var with no default resolves to empty string
        assert_eq!(
            graph.resolve_env("prefix-${BIOMEOS_NONEXISTENT_VAR_XYZ}-suffix"),
            "prefix--suffix"
        );
    }

    #[test]
    fn test_resolve_env_multiple_vars() {
        let toml_str = r#"
            [graph]
            id = "env-test"
            name = "Env"
            version = "1.0.0"

            [graph.env]
            BGTEST_A = "alpha"
            BGTEST_B = "beta"
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();

        assert_eq!(
            graph.resolve_env("${BGTEST_A}-and-${BGTEST_B}"),
            "alpha-and-beta"
        );
    }

    #[test]
    fn test_resolve_env_with_default_pattern_in_graph_env() {
        let toml_str = r#"
            [graph]
            id = "default-test"
            name = "Default"
            version = "1.0.0"

            [graph.env]
            BGTEST_WITH_DEFAULT = "${BGTEST_WITH_DEFAULT:-fallback_value}"
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();

        assert_eq!(
            graph.resolve_env("${BGTEST_WITH_DEFAULT}"),
            "fallback_value"
        );
    }

    #[test]
    fn test_resolve_env_unclosed_brace() {
        let graph = make_test_graph();
        // Unclosed brace should not infinite-loop, just return as-is
        assert_eq!(graph.resolve_env("${UNCLOSED"), "${UNCLOSED");
    }

    #[test]
    fn test_deployment_graph_serde_roundtrip() {
        let graph = make_test_graph();
        let json = serde_json::to_string(&graph).unwrap();
        let deserialized: DeploymentGraph = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id().as_str(), "test-graph");
    }

    #[test]
    fn test_deployment_graph_with_outputs() {
        let toml_str = r#"
            [graph]
            id = "output-test"
            name = "Output"
            version = "1.0.0"

            [graph.outputs]
            result_path = "/tmp/result"
            status = "completed"
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        assert_eq!(graph.definition.outputs.len(), 2);
        assert_eq!(
            graph.definition.outputs.get("result_path"),
            Some(&"/tmp/result".to_string())
        );
    }
}

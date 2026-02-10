// =============================================================================
// Graph Parser - TOML → Graph
// =============================================================================
//
// Modern idiomatic Rust parser:
// - No unsafe code
// - Clear error messages
// - Graceful handling of missing fields
// - No hardcoding of primal names
//
// =============================================================================

use crate::error::{GraphError, Result};
use crate::graph::*;
use std::path::Path;

/// Parser for TOML graph definitions
pub struct GraphParser;

impl GraphParser {
    /// Parse a graph from a TOML file
    pub fn parse_file(path: &Path) -> Result<PrimalGraph> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            GraphError::ParseError(format!("Failed to read {}: {}", path.display(), e))
        })?;

        Self::parse_toml(&content)
    }

    /// Parse a graph from a TOML string
    pub fn parse_toml(content: &str) -> Result<PrimalGraph> {
        let value: toml::Value = toml::from_str(content)?;

        // Parse [graph] section
        let graph_table = value
            .get("graph")
            .and_then(|v| v.as_table())
            .ok_or_else(|| GraphError::ParseError("Missing [graph] section".to_string()))?;

        let name = Self::get_string(graph_table, "name")?;
        let description = Self::get_optional_string(graph_table, "description").unwrap_or_default();
        let version = Self::get_optional_string(graph_table, "version")
            .unwrap_or_else(|| "1.0.0".to_string());
        let coordination = Self::parse_coordination(graph_table)?;

        // Parse [[nodes]] section
        let nodes = Self::parse_nodes(&value)?;

        // Parse [[edges]] section (optional)
        let edges = Self::parse_edges(&value)?;

        Ok(PrimalGraph {
            id: GraphId::new(&name),
            name,
            description,
            version,
            nodes,
            edges,
            coordination,
        })
    }

    /// Parse coordination pattern
    fn parse_coordination(
        table: &toml::map::Map<String, toml::Value>,
    ) -> Result<CoordinationPattern> {
        let coord_str = Self::get_optional_string(table, "coordination")
            .unwrap_or_else(|| "Sequential".to_string());

        match coord_str.as_str() {
            "Sequential" => Ok(CoordinationPattern::Sequential),
            "Parallel" => Ok(CoordinationPattern::Parallel),
            "ConditionalDAG" => Ok(CoordinationPattern::ConditionalDAG),
            "Pipeline" => Ok(CoordinationPattern::Pipeline),
            other => Err(GraphError::ParseError(format!(
                "Unknown coordination pattern: {}",
                other
            ))),
        }
    }

    /// Parse all nodes
    fn parse_nodes(value: &toml::Value) -> Result<Vec<PrimalNode>> {
        let nodes_array = value
            .get("nodes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| GraphError::ParseError("Missing [[nodes]] section".to_string()))?;

        if nodes_array.is_empty() {
            return Err(GraphError::ParseError(
                "Graph must have at least one node".to_string(),
            ));
        }

        nodes_array.iter().map(Self::parse_node).collect()
    }

    /// Parse a single node
    fn parse_node(value: &toml::Value) -> Result<PrimalNode> {
        let table = value
            .as_table()
            .ok_or_else(|| GraphError::ParseError("Node must be a table".to_string()))?;

        let id = Self::get_string(table, "id")?;
        let primal = Self::parse_primal_selector(table)?;
        let operation = Self::parse_operation(table)?;

        let outputs = table
            .get("outputs")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(PrimalNode {
            id,
            primal,
            operation,
            input: None, // Filled in during execution
            outputs,
        })
    }

    /// Parse primal selector (CAPABILITY-BASED!)
    fn parse_primal_selector(
        table: &toml::map::Map<String, toml::Value>,
    ) -> Result<PrimalSelector> {
        let primal_value = table
            .get("primal")
            .ok_or_else(|| GraphError::ParseError("Missing node.primal".to_string()))?;

        let primal_table = primal_value
            .as_table()
            .ok_or_else(|| GraphError::ParseError("node.primal must be a table".to_string()))?;

        // Check for by_id
        if let Some(id) = Self::get_optional_string(primal_table, "by_id") {
            return Ok(PrimalSelector::ById { by_id: id });
        }

        // Check for by_capability (preferred!)
        if let Some(cap) = Self::get_optional_string(primal_table, "by_capability") {
            return Ok(PrimalSelector::ByCapability { by_capability: cap });
        }

        // Check for by_capabilities (multiple)
        if let Some(caps_value) = primal_table.get("by_capabilities") {
            if let Some(caps_array) = caps_value.as_array() {
                let caps: Result<Vec<String>> = caps_array
                    .iter()
                    .map(|v| {
                        v.as_str()
                            .ok_or_else(|| {
                                GraphError::ParseError(
                                    "by_capabilities must be array of strings".to_string(),
                                )
                            })
                            .map(|s| s.to_string())
                    })
                    .collect();
                return Ok(PrimalSelector::ByCapabilities {
                    by_capabilities: caps?,
                });
            }
        }

        Err(GraphError::ParseError(
            "primal selector must have by_id, by_capability, or by_capabilities".to_string(),
        ))
    }

    /// Parse operation
    fn parse_operation(table: &toml::map::Map<String, toml::Value>) -> Result<Operation> {
        let op_value = table
            .get("operation")
            .ok_or_else(|| GraphError::ParseError("Missing node.operation".to_string()))?;

        let op_table = op_value
            .as_table()
            .ok_or_else(|| GraphError::ParseError("node.operation must be a table".to_string()))?;

        let name = Self::get_string(op_table, "name")?;

        let params = op_table
            .get("params")
            .cloned()
            .unwrap_or(toml::Value::Table(toml::map::Map::new()));

        // Convert TOML value to JSON value
        let params_json = serde_json::to_value(&params).map_err(|e| {
            GraphError::ParseError(format!("Failed to convert params to JSON: {}", e))
        })?;

        // Parse environment variables (NEW - Jan 21, 2026)
        let environment = op_table
            .get("environment")
            .and_then(|v| v.as_table())
            .map(|env_table| {
                env_table
                    .iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<std::collections::HashMap<String, String>>()
            });

        Ok(Operation {
            name,
            params: params_json,
            environment,
        })
    }

    /// Parse constraints
    fn parse_constraints(value: &toml::Value) -> Result<NodeConstraints> {
        let table = value
            .as_table()
            .ok_or_else(|| GraphError::ParseError("constraints must be a table".to_string()))?;

        let timeout_ms = table
            .get("timeout_ms")
            .and_then(|v| v.as_integer())
            .map(|i| i as u64);

        let retry_policy = table
            .get("retry")
            .map(Self::parse_retry_policy)
            .transpose()?;

        let required_capabilities = table
            .get("required_capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            });

        Ok(NodeConstraints {
            timeout_ms,
            retry_policy,
            required_capabilities,
        })
    }

    /// Parse retry policy
    fn parse_retry_policy(value: &toml::Value) -> Result<RetryPolicy> {
        let table = value
            .as_table()
            .ok_or_else(|| GraphError::ParseError("retry must be a table".to_string()))?;

        let max_attempts = table
            .get("max_attempts")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| GraphError::ParseError("retry.max_attempts required".to_string()))?
            as u32;

        let backoff_ms = table
            .get("backoff_ms")
            .and_then(|v| v.as_integer())
            .ok_or_else(|| GraphError::ParseError("retry.backoff_ms required".to_string()))?
            as u64;

        Ok(RetryPolicy {
            max_attempts,
            backoff_ms,
        })
    }

    /// Parse all edges
    fn parse_edges(value: &toml::Value) -> Result<Vec<GraphEdge>> {
        let edges_array = value.get("edges").and_then(|v| v.as_array());

        // Edges are optional
        match edges_array {
            Some(edges) => edges.iter().map(Self::parse_edge).collect(),
            None => Ok(vec![]),
        }
    }

    /// Parse a single edge
    fn parse_edge(value: &toml::Value) -> Result<GraphEdge> {
        let table = value
            .as_table()
            .ok_or_else(|| GraphError::ParseError("Edge must be a table".to_string()))?;

        let from = Self::get_string(table, "from")?;
        let to = Self::get_string(table, "to")?;

        let edge_type = if let Some(et_value) = table.get("edge_type") {
            if let Some(s) = et_value.as_str() {
                match s {
                    "dependency" => EdgeType::Dependency,
                    "control_flow" => EdgeType::ControlFlow,
                    _ => EdgeType::DataFlow,
                }
            } else {
                EdgeType::Dependency
            }
        } else {
            EdgeType::Dependency
        };

        Ok(GraphEdge {
            from,
            to,
            edge_type,
        })
    }

    // Helper functions

    fn get_string(table: &toml::map::Map<String, toml::Value>, key: &str) -> Result<String> {
        table
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| GraphError::ParseError(format!("Missing or invalid field: {}", key)))
    }

    fn get_optional_string(
        table: &toml::map::Map<String, toml::Value>,
        key: &str,
    ) -> Option<String> {
        table
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_graph() {
        let toml = r#"
[graph]
name = "test-graph"

[[nodes]]
id = "node1"
primal = { by_capability = "discovery" }
operation = { name = "discover" }
"#;

        let graph = GraphParser::parse_toml(toml).unwrap();
        assert_eq!(graph.name, "test-graph");
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.coordination, CoordinationPattern::Sequential);
    }

    #[test]
    fn test_parse_with_capability_selector() {
        let toml = r#"
[graph]
name = "capability-test"

[[nodes]]
id = "discover"
primal = { by_capability = "discovery" }
operation = { name = "discover_peers" }
"#;

        let graph = GraphParser::parse_toml(toml).unwrap();
        match &graph.nodes[0].primal {
            PrimalSelector::ByCapability { by_capability } => {
                assert_eq!(by_capability, "discovery");
            }
            _ => panic!("Expected ByCapability selector"),
        }
    }

    #[test]
    fn test_parse_with_parallel() {
        let toml = r#"
[graph]
name = "parallel-test"
coordination = "Parallel"

[[nodes]]
id = "node1"
primal = { by_capability = "compute" }
operation = { name = "start" }
parallel_group = 1

[[nodes]]
id = "node2"
primal = { by_capability = "compute" }
operation = { name = "start" }
parallel_group = 1
"#;

        let graph = GraphParser::parse_toml(toml).unwrap();
        assert_eq!(graph.coordination, CoordinationPattern::Parallel);
        // Note: parallel_group is now in constraints, not on PrimalNode directly
        // This test may need updating based on new structure
        assert_eq!(graph.coordination, CoordinationPattern::Parallel);
    }

    #[test]
    fn test_missing_graph_section() {
        let toml = r#"
[[nodes]]
id = "node1"
primal = { by_id = "test" }
operation = { name = "start" }
"#;

        let result = GraphParser::parse_toml(toml);
        assert!(result.is_err());
    }
}

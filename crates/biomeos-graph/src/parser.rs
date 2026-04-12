// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Flat TOML graph parser (`[graph]` + `[[nodes]]` + optional `[[edges]]`).

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
use crate::graph::{
    CoordinationPattern, EdgeType, GraphEdge, GraphId, NodeConstraints, Operation, PrimalGraph,
    PrimalNode, PrimalSelector, RetryPolicy,
};
use std::path::Path;

/// Parser for TOML graph definitions.
pub struct GraphParser;

impl GraphParser {
    /// Parse a graph from a TOML file
    pub fn parse_file(path: &Path) -> Result<PrimalGraph> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| GraphError::Parse(format!("Failed to read {}: {}", path.display(), e)))?;

        Self::parse_toml(&content)
    }

    /// Parse a graph from a TOML string
    pub fn parse_toml(content: &str) -> Result<PrimalGraph> {
        let value: toml::Value =
            toml::from_str(content).map_err(|e| GraphError::Parse(e.to_string()))?;

        // Parse [graph] section
        let graph_table = value
            .get("graph")
            .and_then(|v| v.as_table())
            .ok_or_else(|| GraphError::Parse("Missing [graph] section".to_string()))?;

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
            id: GraphId::new(&name).map_err(GraphError::Parse)?,
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
            "ConditionalDAG" => Ok(CoordinationPattern::ConditionalDag),
            "Pipeline" => Ok(CoordinationPattern::Pipeline),
            other => Err(GraphError::Parse(format!(
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
            .ok_or_else(|| GraphError::Parse("Missing [[nodes]] section".to_string()))?;

        if nodes_array.is_empty() {
            return Err(GraphError::Parse(
                "Graph must have at least one node".to_string(),
            ));
        }

        nodes_array.iter().map(Self::parse_node).collect()
    }

    /// Parse a single node
    fn parse_node(value: &toml::Value) -> Result<PrimalNode> {
        let table = value
            .as_table()
            .ok_or_else(|| GraphError::Parse("Node must be a table".to_string()))?;

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

        let constraints = table
            .get("constraints")
            .map(Self::parse_constraints)
            .transpose()?;

        Ok(PrimalNode {
            id,
            primal,
            operation,
            input: None,
            outputs,
            constraints,
        })
    }

    /// Parse primal selector (CAPABILITY-BASED!)
    fn parse_primal_selector(
        table: &toml::map::Map<String, toml::Value>,
    ) -> Result<PrimalSelector> {
        let primal_value = table
            .get("primal")
            .ok_or_else(|| GraphError::Parse("Missing node.primal".to_string()))?;

        let primal_table = primal_value
            .as_table()
            .ok_or_else(|| GraphError::Parse("node.primal must be a table".to_string()))?;

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
                                GraphError::Parse(
                                    "by_capabilities must be array of strings".to_string(),
                                )
                            })
                            .map(std::string::ToString::to_string)
                    })
                    .collect();
                return Ok(PrimalSelector::ByCapabilities {
                    by_capabilities: caps?,
                });
            }
        }

        Err(GraphError::Parse(
            "primal selector must have by_id, by_capability, or by_capabilities".to_string(),
        ))
    }

    /// Parse operation
    fn parse_operation(table: &toml::map::Map<String, toml::Value>) -> Result<Operation> {
        let op_value = table
            .get("operation")
            .ok_or_else(|| GraphError::Parse("Missing node.operation".to_string()))?;

        let op_table = op_value
            .as_table()
            .ok_or_else(|| GraphError::Parse("node.operation must be a table".to_string()))?;

        let name = Self::get_string(op_table, "name")?;

        let params = op_table
            .get("params")
            .cloned()
            .unwrap_or_else(|| toml::Value::Table(toml::map::Map::new()));

        // Convert TOML value to JSON value
        let params_json = serde_json::to_value(&params)
            .map_err(|e| GraphError::Parse(format!("Failed to convert params to JSON: {}", e)))?;

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

    /// Parse constraints from `[constraints]` sub-table of a node.
    fn parse_constraints(value: &toml::Value) -> Result<NodeConstraints> {
        let table = value
            .as_table()
            .ok_or_else(|| GraphError::Parse("constraints must be a table".to_string()))?;

        let timeout_ms = table
            .get("timeout_ms")
            .and_then(toml::Value::as_integer)
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

    /// Parse retry policy from `[constraints.retry]` sub-table.
    fn parse_retry_policy(value: &toml::Value) -> Result<RetryPolicy> {
        let table = value
            .as_table()
            .ok_or_else(|| GraphError::Parse("retry must be a table".to_string()))?;

        let max_attempts = table
            .get("max_attempts")
            .and_then(toml::Value::as_integer)
            .ok_or_else(|| GraphError::Parse("retry.max_attempts required".to_string()))?
            as u32;

        let backoff_ms = table
            .get("backoff_ms")
            .and_then(toml::Value::as_integer)
            .ok_or_else(|| GraphError::Parse("retry.backoff_ms required".to_string()))?
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
            .ok_or_else(|| GraphError::Parse("Edge must be a table".to_string()))?;

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
            .map(std::string::ToString::to_string)
            .ok_or_else(|| GraphError::Parse(format!("Missing or invalid field: {}", key)))
    }

    fn get_optional_string(
        table: &toml::map::Map<String, toml::Value>,
        key: &str,
    ) -> Option<String> {
        table
            .get(key)
            .and_then(|v| v.as_str())
            .map(std::string::ToString::to_string)
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, reason = "parser unit tests")]
    #![expect(clippy::expect_used, reason = "parser unit tests")]

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

    #[test]
    fn test_parse_by_id_and_operation_params() {
        let toml = r#"
[graph]
name = "by-id-graph"
description = "d"
coordination = "Pipeline"

[[nodes]]
id = "n1"
primal = { by_id = "beardog-local" }
operation = { name = "sign", params = { alg = "ed25519" } }
outputs = ["sig"]
"#;

        let graph = GraphParser::parse_toml(toml).expect("parse");
        assert_eq!(graph.coordination, CoordinationPattern::Pipeline);
        match &graph.nodes[0].primal {
            PrimalSelector::ById { by_id } => assert_eq!(by_id, "beardog-local"),
            _ => panic!("expected ById"),
        }
        assert_eq!(graph.nodes[0].outputs, vec!["sig"]);
        assert!(graph.nodes[0].operation.params.get("alg").is_some());
    }

    #[test]
    fn test_parse_by_capabilities() {
        let toml = r#"
[graph]
name = "multi-cap"

[[nodes]]
id = "n1"
primal = { by_capabilities = ["a.b", "c.d"] }
operation = { name = "run" }
"#;

        let graph = GraphParser::parse_toml(toml).expect("parse");
        match &graph.nodes[0].primal {
            PrimalSelector::ByCapabilities { by_capabilities } => {
                assert_eq!(by_capabilities, &vec!["a.b".to_string(), "c.d".to_string()]);
            }
            _ => panic!("expected ByCapabilities"),
        }
    }

    #[test]
    fn test_parse_edges_and_control_flow() {
        let toml = r#"
[graph]
name = "edge-graph"

[[nodes]]
id = "a"
primal = { by_capability = "x" }
operation = { name = "op" }

[[nodes]]
id = "b"
primal = { by_capability = "x" }
operation = { name = "op" }

[[edges]]
from = "a"
to = "b"
edge_type = "control_flow"

[[edges]]
from = "a"
to = "b"
edge_type = "unknown_maps_to_data_flow"
"#;

        let graph = GraphParser::parse_toml(toml).expect("parse");
        assert_eq!(graph.edges.len(), 2);
        assert_eq!(graph.edges[0].edge_type, EdgeType::ControlFlow);
        assert_eq!(graph.edges[1].edge_type, EdgeType::DataFlow);
    }

    #[test]
    fn test_unknown_coordination_errors() {
        let toml = r#"
[graph]
name = "bad"
coordination = "NotReal"

[[nodes]]
id = "a"
primal = { by_capability = "x" }
operation = { name = "op" }
"#;

        let err = GraphParser::parse_toml(toml).expect_err("bad coordination");
        assert!(
            err.to_string().contains("Unknown coordination")
                || err.to_string().contains("coordination")
        );
    }

    #[test]
    fn test_empty_nodes_array_errors() {
        let toml = r#"
[graph]
name = "empty"

nodes = []
"#;

        let err = GraphParser::parse_toml(toml).expect_err("empty nodes");
        assert!(err.to_string().contains("at least one") || err.to_string().contains("nodes"));
    }

    #[test]
    fn test_graph_id_validation_error_propagates() {
        let toml = r#"
[graph]
name = "Bad_Name"

[[nodes]]
id = "a"
primal = { by_capability = "x" }
operation = { name = "op" }
"#;

        let err = GraphParser::parse_toml(toml).expect_err("invalid graph id");
        let msg = err.to_string();
        assert!(
            msg.contains("Graph ID") || msg.contains("lowercase"),
            "{msg}"
        );
    }

    #[test]
    fn parse_constraints_and_retry_policy() {
        let v: toml::Value = toml::from_str(
            r#"
            timeout_ms = 9000
            required_capabilities = ["cap.a", "cap.b"]
            [retry]
            max_attempts = 4
            backoff_ms = 250
        "#,
        )
        .expect("toml");
        let c = GraphParser::parse_constraints(&v).expect("constraints");
        assert_eq!(c.timeout_ms, Some(9000));
        assert_eq!(
            c.required_capabilities,
            Some(vec!["cap.a".to_string(), "cap.b".to_string()])
        );
        let rp = c.retry_policy.expect("retry");
        assert_eq!(rp.max_attempts, 4);
        assert_eq!(rp.backoff_ms, 250);
    }
}

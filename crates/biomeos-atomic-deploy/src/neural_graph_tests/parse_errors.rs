// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_graph::*;

#[test]
fn test_from_toml_str_missing_graph_section() {
    let toml = r#"
id = "orphan"
[nodes]
"#;
    let result = Graph::from_toml_str(toml);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("[graph]"));
}

#[test]
fn test_from_toml_str_missing_nodes_array() {
    let toml = r#"
[graph]
id = "no_nodes"
version = "1.0.0"
description = "No nodes"
"#;
    let result = Graph::from_toml_str(toml);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("nodes"));
}

#[test]
fn test_from_toml_str_invalid_toml() {
    let toml = "this is not valid [toml = syntax";
    let result = Graph::from_toml_str(toml);
    assert!(result.is_err());
}

#[test]
fn test_from_toml_str_empty_nodes_array() {
    let toml = r#"
nodes = []

[graph]
id = "empty_graph"
version = "1.0.0"
description = "Empty nodes"
"#;
    let result = Graph::from_toml_str(toml);
    assert!(result.is_ok());
    let graph = result.unwrap();
    assert_eq!(graph.id, "empty_graph");
    assert!(graph.nodes.is_empty());
}

#[test]
fn test_parse_simple_graph() {
    let toml = r#"
[graph]
id = "test_graph"
version = "1.0.0"
description = "Test graph"

[[nodes]]
id = "node1"
node_type = "primal"
type = "test.node"
dependencies = []

[[nodes]]
id = "node2"
node_type = "primal"
type = "test.node"
dependencies = ["node1"]

[execution]
mode = "deterministic"
max_parallelism = 2
"#;

    let graph = Graph::from_toml_str(toml).unwrap();
    assert_eq!(graph.id, "test_graph");
    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.config.max_parallelism, 2);
}

#[test]
fn test_is_continuous_false_for_sequential() {
    let toml = r#"
[graph]
id = "seq-test"
version = "1.0.0"
description = "Sequential"
coordination = "Sequential"

nodes = []
"#;
    let graph = Graph::from_toml_str(toml).unwrap();
    assert!(!graph.is_continuous());
}

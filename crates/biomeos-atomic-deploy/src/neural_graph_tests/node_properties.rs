// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_graph::*;

#[test]
fn test_parse_deployment_graph_format() {
    let toml = r#"
[graph]
id = "test-continuous"
name = "Test Continuous Graph"
version = "1.0.0"
description = "A test graph in DeploymentGraph format"
coordination = "continuous"

[graph.tick]
target_hz = 60.0

[[graph.nodes]]
id = "input"
name = "Input Collection"
capability = "interaction.poll"
budget_ms = 1.0

[graph.nodes.config]
primal = "petaltongue"

[graph.nodes.params]
sources = "keyboard"

[[graph.nodes]]
id = "logic"
name = "Game Logic"
capability = "game.tick_logic"
depends_on = ["input"]
feedback_to = "physics"
budget_ms = 4.0

[graph.nodes.config]
primal = "ludospring"

[graph.nodes.params]
input_ref = "${input.output}"
"#;
    let graph = Graph::from_toml_str(toml).unwrap();
    assert_eq!(graph.id, "test-continuous");
    assert_eq!(graph.nodes.len(), 2);
    assert!(graph.is_continuous());

    let input_node = &graph.nodes[0];
    assert_eq!(input_node.id, "input");
    assert!(input_node.operation.is_some());
    let op = input_node.operation.as_ref().unwrap();
    assert_eq!(op.name, "capability_call");
    assert_eq!(
        op.params.get("capability").and_then(|v| v.as_str()),
        Some("interaction.poll")
    );

    let logic_node = &graph.nodes[1];
    assert_eq!(logic_node.id, "logic");
    assert_eq!(logic_node.depends_on, vec!["input"]);
    assert_eq!(
        logic_node
            .config
            .get("feedback_to")
            .and_then(|v| v.as_str()),
        Some("physics")
    );
    assert_eq!(
        logic_node.config.get("primal").and_then(|v| v.as_str()),
        Some("ludospring")
    );
    assert!(logic_node.constraints.is_some());
    assert_eq!(logic_node.constraints.as_ref().unwrap().timeout_ms, Some(4));
}

#[test]
fn graph_node_is_optional_with_skip_fallback() {
    let node = GraphNode {
        id: "test_node".to_string(),
        fallback: Some("skip".to_string()),
        ..Default::default()
    };
    assert!(node.is_optional());
}

#[test]
fn graph_node_is_not_optional_by_default() {
    let node = GraphNode {
        id: "test_node".to_string(),
        fallback: None,
        ..Default::default()
    };
    assert!(!node.is_optional());
}

#[test]
fn graph_node_is_not_optional_with_error_fallback() {
    let node = GraphNode {
        id: "test_node".to_string(),
        fallback: Some("error".to_string()),
        ..Default::default()
    };
    assert!(!node.is_optional());
}

#[test]
fn graph_node_fallback_deserializes_from_toml() {
    let toml_str = r#"
        id = "optional_step"
        fallback = "skip"
    "#;
    let node: GraphNode = toml::from_str(toml_str).expect("should deserialize");
    assert_eq!(node.id, "optional_step");
    assert!(node.is_optional());
}

#[test]
fn graph_node_fallback_absent_deserializes() {
    let toml_str = r#"
        id = "required_step"
    "#;
    let node: GraphNode = toml::from_str(toml_str).expect("should deserialize");
    assert_eq!(node.id, "required_step");
    assert!(!node.is_optional());
}

#[test]
fn graph_node_cost_estimate_ms_from_toml() {
    let toml_str = r#"
        id = "gpu-node"
        cost_estimate_ms = 250
    "#;
    let node: GraphNode = toml::from_str(toml_str).expect("should deserialize");
    assert_eq!(node.cost_estimate_ms, Some(250));
}

#[test]
fn graph_node_cost_estimate_ms_default() {
    let node = GraphNode {
        id: "cheap".to_string(),
        ..Default::default()
    };
    assert_eq!(node.cost_estimate_ms, None);
}

#[test]
fn graph_node_operation_dependencies_from_toml() {
    let toml_str = r#"
        id = "writer"
        operation_dependencies = ["storage.write", "crypto.sign"]
    "#;
    let node: GraphNode = toml::from_str(toml_str).expect("should deserialize");
    assert_eq!(
        node.operation_dependencies,
        vec!["storage.write", "crypto.sign"]
    );
}

#[test]
fn graph_node_operation_dependencies_default_empty() {
    let node = GraphNode {
        id: "pure".to_string(),
        ..Default::default()
    };
    assert!(node.operation_dependencies.is_empty());
}

#[test]
fn convert_deployment_node_carries_cost_estimate() {
    let toml = r#"
[graph]
id = "cost-test"
version = "1.0.0"
description = "Tests cost_estimate_ms in deployment format"

[[graph.nodes]]
id = "expensive"
name = "GPU Compute"
capability = "compute.dispatch"
cost_estimate_ms = 500
operation_dependencies = ["model.load"]

[graph.nodes.config]
primal = "toadstool"
"#;
    let graph = Graph::from_toml_str(toml).unwrap();
    let node = &graph.nodes[0];
    assert_eq!(node.cost_estimate_ms, Some(500));
    assert_eq!(node.operation_dependencies, vec!["model.load"]);
}

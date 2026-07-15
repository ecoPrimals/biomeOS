// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::graph::*;

pub(super) fn make_test_graph() -> DeploymentGraph {
    let toml_str = r#"
            [graph]
            id = "test-graph"
            name = "Test Graph"
            version = "1.0.0"
            description = "A test graph"
        "#;
    toml::from_str(toml_str).unwrap()
}

pub(super) fn make_graph_with_nodes() -> DeploymentGraph {
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
    let pos_a = ordered
        .iter()
        .position(|n| n.id.as_str() == "step-a")
        .unwrap();
    let pos_b = ordered
        .iter()
        .position(|n| n.id.as_str() == "step-b")
        .unwrap();
    let pos_c = ordered
        .iter()
        .position(|n| n.id.as_str() == "step-c")
        .unwrap();
    let pos_d = ordered
        .iter()
        .position(|n| n.id.as_str() == "step-d")
        .unwrap();

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
#[expect(
    clippy::literal_string_with_formatting_args,
    reason = "TOML string embeds ${VAR:-default} env syntax; not a format! placeholder"
)]
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

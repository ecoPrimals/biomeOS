// =============================================================================
// Integration Tests - Graph Parsing
// =============================================================================

use biomeos_graph::{GraphParser, GraphValidator};
use std::path::Path;

#[test]
#[ignore = "Neural API graphs use different format - TODO: unify graph schemas"]
fn test_parse_tower_atomic_bootstrap_graph() {
    // Path relative to workspace root
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/tower_atomic_bootstrap.toml");

    let graph = GraphParser::parse_file(&graph_path).expect("Failed to parse tower_atomic_bootstrap.toml");

    // Validate structure - this graph bootstraps Tower Atomic (BearDog + Songbird)
    assert!(graph.nodes.len() >= 2, "Should have at least 2 nodes for Tower Atomic bootstrap");

    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
fn test_parse_tower_health_check_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/tower_health_check.toml");

    let graph =
        GraphParser::parse_file(&graph_path).expect("Failed to parse tower_health_check.toml");

    assert_eq!(graph.name, "tower-health-check");
    assert_eq!(graph.nodes.len(), 3);

    // Validate all nodes have required fields
    for node in &graph.nodes {
        assert!(!node.id.is_empty());
    }

    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
fn test_parse_tower_shutdown_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/tower_shutdown.toml");

    let graph = GraphParser::parse_file(&graph_path).expect("Failed to parse tower_shutdown.toml");

    assert_eq!(graph.name, "tower-shutdown");
    assert_eq!(graph.nodes.len(), 3);
    assert_eq!(graph.edges.len(), 2);

    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
#[ignore = "Neural API graphs use different format - TODO: unify graph schemas"]
fn test_parse_node_atomic_test_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/node_atomic_test.toml");

    let graph = GraphParser::parse_file(&graph_path).expect("Failed to parse node_atomic_test.toml");

    // Just validate the graph can be parsed and is valid
    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
fn test_parse_node_health_check_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/node_health_check.toml");

    let graph =
        GraphParser::parse_file(&graph_path).expect("Failed to parse node_health_check.toml");

    assert_eq!(graph.name, "node_health_check");
    assert_eq!(graph.nodes.len(), 1); // Simplified version

    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
fn test_parse_node_shutdown_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/node_shutdown.toml");

    let graph = GraphParser::parse_file(&graph_path).expect("Failed to parse node_shutdown.toml");

    assert_eq!(graph.name, "node_shutdown");
    assert_eq!(graph.nodes.len(), 2); // Simplified version

    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
fn test_parse_nest_deploy_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/nest_deploy.toml");

    let graph = GraphParser::parse_file(&graph_path).expect("Failed to parse nest_deploy.toml");

    // Just validate the graph can be parsed
    // Node count may vary as the graph evolves
    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
fn test_parse_nest_health_check_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/nest_health_check.toml");

    let graph =
        GraphParser::parse_file(&graph_path).expect("Failed to parse nest_health_check.toml");

    assert_eq!(graph.name, "nest_health_check");
    assert_eq!(graph.nodes.len(), 3);

    GraphValidator::validate(&graph).expect("Graph validation failed");
}

#[test]
fn test_parse_nest_shutdown_graph() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let graph_path = workspace_root.join("graphs/nest_shutdown.toml");

    let graph = GraphParser::parse_file(&graph_path).expect("Failed to parse nest_shutdown.toml");

    assert_eq!(graph.name, "nest_shutdown");
    assert_eq!(graph.nodes.len(), 4); // Simplified version

    GraphValidator::validate(&graph).expect("Graph validation failed");
}

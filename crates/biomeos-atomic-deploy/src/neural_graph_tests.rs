// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural graph tests - extracted to keep neural_graph.rs under 1000 lines

#![allow(clippy::unwrap_used, clippy::expect_used)]

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

fn find_graphs_dir() -> std::path::PathBuf {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("graphs");
        if candidate.is_dir() {
            return candidate;
        }
        if !dir.pop() {
            panic!(
                "Could not find graphs/ directory from {}",
                env!("CARGO_MANIFEST_DIR")
            );
        }
    }
}

#[test]
fn test_parse_nucleus_complete() {
    let graphs_dir = find_graphs_dir();
    let graph = Graph::from_toml_file(&graphs_dir.join("nucleus_complete.toml")).unwrap();
    assert_eq!(graph.id, "nucleus_complete");
    assert_eq!(graph.version, "2.0.0");
    assert!(
        graph.nodes.len() >= 10,
        "Expected at least 10 nodes, got {}",
        graph.nodes.len()
    );
    assert_eq!(graph.nodes[0].id, "tower_beardog");
    let beardog_caps = graph.nodes[0].capabilities_provided.as_ref().unwrap();
    assert_eq!(
        beardog_caps.get("relay.authorize"),
        Some(&"relay.authorize".to_string())
    );
    let songbird = &graph.nodes[1];
    assert_eq!(songbird.id, "tower_songbird");
    assert!(songbird.capabilities.contains(&"mesh".to_string()));
    assert!(songbird.capabilities.contains(&"punch".to_string()));
    assert!(songbird.capabilities.contains(&"stun".to_string()));
    let songbird_caps = songbird.capabilities_provided.as_ref().unwrap();
    assert_eq!(
        songbird_caps.get("stun.probe_port_pattern"),
        Some(&"stun.probe_port_pattern".to_string())
    );
    assert_eq!(
        songbird_caps.get("punch.coordinate"),
        Some(&"punch.coordinate".to_string())
    );
}

#[test]
fn test_parse_ecosystem_full_bootstrap() {
    let graphs_dir = find_graphs_dir();
    let graph = Graph::from_toml_file(&graphs_dir.join("ecosystem_full_bootstrap.toml")).unwrap();
    assert_eq!(graph.id, "ecosystem_full_bootstrap");
    assert_eq!(graph.version, "2.0.0");
    assert!(
        graph.nodes.len() >= 6,
        "Expected at least 6 nodes, got {}",
        graph.nodes.len()
    );
    let nestgate = graph.nodes.iter().find(|n| n.id == "germinate_nestgate");
    assert!(nestgate.is_some(), "NestGate node should be present");
    let songbird = graph
        .nodes
        .iter()
        .find(|n| n.id == "germinate_songbird")
        .unwrap();
    let env = songbird
        .operation
        .as_ref()
        .unwrap()
        .environment
        .as_ref()
        .unwrap();
    for (key, val) in env {
        assert!(
            !val.contains("/tmp/"),
            "Songbird env {key} should not use /tmp/, got: {val}"
        );
    }
}

#[test]
fn test_parse_gate2_nucleus() {
    let graphs_dir = find_graphs_dir();
    let graph = Graph::from_toml_file(&graphs_dir.join("gate2_nucleus.toml")).unwrap();
    assert_eq!(graph.id, "gate2_nucleus");
    assert!(
        graph.nodes.len() >= 9,
        "Expected at least 9 nodes, got {}",
        graph.nodes.len()
    );
    let discover = graph.nodes.iter().find(|n| n.id == "gate2_discover_tower");
    assert!(discover.is_some(), "gate2_discover_tower node should exist");
    for node in &graph.nodes {
        if let Some(op) = &node.operation {
            if let Some(env) = &op.environment {
                for (key, val) in env {
                    assert!(
                        !val.contains("/run/user/1000"),
                        "Node {} env {} should use ${{XDG_RUNTIME_DIR}}, not hardcoded path: {}",
                        node.id,
                        key,
                        val
                    );
                    assert!(
                        !val.contains("/tmp/"),
                        "Node {} env {} should not use /tmp/: {}",
                        node.id,
                        key,
                        val
                    );
                }
            }
        }
    }
}

#[test]
fn test_parse_tower_atomic_bootstrap() {
    let graphs_dir = find_graphs_dir();
    let graph = Graph::from_toml_file(&graphs_dir.join("tower_atomic_bootstrap.toml")).unwrap();
    assert_eq!(graph.id, "tower_atomic_bootstrap");
    assert!(
        graph.nodes.len() >= 4,
        "Expected at least 4 nodes, got {}",
        graph.nodes.len()
    );
    let songbird = graph
        .nodes
        .iter()
        .find(|n| n.id == "germinate_songbird")
        .unwrap();
    let env = songbird
        .operation
        .as_ref()
        .unwrap()
        .environment
        .as_ref()
        .unwrap();
    let neural_api_sock = env.get("NEURAL_API_SOCKET").unwrap();
    assert!(
        neural_api_sock.contains("XDG_RUNTIME_DIR"),
        "NEURAL_API_SOCKET should use XDG_RUNTIME_DIR, got: {neural_api_sock}"
    );
    for node in &graph.nodes {
        if let Some(op) = &node.operation {
            if let Some(port_val) = op.params.get("port") {
                let port = port_val.as_i64().unwrap_or(0);
                assert_ne!(port, 3492, "Node {} should not use port 3492", node.id);
            }
            if let Some(params_val) = op.params.get("params") {
                if let Some(port_val) = params_val.get("port") {
                    let port = port_val.as_i64().unwrap_or(0);
                    assert_ne!(
                        port, 3492,
                        "Node {} params should not use port 3492",
                        node.id
                    );
                }
            }
        }
    }
}

#[test]
fn test_all_deployment_graphs_parse() {
    let graphs_dir = find_graphs_dir();
    let deployment_graphs = [
        "nucleus_complete.toml",
        "ecosystem_full_bootstrap.toml",
        "gate2_nucleus.toml",
        "tower_atomic_bootstrap.toml",
        "tower_atomic_xdg.toml",
        "tower_atomic.toml",
        "tower_atomic_dynamic.toml",
    ];

    let mut parsed_count = 0;
    let mut errors = Vec::new();

    for filename in &deployment_graphs {
        let path = graphs_dir.join(filename);
        if !path.exists() {
            continue;
        }
        match Graph::from_toml_file(&path) {
            Ok(graph) => {
                assert!(!graph.id.is_empty(), "Graph {filename} has empty id");
                assert!(!graph.nodes.is_empty(), "Graph {filename} has no nodes");
                parsed_count += 1;
            }
            Err(e) => {
                errors.push(format!("{filename}: {e}"));
            }
        }
    }

    assert!(
        parsed_count >= 4,
        "Expected to parse at least 4 deployment graphs, got {parsed_count}"
    );
    if !errors.is_empty() {
        panic!("Deployment graph parse errors:\n{}", errors.join("\n"));
    }
}

#[test]
fn test_no_hardcoded_paths_in_deployment_graphs() {
    let graphs_dir = find_graphs_dir();
    let deployment_graphs = [
        "nucleus_complete.toml",
        "ecosystem_full_bootstrap.toml",
        "gate2_nucleus.toml",
        "tower_atomic_xdg.toml",
    ];

    for filename in &deployment_graphs {
        let path = graphs_dir.join(filename);
        if !path.exists() {
            continue;
        }
        let graph = Graph::from_toml_file(&path).unwrap();
        for node in &graph.nodes {
            if let Some(op) = &node.operation {
                if let Some(env) = &op.environment {
                    for (key, val) in env {
                        assert!(
                            !val.contains("/tmp/"),
                            "[{}] Node {} env {} uses /tmp/: {}",
                            filename,
                            node.id,
                            key,
                            val
                        );
                        assert!(
                            !val.contains("/run/user/1000"),
                            "[{}] Node {} env {} uses hardcoded /run/user/1000: {}",
                            filename,
                            node.id,
                            key,
                            val
                        );
                    }
                }
            }
        }
    }
}

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
fn test_parse_real_game_engine_tick() {
    let graph_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("graphs/game_engine_tick.toml");
    if graph_path.exists() {
        let graph = Graph::from_toml_file(&graph_path).unwrap();
        assert_eq!(graph.id, "game-engine-tick");
        assert!(graph.is_continuous());
        assert_eq!(graph.nodes.len(), 5);
    }
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

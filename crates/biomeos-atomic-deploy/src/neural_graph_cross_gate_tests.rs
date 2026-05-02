// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Cross-gate graph parsing and gate registry tests.

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use crate::neural_graph::*;

#[test]
fn test_cross_gate_graph_parses_gate_field() {
    let toml = r#"
[graph]
id = "cross_gate_tower"
version = "1.0.0"
description = "Deploy across Tower + gate2"
coordination = "Sequential"

[graph.env]
gate2 = "tcp://192.0.2.132:9001"

[[graph.nodes]]
id = "beardog"
name = "BearDog local"
capability = "crypto"
gate = "local"

[graph.nodes.config]
primal = "beardog"

[[graph.nodes]]
id = "nestgate_remote"
name = "NestGate on gate2"
capability = "http"
gate = "gate2"

[graph.nodes.config]
primal = "nestgate"
"#;
    let graph = Graph::from_toml_str(toml).unwrap();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.nodes[0].gate.as_deref(), Some("local"));
    assert_eq!(graph.nodes[1].gate.as_deref(), Some("gate2"));
}

#[test]
fn test_cross_gate_graph_parses_env_section() {
    let toml = r#"
[graph]
id = "env_test"
version = "1.0.0"
description = "Tests [graph.env] parsing"

[graph.env]
gate2 = "tcp://192.0.2.132:9001"
pixel = "@biomeos-pixel"
RUST_LOG = "info"

[[graph.nodes]]
id = "dummy"
name = "Dummy"
capability = "test"
"#;
    let graph = Graph::from_toml_str(toml).unwrap();

    assert_eq!(graph.env.len(), 3);
    assert_eq!(graph.env.get("gate2").unwrap(), "tcp://192.0.2.132:9001");
    assert_eq!(graph.env.get("pixel").unwrap(), "@biomeos-pixel");
    assert_eq!(graph.env.get("RUST_LOG").unwrap(), "info");
}

#[test]
fn test_cross_gate_graph_no_gate_defaults_to_none() {
    let toml = r#"
[graph]
id = "no_gate"
version = "1.0.0"
description = "Node without gate field"

[[graph.nodes]]
id = "local_only"
name = "Local"
capability = "test"
"#;
    let graph = Graph::from_toml_str(toml).unwrap();
    assert!(graph.nodes[0].gate.is_none());
}

#[test]
fn test_gate_registry_from_graph_env() {
    use crate::gate_registry::GateRegistry;

    let toml = r#"
[graph]
id = "gate_reg_test"
version = "1.0.0"
description = "Gate registry from graph env"

[graph.env]
gate2 = "tcp://192.0.2.132:9001"
pixel = "@biomeos-pixel"
RUST_LOG = "info"

[[graph.nodes]]
id = "dummy"
name = "Dummy"
capability = "test"
"#;
    let graph = Graph::from_toml_str(toml).unwrap();
    let registry = GateRegistry::from_graph_env(&graph.env);

    assert_eq!(registry.len(), 2);
    assert!(registry.is_remote("gate2"));
    assert!(registry.is_remote("pixel"));
    assert!(!registry.is_remote("RUST_LOG"));
}

#[test]
fn test_cross_gate_graph_neural_format_with_gate() {
    let toml = r#"
[graph]
id = "neural_gate_test"
version = "1.0.0"
description = "Neural-format graph with gate"

[[nodes]]
id = "remote_beardog"
gate = "gate2"
capabilities = ["crypto", "security"]

[nodes.operation]
name = "start"
"#;
    let graph = Graph::from_toml_str(toml).unwrap();
    assert_eq!(graph.nodes[0].gate.as_deref(), Some("gate2"));
}

#[test]
fn test_cross_gate_graph_empty_env_produces_empty_registry() {
    use crate::gate_registry::GateRegistry;

    let graph = Graph::from_toml_str(
        r#"
[graph]
id = "no_env"
version = "1.0.0"
description = "No env section"

[[graph.nodes]]
id = "n"
name = "N"
capability = "test"
"#,
    )
    .unwrap();
    let registry = GateRegistry::from_graph_env(&graph.env);
    assert!(registry.is_empty());
}

#[test]
fn test_cross_gate_tower_toml_parses_and_wires_registry() {
    use crate::gate_registry::GateRegistry;

    let toml_content = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../graphs/cross_gate_tower.toml"
    ))
    .expect("graphs/cross_gate_tower.toml should exist");
    let graph = Graph::from_toml_str(&toml_content).unwrap();

    assert_eq!(graph.id, "cross_gate_tower");

    let local_node_count = graph.nodes.iter().filter(|n| n.gate.is_none()).count();
    let remote_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| n.gate.as_deref() == Some("gate2"))
        .collect();
    assert!(
        local_node_count >= 2,
        "should have at least 2 local nodes (beardog + songbird)"
    );
    assert!(
        remote_nodes.len() >= 2,
        "should have at least 2 gate2 nodes"
    );

    let registry = GateRegistry::from_graph_env(&graph.env);
    assert!(registry.is_remote("gate2"), "gate2 should be in registry");
    assert!(!registry.is_remote("local"), "local should not resolve");

    for node in &remote_nodes {
        assert!(
            node.id.starts_with("gate2_"),
            "remote node id should be prefixed: {}",
            node.id
        );
    }
}

#[test]
fn test_cross_gate_tower_toml_route_register_nodes() {
    let toml_content = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../graphs/cross_gate_tower.toml"
    ))
    .expect("graphs/cross_gate_tower.toml should exist");
    let graph = Graph::from_toml_str(&toml_content).unwrap();

    let register_nodes: Vec<_> = graph
        .nodes
        .iter()
        .filter(|n| {
            n.operation
                .as_ref()
                .and_then(|op| op.params.get("method"))
                .and_then(|m| m.as_str())
                == Some("route.register")
        })
        .collect();

    assert!(
        register_nodes.len() >= 2,
        "should have route.register nodes for crypto + network"
    );

    for node in &register_nodes {
        let params = node
            .operation
            .as_ref()
            .and_then(|op| op.params.get("params"))
            .expect("route.register node should have params");
        assert!(
            params.get("gate").is_some(),
            "route.register should include gate tag"
        );
        assert!(
            params.get("capabilities").is_some(),
            "route.register should include capabilities array"
        );
    }
}

#[test]
fn test_cross_gate_pixel_toml_parses_and_wires_registry() {
    let toml_content = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../graphs/cross_gate_pixel.toml"
    ))
    .expect("graphs/cross_gate_pixel.toml should exist");
    let graph = Graph::from_toml_str(&toml_content).unwrap();

    assert_eq!(graph.id, "cross_gate_pixel");

    let pixel_node_count = graph
        .nodes
        .iter()
        .filter(|n| n.gate.as_deref() == Some("pixel"))
        .count();
    assert!(
        pixel_node_count >= 2,
        "should have at least 2 pixel-targeted nodes"
    );

    assert!(
        graph.env.contains_key("pixel"),
        "graph.env should contain pixel endpoint"
    );
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::neural_graph::*;
use biomeos_graph::GeneticsTier;

pub(super) fn find_graphs_dir() -> std::path::PathBuf {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("graphs");
        if candidate.is_dir() {
            return candidate;
        }
        assert!(
            dir.pop(),
            "Could not find graphs/ directory from {}",
            env!("CARGO_MANIFEST_DIR")
        );
    }
}

#[test]
fn test_parse_nucleus_complete() {
    let graphs_dir = find_graphs_dir();
    let graph = Graph::from_toml_file(&graphs_dir.join("nucleus_complete.toml")).unwrap();
    assert_eq!(graph.id, "nucleus_complete");
    assert_eq!(graph.version, "2.0.0");
    assert_eq!(graph.genetics_tier, Some(GeneticsTier::Nuclear));
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
    assert_eq!(graph.genetics_tier, Some(GeneticsTier::MitoBeacon));
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
    assert!(
        errors.is_empty(),
        "Deployment graph parse errors:\n{}",
        errors.join("\n")
    );
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

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Level 1: TOML parsing — canonical nucleus_complete.toml loads correctly.

use biomeos_atomic_deploy::*;

use crate::{NUCLEUS_NODE_IDS, graphs_dir};

#[test]
fn test_nucleus_complete_toml_parses() {
    let path = graphs_dir().join("nucleus_complete.toml");
    assert!(
        path.exists(),
        "nucleus_complete.toml missing at {}",
        path.display()
    );

    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    assert_eq!(graph.id, "nucleus_complete");
    assert_eq!(graph.version, "2.0.0");
    assert_eq!(
        graph.nodes.len(),
        NUCLEUS_NODE_IDS.len(),
        "Expected {} nodes, found {}",
        NUCLEUS_NODE_IDS.len(),
        graph.nodes.len()
    );

    for expected_id in NUCLEUS_NODE_IDS {
        assert!(
            graph.nodes.iter().any(|n| n.id == *expected_id),
            "Missing node: {expected_id}"
        );
    }
}

#[test]
fn test_nucleus_complete_has_five_primal_starts() {
    let path = graphs_dir().join("nucleus_complete.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    let start_nodes: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.operation.as_ref().is_some_and(|op| op.name == "start"))
        .map(|n| n.id.as_str())
        .collect();

    assert!(
        start_nodes.len() >= 5,
        "NUCLEUS requires 5+ primal starts (BearDog, Songbird, Toadstool, NestGate, Squirrel), found {}: {:?}",
        start_nodes.len(),
        start_nodes
    );
}

#[test]
fn test_nucleus_complete_capabilities_populated() {
    let path = graphs_dir().join("nucleus_complete.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    let beardog = graph
        .nodes
        .iter()
        .find(|n| n.id == "tower_beardog")
        .unwrap();
    assert!(
        beardog.capabilities.contains(&"crypto".to_string()),
        "BearDog should declare crypto capability"
    );

    let songbird = graph
        .nodes
        .iter()
        .find(|n| n.id == "tower_songbird")
        .unwrap();
    assert!(
        songbird.capabilities.contains(&"discovery".to_string()),
        "Songbird should declare discovery capability"
    );
}

#[test]
fn test_nucleus_complete_dependency_integrity() {
    let path = graphs_dir().join("nucleus_complete.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    let node_ids: Vec<&str> = graph.nodes.iter().map(|n| n.id.as_str()).collect();

    for node in &graph.nodes {
        for dep in &node.depends_on {
            assert!(
                node_ids.contains(&dep.as_str()),
                "Node '{}' depends on '{}' which doesn't exist in the graph",
                node.id,
                dep
            );
        }
    }
}

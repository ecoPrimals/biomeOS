// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Level 4: gate2 NUCLEUS deploy graph — cross-gate composition validation.

use biomeos_atomic_deploy::*;
use std::collections::HashMap;

use crate::graphs_dir;

const GATE2_NODE_IDS: &[&str] = &[
    "gate2_beardog",
    "gate2_songbird",
    "gate2_swarmvine",
    "gate2_mesh_init",
    "gate2_discover_tower",
    "gate2_nestgate",
    "gate2_toadstool",
    "gate2_squirrel",
    "gate2_validate",
    "gate2_announce_relay",
];

#[test]
fn test_gate2_nucleus_toml_parses() {
    let path = graphs_dir().join("gate2_nucleus.toml");
    assert!(
        path.exists(),
        "gate2_nucleus.toml missing at {}",
        path.display()
    );

    let graph = NeuralGraph::from_toml_file(&path).unwrap();
    assert_eq!(graph.id, "gate2_nucleus");
    assert_eq!(graph.nodes.len(), GATE2_NODE_IDS.len());

    for expected_id in GATE2_NODE_IDS {
        assert!(
            graph.nodes.iter().any(|n| n.id == *expected_id),
            "Missing gate2 node: {expected_id}"
        );
    }
}

#[test]
fn test_gate2_nucleus_topological_sort_succeeds() {
    let path = graphs_dir().join("gate2_nucleus.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();
    let executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();
    let total: usize = phases.iter().map(Vec::len).sum();
    assert_eq!(total, GATE2_NODE_IDS.len());
}

#[test]
fn test_gate2_has_parallel_deployment() {
    let path = graphs_dir().join("gate2_nucleus.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();
    let executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();

    let phase_of = |id: &str| -> usize {
        phases
            .iter()
            .position(|p| p.contains(&id.to_string()))
            .unwrap_or_else(|| panic!("node {id} not found"))
    };

    assert_eq!(
        phase_of("gate2_nestgate"),
        phase_of("gate2_toadstool"),
        "NestGate and Toadstool should deploy in parallel on gate2"
    );
    assert_eq!(
        phase_of("gate2_squirrel"),
        phase_of("gate2_toadstool"),
        "Squirrel and Toadstool should deploy in parallel on gate2"
    );
}

#[test]
fn test_gate2_has_five_primal_starts() {
    let path = graphs_dir().join("gate2_nucleus.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    let start_count = graph
        .nodes
        .iter()
        .filter(|n| n.operation.as_ref().is_some_and(|op| op.name == "start"))
        .count();

    assert!(
        start_count >= 5,
        "gate2 NUCLEUS requires 5 primal starts, found {start_count}"
    );
}

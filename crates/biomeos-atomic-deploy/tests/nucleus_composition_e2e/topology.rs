// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Level 2: Topological sort — NUCLEUS phases are correctly ordered.

use biomeos_atomic_deploy::*;
use std::collections::HashMap;

use crate::{graphs_dir, NUCLEUS_NODE_IDS};

#[test]
fn test_nucleus_complete_topological_sort_succeeds() {
    let path = graphs_dir().join("nucleus_complete.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    let executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();

    let total_nodes: usize = phases.iter().map(Vec::len).sum();
    assert_eq!(total_nodes, NUCLEUS_NODE_IDS.len());
}

#[test]
fn test_nucleus_complete_phase_ordering_respects_architecture() {
    let path = graphs_dir().join("nucleus_complete.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    let executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();

    let phase_of = |node_id: &str| -> usize {
        phases
            .iter()
            .position(|phase| phase.contains(&node_id.to_string()))
            .unwrap_or_else(|| panic!("Node {node_id} not found in any phase"))
    };

    assert!(
        phase_of("tower_beardog") < phase_of("tower_songbird"),
        "BearDog must precede Songbird (crypto before network)"
    );

    assert!(
        phase_of("tower_songbird") < phase_of("init_sovereign_onion"),
        "Songbird must precede onion init"
    );

    assert!(
        phase_of("tower_validate") < phase_of("node_toadstool"),
        "Tower validation must precede Toadstool start"
    );
    assert!(
        phase_of("tower_validate") < phase_of("germinate_squirrel"),
        "Tower validation must precede Squirrel start"
    );

    assert_eq!(
        phase_of("node_toadstool"),
        phase_of("germinate_squirrel"),
        "Toadstool and Squirrel should be in the same phase (both depend on tower_validate)"
    );

    assert!(
        phase_of("node_toadstool") < phase_of("node_validate"),
        "Toadstool must precede node validation"
    );
    assert!(
        phase_of("node_validate") < phase_of("nest_nestgate"),
        "Node validation must precede NestGate start"
    );

    assert!(
        phase_of("nest_nestgate") < phase_of("nucleus_validate"),
        "NestGate must precede NUCLEUS validation"
    );
    assert!(
        phase_of("germinate_squirrel") < phase_of("nucleus_validate"),
        "Squirrel must precede NUCLEUS validation"
    );

    assert!(
        phase_of("nucleus_validate") < phase_of("announce_relay"),
        "NUCLEUS validation must precede relay announce"
    );

    let last_phase = phases.len() - 1;
    assert_eq!(
        phase_of("announce_relay"),
        last_phase,
        "announce_relay should be the final phase"
    );
}

#[test]
fn test_nucleus_complete_has_parallel_phases() {
    let path = graphs_dir().join("nucleus_complete.toml");
    let graph = NeuralGraph::from_toml_file(&path).unwrap();

    let executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let phases = executor.topological_sort().unwrap();

    let has_parallel = phases.iter().any(|p| p.len() > 1);
    assert!(
        has_parallel,
        "NUCLEUS graph should have at least one parallel phase (Toadstool + Squirrel)"
    );
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

//! Multi-primal NUCLEUS composition end-to-end tests
//!
//! Validates that biomeOS can correctly parse, order, and execute graphs with
//! 5+ primals — the threshold required for full NUCLEUS composition.
//! This unblocks chimera compositions and garden deployments.
//!
//! Three levels:
//! 1. Parse the canonical `nucleus_complete.toml` and verify all 13 nodes load
//! 2. Verify topological sort produces correct phased execution order
//! 3. Execute a synthetic NUCLEUS-shaped graph end-to-end, proving the executor
//!    handles multi-phase parallel composition correctly

use biomeos_atomic_deploy::*;
use std::collections::HashMap;
use std::path::PathBuf;

/// All expected node IDs in nucleus_complete.toml
const NUCLEUS_NODE_IDS: &[&str] = &[
    "tower_beardog",
    "tower_songbird",
    "init_sovereign_onion",
    "init_beacon_mesh",
    "tower_validate",
    "node_toadstool",
    "node_validate",
    "nest_nestgate",
    "register_barracuda",
    "register_coralreef",
    "germinate_squirrel",
    "nucleus_validate",
    "announce_relay",
];

fn graphs_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("graphs")
}

// ==========================================================================
// Level 1: TOML parsing — canonical nucleus_complete.toml loads correctly
// ==========================================================================

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

// ==========================================================================
// Level 2: Topological sort — NUCLEUS phases are correctly ordered
// ==========================================================================

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

    // Tower foundation comes first
    assert!(
        phase_of("tower_beardog") < phase_of("tower_songbird"),
        "BearDog must precede Songbird (crypto before network)"
    );

    // Songbird must be up before onion init
    assert!(
        phase_of("tower_songbird") < phase_of("init_sovereign_onion"),
        "Songbird must precede onion init"
    );

    // Tower must be validated before Node and Nest primals
    assert!(
        phase_of("tower_validate") < phase_of("node_toadstool"),
        "Tower validation must precede Toadstool start"
    );
    assert!(
        phase_of("tower_validate") < phase_of("germinate_squirrel"),
        "Tower validation must precede Squirrel start"
    );

    // Toadstool and Squirrel can run in parallel (both depend on tower_validate)
    assert_eq!(
        phase_of("node_toadstool"),
        phase_of("germinate_squirrel"),
        "Toadstool and Squirrel should be in the same phase (both depend on tower_validate)"
    );

    // NestGate depends on node_validate which depends on toadstool
    assert!(
        phase_of("node_toadstool") < phase_of("node_validate"),
        "Toadstool must precede node validation"
    );
    assert!(
        phase_of("node_validate") < phase_of("nest_nestgate"),
        "Node validation must precede NestGate start"
    );

    // Full NUCLEUS validation requires both NestGate and Squirrel
    assert!(
        phase_of("nest_nestgate") < phase_of("nucleus_validate"),
        "NestGate must precede NUCLEUS validation"
    );
    assert!(
        phase_of("germinate_squirrel") < phase_of("nucleus_validate"),
        "Squirrel must precede NUCLEUS validation"
    );

    // Relay announce is the final step
    assert!(
        phase_of("nucleus_validate") < phase_of("announce_relay"),
        "NUCLEUS validation must precede relay announce"
    );

    // announce_relay should be in the last phase
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

// ==========================================================================
// Level 3: End-to-end execution — synthetic NUCLEUS-shaped graph
// ==========================================================================

/// Build a NUCLEUS-shaped graph using log.info nodes that execute without
/// real primal sockets. Same dependency structure as nucleus_complete.toml.
fn build_synthetic_nucleus() -> NeuralGraph {
    use biomeos_atomic_deploy::neural_graph::Operation;

    let log_op = |msg: &str| -> Option<Operation> {
        let mut params = HashMap::new();
        params.insert(
            "message".to_string(),
            serde_json::Value::String(msg.to_string()),
        );
        Some(Operation {
            name: "log.info".to_string(),
            params,
            environment: None,
        })
    };

    let node = |id: &str, deps: Vec<&str>, msg: &str| -> NeuralGraphNode {
        let mut c = HashMap::new();
        c.insert(
            "message".to_string(),
            serde_json::Value::String(msg.to_string()),
        );
        NeuralGraphNode {
            id: id.to_string(),
            depends_on: deps.iter().map(ToString::to_string).collect(),
            operation: log_op(msg),
            config: c,
            ..Default::default()
        }
    };

    NeuralGraph {
        id: "synthetic_nucleus".to_string(),
        version: "1.0.0".to_string(),
        description: "Synthetic NUCLEUS for e2e composition test".to_string(),
        nodes: vec![
            node("tower_beardog", vec![], "Germinating BearDog (crypto)"),
            node(
                "tower_songbird",
                vec!["tower_beardog"],
                "Germinating Songbird (network)",
            ),
            node(
                "init_sovereign_onion",
                vec!["tower_songbird"],
                "Initializing Sovereign Onion",
            ),
            node(
                "init_beacon_mesh",
                vec!["init_sovereign_onion"],
                "Initializing Beacon Mesh",
            ),
            node(
                "tower_validate",
                vec!["tower_beardog", "tower_songbird", "init_sovereign_onion"],
                "Validating Tower Atomic",
            ),
            node(
                "node_toadstool",
                vec!["tower_validate"],
                "Germinating Toadstool (compute)",
            ),
            node(
                "node_validate",
                vec!["node_toadstool"],
                "Validating Node Atomic",
            ),
            node(
                "nest_nestgate",
                vec!["node_validate"],
                "Germinating NestGate (storage)",
            ),
            node(
                "germinate_squirrel",
                vec!["tower_validate"],
                "Germinating Squirrel (AI)",
            ),
            node(
                "nucleus_validate",
                vec!["nest_nestgate", "germinate_squirrel"],
                "Validating full NUCLEUS",
            ),
            node(
                "announce_relay",
                vec!["nucleus_validate"],
                "Announcing as family relay",
            ),
        ],
        config: GraphConfig {
            rollback_on_failure: false,
            ..Default::default()
        },
        coordination: Some("Sequential".to_string()),
        env: HashMap::new(),
        genetics_tier: None,
    }
}

#[tokio::test]
async fn test_synthetic_nucleus_executes_end_to_end() {
    let graph = build_synthetic_nucleus();

    assert_eq!(graph.nodes.len(), 11, "NUCLEUS graph should have 11 nodes");

    let mut executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let report = executor.execute().await.unwrap();

    assert!(
        report.success,
        "NUCLEUS composition execution should succeed"
    );
    assert!(
        report.phase_results.len() >= 5,
        "Expected at least 5 phases, got {}",
        report.phase_results.len()
    );
    assert!(
        report.duration_ms < 10_000,
        "Should complete well under 10s"
    );
}

#[tokio::test]
async fn test_synthetic_nucleus_all_nodes_complete() {
    let graph = build_synthetic_nucleus();
    let mut executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let report = executor.execute().await.unwrap();

    let total_completed: usize = report.phase_results.iter().map(|p| p.completed).sum();
    let total_failed: usize = report.phase_results.iter().map(|p| p.failed).sum();

    assert_eq!(total_completed, 11, "All 11 NUCLEUS nodes should complete");
    assert_eq!(total_failed, 0, "No nodes should fail");
}

#[tokio::test]
async fn test_synthetic_nucleus_parallel_phase_speedup() {
    let graph = build_synthetic_nucleus();
    let mut executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let report = executor.execute().await.unwrap();

    let has_parallel = report.phase_results.iter().any(|p| p.completed > 1);

    assert!(
        has_parallel,
        "At least one phase should execute multiple nodes in parallel"
    );
}

// ==========================================================================
// Level 4: gate2 NUCLEUS deploy graph — cross-gate composition validation
// ==========================================================================

const GATE2_NODE_IDS: &[&str] = &[
    "gate2_beardog",
    "gate2_songbird",
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

// ==========================================================================
// Level 5: Failure handling — critical node failures in composition
// ==========================================================================

/// Validate that a failure in a critical node correctly aborts downstream
/// phases while preserving completed phase results.
#[tokio::test]
async fn test_nucleus_critical_node_failure_aborts_graph() {
    use biomeos_atomic_deploy::neural_graph::Operation;

    let fail_op = Some(Operation {
        name: "filesystem.check_exists".to_string(),
        params: HashMap::new(),
        environment: None,
    });

    let log_op = |msg: &str| -> Option<Operation> {
        let mut params = HashMap::new();
        params.insert(
            "message".to_string(),
            serde_json::Value::String(msg.to_string()),
        );
        Some(Operation {
            name: "log.info".to_string(),
            params,
            environment: None,
        })
    };

    let node = |id: &str, deps: Vec<&str>, op: Option<Operation>| -> NeuralGraphNode {
        NeuralGraphNode {
            id: id.to_string(),
            depends_on: deps.iter().map(ToString::to_string).collect(),
            operation: op,
            config: HashMap::new(),
            ..Default::default()
        }
    };

    let graph = NeuralGraph {
        id: "fail_test".to_string(),
        version: "1.0.0".to_string(),
        description: "NUCLEUS with injected failure".to_string(),
        nodes: vec![
            node("beardog", vec![], log_op("ok")),
            node("songbird", vec!["beardog"], log_op("ok")),
            node("onion_init", vec!["songbird"], fail_op),
            node(
                "tower_validate",
                vec!["onion_init"],
                log_op("should not run"),
            ),
            node(
                "toadstool",
                vec!["tower_validate"],
                log_op("should not run"),
            ),
            node("squirrel", vec!["tower_validate"], log_op("should not run")),
        ],
        config: GraphConfig {
            rollback_on_failure: false,
            ..Default::default()
        },
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };

    let mut executor = NeuralGraphExecutor::new(graph, HashMap::new());
    let report = executor.execute().await.unwrap();

    assert!(
        !report.success,
        "Graph should fail when critical node fails"
    );
    assert!(
        report.error.is_some(),
        "Report should contain error details"
    );

    let completed: usize = report.phase_results.iter().map(|p| p.completed).sum();
    assert!(
        completed >= 2,
        "First two phases (beardog, songbird) should have completed before failure"
    );
}

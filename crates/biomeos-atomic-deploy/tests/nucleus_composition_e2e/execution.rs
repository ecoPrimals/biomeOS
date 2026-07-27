// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Level 3 + Level 5: Synthetic NUCLEUS-shaped graph execution and failure handling.

use biomeos_atomic_deploy::neural_graph::Operation;
use biomeos_atomic_deploy::*;
use std::collections::HashMap;

/// Build a NUCLEUS-shaped graph using log.info nodes that execute without
/// real primal sockets. Same dependency structure as nucleus_complete.toml.
fn build_synthetic_nucleus() -> NeuralGraph {
    let log_op = |msg: &str| -> Option<Operation> {
        let mut params = HashMap::new();
        params.insert(
            "message".to_string(),
            serde_json::Value::String(msg.to_string()),
        );
        Some(Operation {
            name: "log.info".to_string(),
            target: None,
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
        composition_model: None,
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

/// Validate that a failure in a critical node correctly aborts downstream
/// phases while preserving completed phase results.
#[tokio::test]
async fn test_nucleus_critical_node_failure_aborts_graph() {
    let fail_op = Some(Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
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
            target: None,
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
        composition_model: None,
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

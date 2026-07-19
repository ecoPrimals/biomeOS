// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unknown node type dispatch tests.

use super::super::neural_executor::GraphExecutor;
use super::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

/// Test execute_node dispatch for unknown node type — returns skipped without error
#[tokio::test]
async fn test_execute_node_unknown_type() {
    let node = create_test_node("unknown_node", vec![]);
    // Node has no operation and no node_type, so node_type_str becomes "unknown"
    let graph = Graph {
        id: "unknown-test".to_string(),
        version: "1.0".to_string(),
        description: "Test unknown node".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "XDG_RUNTIME_DIR".to_string(),
        std::env::temp_dir().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results.len(), 1);
    assert_eq!(report.phase_results[0].completed, 1);
    assert_eq!(report.phase_results[0].failed, 0);
}

/// Test execute_node with explicit unknown node_type string
#[tokio::test]
async fn test_execute_node_explicit_unknown_type() {
    let mut node = create_test_node("explicit_unknown", vec![]);
    node.node_type = Some("custom_unknown_xyz".to_string());

    let graph = Graph {
        id: "explicit-unknown".to_string(),
        version: "1.0".to_string(),
        description: "Test explicit unknown".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "XDG_RUNTIME_DIR".to_string(),
        std::env::temp_dir().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
}

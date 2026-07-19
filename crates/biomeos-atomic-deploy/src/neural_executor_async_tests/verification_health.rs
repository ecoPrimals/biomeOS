// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Verification and health-check node execution tests.

use super::super::neural_executor::GraphExecutor;
use super::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

/// Test node_verification with check_sockets=false (skips socket checks)
#[tokio::test]
async fn test_execute_verification_check_sockets_false() {
    let mut node = create_test_node("verify_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "verification".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config
        .insert("check_sockets".to_string(), serde_json::Value::Bool(false));

    let graph = Graph {
        id: "verify-test".to_string(),
        version: "1.0".to_string(),
        description: "Verification with check_sockets=false".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
}

/// Test node_health_check_all when SOCKET_DIR doesn't exist (returns empty)
#[tokio::test]
async fn test_execute_health_check_all_no_socket_dir() {
    let mut node = create_test_node("health_all", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check_all".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "health-all-test".to_string(),
        version: "1.0".to_string(),
        description: "Health check all".to_string(),
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
        "SOCKET_DIR".to_string(),
        "/nonexistent/path/that/does/not/exist/12345".to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
}

/// Test node_health_check_all with temp dir (may find 0 or more .sock files)
#[tokio::test]
async fn test_execute_health_check_all_with_temp_dir() {
    let mut node = create_test_node("health_all", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check_all".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "health-all-temp".to_string(),
        version: "1.0".to_string(),
        description: "Health check all in temp".to_string(),
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
        "SOCKET_DIR".to_string(),
        temp.path().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
}

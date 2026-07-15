// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Filesystem node execution and failure/rollback tests.

use super::create_test_node;
use super::super::neural_executor::GraphExecutor;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

/// Test execute with filesystem.check_exists node (requires path config)
#[tokio::test]
async fn test_execute_filesystem_check_exists() {
    let mut node = create_test_node("fs_check", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "path".to_string(),
        serde_json::Value::String(std::env::temp_dir().to_string_lossy().to_string()),
    );

    let graph = Graph {
        id: "fs-test".to_string(),
        version: "1.0".to_string(),
        description: "Filesystem check".to_string(),
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
        std::env::temp_dir().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
}

/// Test execute with node that fails (filesystem.check_exists missing path)
#[tokio::test]
async fn test_execute_node_failure_reports_error() {
    let mut node = create_test_node("fail_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    // No path in config - will fail

    let graph = Graph {
        id: "fail-test".to_string(),
        version: "1.0".to_string(),
        description: "Failing node".to_string(),
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

    assert!(!report.success);
    assert!(report.error.is_some());
    if !report.phase_results.is_empty() {
        assert_eq!(report.phase_results[0].failed, 1);
    }
}

/// Test execute with rollback_on_failure triggers rollback path
#[tokio::test]
async fn test_execute_rollback_on_failure() {
    let mut node = create_test_node("fail_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let config = GraphConfig {
        rollback_on_failure: true,
        ..GraphConfig::default()
    };

    let graph = Graph {
        id: "rollback-test".to_string(),
        version: "1.0".to_string(),
        description: "Rollback on failure".to_string(),
        nodes: vec![node],
        config,
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

    assert!(!report.success);
    assert!(report.error.is_some());
}

/// When `rollback_on_failure` is false, a failed phase still produces a failed report (no rollback branch).
#[tokio::test]
async fn test_execute_failure_without_rollback_flag() {
    let mut node = create_test_node("fail_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let config = GraphConfig {
        rollback_on_failure: false,
        ..GraphConfig::default()
    };

    let graph = Graph {
        id: "no-rollback".to_string(),
        version: "1.0".to_string(),
        description: "No rollback".to_string(),
        nodes: vec![node],
        config,
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

    assert!(!report.success);
    assert!(report.error.is_some());
}

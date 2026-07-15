// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::neural_executor::GraphExecutor;
use crate::neural_executor_async_tests::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_execute_verification_socket_dir_missing() {
    let mut node = create_test_node("verify_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "verification".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "verify-no-socket-dir".to_string(),
        version: "1.0".to_string(),
        description: "verify".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let env = HashMap::from([("FAMILY_ID".to_string(), "test".to_string())]);

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
    assert!(report.error.is_some());
}

#[tokio::test]
async fn test_execute_health_check_atomic_alias() {
    let mut node = create_test_node("hc_atomic", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check_atomic".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "hc-atomic".to_string(),
        version: "1.0".to_string(),
        description: "hc".to_string(),
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
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

#[tokio::test]
async fn test_execute_health_check_plain_alias() {
    let mut node = create_test_node("hc_plain", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health_check".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config
        .insert("target".to_string(), serde_json::json!("nonexistent"));

    let graph = Graph {
        id: "hc-plain".to_string(),
        version: "1.0".to_string(),
        description: "hc".to_string(),
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
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

#[tokio::test]
async fn test_execute_health_check_all_missing_socket_dir() {
    let mut node = create_test_node("hca", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check_all".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "hca-no-dir".to_string(),
        version: "1.0".to_string(),
        description: "h".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let env = HashMap::from([("FAMILY_ID".to_string(), "test".to_string())]);

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

/// `verification` with `check_sockets=true` and no `dependencies` completes with zero verified.
#[tokio::test]
async fn test_execute_verification_check_sockets_true_no_deps() {
    let mut node = create_test_node("verify_empty", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "verification".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config
        .insert("check_sockets".to_string(), serde_json::Value::Bool(true));
    node.config
        .insert("check_health".to_string(), serde_json::Value::Bool(true));

    let graph = Graph {
        id: "verify-sockets-empty".to_string(),
        version: "1.0".to_string(),
        description: "verify".to_string(),
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
    let report = executor.execute().await.expect("report");
    assert!(report.success);
}

/// Explicit `health.check` operation name (not only `health_check` alias).
#[tokio::test]
async fn test_execute_health_dot_check_requires_primal_name() {
    let mut node = create_test_node("hc_dot", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "health-dot".to_string(),
        version: "1.0".to_string(),
        description: "hc".to_string(),
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
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Log node execution and metrics attachment tests.

use super::super::neural_executor::GraphExecutor;
use super::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

/// Test execute with log.warn node
#[tokio::test]
async fn test_execute_log_warn_node() {
    let mut node = create_test_node("warn_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "log.warn".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "message".to_string(),
        serde_json::Value::String("test warn".to_string()),
    );

    let graph = Graph {
        id: "log-warn-test".to_string(),
        version: "1.0".to_string(),
        description: "Log warn".to_string(),
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

/// Test execute with log.error node
#[tokio::test]
async fn test_execute_log_error_node() {
    let mut node = create_test_node("error_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "log.error".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "message".to_string(),
        serde_json::Value::String("test error".to_string()),
    );

    let graph = Graph {
        id: "log-error-test".to_string(),
        version: "1.0".to_string(),
        description: "Log error".to_string(),
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

/// Test log.info node
#[tokio::test]
async fn test_execute_log_info_node() {
    let mut node = create_test_node("info_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "log.info".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "message".to_string(),
        serde_json::Value::String("test info".to_string()),
    );

    let graph = Graph {
        id: "log-info-test".to_string(),
        version: "1.0".to_string(),
        description: "Log info".to_string(),
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

/// Test with_metrics builder - executor runs successfully with metrics attached
#[tokio::test]
async fn test_executor_with_metrics() {
    use biomeos_graph::metrics::MetricsCollector;

    let mut node = create_test_node("log1", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "log.info".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "message".to_string(),
        serde_json::Value::String("metrics test".to_string()),
    );

    let graph = Graph {
        id: "metrics-test".to_string(),
        version: "1.0".to_string(),
        description: "Metrics".to_string(),
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

    let executor = GraphExecutor::new(graph, env);

    let temp = tempfile::TempDir::new().expect("tempdir");
    let db_path = temp.path().join("metrics.db");
    let collector = MetricsCollector::new(&db_path).expect("collector");

    let mut executor_with_metrics = executor.with_metrics(collector);
    let report = executor_with_metrics.execute().await.unwrap();

    assert!(report.success);
}

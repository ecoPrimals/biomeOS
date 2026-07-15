// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::neural_executor::GraphExecutor;
use crate::neural_executor_async_tests::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_execute_rpc_call_missing_target() {
    let mut node = create_test_node("rpc_bad", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "rpc_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "method".to_string(),
        serde_json::Value::String("ping".to_string()),
    );

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "rpc-missing-target".to_string(),
        version: "1.0".to_string(),
        description: "rpc".to_string(),
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
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

#[tokio::test]
async fn test_execute_rpc_call_missing_method() {
    let mut node = create_test_node("rpc_bad2", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "rpc_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "target".to_string(),
        serde_json::Value::String("any".to_string()),
    );

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "rpc-missing-method".to_string(),
        version: "1.0".to_string(),
        description: "rpc".to_string(),
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
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

#[tokio::test]
async fn test_execute_capability_call_missing_capability_key() {
    let mut node = create_test_node("cap_bad", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "capability_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "cap-missing-cap".to_string(),
        version: "1.0".to_string(),
        description: "c".to_string(),
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
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

/// `capability_call` with explicit `timeout_ms` (neural-api path may skip; direct fallback may run).
#[tokio::test]
async fn test_execute_capability_call_with_timeout_ms_config() {
    let mut node = create_test_node("cap_timeout", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "capability_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "capability".to_string(),
        serde_json::Value::String("nonexistent.domain.op".to_string()),
    );
    node.config
        .insert("timeout_ms".to_string(), serde_json::json!(50u64));

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "cap-timeout".to_string(),
        version: "1.0".to_string(),
        description: "c".to_string(),
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
    env.insert(
        "XDG_RUNTIME_DIR".to_string(),
        temp.path().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

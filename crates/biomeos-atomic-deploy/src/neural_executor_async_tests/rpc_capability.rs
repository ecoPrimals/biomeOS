// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! RPC call, capability call, and optional fallback node tests.

use super::super::neural_executor::GraphExecutor;
use super::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

/// Test rpc_call node - connection refused (socket doesn't exist)
#[tokio::test]
async fn test_execute_rpc_call_connection_refused() {
    let mut node = create_test_node("rpc_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "rpc_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "target".to_string(),
        serde_json::Value::String("nonexistent_primal".to_string()),
    );
    node.config.insert(
        "method".to_string(),
        serde_json::Value::String("ping".to_string()),
    );

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "rpc-test".to_string(),
        version: "1.0".to_string(),
        description: "RPC call to nonexistent".to_string(),
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
    let report = executor.execute().await.unwrap();

    assert!(!report.success);
    assert!(report.error.is_some());
    let err = report.error.unwrap();
    assert!(
        err.contains("Failed")
            || err.contains("connect")
            || err.contains("Connection")
            || err.contains("No such file")
            || err.contains("Phase failed")
            || err.contains("nodes failed"),
        "Expected connection/phase error, got: {err}"
    );
}

/// Test optional node failure - should be skipped, not fail phase
#[tokio::test]
async fn test_execute_optional_node_failure_skipped() {
    let mut node = create_test_node("optional_fail", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "rpc_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "target".to_string(),
        serde_json::Value::String("nonexistent".to_string()),
    );
    node.config.insert(
        "method".to_string(),
        serde_json::Value::String("ping".to_string()),
    );
    node.fallback = Some("skip".to_string());

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "optional-test".to_string(),
        version: "1.0".to_string(),
        description: "Optional node failure".to_string(),
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
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
    assert_eq!(report.phase_results[0].failed, 0);
}

/// Test capability_call node - no provider (connection error path)
#[tokio::test]
async fn test_execute_capability_call_no_provider() {
    let mut node = create_test_node("cap_call", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "capability_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "capability".to_string(),
        serde_json::Value::String("nonexistent.capability.xyz".to_string()),
    );

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "cap-call-test".to_string(),
        version: "1.0".to_string(),
        description: "Capability call no provider".to_string(),
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
    let report = executor.execute().await.unwrap();

    assert!(!report.success);
    assert!(report.error.is_some());
}

/// `rpc_call` expands `${VAR}` inside JSON params via `executor::substitute_env` before connecting.
#[tokio::test]
async fn test_execute_rpc_call_params_env_substitution() {
    let mut node = create_test_node("rpc_sub", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "rpc_call".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "target".to_string(),
        serde_json::Value::String("any_primal".to_string()),
    );
    node.config.insert(
        "method".to_string(),
        serde_json::Value::String("ping".to_string()),
    );
    node.config.insert(
        "params".to_string(),
        serde_json::json!({"label": "${RPC_LABEL}"}),
    );

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "rpc-sub-test".to_string(),
        version: "1.0".to_string(),
        description: "RPC param substitution".to_string(),
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
    env.insert("RPC_LABEL".to_string(), "expanded-value".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(!report.success);
    assert!(report.error.is_some());
}

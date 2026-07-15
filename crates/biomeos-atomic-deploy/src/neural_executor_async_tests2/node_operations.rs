// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::neural_executor::GraphExecutor;
use crate::neural_executor_async_tests::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use std::collections::HashMap;

#[tokio::test]
async fn test_execute_report_deployment_success() {
    let mut node = create_test_node("report1", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "report.deployment_success".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "report-dep".to_string(),
        version: "1.0".to_string(),
        description: "r".to_string(),
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
    assert!(report.success);
}

#[tokio::test]
async fn test_execute_register_capabilities_node() {
    let mut node = create_test_node("regcap", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "register_capabilities".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "reg-cap".to_string(),
        version: "1.0".to_string(),
        description: "r".to_string(),
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
    assert!(report.success);
}

#[tokio::test]
async fn test_execute_node_type_legacy_primal_launch_string() {
    let mut node = create_test_node("legacy_type", vec![]);
    node.node_type = Some("primal.launch".to_string());
    node.config.insert(
        "primal_name".to_string(),
        serde_json::Value::String("nope".to_string()),
    );

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "legacy-nt".to_string(),
        version: "1.0".to_string(),
        description: "legacy".to_string(),
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

#[tokio::test]
async fn test_execute_lineage_verify_siblings() {
    let mut node = create_test_node("lin1", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "lineage.verify_siblings".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "lineage".to_string(),
        version: "1.0".to_string(),
        description: "l".to_string(),
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
async fn test_execute_start_alias_node_type() {
    let mut node = create_test_node("start_alias", vec![]);
    node.node_type = Some("start".to_string());

    let temp = tempfile::TempDir::new().expect("tempdir");
    let graph = Graph {
        id: "start-alias".to_string(),
        version: "1.0".to_string(),
        description: "s".to_string(),
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
async fn test_execute_two_phase_second_fails_without_rollback() {
    let mut n1 = create_test_node("ok_node", vec![]);
    n1.operation = Some(crate::neural_graph::Operation {
        name: "log.info".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    n1.config.insert(
        "message".to_string(),
        serde_json::Value::String("ok".to_string()),
    );

    let mut n2 = create_test_node("bad_node", vec!["ok_node".to_string()]);
    n2.operation = Some(crate::neural_graph::Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "two-phase-fail".to_string(),
        version: "1.0".to_string(),
        description: "t".to_string(),
        nodes: vec![n1, n2],
        config: GraphConfig {
            rollback_on_failure: false,
            ..GraphConfig::default()
        },
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
    assert!(!report.phase_results.is_empty());
}

/// `crypto.derive_child_seed` falls back to deterministic seed when no security primal is up.
#[tokio::test]
async fn test_execute_crypto_derive_child_seed_deterministic_fallback() {
    let mut node = create_test_node("crypto1", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "crypto.derive_child_seed".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "source".to_string(),
        serde_json::Value::String("unit-test-source".to_string()),
    );

    let graph = Graph {
        id: "crypto-fallback".to_string(),
        version: "1.0".to_string(),
        description: "crypto".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test-family-xyz".to_string());
    env.insert(
        "SOCKET_DIR".to_string(),
        std::env::temp_dir().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
}

/// Legacy `node_type` only — selects handler when `operation` is absent.
#[tokio::test]
async fn test_execute_node_type_crypto_derive_without_operation() {
    let mut node = create_test_node("nt_crypto", vec![]);
    node.node_type = Some("crypto.derive_child_seed".to_string());
    node.config.insert(
        "source".to_string(),
        serde_json::Value::String("legacy".to_string()),
    );

    let graph = Graph {
        id: "nt-crypto".to_string(),
        version: "1.0".to_string(),
        description: "nt".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam-nt".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
}

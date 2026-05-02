// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! GraphExecutor utility tests: topological sort, env substitution,
//! metrics, optional node handling, and edge-case coverage.

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::neural_executor::GraphExecutor;
use crate::neural_executor_async_tests::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use biomeos_graph::metrics::MetricsCollector;
use std::collections::HashMap;

#[test]
fn test_topological_sort_includes_all_node_ids() {
    let graph = Graph {
        id: "ids".to_string(),
        version: "1.0".to_string(),
        description: "d".to_string(),
        nodes: vec![
            create_test_node("x", vec![]),
            create_test_node("y", vec!["x".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let ex = GraphExecutor::new(graph, HashMap::new());
    let phases = ex.topological_sort().expect("sort");
    let mut seen = std::collections::HashSet::new();
    for p in &phases {
        for id in p {
            assert!(seen.insert(id.clone()));
        }
    }
    assert_eq!(seen.len(), 2);
}

#[test]
fn test_split_capability_dotted_and_plain() {
    assert_eq!(
        GraphExecutor::split_capability("ecology.et0_fao56"),
        ("ecology".to_string(), "et0_fao56".to_string())
    );
    assert_eq!(
        GraphExecutor::split_capability("solo"),
        ("solo".to_string(), "execute".to_string())
    );
}

#[test]
fn test_substitute_env_replaces_placeholders() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    assert_eq!(
        GraphExecutor::substitute_env("pre-${FOO}-post", &env),
        "pre-bar-post"
    );
    assert_eq!(
        GraphExecutor::substitute_env("no placeholders", &env),
        "no placeholders"
    );
}

#[test]
fn test_topological_sort_empty_graph() {
    let graph = Graph {
        id: "t-empty".to_string(),
        version: "1.0".to_string(),
        description: "empty".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let ex = GraphExecutor::new(graph, HashMap::new());
    let phases = ex.topological_sort().expect("sort");
    assert!(phases.is_empty());
}

#[test]
fn test_topological_sort_cycle_fails() {
    let graph = Graph {
        id: "t-cycle".to_string(),
        version: "1.0".to_string(),
        description: "cycle".to_string(),
        nodes: vec![
            create_test_node("a", vec!["b".to_string()]),
            create_test_node("b", vec!["a".to_string()]),
        ],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let ex = GraphExecutor::new(graph, HashMap::new());
    let err = ex.topological_sort().expect_err("cycle");
    assert!(err.to_string().contains("cycles"));
}

#[tokio::test]
async fn test_execute_empty_graph_succeeds() {
    let graph = Graph {
        id: "exec-empty".to_string(),
        version: "1.0".to_string(),
        description: "empty".to_string(),
        nodes: vec![],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
    assert!(report.phase_results.is_empty());
}

#[tokio::test]
async fn test_execute_optional_rpc_missing_target_skipped() {
    let mut node = create_test_node("opt_rpc", vec![]);
    node.fallback = Some("skip".to_string());
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
        id: "opt-rpc".to_string(),
        version: "1.0".to_string(),
        description: "optional rpc".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SOCKET_DIR".to_string(),
        temp.path().to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
    assert_eq!(report.phase_results[0].failed, 0);
}

#[tokio::test]
async fn test_execute_with_metrics_collector() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    let metrics =
        MetricsCollector::new(temp.path().join("pathway-metrics.redb")).expect("metrics db");

    let mut node = create_test_node("metrics_log", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "log.info".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config.insert(
        "message".to_string(),
        serde_json::Value::String("metrics coverage".to_string()),
    );

    let graph = Graph {
        id: "metrics-graph".to_string(),
        version: "1.0".to_string(),
        description: "m".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env).with_metrics(metrics);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
}

#[tokio::test]
async fn test_execute_health_check_all_nonexistent_socket_dir() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    let missing_dir = temp.path().join("definitely_missing_sock_dir");

    let mut node = create_test_node("hca_empty", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check_all".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "hca-missing-dir".to_string(),
        version: "1.0".to_string(),
        description: "hca".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SOCKET_DIR".to_string(),
        missing_dir.to_string_lossy().to_string(),
    );

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
}

#[tokio::test]
async fn test_execute_verification_check_sockets_false_without_socket_dir() {
    let mut node = create_test_node("verify_no_sock", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "verification".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node.config
        .insert("check_sockets".to_string(), serde_json::Value::Bool(false));

    let graph = Graph {
        id: "verify-no-socket-dir-needed".to_string(),
        version: "1.0".to_string(),
        description: "v".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
}

#[tokio::test]
async fn test_execute_unknown_operation_yields_skipped_success() {
    let mut node = create_test_node("unknown_op", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "totally.unknown.node_type".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "unknown-op-graph".to_string(),
        version: "1.0".to_string(),
        description: "u".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
    assert_eq!(report.phase_results.len(), 1);
    assert_eq!(report.phase_results[0].completed, 1);
}

#[tokio::test]
async fn test_execute_metrics_with_failing_phase_still_produces_report() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    let metrics = MetricsCollector::new(temp.path().join("metrics.redb")).expect("metrics");

    let mut node = create_test_node("m_fail", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = Graph {
        id: "metrics-fail-graph".to_string(),
        version: "1.0".to_string(),
        description: "m".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env).with_metrics(metrics);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

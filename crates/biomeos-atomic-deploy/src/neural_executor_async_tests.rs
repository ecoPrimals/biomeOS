// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Async integration tests for GraphExecutor (split from `neural_executor_tests.rs`).
//! Branch-coverage tests live in `neural_executor_async_tests2.rs`.

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::neural_executor::GraphExecutor;
use crate::neural_graph::{Graph, GraphConfig, GraphNode};
use std::collections::HashMap;

pub(crate) fn create_test_node(id: &str, depends_on: Vec<String>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        depends_on,
        ..Default::default()
    }
}

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
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

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
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(!report.success);
    assert!(report.error.is_some());
}

#[test]
fn test_execution_report_new_and_builder() {
    use crate::executor::types::ExecutionReport;

    let mut report = ExecutionReport::new("graph-1");
    assert_eq!(report.graph_id, "graph-1");
    assert!(report.success);
    assert_eq!(report.duration_ms, 0);

    report = report.mark_failed("test error");
    assert!(!report.success);
    assert_eq!(report.error.as_deref(), Some("test error"));

    report = report.with_duration(500).with_phases(2).with_nodes(4);
    assert_eq!(report.duration_ms, 500);
    assert_eq!(report.total_phases(), 2);
    assert_eq!(report.total_nodes(), 4);
}

#[test]
fn test_phase_result_builder() {
    use crate::executor::types::PhaseResult;

    let mut result = PhaseResult::new(3);
    result.add_completed();
    result.add_completed();
    result.add_failed("node3", "timeout");
    result.duration_ms = 100;

    assert_eq!(result.completed, 2);
    assert_eq!(result.failed, 1);
    assert!(!result.is_success());
    assert_eq!(
        result.errors,
        vec![("node3".to_string(), "timeout".to_string())]
    );
}

#[test]
fn test_execution_report_add_phase_result() {
    use crate::executor::types::{ExecutionReport, PhaseResult};

    let mut report = ExecutionReport::new("test");
    let mut phase = PhaseResult::new(2);
    phase.completed = 2;
    phase.duration_ms = 50;
    report.add_phase_result(&phase);

    assert_eq!(report.phase_results.len(), 1);
    assert_eq!(report.total_phases(), 1);
    assert_eq!(report.total_nodes(), 2);
}

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
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results[0].completed, 1);
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

/// Semaphore limits concurrent node tasks (`max_parallelism`).
#[tokio::test]
async fn test_execute_parallel_nodes_with_max_parallelism_one() {
    let temp = tempfile::TempDir::new().expect("tempdir");
    let mut nodes = Vec::new();
    for i in 0..4 {
        let mut node = create_test_node(&format!("n{i}"), vec![]);
        node.operation = Some(crate::neural_graph::Operation {
            name: "log.info".to_string(),
            target: None,
            params: HashMap::new(),
            environment: None,
        });
        node.config.insert(
            "message".to_string(),
            serde_json::Value::String(format!("msg {i}")),
        );
        nodes.push(node);
    }
    let graph = Graph {
        id: "parallel-one".to_string(),
        version: "1.0".to_string(),
        description: "Parallelism cap".to_string(),
        nodes,
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
    executor.max_parallelism = 1;
    let report = executor.execute().await.unwrap();

    assert!(report.success);
    assert_eq!(report.phase_results.len(), 1);
    assert_eq!(report.phase_results[0].completed, 4);
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
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.unwrap();

    assert!(!report.success);
    assert!(report.error.is_some());
}

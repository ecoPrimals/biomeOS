// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Additional async integration tests for GraphExecutor (branch coverage — split from `neural_executor_async_tests.rs`).

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::neural_executor::GraphExecutor;
use crate::neural_executor_async_tests::create_test_node;
use crate::neural_graph::{Graph, GraphConfig};
use biomeos_graph::metrics::MetricsCollector;
use std::collections::HashMap;

// --- Additional branch coverage (verification, rpc_call config, node_type paths) ---

#[tokio::test]
async fn test_execute_verification_socket_dir_missing() {
    let mut node = create_test_node("verify_node", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "verification".to_string(),
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
    };
    let env = HashMap::from([("FAMILY_ID".to_string(), "test".to_string())]);

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
    assert!(report.error.is_some());
}

#[tokio::test]
async fn test_execute_rpc_call_missing_target() {
    let mut node = create_test_node("rpc_bad", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "rpc_call".to_string(),
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
async fn test_execute_health_check_atomic_alias() {
    let mut node = create_test_node("hc_atomic", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check_atomic".to_string(),
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
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

#[tokio::test]
async fn test_execute_report_deployment_success() {
    let mut node = create_test_node("report1", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "report.deployment_success".to_string(),
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
async fn test_execute_capability_call_missing_capability_key() {
    let mut node = create_test_node("cap_bad", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "capability_call".to_string(),
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
async fn test_execute_lineage_verify_siblings() {
    let mut node = create_test_node("lin1", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "lineage.verify_siblings".to_string(),
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
async fn test_execute_health_check_all_missing_socket_dir() {
    let mut node = create_test_node("hca", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "health.check_all".to_string(),
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
    };
    let env = HashMap::from([("FAMILY_ID".to_string(), "test".to_string())]);

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

#[tokio::test]
async fn test_execute_two_phase_second_fails_without_rollback() {
    let mut n1 = create_test_node("ok_node", vec![]);
    n1.operation = Some(crate::neural_graph::Operation {
        name: "log.info".to_string(),
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

/// `verification` with `check_sockets=true` and no `dependencies` completes with zero verified.
#[tokio::test]
async fn test_execute_verification_check_sockets_true_no_deps() {
    let mut node = create_test_node("verify_empty", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "verification".to_string(),
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
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
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
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam-nt".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env);
    let report = executor.execute().await.expect("report");
    assert!(report.success);
}

/// `capability_call` with explicit `timeout_ms` (neural-api path may skip; direct fallback may run).
#[tokio::test]
async fn test_execute_capability_call_with_timeout_ms_config() {
    let mut node = create_test_node("cap_timeout", vec![]);
    node.operation = Some(crate::neural_graph::Operation {
        name: "capability_call".to_string(),
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

/// Phase failure on missing node id in graph — defensive (topological sort prevents this).
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

// --- split_capability, substitute_env, empty graph, metrics, optional failure, config errors ---

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
    };
    // Intentionally no SOCKET_DIR — branch when check_sockets is false
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
        params: HashMap::new(),
        environment: None,
    });
    // Missing `path` config → phase fails; metrics path still runs (warn on graph metrics ok)

    let graph = Graph {
        id: "metrics-fail-graph".to_string(),
        version: "1.0".to_string(),
        description: "m".to_string(),
        nodes: vec![node],
        config: GraphConfig::default(),
        coordination: None,
        env: HashMap::new(),
    };
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert("SOCKET_DIR".to_string(), "/tmp".to_string());

    let mut executor = GraphExecutor::new(graph, env).with_metrics(metrics);
    let report = executor.execute().await.expect("report");
    assert!(!report.success);
}

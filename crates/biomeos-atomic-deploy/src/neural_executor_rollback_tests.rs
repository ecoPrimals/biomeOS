// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for graph rollback (`neural_executor_rollback.rs`).

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::super::neural_executor::{GraphExecutor, NodeStatus};
use crate::neural_executor_async_tests::create_test_node;
use crate::neural_graph::{Graph, GraphConfig, GraphNode, Operation};
use crate::nucleation::SocketNucleation;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

pub(crate) fn test_graph(nodes: Vec<GraphNode>, config: GraphConfig) -> Graph {
    Graph {
        id: "rollback-test".to_string(),
        version: "1.0".to_string(),
        description: "rollback coverage".to_string(),
        nodes,
        config,
        coordination: None,
        env: HashMap::new(),
        genetics_tier: None,
        composition_model: None,
    }
}

pub(crate) fn node_with_operation(id: &str, op_name: &str, depends_on: Vec<String>) -> GraphNode {
    let mut node = create_test_node(id, depends_on);
    node.operation = Some(Operation {
        name: op_name.to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node
}

pub(crate) fn env_with_runtime(family_id: &str, runtime_dir: &Path) -> HashMap<String, String> {
    HashMap::from([
        ("FAMILY_ID".to_string(), family_id.to_string()),
        (
            "XDG_RUNTIME_DIR".to_string(),
            runtime_dir.to_string_lossy().into_owned(),
        ),
    ])
}

pub(crate) fn neural_api_socket_path(family_id: &str, runtime_dir: &Path) -> PathBuf {
    let mut nuc = SocketNucleation::default();
    nuc.assign_socket_with_runtime_dir("neural-api", family_id, Some(runtime_dir))
}

pub(crate) async fn set_completed(
    executor: &GraphExecutor,
    node_id: &str,
    output: serde_json::Value,
) {
    executor
        .context
        .set_status(node_id, NodeStatus::Completed(output))
        .await;
}

// ---------------------------------------------------------------------------
// resolve_node_type
// ---------------------------------------------------------------------------

#[test]
fn neural_executor_rollback_resolve_node_type_prefers_operation_name() {
    let node = GraphNode {
        operation: Some(Operation {
            name: "primal.launch".to_string(),
            target: None,
            params: HashMap::new(),
            environment: None,
        }),
        node_type: Some("ignored".to_string()),
        ..Default::default()
    };
    assert_eq!(GraphExecutor::resolve_node_type(&node), "primal.launch");
}

#[test]
fn neural_executor_rollback_resolve_node_type_falls_back_to_node_type() {
    let node = GraphNode {
        operation: Some(Operation {
            name: String::new(),
            target: None,
            params: HashMap::new(),
            environment: None,
        }),
        node_type: Some("register_capabilities".to_string()),
        ..Default::default()
    };
    assert_eq!(
        GraphExecutor::resolve_node_type(&node),
        "register_capabilities"
    );
}

#[test]
fn neural_executor_rollback_resolve_node_type_unknown() {
    let node = GraphNode::default();
    assert_eq!(GraphExecutor::resolve_node_type(&node), "unknown");
}

#[test]
fn neural_executor_rollback_resolve_node_type_from_node_type_field_only() {
    let node = GraphNode {
        node_type: Some("primal_start".to_string()),
        ..Default::default()
    };
    assert_eq!(GraphExecutor::resolve_node_type(&node), "primal_start");
}

// ---------------------------------------------------------------------------
// rollback() entry points and status filtering
// ---------------------------------------------------------------------------

#[tokio::test]
async fn neural_executor_rollback_empty_graph_completes() {
    let executor = GraphExecutor::new(test_graph(vec![], GraphConfig::default()), HashMap::new());
    executor.rollback().await.expect("rollback ok");
}

#[tokio::test]
async fn neural_executor_rollback_topological_sort_failure_returns_ok() {
    let graph = test_graph(
        vec![create_test_node("cycle", vec!["cycle".to_string()])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "cycle",
        json!({ "primal": "beardog", "socket": "/tmp/unused.sock" }),
    )
    .await;
    executor
        .rollback()
        .await
        .expect("rollback tolerates sort failure");
}

#[tokio::test]
async fn neural_executor_rollback_skips_non_completed_statuses() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("primal.sock");
    let calls = Arc::new(Mutex::new(0_u32));
    let calls_clone = Arc::clone(&calls);
    let _server = MockJsonRpcServer::spawn(&sock, move |req| {
        if req.contains("lifecycle.stop") {
            *calls_clone.lock().expect("lock") += 1;
        }
        r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
    })
    .await;

    let mut completed = node_with_operation("done", "start", vec![]);
    completed
        .config
        .insert("primal_name".to_string(), json!("beardog"));

    let graph = test_graph(
        vec![
            create_test_node("pending", vec![]),
            create_test_node("running", vec![]),
            create_test_node("failed", vec![]),
            create_test_node("skipped", vec![]),
            completed,
        ],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());

    executor
        .context
        .set_status("pending", NodeStatus::Pending)
        .await;
    executor
        .context
        .set_status("running", NodeStatus::Running)
        .await;
    executor
        .context
        .set_status("failed", NodeStatus::Failed("boom".to_string()))
        .await;
    executor
        .context
        .set_status("skipped", NodeStatus::Skipped)
        .await;
    set_completed(
        &executor,
        "done",
        json!({ "primal": "beardog", "socket": sock.to_string_lossy() }),
    )
    .await;

    executor.rollback().await.expect("rollback ok");
    assert_eq!(*calls.lock().expect("lock"), 1);
}

#[tokio::test]
async fn neural_executor_rollback_skips_nodes_without_status() {
    let graph = test_graph(
        vec![node_with_operation("orphan", "rpc_call", vec![])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    executor.rollback().await.expect("rollback ok");
}

#[tokio::test]
async fn neural_executor_rollback_execute_skips_when_rollback_disabled() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("would-stop.sock");
    let calls = Arc::new(Mutex::new(0_u32));
    let calls_clone = Arc::clone(&calls);
    let _server = MockJsonRpcServer::spawn(&sock, move |req| {
        if req.contains("lifecycle.stop") {
            *calls_clone.lock().expect("lock") += 1;
        }
        r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
    })
    .await;

    let mut completed = node_with_operation("done", "start", vec![]);
    completed
        .config
        .insert("primal_name".to_string(), json!("beardog"));

    let mut fail_node = create_test_node("fail", vec!["done".to_string()]);
    fail_node.operation = Some(Operation {
        name: "filesystem.check_exists".to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });

    let graph = test_graph(
        vec![completed, fail_node],
        GraphConfig {
            rollback_on_failure: false,
            ..GraphConfig::default()
        },
    );
    let mut executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "done",
        json!({ "primal": "beardog", "socket": sock.to_string_lossy() }),
    )
    .await;

    let report = executor.execute().await.expect("execute report");
    assert!(!report.success);
    assert_eq!(
        *calls.lock().expect("lock"),
        0,
        "rollback must not run when disabled"
    );
}

#[tokio::test]
async fn neural_executor_rollback_execute_triggers_on_phase_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "rollback-exec-fam";
    let unregister_calls = Arc::new(Mutex::new(0_u32));
    let unregister_calls_clone = Arc::clone(&unregister_calls);

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let _server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                if req.contains("capability.unregister") {
                    *unregister_calls_clone.lock().expect("lock") += 1;
                }
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let mut ok_node = node_with_operation("ok", "register_capabilities", vec![]);
            ok_node
                .config
                .insert("primal_name".to_string(), json!("nestgate"));
            ok_node.capabilities = vec!["storage.read".to_string()];

            let mut fail_node = create_test_node("fail", vec!["ok".to_string()]);
            fail_node.operation = Some(Operation {
                name: "filesystem.check_exists".to_string(),
                target: None,
                params: HashMap::new(),
                environment: None,
            });
            // Missing `path` config — node execution fails and triggers rollback.

            let graph = test_graph(
                vec![ok_node, fail_node],
                GraphConfig {
                    rollback_on_failure: true,
                    ..GraphConfig::default()
                },
            );
            let mut executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            let report = executor.execute().await.expect("execute report");

            assert!(!report.success);
            assert!(report.error.is_some());
            assert_eq!(
                *unregister_calls.lock().expect("lock"),
                1,
                "execute() must invoke rollback capability.unregister on phase failure"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_execute_triggers_on_checkpoint_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let checkpoint_blocker = iso.join("checkpoint-blocker");
    std::fs::write(&checkpoint_blocker, b"not-a-directory").expect("blocker file");

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let mut ok_node = create_test_node("ok", vec![]);
            ok_node.operation = Some(Operation {
                name: "log.info".to_string(),
                target: None,
                params: HashMap::new(),
                environment: None,
            });
            ok_node.config.insert("message".to_string(), json!("done"));

            let graph = test_graph(
                vec![ok_node],
                GraphConfig {
                    rollback_on_failure: true,
                    ..GraphConfig::default()
                },
            );
            let mut executor = GraphExecutor::new(graph, env_with_runtime("ckpt-err-fam", &iso));
            executor.context.checkpoint_dir = Some(checkpoint_blocker);

            let report = executor.execute().await.expect("execute report");
            assert!(!report.success);
            assert!(report.error.is_some());
        },
    )
    .await;
}

// ---------------------------------------------------------------------------
// primal lifecycle rollback
// ---------------------------------------------------------------------------

#[path = "neural_executor_rollback_lifecycle_tests.rs"]
mod lifecycle;

#[path = "neural_executor_rollback_capability_tests.rs"]
mod capability;

#[path = "neural_executor_rollback_order_tests.rs"]
mod order;

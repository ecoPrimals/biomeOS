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

fn test_graph(nodes: Vec<GraphNode>, config: GraphConfig) -> Graph {
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

fn node_with_operation(id: &str, op_name: &str, depends_on: Vec<String>) -> GraphNode {
    let mut node = create_test_node(id, depends_on);
    node.operation = Some(Operation {
        name: op_name.to_string(),
        target: None,
        params: HashMap::new(),
        environment: None,
    });
    node
}

fn env_with_runtime(family_id: &str, runtime_dir: &Path) -> HashMap<String, String> {
    HashMap::from([
        ("FAMILY_ID".to_string(), family_id.to_string()),
        (
            "XDG_RUNTIME_DIR".to_string(),
            runtime_dir.to_string_lossy().into_owned(),
        ),
    ])
}

fn neural_api_socket_path(family_id: &str, runtime_dir: &Path) -> PathBuf {
    let mut nuc = SocketNucleation::default();
    nuc.assign_socket_with_runtime_dir("neural-api", family_id, Some(runtime_dir))
}

async fn set_completed(executor: &GraphExecutor, node_id: &str, output: serde_json::Value) {
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

#[tokio::test]
async fn neural_executor_rollback_primal_start_sends_lifecycle_stop_success() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("beardog.sock");
    let method = Arc::new(Mutex::new(String::new()));
    let method_clone = Arc::clone(&method);
    let _server = MockJsonRpcServer::spawn(&sock, move |req| {
        let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
        *method_clone.lock().expect("lock") = parsed["method"].as_str().unwrap_or("").to_string();
        r#"{"jsonrpc":"2.0","id":1,"result":{"stopped":true}}"#.to_string()
    })
    .await;

    let graph = test_graph(
        vec![node_with_operation("launch", "start", vec![])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "launch",
        json!({ "primal": "beardog", "socket": sock.to_string_lossy() }),
    )
    .await;

    executor.rollback().await.expect("rollback ok");
    assert_eq!(*method.lock().expect("lock"), "lifecycle.stop");
}

#[tokio::test]
async fn neural_executor_rollback_primal_start_aliases_invoke_lifecycle_stop() {
    for op in ["primal.launch", "primal_start", "start"] {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join(format!("{op}.sock"));
        let seen = Arc::new(Mutex::new(false));
        let seen_clone = Arc::clone(&seen);
        let _server = MockJsonRpcServer::spawn(&sock, move |req| {
            if req.contains("lifecycle.stop") {
                *seen_clone.lock().expect("lock") = true;
            }
            r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
        })
        .await;

        let graph = test_graph(
            vec![node_with_operation("n", op, vec![])],
            GraphConfig::default(),
        );
        let executor = GraphExecutor::new(graph, HashMap::new());
        set_completed(
            &executor,
            "n",
            json!({ "primal": "p", "socket": sock.to_string_lossy() }),
        )
        .await;
        executor.rollback().await.expect("rollback ok");
        assert!(
            *seen.lock().expect("lock"),
            "expected lifecycle.stop for {op}"
        );
    }
}

#[tokio::test]
async fn neural_executor_rollback_primal_start_missing_primal_name_skips() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("unused.sock");
    let calls = Arc::new(Mutex::new(0_u32));
    let calls_clone = Arc::clone(&calls);
    let _server = MockJsonRpcServer::spawn(&sock, move |_req| {
        *calls_clone.lock().expect("lock") += 1;
        r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
    })
    .await;

    let graph = test_graph(
        vec![node_with_operation("launch", "start", vec![])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "launch",
        json!({ "socket": sock.to_string_lossy() }),
    )
    .await;

    executor.rollback().await.expect("rollback ok");
    assert_eq!(*calls.lock().expect("lock"), 0);
}

#[tokio::test]
async fn neural_executor_rollback_primal_start_uses_config_primal_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let mut node = node_with_operation("launch", "start", vec![]);
            node.config
                .insert("primal_name".to_string(), json!("songbird"));

            let mut nuc = SocketNucleation::default();
            let sock = nuc.assign_socket_with_runtime_dir("songbird", "cfg-fam", Some(&iso));
            let _ = std::fs::remove_file(&sock);
            let seen = Arc::new(Mutex::new(false));
            let seen_clone = Arc::clone(&seen);
            let _server = MockJsonRpcServer::spawn(&sock, move |req| {
                if req.contains("lifecycle.stop") {
                    *seen_clone.lock().expect("lock") = true;
                }
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let graph = test_graph(vec![node], GraphConfig::default());
            let executor = GraphExecutor::new(graph, env_with_runtime("cfg-fam", &iso));
            set_completed(&executor, "launch", json!({})).await;

            executor.rollback().await.expect("rollback ok");
            assert!(*seen.lock().expect("lock"));
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_primal_start_missing_socket_skips() {
    let graph = test_graph(
        vec![node_with_operation("launch", "start", vec![])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "launch",
        json!({ "primal": "beardog", "socket": "/no/such/primal.sock" }),
    )
    .await;

    executor.rollback().await.expect("rollback ok");
}

#[tokio::test]
async fn neural_executor_rollback_primal_start_rpc_error_response() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("err.sock");
    let _server = MockJsonRpcServer::spawn_echo_error(&sock, -32000, "stop rejected").await;

    let graph = test_graph(
        vec![node_with_operation("launch", "start", vec![])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "launch",
        json!({ "primal": "beardog", "socket": sock.to_string_lossy() }),
    )
    .await;

    executor
        .rollback()
        .await
        .expect("rollback tolerates rpc error");
}

#[tokio::test]
async fn neural_executor_rollback_primal_start_connection_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("fake.sock");
    std::fs::write(&sock, b"not-a-socket").expect("write fake socket file");

    let graph = test_graph(
        vec![node_with_operation("launch", "start", vec![])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "launch",
        json!({ "primal": "beardog", "socket": sock.to_string_lossy() }),
    )
    .await;

    executor
        .rollback()
        .await
        .expect("rollback tolerates connect error");
}

// ---------------------------------------------------------------------------
// register_capabilities rollback
// ---------------------------------------------------------------------------

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_success() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-rollback-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let captured = Arc::new(Mutex::new(None::<serde_json::Value>));
            let captured_clone = Arc::clone(&captured);
            let _server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                *captured_clone.lock().expect("lock") = Some(parsed);
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let graph = test_graph(
                vec![node_with_operation("reg", "register_capabilities", vec![])],
                GraphConfig::default(),
            );
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(
                &executor,
                "reg",
                json!({
                    "primal": "nestgate",
                    "registered": ["storage.read", "storage.write"]
                }),
            )
            .await;

            executor.rollback().await.expect("rollback ok");

            let req = captured.lock().expect("lock").take().expect("rpc sent");
            assert_eq!(req["method"], "capability.unregister");
            assert_eq!(req["params"]["primal"], "nestgate");
            assert_eq!(
                req["params"]["capabilities"],
                json!(["storage.read", "storage.write"])
            );
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_unknown_primal_fallback() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-unknown-primal-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let captured = Arc::new(Mutex::new(None::<serde_json::Value>));
            let captured_clone = Arc::clone(&captured);
            let _server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                *captured_clone.lock().expect("lock") = Some(parsed);
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let graph = test_graph(
                vec![node_with_operation("reg", "register_capabilities", vec![])],
                GraphConfig::default(),
            );
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(&executor, "reg", json!({ "registered": ["a.b"] })).await;

            executor.rollback().await.expect("rollback ok");

            let req = captured.lock().expect("lock").take().expect("rpc sent");
            assert_eq!(req["params"]["primal"], "unknown");
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_uses_config_primal_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-config-primal-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let captured = Arc::new(Mutex::new(None::<serde_json::Value>));
            let captured_clone = Arc::clone(&captured);
            let _server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                *captured_clone.lock().expect("lock") = Some(parsed);
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let mut node = node_with_operation("reg", "register_capabilities", vec![]);
            node.config
                .insert("primal_name".to_string(), json!("from-config"));

            let graph = test_graph(vec![node], GraphConfig::default());
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(&executor, "reg", json!({ "registered": ["cap.one"] })).await;

            executor.rollback().await.expect("rollback ok");

            let req = captured.lock().expect("lock").take().expect("rpc sent");
            assert_eq!(req["params"]["primal"], "from-config");
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_uses_node_capabilities_fallback() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-fallback-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let captured = Arc::new(Mutex::new(None::<serde_json::Value>));
            let captured_clone = Arc::clone(&captured);
            let _server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                *captured_clone.lock().expect("lock") = Some(parsed);
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let mut node = node_with_operation("reg", "register_capabilities", vec![]);
            node.capabilities = vec!["mesh.sync".to_string()];

            let graph = test_graph(vec![node], GraphConfig::default());
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(&executor, "reg", json!({ "primal": "toadstool" })).await;

            executor.rollback().await.expect("rollback ok");

            let req = captured.lock().expect("lock").take().expect("rpc sent");
            assert_eq!(req["params"]["capabilities"], json!(["mesh.sync"]));
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_missing_neural_socket() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let graph = test_graph(
                vec![node_with_operation("reg", "register_capabilities", vec![])],
                GraphConfig::default(),
            );
            let executor = GraphExecutor::new(graph, env_with_runtime("missing-neural-fam", &iso));
            set_completed(
                &executor,
                "reg",
                json!({ "primal": "nestgate", "registered": ["a.b"] }),
            )
            .await;

            executor
                .rollback()
                .await
                .expect("rollback ok without neural socket");
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_rpc_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-error-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let _server =
                MockJsonRpcServer::spawn_echo_error(&neural_sock, -32001, "unregister failed")
                    .await;

            let graph = test_graph(
                vec![node_with_operation("reg", "register_capabilities", vec![])],
                GraphConfig::default(),
            );
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(
                &executor,
                "reg",
                json!({ "primal": "nestgate", "registered": ["x.y"] }),
            )
            .await;

            executor
                .rollback()
                .await
                .expect("rollback tolerates unregister error");
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_connection_failure() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-badconn-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            std::fs::create_dir_all(neural_sock.parent().expect("parent")).expect("mkdir");
            std::fs::write(&neural_sock, b"not-a-socket").expect("fake socket file");

            let graph = test_graph(
                vec![node_with_operation("reg", "register_capabilities", vec![])],
                GraphConfig::default(),
            );
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(
                &executor,
                "reg",
                json!({ "primal": "nestgate", "registered": ["x.y"] }),
            )
            .await;

            executor
                .rollback()
                .await
                .expect("rollback tolerates connect error");
        },
    )
    .await;
}

// ---------------------------------------------------------------------------
// non-reversible and unknown node types
// ---------------------------------------------------------------------------

#[tokio::test]
async fn neural_executor_rollback_rpc_and_capability_call_nodes_not_reversed() {
    let graph = test_graph(
        vec![
            node_with_operation("rpc", "rpc_call", vec![]),
            node_with_operation("cap", "capability_call", vec![]),
        ],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(&executor, "rpc", json!({ "method": "ping" })).await;
    set_completed(&executor, "cap", json!({ "capability": "x.y" })).await;

    executor.rollback().await.expect("rollback ok");
}

#[tokio::test]
async fn neural_executor_rollback_unknown_node_type_no_action() {
    let mut node = create_test_node("custom", vec![]);
    node.node_type = Some("custom.operation".to_string());

    let graph = test_graph(vec![node], GraphConfig::default());
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(&executor, "custom", json!({ "done": true })).await;

    executor.rollback().await.expect("rollback ok");
}

#[tokio::test]
async fn neural_executor_rollback_reverse_topological_order() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_c = dir.path().join("c.sock");
    let sock_b = dir.path().join("b.sock");
    let order = Arc::new(Mutex::new(Vec::<String>::new()));

    let _server_c = MockJsonRpcServer::spawn(&sock_c, {
        let order_clone = Arc::clone(&order);
        move |req| {
            if req.contains("lifecycle.stop") {
                order_clone.lock().expect("lock").push("c".to_string());
            }
            r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
        }
    })
    .await;
    let _server_b = MockJsonRpcServer::spawn(&sock_b, {
        let order_clone = Arc::clone(&order);
        move |req| {
            if req.contains("lifecycle.stop") {
                order_clone.lock().expect("lock").push("b".to_string());
            }
            r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
        }
    })
    .await;

    let graph = test_graph(
        vec![
            node_with_operation("a", "log.info", vec![]),
            node_with_operation("b", "start", vec!["a".to_string()]),
            node_with_operation("c", "start", vec!["b".to_string()]),
        ],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(&executor, "a", json!({ "message": "done" })).await;
    set_completed(
        &executor,
        "b",
        json!({ "primal": "p-b", "socket": sock_b.to_string_lossy() }),
    )
    .await;
    set_completed(
        &executor,
        "c",
        json!({ "primal": "p-c", "socket": sock_c.to_string_lossy() }),
    )
    .await;

    executor.rollback().await.expect("rollback ok");
    assert_eq!(*order.lock().unwrap(), vec!["c", "b"]);
}

#[tokio::test]
async fn neural_executor_rollback_skips_completed_status_when_node_removed_from_graph() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("ghost.sock");
    let calls = Arc::new(Mutex::new(0_u32));
    let calls_clone = Arc::clone(&calls);
    let _server = MockJsonRpcServer::spawn(&sock, move |req| {
        if req.contains("lifecycle.stop") {
            *calls_clone.lock().expect("lock") += 1;
        }
        r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
    })
    .await;

    let graph = test_graph(
        vec![node_with_operation("ghost", "start", vec![])],
        GraphConfig::default(),
    );
    let mut executor = GraphExecutor::new(graph, HashMap::new());
    set_completed(
        &executor,
        "ghost",
        json!({ "primal": "beardog", "socket": sock.to_string_lossy() }),
    )
    .await;

    // Simulate stale execution state: status recorded, node definition removed before rollback.
    executor.graph.nodes.clear();

    executor
        .rollback()
        .await
        .expect("rollback tolerates missing node definitions");
    assert_eq!(
        *calls.lock().expect("lock"),
        0,
        "lifecycle.stop must not run when node is absent from graph"
    );
}

#[tokio::test]
async fn neural_executor_rollback_mixed_completed_nodes_in_single_pass() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "mixed-rollback-fam";
    let primal_sock = iso.join("primal.sock");
    let methods = Arc::new(Mutex::new(Vec::<String>::new()));

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let methods_neural = Arc::clone(&methods);
            let _neural_server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                methods_neural
                    .lock()
                    .expect("lock")
                    .push(parsed["method"].as_str().unwrap_or("").to_string());
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let methods_primal = Arc::clone(&methods);
            let _primal_server = MockJsonRpcServer::spawn(&primal_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                methods_primal
                    .lock()
                    .expect("lock")
                    .push(parsed["method"].as_str().unwrap_or("").to_string());
                r#"{"jsonrpc":"2.0","id":1,"result":{"stopped":true}}"#.to_string()
            })
            .await;

            let graph = test_graph(
                vec![
                    node_with_operation("launch", "start", vec![]),
                    node_with_operation("reg", "register_capabilities", vec!["launch".to_string()]),
                ],
                GraphConfig::default(),
            );
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(
                &executor,
                "launch",
                json!({ "primal": "beardog", "socket": primal_sock.to_string_lossy() }),
            )
            .await;
            set_completed(
                &executor,
                "reg",
                json!({ "primal": "nestgate", "registered": ["mesh.sync"] }),
            )
            .await;

            executor.rollback().await.expect("rollback ok");

            let seen = methods.lock().expect("lock");
            assert!(seen.contains(&"lifecycle.stop".to_string()));
            assert!(seen.contains(&"capability.unregister".to_string()));
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_status_transitions_only_completed_reversed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("phase.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "stopped": true })).await;

    let graph = test_graph(
        vec![node_with_operation("start_node", "start", vec![])],
        GraphConfig::default(),
    );
    let executor = GraphExecutor::new(graph, HashMap::new());

    executor
        .context
        .set_status("start_node", NodeStatus::Pending)
        .await;
    executor.rollback().await.expect("pending not reversed");

    executor
        .context
        .set_status("start_node", NodeStatus::Running)
        .await;
    executor.rollback().await.expect("running not reversed");

    executor
        .context
        .set_status("start_node", NodeStatus::Failed("phase error".to_string()))
        .await;
    executor.rollback().await.expect("failed not reversed");

    executor
        .context
        .set_status("start_node", NodeStatus::Skipped)
        .await;
    executor.rollback().await.expect("skipped not reversed");

    set_completed(
        &executor,
        "start_node",
        json!({ "primal": "beardog", "socket": sock.to_string_lossy() }),
    )
    .await;
    executor
        .rollback()
        .await
        .expect("completed node reversed (rolled back)");
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_filters_non_string_registered() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-filter-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let captured = Arc::new(Mutex::new(None::<serde_json::Value>));
            let captured_clone = Arc::clone(&captured);
            let _server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                *captured_clone.lock().expect("lock") = Some(parsed);
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let graph = test_graph(
                vec![node_with_operation("reg", "register_capabilities", vec![])],
                GraphConfig::default(),
            );
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(
                &executor,
                "reg",
                json!({
                    "primal": "nestgate",
                    "registered": ["storage.read", 42, null, true, "mesh.sync"]
                }),
            )
            .await;

            executor.rollback().await.expect("rollback ok");

            let req = captured.lock().expect("lock").take().expect("rpc sent");
            assert_eq!(
                req["params"]["capabilities"],
                json!(["storage.read", "mesh.sync"])
            );
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_register_capabilities_prefers_output_over_node_caps() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "reg-precedence-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let neural_sock = neural_api_socket_path(family, &iso);
            let captured = Arc::new(Mutex::new(None::<serde_json::Value>));
            let captured_clone = Arc::clone(&captured);
            let _server = MockJsonRpcServer::spawn(&neural_sock, move |req| {
                let parsed: serde_json::Value = serde_json::from_str(req).expect("json");
                *captured_clone.lock().expect("lock") = Some(parsed);
                r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string()
            })
            .await;

            let mut node = node_with_operation("reg", "register_capabilities", vec![]);
            node.capabilities = vec!["node.fallback".to_string()];

            let graph = test_graph(vec![node], GraphConfig::default());
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(
                &executor,
                "reg",
                json!({
                    "primal": "nestgate",
                    "registered": ["output.cap"]
                }),
            )
            .await;

            executor.rollback().await.expect("rollback ok");

            let req = captured.lock().expect("lock").take().expect("rpc sent");
            assert_eq!(req["params"]["capabilities"], json!(["output.cap"]));
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_execute_triggers_rollback_on_execute_phase_error() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "phase-err-rollback-fam";
    let primal_sock = iso.join("phase-err-primal.sock");
    let stop_calls = Arc::new(Mutex::new(0_u32));
    let stop_calls_clone = Arc::clone(&stop_calls);

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let _primal_server = MockJsonRpcServer::spawn(&primal_sock, move |req| {
                if req.contains("lifecycle.stop") {
                    *stop_calls_clone.lock().expect("lock") += 1;
                }
                r#"{"jsonrpc":"2.0","id":1,"result":{"stopped":true}}"#.to_string()
            })
            .await;

            let mut launch = node_with_operation("launch", "start", vec![]);
            launch
                .config
                .insert("primal_name".to_string(), json!("beardog"));

            let graph = test_graph(
                vec![launch],
                GraphConfig {
                    rollback_on_failure: true,
                    ..GraphConfig::default()
                },
            );
            let mut executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));

            // Checkpoint before phase 0 fails (path is a file, not a directory).
            let ckpt_blocker = iso.join("checkpoint-blocker");
            std::fs::write(&ckpt_blocker, b"not-a-directory").expect("blocker file");
            executor.context.checkpoint_dir = Some(ckpt_blocker);

            // Stale completed state from an earlier partial run — rollback must still reverse it.
            set_completed(
                &executor,
                "launch",
                json!({
                    "primal": "beardog",
                    "socket": primal_sock.to_string_lossy()
                }),
            )
            .await;

            let report = executor.execute().await.expect("execute report");
            assert!(!report.success);
            assert!(report.error.is_some());
            assert_eq!(
                *stop_calls.lock().expect("lock"),
                1,
                "execute_phase Err path must invoke rollback for completed nodes"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn neural_executor_rollback_primal_start_resolves_socket_via_nucleation() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_path_buf();
    let family = "nuc-socket-fam";

    temp_env::async_with_vars(
        [("XDG_RUNTIME_DIR", Some(iso.to_str().expect("utf8")))],
        async {
            let mut nuc = SocketNucleation::default();
            let sock = nuc.assign_socket_with_runtime_dir("toadstool", family, Some(&iso));
            let seen = Arc::new(Mutex::new(false));
            let seen_clone = Arc::clone(&seen);
            let _server = MockJsonRpcServer::spawn(&sock, move |req| {
                if req.contains("lifecycle.stop") {
                    *seen_clone.lock().expect("lock") = true;
                }
                r#"{"jsonrpc":"2.0","id":1,"result":{"stopped":true}}"#.to_string()
            })
            .await;

            let mut node = node_with_operation("launch", "start", vec![]);
            node.config
                .insert("primal_name".to_string(), json!("toadstool"));

            let graph = test_graph(vec![node], GraphConfig::default());
            let executor = GraphExecutor::new(graph, env_with_runtime(family, &iso));
            set_completed(&executor, "launch", json!({ "primal": "toadstool" })).await;

            executor.rollback().await.expect("rollback ok");
            assert!(*seen.lock().expect("lock"));
        },
    )
    .await;
}

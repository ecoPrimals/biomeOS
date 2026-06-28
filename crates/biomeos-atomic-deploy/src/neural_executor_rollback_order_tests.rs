// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Rollback tests: execution ordering and mixed-status passes.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::{
    env_with_runtime, neural_api_socket_path, node_with_operation, set_completed, test_graph,
};
use crate::neural_executor::{GraphExecutor, NodeStatus};
use crate::neural_executor_async_tests::create_test_node;
use crate::neural_graph::GraphConfig;
use crate::nucleation::SocketNucleation;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

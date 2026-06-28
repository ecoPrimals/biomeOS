// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Rollback tests: capability registration paths.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::{
    env_with_runtime, neural_api_socket_path, node_with_operation, set_completed, test_graph,
};
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::GraphConfig;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use std::sync::{Arc, Mutex};

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

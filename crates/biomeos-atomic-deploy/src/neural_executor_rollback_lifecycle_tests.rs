// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Rollback tests: primal lifecycle stop paths.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::{env_with_runtime, node_with_operation, set_completed, test_graph};
use crate::neural_executor::GraphExecutor;
use crate::neural_graph::GraphConfig;
use crate::nucleation::SocketNucleation;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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

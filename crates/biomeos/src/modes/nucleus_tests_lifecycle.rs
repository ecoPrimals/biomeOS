// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::super::*;
use std::path::PathBuf;
use std::time::Duration;

// --- Lifecycle management (run_start/stop/status/deploy/undeploy) ---

fn valid_spore_manifest_toml() -> &'static str {
    r#"[spore]
mode = "tower"
node_id = "node-test-1"
graph_id = "nucleus_simple"
"#
}

#[test]
fn test_parse_spore_deploy_manifest_valid() {
    let manifest = parse_spore_deploy_manifest(valid_spore_manifest_toml()).expect("valid");
    assert_eq!(manifest.spore.mode, "tower");
    assert_eq!(manifest.spore.node_id, "node-test-1");
    assert_eq!(manifest.spore.graph_id, "nucleus_simple");
}

#[test]
fn test_parse_spore_deploy_manifest_invalid_toml() {
    let err = parse_spore_deploy_manifest("[spore\nbroken = {{{").unwrap_err();
    assert!(err.to_string().contains("Invalid spore manifest"));
}

#[test]
fn test_parse_spore_deploy_manifest_missing_graph_id() {
    let err = parse_spore_deploy_manifest(
        r#"[spore]
mode = "tower"
node_id = "n1"
graph_id = ""
"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("graph_id"));
}

#[test]
fn test_parse_nucleus_status_with_primals() {
    let result = serde_json::json!({
        "primals": [
            { "name": "beardog", "state": "active" },
            { "name": "songbird", "state": "active" }
        ],
        "count": 2,
        "healthy": 2
    });
    let summary = parse_nucleus_status(&result).expect("parse");
    assert_eq!(summary.count, 2);
    assert_eq!(summary.healthy, 2);
    assert_eq!(summary.primal_names, vec!["beardog", "songbird"]);
}

#[test]
fn test_parse_nucleus_status_empty_nucleus() {
    let result = serde_json::json!({
        "primals": [],
        "count": 0,
        "healthy": 0
    });
    let summary = parse_nucleus_status(&result).expect("parse empty");
    assert_eq!(summary.count, 0);
    assert_eq!(summary.healthy, 0);
    assert!(summary.primal_names.is_empty());
}

#[tokio::test]
async fn test_run_start_missing_socket() {
    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let missing = temp.path().join("neural-api-missing.sock");

    temp_env::async_with_vars([("BIOMEOS_SOCKET_DIR", Some(iso_path))], async {
        let err = run_start(
            Some(missing),
            Some("test-family".to_string()),
            "tower".to_string(),
            "node1".to_string(),
        )
        .await
        .expect_err("start should fail without socket");
        assert!(
            err.to_string().contains("socket not found")
                || err.to_string().contains("Failed to connect"),
            "expected socket error: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn test_run_start_success() {
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("neural-api-test.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "started": true, "mode": "tower" }))
            .await;

    let result = run_start(
        Some(sock),
        Some("test-family".to_string()),
        "tower".to_string(),
        "node1".to_string(),
    )
    .await;
    assert!(result.is_ok(), "start should succeed: {result:?}");
}

#[tokio::test]
async fn test_run_stop_missing_socket() {
    let temp = tempfile::tempdir().expect("temp dir");
    let missing = temp.path().join("no-neural-api.sock");

    let err = run_stop(Some(missing), Some("fam".to_string()))
        .await
        .expect_err("stop should fail");
    assert!(
        err.to_string().contains("socket not found")
            || err.to_string().contains("Failed to connect"),
        "expected connect error: {err}"
    );
}

#[tokio::test]
async fn test_run_stop_success() {
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("neural-api-stop.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "shutdown": "initiated" })).await;

    run_stop(Some(sock), Some("fam".to_string()))
        .await
        .expect("stop should succeed");
}

#[tokio::test]
async fn test_run_status_success() {
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("neural-api-status.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        json!({
            "primals": [{ "name": "beardog", "state": "active" }],
            "count": 1,
            "healthy": 1
        }),
    )
    .await;

    let summary = run_status(Some(sock), Some("fam".to_string()))
        .await
        .expect("status should succeed");
    assert_eq!(summary.count, 1);
    assert_eq!(summary.healthy, 1);
    assert_eq!(summary.primal_names, vec!["beardog"]);
}

#[tokio::test]
async fn test_run_status_empty_nucleus() {
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("neural-api-empty.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        json!({ "primals": [], "count": 0, "healthy": 0 }),
    )
    .await;

    let summary = run_status(Some(sock), None)
        .await
        .expect("empty status should succeed");
    assert_eq!(summary.count, 0);
    assert_eq!(summary.healthy, 0);
    assert!(summary.primal_names.is_empty());
}

#[tokio::test]
async fn test_run_deploy_missing_spore_file() {
    let temp = tempfile::tempdir().expect("temp dir");
    let missing = temp.path().join("missing-spore.toml");

    let err = run_deploy(missing, None, None)
        .await
        .expect_err("missing file");
    assert!(
        err.to_string().contains("Spore file not found"),
        "expected not found: {err}"
    );
}

#[tokio::test]
async fn test_run_deploy_invalid_manifest() {
    let temp = tempfile::tempdir().expect("temp dir");
    let bad = temp.path().join("bad-spore.toml");
    std::fs::write(&bad, "not valid [[[").expect("write");

    let err = run_deploy(bad, None, None)
        .await
        .expect_err("invalid manifest");
    assert!(
        err.to_string().contains("Invalid spore manifest"),
        "expected parse error: {err}"
    );
}

#[tokio::test]
async fn test_run_deploy_success() {
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    let temp = tempfile::tempdir().expect("temp dir");
    let spore = temp.path().join("spore.toml");
    std::fs::write(&spore, valid_spore_manifest_toml()).expect("write spore");

    let sock = temp.path().join("neural-api-deploy.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "execution_id": "exec-1" })).await;

    run_deploy(spore, Some(sock), Some("deploy-fam".to_string()))
        .await
        .expect("deploy should succeed");
}

#[tokio::test]
async fn test_run_undeploy_missing_primal_name() {
    let err = run_undeploy(None, None, None)
        .await
        .expect_err("missing name");
    assert!(
        err.to_string().contains("Primal name required"),
        "expected name error: {err}"
    );
}

#[tokio::test]
async fn test_run_undeploy_success() {
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("neural-api-undeploy.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        json!({ "initiated": "beardog", "state": "apoptosis" }),
    )
    .await;

    run_undeploy(
        Some("beardog".to_string()),
        Some(sock),
        Some("fam".to_string()),
    )
    .await
    .expect("undeploy should succeed");
}

#[tokio::test]
async fn test_send_lifecycle_rpc_malformed_response() {
    use std::sync::Arc;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::Notify;

    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("bad-json.sock");
    let ready = Arc::new(Notify::new());
    let ready_c = Arc::clone(&ready);
    let path = sock.clone();

    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&path).expect("bind");
        ready_c.notify_one();
        if let Ok((stream, _)) = listener.accept().await {
            let (mut r, mut w) = stream.into_split();
            let mut line = String::new();
            let _ = BufReader::new(&mut r).read_line(&mut line).await;
            let _ = w.write_all(b"NOT VALID JSON\n").await;
        }
    });

    ready.notified().await;
    let err = send_lifecycle_rpc(&sock, "lifecycle.status", serde_json::json!({}))
        .await
        .expect_err("malformed JSON should fail");
    server.abort();

    assert!(
        err.to_string().contains("Invalid JSON-RPC response"),
        "expected parse error: {err}"
    );
}

#[tokio::test]
async fn test_send_lifecycle_rpc_jsonrpc_error_response() {
    use biomeos_test_utils::MockJsonRpcServer;

    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("rpc-error.sock");
    let _server = MockJsonRpcServer::spawn_echo_error(&sock, -32603, "Internal error").await;

    let err = send_lifecycle_rpc(&sock, "lifecycle.start", serde_json::json!({}))
        .await
        .expect_err("RPC error should fail");
    assert!(
        err.to_string().contains("JSON-RPC error"),
        "expected RPC error: {err}"
    );
}

#[tokio::test]
async fn test_run_start_socket_discovery_isolation() {
    use biomeos_test_utils::MockJsonRpcServer;
    use serde_json::json;

    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let sock = temp.path().join("neural-api-isolated.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "started": true })).await;

    temp_env::async_with_vars([("BIOMEOS_SOCKET_DIR", Some(iso_path))], async {
        run_start(
            Some(sock),
            Some("iso-family".to_string()),
            "full".to_string(),
            "node-iso".to_string(),
        )
        .await
        .expect("start with isolated env should succeed");
    })
    .await;
}

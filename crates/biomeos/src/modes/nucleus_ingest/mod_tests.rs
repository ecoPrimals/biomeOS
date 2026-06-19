// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::{poll_execution, run_emit, run_ingest, send_jsonrpc};
use biomeos_types::JsonRpcRequest;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::sync::{Notify, oneshot};

/// Create a valid pseudoSpore 2.0 directory for ingest tests.
fn create_valid_pseudospore(dir: &Path) {
    std::fs::write(
        dir.join("scope.toml"),
        r#"[artifact]
name = "test-spore-001"
version = "1.0.0"
type = "pseudoSpore"
date = "2026-05-27"
origin = "biomeOS-test"
license = "AGPL-3.0"
"#,
    )
    .unwrap();

    std::fs::write(
        dir.join("validation.json"),
        r#"{"artifact":"test-spore-001","version":"1.0.0","date":"2026-05-27","modules":[{"name":"structural","status":"PASS","checks_total":3,"checks_passed":3}]}"#,
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("receipts")).unwrap();
    std::fs::write(
        dir.join("receipts/environment.toml"),
        "[hardware]\ncpu = \"x86_64\"\ncores = 8\n\n[software]\nos = \"Linux\"\nrust = \"1.82\"\n",
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("data")).unwrap();
    std::fs::write(dir.join("data/payload.bin"), b"hello world").unwrap();

    let hash = biomeos_pseudospore::compute_checksums(dir, &["data"]);
    std::fs::write(
        dir.join("receipts/checksums.blake3"),
        biomeos_pseudospore::format_checksums(&hash),
    )
    .unwrap();

    std::fs::create_dir_all(dir.join("provenance")).unwrap();
    std::fs::write(
        dir.join("provenance/ferment_transcript.json"),
        r#"{"dataset_id":"ds-001","spring":"hotSpring","spring_version":"1.5.0"}"#,
    )
    .unwrap();

    std::fs::write(dir.join("README.md"), "# Test pseudoSpore\n").unwrap();
}

/// Spawn a mock Neural API that returns one fixed JSON-RPC response per accepted connection.
async fn spawn_mock_neural_api(
    responses: Vec<serde_json::Value>,
) -> (PathBuf, tokio::task::JoinHandle<()>) {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("neural-api.sock");
    let path_for_listener = socket_path.clone();
    let (ready_tx, ready_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        let _temp = temp;
        let listener = UnixListener::bind(&path_for_listener).expect("bind mock socket");
        let _ = ready_tx.send(());

        for response in responses {
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = stream.into_split();
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok() {
                    let response_str = serde_json::to_string(&response).expect("serialize") + "\n";
                    let _ = writer.write_all(response_str.as_bytes()).await;
                    let _ = writer.flush().await;
                }
            }
        }
    });

    ready_rx.await.expect("mock server ready");
    (socket_path, handle)
}

/// Spawn a mock server at an explicit path (for socket discovery tests).
async fn spawn_mock_at_path(
    socket_path: PathBuf,
    response: serde_json::Value,
    ready: Arc<Notify>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let listener = UnixListener::bind(&socket_path).expect("bind discovered socket");
        ready.notify_one();
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                let response_str = serde_json::to_string(&response).expect("serialize") + "\n";
                let _ = writer.write_all(response_str.as_bytes()).await;
            }
        }
    })
}

#[tokio::test]
async fn test_run_ingest_dry_run() {
    let dir = tempfile::tempdir().expect("temp dir");
    create_valid_pseudospore(dir.path());

    let result = run_ingest(
        dir.path().to_path_buf(),
        None,
        Some("test-family".to_string()),
        true,
    )
    .await;

    assert!(result.is_ok(), "dry run should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_run_ingest_invalid_pseudospore_dir() {
    let result = run_ingest(
        PathBuf::from("/nonexistent/pseudospore-dir-xyz"),
        None,
        Some("test-family".to_string()),
        false,
    )
    .await;

    let err = result.expect_err("missing dir should fail");
    assert!(
        err.to_string().contains("not a directory"),
        "expected directory error: {err}"
    );
}

#[tokio::test]
async fn test_run_ingest_missing_socket() {
    let dir = tempfile::tempdir().expect("temp dir");
    create_valid_pseudospore(dir.path());

    let result = run_ingest(
        dir.path().to_path_buf(),
        Some(PathBuf::from("/tmp/nonexistent-neural-api-ingest.sock")),
        Some("test-family".to_string()),
        false,
    )
    .await;

    let err = result.expect_err("missing socket should fail");
    assert!(
        err.to_string().contains("Neural API") || err.to_string().contains("connect"),
        "expected connection error: {err}"
    );
}

#[tokio::test]
async fn test_run_ingest_success_writes_receipt() {
    let dir = tempfile::tempdir().expect("temp dir");
    create_valid_pseudospore(dir.path());

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "receipt": {
                "store_id": "store-ingest-001",
                "dag_session_id": "dag-001",
                "ledger_entry_id": "ledger-001",
                "braid_id": "braid-001",
                "signature": "sig-ingest"
            }
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![response]).await;

    let result = run_ingest(
        dir.path().to_path_buf(),
        Some(socket),
        Some("ingest-family".to_string()),
        false,
    )
    .await;

    server.abort();
    result.expect("ingest with mock socket should succeed");

    let receipt_path = dir.path().join("receipts/nucleus_ingest.toml");
    assert!(receipt_path.exists(), "ingest receipt should be written");
    let content = std::fs::read_to_string(receipt_path).unwrap();
    assert!(content.contains("store_id = \"store-ingest-001\""));
    assert!(content.contains("family_id = \"ingest-family\""));
}

#[tokio::test]
async fn test_run_ingest_jsonrpc_error_response() {
    let dir = tempfile::tempdir().expect("temp dir");
    create_valid_pseudospore(dir.path());

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": -32000, "message": "nest ingest failed" }
    });

    let (socket, server) = spawn_mock_neural_api(vec![response]).await;

    let result = run_ingest(
        dir.path().to_path_buf(),
        Some(socket),
        Some("test-family".to_string()),
        false,
    )
    .await;

    server.abort();
    let err = result.expect_err("jsonrpc error should fail ingest");
    assert!(
        err.to_string().contains("NUCLEUS ingest failed"),
        "expected ingest failure: {err}"
    );
}

#[tokio::test]
async fn test_run_ingest_uses_discovered_socket() {
    let dir = tempfile::tempdir().expect("temp dir");
    create_valid_pseudospore(dir.path());
    let iso_path = dir.path().to_str().expect("utf8 path");
    let family = "discovered-family";
    let socket_path = dir.path().join(format!("neural-api-{family}.sock"));

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "receipt": { "store_id": "from-discovered-socket" }
        }
    });

    let ready = Arc::new(Notify::new());
    let server = spawn_mock_at_path(socket_path, response, Arc::clone(&ready)).await;

    temp_env::async_with_vars([("BIOMEOS_SOCKET_DIR", Some(iso_path))], async {
        ready.notified().await;
        let result = run_ingest(
            dir.path().to_path_buf(),
            None,
            Some(family.to_string()),
            false,
        )
        .await;
        assert!(
            result.is_ok(),
            "discovered socket ingest failed: {:?}",
            result.err()
        );

        let receipt = std::fs::read_to_string(dir.path().join("receipts/nucleus_ingest.toml"))
            .expect("receipt");
        assert!(receipt.contains("store_id = \"from-discovered-socket\""));
    })
    .await;

    server.abort();
}

#[tokio::test]
async fn test_run_emit_dry_run() {
    let result = run_emit(
        "spore-dry-run".to_string(),
        None,
        None,
        Some("emit-family".to_string()),
        true,
    )
    .await;

    assert!(
        result.is_ok(),
        "emit dry run should succeed: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_run_emit_missing_socket() {
    let result = run_emit(
        "spore-001".to_string(),
        None,
        Some(PathBuf::from("/tmp/nonexistent-neural-api-emit.sock")),
        Some("test-family".to_string()),
        false,
    )
    .await;

    let err = result.expect_err("missing socket should fail emit");
    assert!(
        err.to_string().contains("Neural API") || err.to_string().contains("connect"),
        "expected connection error: {err}"
    );
}

#[tokio::test]
async fn test_run_emit_no_result_in_response() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": -1, "message": "dispatch failed" }
    });

    let (socket, server) = spawn_mock_neural_api(vec![response]).await;
    let out = tempfile::tempdir().expect("temp dir");

    let result = run_emit(
        "spore-no-result".to_string(),
        Some(out.path().to_path_buf()),
        Some(socket),
        Some("test-family".to_string()),
        false,
    )
    .await;

    server.abort();
    let err = result.expect_err("missing result should fail");
    assert!(
        err.to_string().contains("No result"),
        "expected no-result error: {err}"
    );
}

#[tokio::test]
async fn test_run_emit_success_without_polling() {
    let dispatch_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "state": "completed",
            "nodes": {
                "retrieve_content": {
                    "result": { "data": "emit-content", "spore_id": "spore-direct" }
                },
                "resolve_braid": { "result": { "braid_id": "braid-direct" } },
                "sign_emission": { "result": { "signature": "sig-direct" } }
            }
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![dispatch_response]).await;
    let out = tempfile::tempdir().expect("temp dir");

    let result = run_emit(
        "spore-direct".to_string(),
        Some(out.path().to_path_buf()),
        Some(socket),
        Some("emit-family".to_string()),
        false,
    )
    .await;

    server.abort();
    result.expect("emit without polling should succeed");

    let emit_dir = out.path().join("spore_spore-direct");
    assert!(emit_dir.join("scope.toml").exists());
    assert!(emit_dir.join("emit_manifest.json").exists());
    assert!(emit_dir.join("receipts/nucleus_emit.toml").exists());

    let receipt = std::fs::read_to_string(emit_dir.join("receipts/nucleus_emit.toml")).unwrap();
    assert!(receipt.contains("braid_id = \"braid-direct\""));
    assert!(receipt.contains("signature = \"sig-direct\""));
}

#[tokio::test]
async fn test_run_emit_success_with_polling() {
    let dispatch_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "execution": { "execution_id": "exec-emit-poll-001" },
            "signal": "nest.emit_spore"
        }
    });
    let status_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "state": "completed",
            "execution_id": "exec-emit-poll-001",
            "nodes": {
                "retrieve_content": {
                    "result": { "data": "polled-content", "spore_id": "spore-polled" }
                },
                "resolve_braid": { "result": { "braid_id": "braid-polled" } },
                "sign_emission": { "result": { "signature": "sig-polled" } }
            }
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![dispatch_response, status_response]).await;
    let out = tempfile::tempdir().expect("temp dir");

    let result = run_emit(
        "spore-polled".to_string(),
        Some(out.path().to_path_buf()),
        Some(socket),
        Some("poll-family".to_string()),
        false,
    )
    .await;

    server.abort();
    result.expect("emit with polling should succeed");

    let emit_dir = out.path().join("spore_spore-polled");
    let content = std::fs::read_to_string(emit_dir.join("data/content.json")).unwrap();
    assert!(content.contains("polled-content"));
}

#[tokio::test]
async fn test_run_emit_execution_failed_during_poll() {
    let dispatch_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "execution_id": "exec-fail-001",
            "signal": "nest.emit_spore"
        }
    });
    let status_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "result": {
            "state": "failed",
            "error": "materialization node crashed"
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![dispatch_response, status_response]).await;
    let out = tempfile::tempdir().expect("temp dir");

    let result = run_emit(
        "spore-fail".to_string(),
        Some(out.path().to_path_buf()),
        Some(socket),
        Some("test-family".to_string()),
        false,
    )
    .await;

    server.abort();
    let err = result.expect_err("failed execution should fail emit");
    assert!(
        err.to_string().contains("failed"),
        "expected execution failure: {err}"
    );
}

#[tokio::test]
async fn test_send_jsonrpc_connection_failure() {
    let request = JsonRpcRequest::new("health.check", serde_json::json!({}));
    let result = send_jsonrpc(
        Path::new("/tmp/nonexistent-neural-api-send-jsonrpc.sock"),
        &request,
    )
    .await;

    let err = result.expect_err("connection to missing socket should fail");
    assert!(
        err.to_string().contains("Neural API") || err.to_string().contains("connect"),
        "expected connection error: {err}"
    );
}

#[tokio::test]
async fn test_send_jsonrpc_invalid_json_response() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("invalid-json.sock");
    let sock = socket_path.clone();
    let (ready_tx, ready_rx) = oneshot::channel();

    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock).expect("bind");
        let _ = ready_tx.send(());
        if let Ok((stream, _)) = listener.accept().await {
            let (_, mut writer) = stream.into_split();
            let _ = writer.write_all(b"not valid json\n").await;
        }
    });

    ready_rx.await.expect("server ready");

    let request = JsonRpcRequest::new("test.method", serde_json::json!({}));
    let result = send_jsonrpc(&socket_path, &request).await;
    server.abort();

    let err = result.expect_err("invalid JSON should fail");
    assert!(
        err.to_string().contains("Invalid JSON-RPC response"),
        "expected parse error: {err}"
    );
}

#[tokio::test]
async fn test_send_jsonrpc_success() {
    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 42,
        "result": { "ok": true }
    });

    let (socket, server) = spawn_mock_neural_api(vec![response]).await;
    let request = JsonRpcRequest::new("signal.dispatch", serde_json::json!({"signal": "test"}));

    let result = send_jsonrpc(&socket, &request).await;
    server.abort();

    let value = result.expect("send_jsonrpc should succeed");
    assert_eq!(
        value.get("result").and_then(|v| v.get("ok")),
        Some(&serde_json::json!(true))
    );
}

#[tokio::test]
async fn test_poll_execution_completed() {
    let status_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "state": "completed",
            "execution_id": "exec-poll-done",
            "nodes": { "retrieve_content": { "result": { "data": "x" } } }
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![status_response]).await;

    let status = poll_execution(&socket, "exec-poll-done")
        .await
        .expect("poll should return completed status");

    server.abort();
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("completed")
    );
}

#[tokio::test]
async fn test_poll_execution_failed() {
    let status_response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "state": "failed",
            "error": "node timeout"
        }
    });

    let (socket, server) = spawn_mock_neural_api(vec![status_response]).await;

    let result = poll_execution(&socket, "exec-poll-fail").await;
    server.abort();

    let err = result.expect_err("failed state should bail");
    assert!(
        err.to_string().contains("failed"),
        "expected failure message: {err}"
    );
}

#[tokio::test]
async fn test_poll_execution_retries_on_transient_socket_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("retry-poll.sock");
    let sock = socket_path.clone();
    let (ready_tx, ready_rx) = oneshot::channel();

    let server = tokio::spawn(async move {
        let listener = UnixListener::bind(&sock).expect("bind");
        let _ = ready_tx.send(());

        // First accept: drop connection without responding (client gets EOF / parse issues on retry path)
        if let Ok((stream, _)) = listener.accept().await {
            drop(stream);
        }

        // Second accept: return completed status
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_ok() {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "result": { "state": "completed", "execution_id": "exec-retry" }
                });
                let _ = writer.write_all(format!("{response}\n").as_bytes()).await;
            }
        }
    });

    ready_rx.await.expect("server ready");

    // First poll attempt hits dropped stream; poll_execution logs and retries with backoff.
    let status = poll_execution(&socket_path, "exec-retry")
        .await
        .expect("poll should eventually succeed after retry");

    server.abort();
    assert_eq!(
        status.get("state").and_then(|v| v.as_str()),
        Some("completed")
    );
}

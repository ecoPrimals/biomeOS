// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::PathBuf;
use std::sync::Arc;

use super::common::{create_valid_pseudospore, spawn_mock_at_path, spawn_mock_neural_api};
use super::super::run_ingest;

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

    let ready = Arc::new(tokio::sync::Notify::new());
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

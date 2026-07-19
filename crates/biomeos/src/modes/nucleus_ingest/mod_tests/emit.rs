// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::PathBuf;

use super::super::run_emit;
use super::common::spawn_mock_neural_api;

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

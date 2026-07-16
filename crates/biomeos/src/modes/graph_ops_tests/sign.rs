// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::PathBuf;

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::super::*;

#[tokio::test]
async fn test_sign_fails_when_path_missing() {
    let path = PathBuf::from("/nonexistent/graph/path/missing.toml");
    let result = sign(path).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Cannot read graph"),
        "expected read error: {err}"
    );
}

#[tokio::test]
async fn test_sign_fails_when_neural_api_socket_missing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let graph_path = temp.path().join("unsigned.toml");
    std::fs::write(&graph_path, "[graph]\nid = \"test\"\n").expect("write graph");

    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
            ("NEURAL_API_SOCKET", None::<&str>),
        ],
        async {
            let result = sign(graph_path).await;
            assert!(result.is_err(), "sign should fail without socket");
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("Neural API socket not found")
                    || err.to_string().contains("NUCLEUS"),
                "expected socket discovery error: {err}"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_sign_succeeds_with_invalid_toml_content() {
    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let graph_path = temp.path().join("broken.toml");
    std::fs::write(&graph_path, "[graph\nid = {{{not valid toml").expect("write graph");
    let sock = temp.path().join("neural-api-mock.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        json!({
            "signature": "deadbeef".repeat(16),
            "public_key": "0123456789abcdef".repeat(4),
        }),
    )
    .await;

    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
            ("NEURAL_API_SOCKET", Some(sock.to_str().expect("utf8"))),
        ],
        async {
            let result = sign(graph_path.clone()).await;
            assert!(
                result.is_ok(),
                "sign should not parse TOML; invalid content is still signable: {:?}",
                result.err()
            );
            let signed = std::fs::read_to_string(&graph_path).expect("read signed");
            assert!(signed.contains("content_hash = \""));
            assert!(signed.contains("signature = \""));
            assert!(signed.contains("signed_by = \""));
        },
    )
    .await;
}

#[tokio::test]
async fn test_sign_succeeds_with_valid_graph() {
    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let graph_path = temp.path().join("valid.toml");
    let content = "[graph]\nid = \"rootpulse_commit\"\n";
    std::fs::write(&graph_path, content).expect("write graph");
    let expected_hash = biomeos_graph::integrity::compute_content_hash(content);

    let sock = temp.path().join("neural-api-mock.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        json!({
            "signature": "aa".repeat(64),
            "public_key": "bb".repeat(32),
        }),
    )
    .await;

    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
            ("NEURAL_API_SOCKET", Some(sock.to_str().expect("utf8"))),
        ],
        async {
            let result = sign(graph_path.clone()).await;
            assert!(result.is_ok(), "sign failed: {:?}", result.err());

            let signed = std::fs::read_to_string(&graph_path).expect("read signed");
            assert!(signed.contains(&format!("content_hash = \"{expected_hash}\"")));
            assert!(signed.contains(&format!("signature = \"{}\"", "aa".repeat(64))));
            assert!(signed.contains(&format!("signed_by = \"{}\"", "bb".repeat(32))));
        },
    )
    .await;
}

#[tokio::test]
async fn test_sign_fails_when_crypto_sign_rpc_errors() {
    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let graph_path = temp.path().join("graph.toml");
    std::fs::write(&graph_path, "[graph]\nid = \"test\"\n").expect("write graph");
    let sock = temp.path().join("neural-api-mock.sock");
    let _server = MockJsonRpcServer::spawn_echo_error(&sock, -32000, "signing unavailable").await;

    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
            ("NEURAL_API_SOCKET", Some(sock.to_str().expect("utf8"))),
        ],
        async {
            let result = sign(graph_path).await;
            assert!(result.is_err());
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("crypto.sign RPC failed")
                    || err.to_string().contains("signing unavailable"),
                "expected RPC error: {err}"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn test_sign_fails_when_response_missing_signature_field() {
    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let graph_path = temp.path().join("graph.toml");
    std::fs::write(&graph_path, "[graph]\nid = \"test\"\n").expect("write graph");
    let sock = temp.path().join("neural-api-mock.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "public_key": "abc123" })).await;

    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
            ("NEURAL_API_SOCKET", Some(sock.to_str().expect("utf8"))),
        ],
        async {
            let result = sign(graph_path).await;
            assert!(result.is_err());
            assert!(
                result
                    .unwrap_err()
                    .to_string()
                    .contains("missing 'signature' field")
            );
        },
    )
    .await;
}

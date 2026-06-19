// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use std::path::PathBuf;

use biomeos_graph::integrity::compute_content_hash;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::*;

#[test]
fn test_inject_signing_with_existing_metadata() {
    let toml = "[graph]\nid = \"test\"\n\n[graph.metadata]\nauthor = \"me\"\n";
    let result = inject_signing_metadata(toml, "abc123", "sig456", "pub789");
    assert!(result.contains("content_hash = \"abc123\""));
    assert!(result.contains("signature = \"sig456\""));
    assert!(result.contains("signed_by = \"pub789\""));
    assert!(result.contains("author = \"me\""));
}

#[test]
fn test_inject_signing_without_metadata() {
    let toml = "[graph]\nid = \"test\"\n";
    let result = inject_signing_metadata(toml, "abc", "sig", "pub");
    assert!(result.contains("[graph.metadata]"));
    assert!(result.contains("content_hash = \"abc\""));
}

#[test]
fn test_strip_old_signing_idempotent() {
    let toml = "[graph.metadata]\ncontent_hash = \"old\"\nsignature = \"old\"\nsigned_by = \"old\"\nauthor = \"me\"\n";
    let stripped = strip_old_signing(toml);
    assert!(!stripped.contains("content_hash"));
    assert!(!stripped.contains("signature"));
    assert!(!stripped.contains("signed_by"));
    assert!(stripped.contains("author = \"me\""));
}

#[test]
fn test_extract_signing_metadata() {
    let toml =
        "[graph.metadata]\ncontent_hash = \"abc\"\nsignature = \"def\"\nsigned_by = \"012\"\n";
    let (h, s, p) = extract_signing_metadata(toml);
    assert_eq!(h.unwrap(), "abc");
    assert_eq!(s.unwrap(), "def");
    assert_eq!(p.unwrap(), "012");
}

#[test]
fn test_extract_signing_metadata_none() {
    let toml = "[graph]\nid = \"test\"\n";
    let (h, s, p) = extract_signing_metadata(toml);
    assert!(h.is_none());
    assert!(s.is_none());
    assert!(p.is_none());
}

#[test]
fn test_extract_toml_string_value_malformed() {
    assert!(extract_toml_string_value(" = noquotes").is_none());
    assert!(extract_toml_string_value("= \"unclosed").is_none());
}

// --- sign() ---

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
    let expected_hash = compute_content_hash(content);

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

// --- verify() ---

#[tokio::test]
async fn test_verify_fails_when_path_missing() {
    let path = PathBuf::from("/nonexistent/verify/graph.toml");
    let result = verify(path).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Cannot read graph")
    );
}

#[tokio::test]
async fn test_verify_succeeds_for_unsigned_graph() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("unsigned.toml");
    std::fs::write(&path, "[graph]\nid = \"test\"\n").expect("write graph");

    let result = verify(path).await;
    assert!(
        result.is_ok(),
        "unsigned graph should verify OK: {:?}",
        result.err()
    );
}

#[tokio::test]
async fn test_verify_fails_on_hash_mismatch() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("bad-hash.toml");
    std::fs::write(
        &path,
        "[graph]\nid = \"test\"\n\n[graph.metadata]\ncontent_hash = \"deadbeef\"\n",
    )
    .expect("write graph");

    let result = verify(path).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Content hash mismatch")
    );
}

#[tokio::test]
async fn test_verify_fails_on_invalid_signature() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("bad-sig.toml");
    let content = "[graph]\nid = \"test\"\n\n[graph.metadata]\n";
    let hash = compute_content_hash(content);
    let signed = format!(
        "{content}content_hash = \"{hash}\"\nsignature = \"not-valid-hex-signature\"\nsigned_by = \"also-not-valid-hex\"\n"
    );
    std::fs::write(&path, signed).expect("write graph");

    let result = verify(path).await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Signature verification failed")
    );
}

#[tokio::test]
async fn test_verify_succeeds_with_matching_hash() {
    let temp = tempfile::tempdir().expect("temp dir");
    let path = temp.path().join("good-hash.toml");
    let base = "[graph]\nid = \"test\"\n\n[graph.metadata]\n";
    let hash = compute_content_hash(base);
    std::fs::write(&path, format!("{base}content_hash = \"{hash}\"\n")).expect("write graph");

    let result = verify(path).await;
    assert!(
        result.is_ok(),
        "graph with matching hash should verify: {:?}",
        result.err()
    );
}

// --- execute() ---

#[tokio::test]
async fn test_execute_fails_when_graph_file_missing() {
    let result = execute(
        "/nonexistent/graph/file.toml".to_string(),
        vec![],
        None,
        Some("test-family".to_string()),
        false,
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Cannot read graph")
    );
}

#[tokio::test]
async fn test_execute_fails_on_invalid_toml() {
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_path = temp.path().join("invalid.toml");
    std::fs::write(&graph_path, "[[[broken").expect("write graph");
    let sock = temp.path().join("unused.sock");

    let result = execute(
        graph_path.to_string_lossy().into_owned(),
        vec![],
        Some(sock),
        Some("test-family".to_string()),
        false,
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid TOML"));
}

#[tokio::test]
async fn test_execute_dry_run_skips_rpc() {
    let temp = tempfile::tempdir().expect("temp dir");
    let iso_path = temp.path().to_str().expect("utf8");
    let graph_path = temp.path().join("dry_run.toml");
    std::fs::write(&graph_path, "[graph]\nid = \"explicit_graph_id\"\n").expect("write graph");

    temp_env::async_with_vars(
        [
            ("BIOMEOS_SOCKET_DIR", Some(iso_path)),
            ("XDG_RUNTIME_DIR", Some(iso_path)),
        ],
        async {
            let result = execute(
                graph_path.to_string_lossy().into_owned(),
                vec!["FOO=bar".to_string()],
                None,
                Some("iso-family".to_string()),
                true,
            )
            .await;
            assert!(result.is_ok(), "dry run should succeed: {:?}", result.err());
        },
    )
    .await;
}

#[tokio::test]
async fn test_execute_fails_when_socket_unavailable() {
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_path = temp.path().join("run.toml");
    std::fs::write(&graph_path, "[graph]\nid = \"g1\"\n").expect("write graph");
    let missing_sock = temp.path().join("missing-neural-api.sock");

    let result = execute(
        graph_path.to_string_lossy().into_owned(),
        vec![],
        Some(missing_sock),
        Some("test-family".to_string()),
        false,
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("graph.execute RPC failed")
    );
}

#[tokio::test]
async fn test_execute_fails_on_invalid_param_format() {
    let result = execute(
        "plain_graph_id".to_string(),
        vec!["not-a-key-value-pair".to_string()],
        Some(PathBuf::from("/tmp/unused.sock")),
        Some("fam".to_string()),
        true,
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid param format")
    );
}

#[tokio::test]
async fn test_execute_parses_params_and_resolves_graph_id_from_toml() {
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_path = temp.path().join("with_id.toml");
    std::fs::write(&graph_path, "[graph]\nid = \"from_toml_id\"\n").expect("write graph");
    let sock = temp.path().join("neural-api.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |req| {
        let v: serde_json::Value = serde_json::from_str(req).expect("json");
        assert_eq!(v["method"], "graph.execute");
        let params = &v["params"];
        assert_eq!(params["graph_id"], "from_toml_id");
        assert_eq!(params["params"]["KEY"], "VALUE");
        assert_eq!(params["params"]["FAMILY_ID"], "custom-family");
        let id = v["id"].clone();
        format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"ok":true}}}}"#)
    })
    .await;

    let result = execute(
        graph_path.to_string_lossy().into_owned(),
        vec!["KEY=VALUE".to_string()],
        Some(sock),
        Some("custom-family".to_string()),
        false,
    )
    .await;
    assert!(result.is_ok(), "execute failed: {:?}", result.err());
}

#[tokio::test]
async fn test_execute_uses_file_stem_when_graph_id_missing() {
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_path = temp.path().join("stem_id.toml");
    std::fs::write(&graph_path, "[graph]\nversion = 1\n").expect("write graph");
    let sock = temp.path().join("neural-api.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |req| {
        let v: serde_json::Value = serde_json::from_str(req).expect("json");
        assert_eq!(v["params"]["graph_id"], "stem_id");
        let id = v["id"].clone();
        format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"status":"ok"}}}}"#)
    })
    .await;

    let result = execute(
        graph_path.to_string_lossy().into_owned(),
        vec![],
        Some(sock),
        Some("fam".to_string()),
        false,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_non_toml_graph_id_passed_through() {
    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("neural-api.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |req| {
        let v: serde_json::Value = serde_json::from_str(req).expect("json");
        assert_eq!(v["params"]["graph_id"], "rootpulse_commit");
        let id = v["id"].clone();
        format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{"done":true}}}}"#)
    })
    .await;

    let result = execute(
        "rootpulse_commit".to_string(),
        vec![],
        Some(sock),
        Some("fam".to_string()),
        false,
    )
    .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_execute_fails_when_response_contains_error() {
    let temp = tempfile::tempdir().expect("temp dir");
    let sock = temp.path().join("neural-api.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(
        &sock,
        json!({ "error": "graph not found in registry" }),
    )
    .await;

    let result = execute(
        "missing_graph".to_string(),
        vec![],
        Some(sock),
        Some("fam".to_string()),
        false,
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Graph execution failed")
    );
}

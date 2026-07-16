// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::path::PathBuf;

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

use super::super::*;

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

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use serde_json::json;
use tempfile::tempdir;

// ── graph.execute ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_execute_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler.execute(&None).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_execute_missing_graph_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({}));
    let err = handler.execute(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing graph_id"));
}

#[tokio::test]
async fn test_execute_graph_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "nonexistent"}));
    let err = handler.execute(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Graph file not found"));
}

#[tokio::test]
async fn test_execute_success_returns_immediate_response() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("test_minimal.toml");
    std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
    let (handler, executions) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "test_minimal"}));
    let result = handler.execute(&params).await.expect("execute");

    assert!(
        result["execution_id"]
            .as_str()
            .unwrap()
            .starts_with("test_minimal-")
    );
    assert_eq!(result["graph_id"], "test_minimal");
    assert!(result["started_at"].as_str().is_some());

    let exec_id = result["execution_id"].as_str().expect("execution_id");
    let execs = executions.read().await;
    assert!(execs.contains_key(exec_id));
    let status = execs.get(exec_id).expect("status");
    assert_eq!(status.state, "running");
}

#[tokio::test]
async fn test_execute_with_family_id_param() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("test_minimal.toml");
    std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({
        "graph_id": "test_minimal",
        "family_id": "custom-family"
    }));
    let result = handler.execute(&params).await.expect("execute");
    assert_eq!(result["graph_id"], "test_minimal");
    assert!(
        result["execution_id"]
            .as_str()
            .unwrap()
            .starts_with("test_minimal-")
    );
}

#[tokio::test]
async fn test_execute_uses_handler_family_id_when_param_missing() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("test_minimal.toml");
    std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "test_minimal"}));
    let result = handler.execute(&params).await.expect("execute");
    assert!(result["execution_id"].as_str().is_some());
}

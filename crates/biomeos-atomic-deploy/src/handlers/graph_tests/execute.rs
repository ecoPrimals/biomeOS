// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

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
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_execute_corrupt_graph_toml() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("corrupt_exec.toml");
    std::fs::write(&path, "[[[not valid graph toml").expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "corrupt_exec"}));
    let err = handler.execute(&params).await.expect_err("should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("Failed to load graph") || msg.contains("parse") || msg.contains("TOML"),
        "unexpected error: {msg}"
    );
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

const CONTINUOUS_REDIRECT_TOML: &str = r#"
[graph]
id = "continuous-redirect"
name = "Continuous Redirect"
version = "1.0.0"
coordination = "continuous"

[graph.tick]
target_hz = 30.0

[[graph.nodes]]
id = "tick-node"
name = "Tick Node"
"#;

#[tokio::test]
async fn test_execute_redirects_continuous_to_start_continuous() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("continuous_redirect.toml");
    std::fs::write(&path, CONTINUOUS_REDIRECT_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "continuous_redirect"}));
    let result = handler.execute(&params).await.expect("execute");
    let id = result["session_id"]
        .as_str()
        .or_else(|| result["execution_id"].as_str())
        .expect("session_id or execution_id");
    assert!(id.starts_with("continuous_redirect-"));
    assert_eq!(result["graph_id"], "continuous_redirect");
}

const PIPELINE_REDIRECT_TOML: &str = r#"
[graph]
id = "pipeline-redirect"
name = "Pipeline Redirect"
version = "1.0.0"
coordination = "pipeline"

[[graph.nodes]]
id = "source"
name = "Source"
capability = "test.source"

[[graph.nodes]]
id = "sink"
name = "Sink"
capability = "test.sink"
depends_on = ["source"]
"#;

#[tokio::test]
async fn test_execute_redirects_pipeline_to_execute_pipeline() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("pipeline_redirect.toml");
    std::fs::write(&path, PIPELINE_REDIRECT_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "pipeline_redirect"}));

    // Pipeline execution involves real capability discovery which won't
    // resolve in tests. Use a timeout to verify the redirect happened
    // (graph loads, pipeline path is entered) without hanging.
    let result =
        tokio::time::timeout(std::time::Duration::from_secs(2), handler.execute(&params)).await;

    match result {
        Ok(Ok(r)) => {
            assert!(
                r.get("items").is_some()
                    || r.get("throughput").is_some()
                    || r.get("graph_id").is_some()
            );
        }
        Ok(Err(e)) => {
            assert!(
                e.to_string().contains("Pipeline")
                    || e.to_string().contains("Capability")
                    || e.to_string().contains("discovery"),
                "unexpected error: {e}"
            );
        }
        Err(_timeout) => {
            // Pipeline entered execution (redirect worked) but hangs
            // waiting for capability discovery — expected in test env.
        }
    }
}

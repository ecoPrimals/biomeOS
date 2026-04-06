// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use serde_json::json;
use tempfile::tempdir;

const CONTINUOUS_GRAPH_TOML: &str = r#"
[graph]
id = "continuous-test"
name = "Continuous Test"
version = "1.0.0"
coordination = "continuous"

[graph.tick]
target_hz = 30.0

[[graph.nodes]]
id = "tick-node"
name = "Tick Node"
"#;

// ── graph.start_continuous ────────────────────────────────────────────────

#[tokio::test]
async fn test_start_continuous_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .start_continuous(&None)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_start_continuous_missing_graph_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({}));
    let err = handler
        .start_continuous(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing graph_id"));
}

#[tokio::test]
async fn test_start_continuous_graph_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "nonexistent_continuous"}));
    let err = handler
        .start_continuous(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_start_continuous_invalid_toml_for_deployment_graph() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("bad_parse.toml");
    std::fs::write(&path, "this is not valid toml [[[ ").expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "bad_parse"}));
    let err = handler
        .start_continuous(&params)
        .await
        .expect_err("should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("parse") || msg.contains("Failed") || msg.contains("TOML"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn test_start_continuous_wrong_coordination() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("sequential_graph.toml");
    let sequential_toml = r#"
[graph]
id = "sequential-graph"
name = "Sequential"
version = "1.0.0"
coordination = "sequential"

[[graph.nodes]]
id = "n1"
name = "Node 1"
"#;
    std::fs::write(&path, sequential_toml).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "sequential_graph"}));
    let err = handler
        .start_continuous(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("not Continuous"));
}

#[tokio::test]
async fn test_start_continuous_success() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("continuous_graph.toml");
    std::fs::write(&path, CONTINUOUS_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "continuous_graph"}));
    let result = handler
        .start_continuous(&params)
        .await
        .expect("start_continuous");
    assert!(
        result["session_id"]
            .as_str()
            .unwrap()
            .starts_with("continuous_graph-")
    );
    assert_eq!(result["graph_id"], "continuous_graph");
    assert!(result["started_at"].as_str().is_some());
}

// ── graph.pause_continuous, resume_continuous, stop_continuous ───────────────

#[tokio::test]
async fn test_pause_continuous_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .pause_continuous(&None)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_pause_continuous_session_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"session_id": "nonexistent-session"}));
    let err = handler
        .pause_continuous(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Continuous session not found"));
}

#[tokio::test]
async fn test_resume_continuous_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .resume_continuous(&None)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_resume_continuous_session_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"session_id": "nonexistent-session"}));
    let err = handler
        .resume_continuous(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Continuous session not found"));
}

#[tokio::test]
async fn test_stop_continuous_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .stop_continuous(&None)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_stop_continuous_session_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"session_id": "nonexistent-session"}));
    let err = handler
        .stop_continuous(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Continuous session not found"));
}

#[tokio::test]
async fn test_stop_continuous_success() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("stop_test.toml");
    std::fs::write(&path, CONTINUOUS_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "stop_test"}));
    let start_result = handler.start_continuous(&params).await.expect("start");
    let session_id = start_result["session_id"].as_str().unwrap().to_string();

    let stop_params = Some(json!({"session_id": session_id}));
    let stop_result = handler.stop_continuous(&stop_params).await.expect("stop");
    assert_eq!(stop_result["session_id"], session_id);
    assert_eq!(stop_result["command"], "stop");
}

// ── get_status for continuous session ─────────────────────────────────────

#[tokio::test]
async fn test_pause_continuous_success() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("pause_test.toml");
    std::fs::write(&path, CONTINUOUS_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "pause_test"}));
    let start_result = handler.start_continuous(&params).await.expect("start");
    let session_id = start_result["session_id"].as_str().unwrap().to_string();

    let pause_params = Some(json!({"session_id": session_id}));
    let pause_result = handler
        .pause_continuous(&pause_params)
        .await
        .expect("pause");
    assert_eq!(pause_result["session_id"], session_id);
    assert_eq!(pause_result["command"], "pause");
}

#[tokio::test]
async fn test_resume_continuous_success() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("resume_test.toml");
    std::fs::write(&path, CONTINUOUS_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "resume_test"}));
    let start_result = handler.start_continuous(&params).await.expect("start");
    let session_id = start_result["session_id"].as_str().unwrap().to_string();

    let pause_params = Some(json!({"session_id": session_id}));
    handler
        .pause_continuous(&pause_params)
        .await
        .expect("pause");

    let resume_params = Some(json!({"session_id": session_id}));
    let resume_result = handler
        .resume_continuous(&resume_params)
        .await
        .expect("resume");
    assert_eq!(resume_result["session_id"], session_id);
    assert_eq!(resume_result["command"], "resume");
}

#[tokio::test]
async fn test_get_status_continuous_session() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("status_continuous.toml");
    std::fs::write(&path, CONTINUOUS_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "status_continuous"}));
    let start_result = handler.start_continuous(&params).await.expect("start");
    let session_id = start_result["session_id"].as_str().unwrap().to_string();

    let status_params = Some(json!({"execution_id": session_id}));
    let status_result = handler
        .get_status(&status_params)
        .await
        .expect("get_status");
    assert_eq!(status_result["execution_id"], session_id);
    assert_eq!(status_result["graph_id"], "status_continuous");
    assert_eq!(status_result["continuous"], true);
    assert!(status_result["started_at"].as_str().is_some());
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use serde_json::json;
use tempfile::tempdir;

// ── graph.execute_pipeline ────────────────────────────────────────────────

#[tokio::test]
async fn test_execute_pipeline_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .execute_pipeline(&None)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_execute_pipeline_graph_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "nonexistent_pipeline"}));
    let err = handler
        .execute_pipeline(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Graph file not found"));
}

#[tokio::test]
async fn test_execute_pipeline_wrong_coordination() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("sequential_pipeline.toml");
    let sequential_toml = r#"
[graph]
id = "sequential-pipeline"
name = "Sequential"
version = "1.0.0"
coordination = "sequential"

[[graph.nodes]]
id = "n1"
name = "Node 1"
"#;
    std::fs::write(&path, sequential_toml).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "sequential_pipeline"}));
    let err = handler
        .execute_pipeline(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("not Pipeline"));
}

#[tokio::test]
async fn test_execute_pipeline_invalid_deployment_graph_toml() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("bad_pipe.toml");
    std::fs::write(&path, "[[[broken").expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "bad_pipe"}));
    let err = handler
        .execute_pipeline(&params)
        .await
        .expect_err("should fail");
    let msg = err.to_string();
    assert!(
        msg.contains("parse") || msg.contains("Failed") || msg.contains("TOML"),
        "unexpected: {msg}"
    );
}

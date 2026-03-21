// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use serde_json::json;
use tempfile::tempdir;

// ── graph.suggest_optimizations ───────────────────────────────────────────

#[tokio::test]
async fn test_suggest_optimizations_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .suggest_optimizations(&None)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_suggest_optimizations_graph_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "nonexistent_opt"}));
    let err = handler
        .suggest_optimizations(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Graph file not found"));
}

#[tokio::test]
async fn test_suggest_optimizations_missing_graph_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"min_samples": 3}));
    let err = handler
        .suggest_optimizations(&params)
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("Missing graph_id"));
}

#[serial_test::serial]
#[tokio::test]
async fn test_suggest_optimizations_with_min_samples() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("opt_graph.toml");
    let opt_toml = r#"
[graph]
id = "opt-graph"
name = "Opt Graph"
version = "1.0.0"
coordination = "sequential"

[[graph.nodes]]
id = "n1"
name = "Node 1"
"#;
    std::fs::write(&path, opt_toml).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({
        "graph_id": "opt_graph",
        "min_samples": 5
    }));
    let result = handler
        .suggest_optimizations(&params)
        .await
        .expect("suggest_optimizations");
    assert!(result.get("suggestions").is_some());
    assert!(result.get("sample_size").is_some());
}

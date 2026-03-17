// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use serde_json::json;
use tempfile::tempdir;

// ── GraphHandler constructor ───────────────────────────────────────────

#[tokio::test]
async fn test_graph_handler_creation() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let result = handler.list().await.expect("list");
    assert!(result.is_array());
}

#[tokio::test]
async fn test_graph_handler_clone() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let cloned = handler.clone();
    let result = cloned.list().await.expect("list");
    assert!(result.is_array());
}

// ── graph.list ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_graph_handler_list_empty() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let result = handler.list().await.expect("list");
    assert!(result.as_array().expect("array").is_empty());
}

#[tokio::test]
async fn test_list_with_valid_graphs() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("test_graph.toml");
    std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write graph");
    let (handler, _) = make_handler(temp.path());

    let result = handler.list().await.expect("list");
    let arr = result.as_array().expect("array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], "test_minimal");
    assert_eq!(arr[0]["version"], "1.0.0");
    assert_eq!(arr[0]["node_count"], 1);
}

#[tokio::test]
async fn test_list_skips_invalid_toml() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("invalid.toml");
    std::fs::write(&path, "not valid toml {{{").expect("write");
    let (handler, _) = make_handler(temp.path());

    let result = handler.list().await.expect("list");
    let arr = result.as_array().expect("array");
    assert!(arr.is_empty(), "invalid TOML should be skipped");
}

#[tokio::test]
async fn test_list_skips_non_toml_files() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("other.txt");
    std::fs::write(&path, "hello").expect("write");
    let (handler, _) = make_handler(temp.path());

    let result = handler.list().await.expect("list");
    assert!(result.as_array().expect("array").is_empty());
}

#[tokio::test]
async fn test_list_nonexistent_directory() {
    let temp = tempdir().expect("tempdir");
    let bad_path = temp.path().join("nonexistent_subdir");
    let (handler, _) = make_handler(&bad_path);

    let err = handler.list().await.expect_err("should fail");
    assert!(err.to_string().contains("Failed to read graphs directory"));
}

#[tokio::test]
async fn test_list_empty_graphs_dir_created() {
    let temp = tempdir().expect("tempdir");
    let empty = temp.path().join("empty");
    std::fs::create_dir(&empty).expect("create dir");
    let (handler, _) = make_handler(&empty);

    let result = handler.list().await.expect("list");
    assert!(result.as_array().expect("array").is_empty());
}

#[tokio::test]
async fn test_list_includes_coordination_field() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("coord_graph.toml");
    let toml_with_coord = r#"
[graph]
id = "coord_test"
version = "1.0.0"
description = "With coordination"

[[nodes]]
id = "n1"
depends_on = []
capabilities = []
"#;
    std::fs::write(&path, toml_with_coord).expect("write");
    let (handler, _) = make_handler(temp.path());

    let result = handler.list().await.expect("list");
    let arr = result.as_array().expect("array");
    assert!(!arr.is_empty());
    assert!(arr[0].get("coordination").is_some());
}

// ── graph.get ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_success() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("my_graph.toml");
    std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "my_graph"}));
    let result = handler.get(&params).await.expect("get");
    assert_eq!(result["id"], "test_minimal");
    assert_eq!(result["version"], "1.0.0");
    assert!(result["nodes"].is_array());
}

#[tokio::test]
async fn test_get_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler.get(&None).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_get_missing_graph_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({}));
    let err = handler.get(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing graph_id"));
}

#[tokio::test]
async fn test_get_graph_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "nonexistent_graph"}));
    let err = handler.get(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Failed to load graph"));
}

#[tokio::test]
async fn test_get_graph_id_as_number_fails() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": 12345}));
    let err = handler.get(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing graph_id"));
}

#[tokio::test]
async fn test_get_with_json_rpc_params_object() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("rpc_graph.toml");
    std::fs::write(&path, MINIMAL_GRAPH_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "rpc_graph"}));
    let result = handler.get(&params).await.expect("get");
    assert_eq!(result["id"], "test_minimal");
}

// ── graph.save ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_save_success() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let graph_value = json!({
        "id": "saved_graph",
        "version": "1.0.0",
        "description": "Saved for test",
        "nodes": [],
        "config": {
            "deterministic": true,
            "parallel_phases": true,
            "max_parallelism": 3,
            "timeout_total_ms": 60000,
            "checkpoint_enabled": false,
            "rollback_on_failure": true
        }
    });
    let params = Some(graph_value);

    let result = handler.save(&params).await.expect("save");
    assert_eq!(result["graph_id"], "saved_graph");

    let path = temp.path().join("saved_graph.toml");
    assert!(path.exists(), "graph file should exist");
    let content = std::fs::read_to_string(&path).expect("read");
    assert!(content.contains("saved_graph"));
}

#[tokio::test]
async fn test_save_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler.save(&None).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_save_invalid_graph_structure() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({
        "id": "bad",
        "version": "1.0",
        "nodes": "not an array"
    }));
    let err = handler.save(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Failed to parse graph"));
}

#[tokio::test]
async fn test_save_overwrites_existing() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("overwrite.toml");
    std::fs::write(&path, "old content").expect("write");
    let (handler, _) = make_handler(temp.path());

    let graph_value = json!({
        "id": "overwrite",
        "version": "2.0.0",
        "description": "Updated",
        "nodes": [],
        "config": {
            "deterministic": true,
            "parallel_phases": true,
            "max_parallelism": 3,
            "timeout_total_ms": 60000,
            "checkpoint_enabled": false,
            "rollback_on_failure": true
        }
    });
    handler.save(&Some(graph_value)).await.expect("save");

    let content = std::fs::read_to_string(&path).expect("read");
    assert!(content.contains("2.0.0"));
    assert!(content.contains("Updated"));
}

#[tokio::test]
async fn test_save_persists_file() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let graph_value = json!({
        "id": "roundtrip_graph",
        "version": "1.0.0",
        "description": "Roundtrip test",
        "nodes": [{
            "id": "node1",
            "depends_on": [],
            "capabilities": []
        }],
        "config": {
            "deterministic": true,
            "parallel_phases": true,
            "max_parallelism": 3,
            "timeout_total_ms": 60000,
            "checkpoint_enabled": false,
            "rollback_on_failure": true
        }
    });
    handler.save(&Some(graph_value)).await.expect("save");

    let path = temp.path().join("roundtrip_graph.toml");
    assert!(path.exists(), "saved graph file should exist");
    let content = std::fs::read_to_string(&path).expect("read file");
    assert!(content.contains("roundtrip_graph"));
    assert!(content.contains("1.0.0"));
    assert!(content.contains("node1"));
}

// ── graph.status ───────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_status_success() {
    let temp = tempdir().expect("tempdir");
    let (handler, executions) = make_handler(temp.path());

    let status = ExecutionStatus {
        execution_id: "exec-123".to_string(),
        state: "running".to_string(),
        current_phase: Some(1),
        total_phases: 3,
        completed_nodes: vec!["n1".to_string()],
        failed_nodes: Vec::new(),
        duration_ms: 100,
        error: None,
    };
    executions
        .write()
        .await
        .insert("exec-123".to_string(), status);

    let params = Some(json!({"execution_id": "exec-123"}));
    let result = handler.get_status(&params).await.expect("get_status");
    assert_eq!(result["execution_id"], "exec-123");
    assert_eq!(result["state"], "running");
    assert_eq!(result["current_phase"], 1);
    assert_eq!(result["total_phases"], 3);
    assert_eq!(result["completed_nodes"], json!(["n1"]));
}

#[tokio::test]
async fn test_get_status_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler.get_status(&None).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_get_status_missing_execution_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({}));
    let err = handler.get_status(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing execution_id"));
}

#[tokio::test]
async fn test_get_status_execution_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"execution_id": "nonexistent-exec"}));
    let err = handler.get_status(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Execution not found"));
}

#[tokio::test]
async fn test_get_status_completed_with_error_field() {
    let temp = tempdir().expect("tempdir");
    let (handler, executions) = make_handler(temp.path());

    let status = ExecutionStatus {
        execution_id: "exec-failed".to_string(),
        state: "failed".to_string(),
        current_phase: Some(2),
        total_phases: 3,
        completed_nodes: vec!["a".to_string(), "b".to_string()],
        failed_nodes: vec!["c".to_string()],
        duration_ms: 500,
        error: Some("Node c failed".to_string()),
    };
    executions
        .write()
        .await
        .insert("exec-failed".to_string(), status);

    let params = Some(json!({"execution_id": "exec-failed"}));
    let result = handler.get_status(&params).await.expect("get_status");
    assert_eq!(result["state"], "failed");
    assert_eq!(result["error"], "Node c failed");
}

#[tokio::test]
async fn test_get_status_with_non_string_execution_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"execution_id": 12345}));
    let err = handler.get_status(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing execution_id"));
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unit tests for graph handlers (graph.list, graph.get, graph.save, graph.execute, graph.status).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::graph::{ExecutionStatus, GraphHandler};
use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tempfile::tempdir;
use tokio::sync::RwLock;

/// Helper to create a minimal GraphHandler for tests.
fn make_handler(
    graphs_dir: &std::path::Path,
) -> (GraphHandler, Arc<RwLock<HashMap<String, ExecutionStatus>>>) {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    let executions = Arc::new(RwLock::new(HashMap::new()));
    let handler = GraphHandler::new(
        graphs_dir,
        "test-family",
        executions.clone(),
        router,
        registry,
    );
    (handler, executions)
}

/// Minimal valid graph TOML for execute tests (log.info completes quickly).
const MINIMAL_GRAPH_TOML: &str = r#"
[graph]
id = "test_minimal"
version = "1.0.0"
description = "Minimal graph for tests"

[[nodes]]
id = "log1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "test execution"
"#;

// ── ExecutionStatus tests ─────────────────────────────────────────────

#[test]
fn test_execution_status_construction() {
    let status = ExecutionStatus {
        execution_id: "graph-123".to_string(),
        state: "running".to_string(),
        current_phase: Some(1),
        total_phases: 3,
        completed_nodes: vec!["node1".to_string()],
        failed_nodes: Vec::new(),
        duration_ms: 100,
        error: None,
    };
    assert_eq!(status.execution_id, "graph-123");
    assert_eq!(status.state, "running");
    assert_eq!(status.current_phase, Some(1));
    assert_eq!(status.total_phases, 3);
    assert_eq!(status.completed_nodes, vec!["node1"]);
    assert!(status.failed_nodes.is_empty());
    assert_eq!(status.duration_ms, 100);
    assert!(status.error.is_none());
}

#[test]
fn test_execution_status_with_error() {
    let status = ExecutionStatus {
        execution_id: "graph-456".to_string(),
        state: "failed".to_string(),
        current_phase: Some(2),
        total_phases: 3,
        completed_nodes: vec!["node1".to_string(), "node2".to_string()],
        failed_nodes: vec!["node3".to_string()],
        duration_ms: 500,
        error: Some("Node execution failed".to_string()),
    };
    assert_eq!(status.state, "failed");
    assert_eq!(status.failed_nodes, vec!["node3"]);
    assert_eq!(status.error.as_deref(), Some("Node execution failed"));
}

#[test]
fn test_execution_status_serialization_roundtrip() {
    let status = ExecutionStatus {
        execution_id: "exec-789".to_string(),
        state: "completed".to_string(),
        current_phase: Some(3),
        total_phases: 3,
        completed_nodes: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        failed_nodes: Vec::new(),
        duration_ms: 1234,
        error: None,
    };
    let json = serde_json::to_value(&status).expect("serialize");
    let restored: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
    assert_eq!(restored.execution_id, status.execution_id);
    assert_eq!(restored.state, status.state);
    assert_eq!(restored.completed_nodes, status.completed_nodes);
    assert_eq!(restored.duration_ms, status.duration_ms);
}

#[test]
fn test_execution_status_deserialize_from_json() {
    let json = json!({
        "execution_id": "test-123",
        "state": "running",
        "current_phase": 0,
        "total_phases": 2,
        "completed_nodes": [],
        "failed_nodes": [],
        "duration_ms": 0,
        "error": null
    });
    let status: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
    assert_eq!(status.execution_id, "test-123");
    assert_eq!(status.state, "running");
    assert_eq!(status.current_phase, Some(0));
    assert_eq!(status.total_phases, 2);
}

#[test]
fn test_execution_status_deserialize_with_optional_error() {
    let json = json!({
        "execution_id": "fail-1",
        "state": "failed",
        "current_phase": null,
        "total_phases": 1,
        "completed_nodes": [],
        "failed_nodes": ["n1"],
        "duration_ms": 50,
        "error": "Something went wrong"
    });
    let status: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
    assert_eq!(status.error, Some("Something went wrong".to_string()));
    assert_eq!(status.current_phase, None);
}

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

    assert!(result["execution_id"]
        .as_str()
        .unwrap()
        .starts_with("test_minimal-"));
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
    assert!(result["execution_id"]
        .as_str()
        .unwrap()
        .starts_with("test_minimal-"));
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

// ── JSON-RPC style params ─────────────────────────────────────────────

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

// ── Edge cases ────────────────────────────────────────────────────────

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
async fn test_get_status_with_non_string_execution_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"execution_id": 12345}));
    let err = handler.get_status(&params).await.expect_err("should fail");
    assert!(err.to_string().contains("Missing execution_id"));
}

// ── graph.start_continuous ────────────────────────────────────────────────

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
    assert!(err.to_string().contains("Graph file not found"));
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
    assert!(result["session_id"]
        .as_str()
        .unwrap()
        .starts_with("continuous_graph-"));
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

// ── get_status for continuous session ─────────────────────────────────────

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

// ── resolve_primal_name (pure logic) ────────────────────────────────────────

fn make_node(
    id: &str,
    primal_by_name: Option<&str>,
    primal_by_capability: Option<&str>,
) -> crate::neural_graph::GraphNode {
    use crate::neural_graph::{GraphNode, PrimalSelector};

    let primal = match (primal_by_name, primal_by_capability) {
        (Some(name), cap) => Some(PrimalSelector {
            by_name: Some(name.to_string()),
            by_capability: cap.map(String::from),
        }),
        (None, Some(cap)) => Some(PrimalSelector {
            by_name: None,
            by_capability: Some(cap.to_string()),
        }),
        _ => None,
    };
    GraphNode {
        id: id.to_string(),
        primal,
        depends_on: vec![],
        operation: None,
        output: None,
        constraints: None,
        capabilities: vec![],
        capabilities_provided: None,
        parameter_mappings: None,
        node_type: None,
        dependencies: vec![],
        config: HashMap::new(),
        outputs: vec![],
    }
}

#[test]
fn test_resolve_primal_name_from_by_name() {
    use crate::handlers::graph::GraphHandler;

    let node = make_node("node-id", Some("beardog"), None);
    assert_eq!(GraphHandler::resolve_primal_name(&node), "beardog");
}

#[test]
fn test_resolve_primal_name_fallback_to_node_id() {
    use crate::handlers::graph::GraphHandler;

    let node = make_node("songbird", None, None);
    assert_eq!(GraphHandler::resolve_primal_name(&node), "songbird");
}

#[test]
fn test_resolve_primal_name_by_capability_only_uses_node_id() {
    use crate::handlers::graph::GraphHandler;

    let node = make_node("security-node", None, Some("security"));
    assert_eq!(GraphHandler::resolve_primal_name(&node), "security-node");
}

// ── extract_session_id (pure logic) ────────────────────────────────────────

#[test]
fn test_extract_session_id_success() {
    use crate::handlers::graph::GraphHandler;

    let params = Some(json!({"session_id": "session-abc-123"}));
    let result = GraphHandler::extract_session_id(&params).expect("extract");
    assert_eq!(result, "session-abc-123");
}

#[test]
fn test_extract_session_id_missing_params() {
    use crate::handlers::graph::GraphHandler;

    let err = GraphHandler::extract_session_id(&None).expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[test]
fn test_extract_session_id_missing_session_id() {
    use crate::handlers::graph::GraphHandler;

    let params = Some(json!({}));
    let err = GraphHandler::extract_session_id(&params).expect_err("should fail");
    assert!(err.to_string().contains("Missing session_id"));
}

// ── list coordination field ────────────────────────────────────────────────

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

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for executor introspection handlers.

use super::executor::ExecutorHandler;
use super::graph::ExecutionStatus;
use crate::handlers::graph_tests::make_handler;
use serde_json::json;
use std::time::Instant;

#[tokio::test]
async fn executor_list_returns_all_executor_types() {
    let temp = tempfile::tempdir().expect("temp dir");
    let (handler, _executions) = make_handler(temp.path());

    let result = ExecutorHandler::new(&handler, Instant::now())
        .list()
        .await
        .expect("list");

    let executors = result["executors"]
        .as_array()
        .expect("executors array");
    assert_eq!(executors.len(), 3);
    assert_eq!(result["count"], 3);

    let names: Vec<&str> = executors
        .iter()
        .filter_map(|e| e["name"].as_str())
        .collect();
    assert!(names.contains(&"continuous"));
    assert!(names.contains(&"pipeline"));
    assert!(names.contains(&"single-shot"));
}

#[tokio::test]
async fn executor_list_reflects_active_single_shot_sessions() {
    let temp = tempfile::tempdir().expect("temp dir");
    let (handler, executions) = make_handler(temp.path());

    executions.write().await.insert(
        "exec-1".to_string(),
        ExecutionStatus {
            execution_id: "exec-1".to_string(),
            state: "running".to_string(),
            current_phase: Some(0),
            total_phases: 1,
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            duration_ms: 0,
            error: None,
        },
    );

    let result = ExecutorHandler::new(&handler, Instant::now())
        .list()
        .await
        .expect("list");

    let single_shot = result["executors"]
        .as_array()
        .expect("executors")
        .iter()
        .find(|e| e["name"] == "single-shot")
        .expect("single-shot entry");

    assert_eq!(single_shot["active_sessions"], 1);
    assert_eq!(single_shot["status"], "active");
}

#[tokio::test]
async fn executor_status_returns_all_executors_by_default() {
    let temp = tempfile::tempdir().expect("temp dir");
    let (handler, executions) = make_handler(temp.path());

    executions.write().await.insert(
        "exec-done".to_string(),
        ExecutionStatus {
            execution_id: "exec-done".to_string(),
            state: "completed".to_string(),
            current_phase: Some(1),
            total_phases: 1,
            completed_nodes: vec!["node1".to_string()],
            failed_nodes: Vec::new(),
            duration_ms: 42,
            error: None,
        },
    );
    executions.write().await.insert(
        "exec-fail".to_string(),
        ExecutionStatus {
            execution_id: "exec-fail".to_string(),
            state: "failed".to_string(),
            current_phase: Some(0),
            total_phases: 1,
            completed_nodes: Vec::new(),
            failed_nodes: vec!["node1".to_string()],
            duration_ms: 10,
            error: Some("boom".to_string()),
        },
    );

    let started_at = Instant::now();
    let result = ExecutorHandler::new(&handler, started_at)
        .status(&None)
        .await
        .expect("status");

    let single_shot = &result["executors"]["single-shot"];
    assert_eq!(single_shot["completed"], 1);
    assert_eq!(single_shot["failed"], 1);
    assert_eq!(single_shot["active"], 0);
    assert!(result["uptime_s"].as_u64().is_some());

    let pipeline = &result["executors"]["pipeline"];
    assert_eq!(pipeline["active_sessions"], 0);
    assert_eq!(pipeline["status"], "idle");
}

#[tokio::test]
async fn executor_status_filters_by_executor_type() {
    let temp = tempfile::tempdir().expect("temp dir");
    let (handler, _executions) = make_handler(temp.path());

    let result = ExecutorHandler::new(&handler, Instant::now())
        .status(&Some(json!({"executor_type": "continuous"})))
        .await
        .expect("status");

    assert_eq!(result["executor_type"], "continuous");
    assert_eq!(result["executor"]["type"], "continuous");
    assert!(result["executor"]["sessions"].is_array());
}

#[tokio::test]
async fn executor_status_rejects_unknown_executor_type() {
    let temp = tempfile::tempdir().expect("temp dir");
    let (handler, _executions) = make_handler(temp.path());

    let err = ExecutorHandler::new(&handler, Instant::now())
        .status(&Some(json!({"executor_type": "quantum"})))
        .await
        .expect_err("unknown type");

    assert!(err.to_string().contains("Unknown executor_type"));
}

#[tokio::test]
async fn executor_list_route_via_neural_api_server() {
    use crate::neural_api_server::NeuralApiServer;

    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir_all(temp.path()).expect("create graphs dir");
    let server = NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"));
    server
        .router
        .lazy_rescan_attempted
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let req = r#"{"jsonrpc":"2.0","method":"executor.list","id":1}"#;
    let response = server.handle_request_json(req).await;
    assert!(response.get("result").is_some());
    assert_eq!(response["result"]["count"], 3);
}

#[tokio::test]
async fn executor_status_route_via_neural_api_server() {
    use crate::neural_api_server::NeuralApiServer;

    let temp = tempfile::tempdir().expect("temp dir");
    std::fs::create_dir_all(temp.path()).expect("create graphs dir");
    let server = NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"));
    server
        .router
        .lazy_rescan_attempted
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let req = r#"{"jsonrpc":"2.0","method":"executor.status","params":{},"id":2}"#;
    let response = server.handle_request_json(req).await;
    assert!(response.get("result").is_some());
    assert!(response["result"]["executors"]["continuous"].is_object());
}

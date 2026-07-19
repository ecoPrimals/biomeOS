// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::super::*;
use super::helpers::make_handler;
use crate::handlers::graph::ExecutionStatus;
use crate::neural_router::NeuralRouter;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::test]
async fn test_get_metrics_response_structure() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get_metrics().await.expect("get_metrics");

    assert!(result.get("timestamp").is_some());
    assert!(result.get("system").is_some());
    assert!(result.get("neural_api").is_some());

    let system = &result["system"];
    assert!(system.get("cpu_percent").is_some());
    assert!(system.get("memory_used_mb").is_some());
    assert!(system.get("memory_total_mb").is_some());
    assert!(system.get("memory_percent").is_some());
    assert!(system.get("uptime_seconds").is_some());

    let neural = &result["neural_api"];
    assert_eq!(neural["family_id"], "test-family");
    assert!(neural.get("active_primals").is_some());
    assert!(neural.get("graphs_available").is_some());
    assert!(neural.get("active_executions").is_some());
}

#[tokio::test]
async fn test_get_metrics_with_graphs_dir() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let graph_file = temp.path().join("test.toml");
    std::fs::write(
        &graph_file,
        r#"
[graph]
id = "test"
version = "1.0"
description = "Test"

[[nodes]]
id = "node1"
"#,
    )
    .expect("write graph");
    let handler = make_handler("test-family", router, temp.path().to_path_buf());

    let result = handler.get_metrics().await.expect("get_metrics");
    assert_eq!(result["neural_api"]["graphs_available"], 1);
}

#[tokio::test]
async fn test_get_metrics_with_nonexistent_graphs_dir() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let handler = make_handler("test-family", router, "/nonexistent/path/12345");

    let result = handler.get_metrics().await.expect("get_metrics");
    assert_eq!(result["neural_api"]["graphs_available"], 0);
}

#[tokio::test]
async fn test_get_metrics_active_executions_count() {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let temp = tempfile::tempdir().expect("temp dir");
    let executions = Arc::new(RwLock::new(HashMap::from([(
        "exec-1".to_string(),
        ExecutionStatus {
            execution_id: "exec-1".to_string(),
            state: "running".to_string(),
            current_phase: Some(1),
            total_phases: 2,
            completed_nodes: vec![],
            failed_nodes: vec![],
            duration_ms: 100,
            error: None,
        },
    )])));
    let handler = TopologyHandler::new("test-family", router, executions, temp.path());

    let result = handler.get_metrics().await.expect("get_metrics");
    assert_eq!(result["neural_api"]["active_executions"], 1);
}

#[tokio::test]
async fn get_metrics_has_neural_api_block() {
    let router = Arc::new(NeuralRouter::new("topo-cov-fam3"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("topo-cov-fam3", router, exec, tmp.path());
    let v = h.get_metrics().await.expect("metrics");
    assert!(v.get("system").is_some());
    assert_eq!(v["neural_api"]["family_id"], "topo-cov-fam3");
    assert!(v["neural_api"].get("active_primals").is_some());
}

#[tokio::test]
async fn get_metrics_timestamp_and_system_keys() {
    let router = Arc::new(NeuralRouter::new("metrics-keys"));
    let exec = Arc::new(RwLock::new(HashMap::new()));
    let tmp = tempfile::tempdir().expect("tempdir");
    let h = TopologyHandler::new("metrics-keys", router, exec, tmp.path());
    let v = h.get_metrics().await.expect("metrics");
    assert!(v["timestamp"].as_str().is_some());
    let sys = v["system"].as_object().unwrap();
    assert!(sys.contains_key("cpu_percent"));
    assert!(sys.contains_key("uptime_seconds"));
}

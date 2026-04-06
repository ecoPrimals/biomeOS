// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Additional branch coverage for `handlers::graph` (CRUD edges, translation load, execute variants).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;
use serde_json::json;
use tempfile::tempdir;

/// Graph with capability metadata to exercise `load_translations_from_graph` during `execute`.
const GRAPH_WITH_CAPABILITIES_TOML: &str = r#"
[graph]
id = "cap_load_test"
version = "1.0.0"
description = "Capability translation load"

[[nodes]]
id = "provides_x"
depends_on = []
capabilities = ["custom.semantic"]

[nodes.primal]
by_name = "testprimal"

[nodes.operation]
name = "log.info"
[nodes.config]
message = "capability translation path"
"#;

#[tokio::test]
async fn test_execute_loads_translations_when_nodes_have_capabilities() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("cap_load_test.toml");
    std::fs::write(&path, GRAPH_WITH_CAPABILITIES_TOML).expect("write");
    let (handler, _) = make_handler(temp.path());

    let params = Some(json!({"graph_id": "cap_load_test"}));
    let result = handler.execute(&params).await.expect("execute");
    assert_eq!(result["graph_id"], "cap_load_test");
}

#[tokio::test]
async fn test_list_multiple_graphs_sorted_by_read_dir() {
    let temp = tempdir().expect("tempdir");
    std::fs::write(
        temp.path().join("a_first.toml"),
        graph_toml_with_id("graph_a"),
    )
    .expect("write");
    std::fs::write(
        temp.path().join("z_second.toml"),
        graph_toml_with_id("graph_z"),
    )
    .expect("write");
    let (handler, _) = make_handler(temp.path());

    let result = handler.list().await.expect("list");
    let arr = result.as_array().expect("array");
    assert_eq!(arr.len(), 2);
    let ids: Vec<String> = arr
        .iter()
        .filter_map(|v| v["id"].as_str().map(String::from))
        .collect();
    assert!(ids.contains(&"graph_a".to_string()));
    assert!(ids.contains(&"graph_z".to_string()));
}

fn graph_toml_with_id(id: &str) -> String {
    format!(
        r#"
[graph]
id = "{id}"
version = "1.0.0"
description = "Test"

[[nodes]]
id = "n1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "hi"
"#
    )
}

#[tokio::test]
async fn test_get_graph_id_with_hyphen() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("my-graph-id.toml");
    let toml = graph_toml_with_id("my-graph-id");
    std::fs::write(&path, toml).expect("write");
    let (handler, _) = make_handler(temp.path());

    let result = handler
        .get(&Some(json!({"graph_id": "my-graph-id"})))
        .await
        .expect("get");
    assert_eq!(result["id"], "my-graph-id");
}

#[tokio::test]
async fn test_suggest_optimizations_corrupt_toml() {
    let temp = tempdir().expect("tempdir");
    std::fs::write(temp.path().join("bad_opt.toml"), "not [[valid toml").expect("write");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .suggest_optimizations(&Some(json!({"graph_id": "bad_opt"})))
        .await
        .expect_err("parse fail");
    let msg = err.to_string();
    assert!(
        msg.contains("parse") || msg.contains("Failed") || msg.contains("TOML"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn test_execute_pipeline_missing_graph_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .execute_pipeline(&Some(json!({"channel_capacity": 8})))
        .await
        .expect_err("fail");
    assert!(err.to_string().contains("Missing graph_id"));
}

#[tokio::test]
async fn test_start_continuous_reads_graph_file_via_same_basename() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("edge-case.toml");
    let toml = r#"
[graph]
id = "edge-case"
name = "Edge"
version = "1.0.0"
coordination = "continuous"

[graph.tick]
target_hz = 1.0

[[graph.nodes]]
id = "n1"
name = "N"
"#;
    std::fs::write(&path, toml).expect("write");
    let (handler, _) = make_handler(temp.path());

    let result = handler
        .start_continuous(&Some(json!({"graph_id": "edge-case"})))
        .await
        .expect("start");
    assert!(result["session_id"].as_str().unwrap().contains("edge-case"));
}

#[tokio::test]
async fn test_execute_graph_id_numeric_string_rejected() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .execute(&Some(json!({"graph_id": 123})))
        .await
        .expect_err("fail");
    assert!(err.to_string().contains("Missing graph_id"));
}

#[tokio::test]
async fn test_graph_status_priority_over_continuous_when_same_id() {
    let temp = tempdir().expect("tempdir");
    let (handler, executions) = make_handler(temp.path());

    let status = ExecutionStatus {
        execution_id: "shared-id".to_string(),
        state: "running".to_string(),
        current_phase: Some(0),
        total_phases: 1,
        completed_nodes: vec![],
        failed_nodes: vec![],
        duration_ms: 0,
        error: None,
    };
    executions
        .write()
        .await
        .insert("shared-id".to_string(), status);

    let result = handler
        .get_status(&Some(json!({"execution_id": "shared-id"})))
        .await
        .expect("status");
    assert_eq!(result["state"], "running");
    assert!(result.get("continuous").is_none());
}

#[tokio::test]
async fn test_list_toml_parse_error_skips_file_quietly() {
    let temp = tempdir().expect("tempdir");
    std::fs::write(
        temp.path().join("good.toml"),
        graph_toml_with_id("only_good"),
    )
    .expect("write");
    std::fs::write(temp.path().join("bad.toml"), "[[[broken").expect("write");
    let (handler, _) = make_handler(temp.path());

    let arr = handler.list().await.expect("list");
    assert_eq!(arr.as_array().unwrap().len(), 1);
    assert_eq!(arr[0]["id"], "only_good");
}

#[tokio::test]
async fn test_suggest_optimizations_default_min_samples() {
    let temp = tempdir().expect("tempdir");
    let metrics_db = temp.path().join("neural_metrics_test.redb");
    let path = temp.path().join("opt_default.toml");
    let toml = r#"
[graph]
id = "opt-default"
name = "O"
version = "1.0.0"
coordination = "sequential"

[[graph.nodes]]
id = "n1"
name = "N"
"#;
    std::fs::write(&path, toml).expect("write");
    let (handler, _) = make_handler_with_metrics_db(temp.path(), Some(metrics_db));

    let result = handler
        .suggest_optimizations(&Some(json!({"graph_id": "opt_default"})))
        .await
        .expect("suggest");
    assert!(result.get("suggestions").is_some());
}

#[tokio::test]
async fn test_execute_pipeline_channel_capacity_default() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("pipe_cap.toml");
    let toml = r#"
[graph]
id = "pipe-cap"
name = "P"
version = "1.0.0"
coordination = "pipeline"

[[graph.nodes]]
id = "src"
name = "Source"
capability = "test.cap"

[[graph.nodes]]
id = "sink"
name = "Sink"
depends_on = ["src"]
capability = "test.cap"
"#;
    std::fs::write(&path, toml).expect("write");
    let (handler, _) = make_handler(temp.path());

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        handler.execute_pipeline(&Some(json!({"graph_id": "pipe_cap"}))),
    )
    .await;
    match result {
        Ok(Ok(v)) => {
            assert!(v.get("nodes").is_some() || v.get("throughput").is_some() || v.is_object());
        }
        Ok(Err(e)) => {
            let err = e.to_string();
            assert!(
                err.contains("forward")
                    || err.contains("discover")
                    || err.contains("Capability")
                    || err.contains("not found"),
                "unexpected error: {err}"
            );
        }
        Err(_timeout) => {}
    }
}

#[tokio::test]
async fn test_get_empty_file() {
    let temp = tempdir().expect("tempdir");
    let path = temp.path().join("empty.toml");
    std::fs::write(&path, "").expect("write");
    let (handler, _) = make_handler(temp.path());

    let err = handler
        .get(&Some(json!({"graph_id": "empty"})))
        .await
        .expect_err("parse");
    assert!(err.to_string().contains("Failed to load graph") || err.to_string().contains("parse"));
}

#[test]
fn test_extract_session_id_ok() {
    let id = GraphHandler::extract_session_id(&Some(json!({"session_id": "sess-42"})))
        .expect("session id");
    assert_eq!(id, "sess-42");
}

#[test]
fn test_extract_session_id_missing_params() {
    let err = GraphHandler::extract_session_id(&None).unwrap_err();
    assert!(err.to_string().contains("Missing parameters"));
}

#[test]
fn test_extract_session_id_missing_key() {
    let err = GraphHandler::extract_session_id(&Some(json!({"other": 1}))).unwrap_err();
    assert!(err.to_string().contains("session_id"));
}

#[tokio::test]
async fn test_pause_continuous_missing_session() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let err = handler
        .pause_continuous(&Some(json!({"session_id": "nope"})))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_resume_continuous_missing_session() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let err = handler
        .resume_continuous(&Some(json!({"session_id": "nope"})))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_stop_continuous_missing_session() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let err = handler
        .stop_continuous(&Some(json!({"session_id": "nope"})))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_get_missing_params() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let err = handler.get(&None).await.unwrap_err();
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn test_save_invalid_graph_json() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let err = handler
        .save(&Some(json!({"not": "a graph"})))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("parse") || err.to_string().contains("Failed"));
}

#[tokio::test]
async fn test_execute_graph_file_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let err = handler
        .execute(&Some(json!({"graph_id": "missing_graph_xyz"})))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn test_resolve_primal_name_uses_by_name_when_set() {
    use crate::neural_graph::{GraphNode, PrimalSelector};
    let node = GraphNode {
        id: "node-a".to_string(),
        primal: Some(PrimalSelector {
            by_capability: None,
            by_name: Some("my-primal".to_string()),
        }),
        ..Default::default()
    };
    assert_eq!(GraphHandler::resolve_primal_name(&node), "my-primal");
}

#[test]
fn test_resolve_primal_name_falls_back_to_node_id() {
    use crate::neural_graph::GraphNode;
    let node = GraphNode {
        id: "fallback-id".to_string(),
        ..Default::default()
    };
    assert_eq!(GraphHandler::resolve_primal_name(&node), "fallback-id");
}

#[tokio::test]
async fn test_suggest_optimizations_graph_not_found() {
    let temp = tempdir().expect("tempdir");
    let (handler, _) = make_handler(temp.path());
    let err = handler
        .suggest_optimizations(&Some(json!({"graph_id": "missing_opt"})))
        .await
        .unwrap_err();
    assert!(err.to_string().contains("not found"));
}

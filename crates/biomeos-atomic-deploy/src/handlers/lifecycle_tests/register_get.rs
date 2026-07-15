// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::make_graph_node;
use crate::handlers::lifecycle::LifecycleHandler;
use serde_json::json;

#[tokio::test]
async fn register_returns_full_state() {
    let handler = LifecycleHandler::new("test-family");

    let params = json!({
        "name": "test-primal",
        "socket_path": "/tmp/test-primal.sock",
        "pid": 12345
    });

    let result = handler.register(&Some(params)).await.expect("register");
    assert_eq!(result["registered"], "test-primal");
    assert_eq!(result["state"], "incubating");
    assert_eq!(result["socket_path"], "/tmp/test-primal.sock");
    assert_eq!(result["pid"], 12345);

    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 1);
    assert_eq!(status["healthy"], 0);

    let primals = status["primals"].as_array().expect("primals");
    let p = &primals[0];
    assert_eq!(p["name"], "test-primal");
    assert_eq!(p["state"], "incubating");
    assert!(p.get("details").is_some());
}

#[tokio::test]
async fn register_with_deployment_node_tracks_deps() {
    let handler = LifecycleHandler::new("test-family");

    let deployment_node = make_graph_node("beardog", vec![]);
    let params = json!({
        "name": "beardog",
        "socket_path": "/tmp/beardog.sock",
        "pid": 42,
        "deployment_node": serde_json::to_value(&deployment_node).unwrap()
    });

    let result = handler.register(&Some(params)).await.expect("register");
    assert_eq!(result["registered"], "beardog");

    let get_result = handler
        .get(&Some(json!({"name": "beardog"})))
        .await
        .expect("get");
    assert_eq!(get_result["name"], "beardog");
    assert!(get_result.get("depends_on").is_some());
    assert!(get_result.get("depended_by").is_some());
}

#[tokio::test]
async fn register_without_pid_is_null() {
    let handler = LifecycleHandler::new("test-family");

    let params = json!({
        "name": "no-pid-primal",
        "socket_path": "/tmp/no-pid.sock"
    });

    let result = handler.register(&Some(params)).await.expect("register");
    assert_eq!(result["registered"], "no-pid-primal");
    assert!(result["pid"].is_null());
}

#[tokio::test]
async fn get_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .get(&None)
        .await
        .expect_err("get with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn get_missing_name_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .get(&Some(json!({})))
        .await
        .expect_err("get with empty params should fail");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn get_nonexistent_primal_returns_error_body() {
    let handler = LifecycleHandler::new("test-family");
    let result = handler
        .get(&Some(json!({"name": "nonexistent"})))
        .await
        .expect("get returns Ok with error in body");
    assert!(result.get("error").is_some());
    assert!(
        result["error"]
            .as_str()
            .expect("error string")
            .contains("nonexistent")
    );
}

#[tokio::test]
async fn register_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .register(&None)
        .await
        .expect_err("register with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn register_missing_name_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .register(&Some(json!({"socket_path": "/tmp/x.sock"})))
        .await
        .expect_err("register without name should fail");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn register_missing_socket_path_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .register(&Some(json!({"name": "x"})))
        .await
        .expect_err("register without socket_path should fail");
    assert!(err.to_string().contains("socket_path"));
}

#[tokio::test]
async fn get_full_serialization_includes_all_fields() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "full-details",
            "socket_path": "/tmp/full.sock",
            "pid": 9999
        })))
        .await
        .expect("register");

    let result = handler
        .get(&Some(json!({"name": "full-details"})))
        .await
        .expect("get");

    assert_eq!(result["name"], "full-details");
    assert_eq!(result["family_id"], "test-family");
    assert_eq!(result["socket_path"], "/tmp/full.sock");
    assert_eq!(result["pid"], 9999);
    assert_eq!(result["state"], "incubating");

    assert!(result.get("state_details").is_some());
    assert!(result.get("depends_on").is_some());
    assert!(result.get("depended_by").is_some());
    assert!(result.get("metrics").is_some());
    assert!(result.get("health_config").is_some());
    assert!(result.get("resurrection_config").is_some());

    let metrics = &result["metrics"];
    assert!(metrics.get("total_uptime_secs").is_some());
    assert!(metrics.get("resurrection_count").is_some());
    assert!(metrics.get("health_failures").is_some());
    assert!(metrics.get("last_health_latency_ms").is_some());
    assert!(metrics.get("requests_served").is_some());

    let health_config = &result["health_config"];
    assert!(health_config.get("check_interval_secs").is_some());
    assert!(health_config.get("timeout_secs").is_some());
    assert!(health_config.get("failure_threshold").is_some());
    assert!(health_config.get("health_method").is_some());

    let res_config = &result["resurrection_config"];
    assert!(res_config.get("enabled").is_some());
    assert!(res_config.get("max_attempts").is_some());
    assert!(res_config.get("base_delay_secs").is_some());
    assert!(res_config.get("max_delay_secs").is_some());
}

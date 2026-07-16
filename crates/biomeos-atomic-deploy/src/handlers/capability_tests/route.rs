// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use tempfile::tempdir;

use super::{handler_with_registration, make_handler};

#[tokio::test]
async fn test_route_success_via_mock_socket() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("route-test.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "echo": true, "method": "pong" }))
            .await;

    let handler = make_handler();
    let reg = Some(json!({
        "capability": "mesh",
        "primal": "songbird",
        "socket": sock.to_str().unwrap(),
        "source": "test"
    }));
    handler.register(&reg).await.expect("register");

    let params = Some(json!({
        "capability": "mesh",
        "method": "any.method",
        "params": { "a": 1 }
    }));
    let result = handler.route(&params).await.expect("route");
    assert_eq!(result["echo"], true);

    let metrics = handler.get_metrics().await.expect("metrics");
    assert_eq!(metrics["total_requests"], 1);
    let m0 = &metrics["metrics"].as_array().expect("arr")[0];
    assert_eq!(m0["success"], true);
}
#[tokio::test]
async fn test_route_missing_capability_field() {
    let handler = make_handler();
    let params = Some(json!({ "method": "x", "params": {} }));
    assert!(handler.route(&params).await.is_err());
}
#[tokio::test]
async fn test_route_uses_default_empty_params_object() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("route-default-params.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "ok": true })).await;

    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "mesh",
            "primal": "songbird",
            "socket": sock.to_str().unwrap(),
            "source": "t"
        })))
        .await
        .unwrap();

    let params = Some(json!({
        "capability": "mesh",
        "method": "ping"
    }));
    let result = handler.route(&params).await.expect("route");
    assert_eq!(result["ok"], true);
}
#[tokio::test]
async fn test_route_logs_metric_on_success() {
    let dir = tempdir().expect("tempdir");
    let sock = dir.path().join("metric-route.sock");
    let _server = MockJsonRpcServer::spawn_echo_success(&sock, json!({ "metric": true })).await;

    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "metric-cap",
            "primal": "m",
            "socket": sock.to_str().unwrap(),
            "source": "t"
        })))
        .await
        .unwrap();

    let params = Some(json!({
        "capability": "metric-cap",
        "method": "x",
        "params": {}
    }));
    let out = handler.route(&params).await.expect("route");
    assert_eq!(out["metric"], true);
    let m = handler.get_metrics().await.expect("metrics");
    assert_eq!(m["total_requests"], 1);
    assert_eq!(m["metrics"].as_array().unwrap()[0]["success"], true);
}

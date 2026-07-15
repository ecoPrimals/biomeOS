// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{create_test_handler, make_handler};
use serde_json::json;

#[tokio::test]
async fn escalate_and_fallback_return_mode_strings() {
    let h = make_handler();
    h.register_connection(&Some(json!({ "from": "u", "to": "v" })))
        .await
        .unwrap();
    let esc = h
        .escalate(&Some(json!({ "from": "u", "to": "v" })))
        .await
        .unwrap();
    assert!(esc.get("previous_mode").is_some());
    assert!(esc.get("current_mode").is_some());
    let fb = h
        .fallback(&Some(json!({ "from": "u", "to": "v", "reason": "t" })))
        .await
        .unwrap();
    assert!(fb.get("message").is_some());
}

#[tokio::test]
async fn stop_monitoring_idempotent() {
    let h = make_handler();
    let a = h.stop_monitoring().await.unwrap();
    let b = h.stop_monitoring().await.unwrap();
    assert_eq!(a["status"], "stopped");
    assert_eq!(b["status"], "stopped");
}

#[tokio::test]
async fn test_metrics() {
    let handler = create_test_handler();

    handler.living_graph().register_connection("x", "y").await;

    for i in 0..10 {
        handler
            .living_graph()
            .record_request("x", "y", 100 + i * 10, true)
            .await;
    }

    let params = Some(json!({
        "from": "x",
        "to": "y"
    }));

    let result = handler.metrics(&params).await.unwrap();

    assert!(result.get("connection").is_some());
    assert!(result.get("metrics").is_some());
    assert_eq!(result["metrics"]["request_count"], 10);
}

#[tokio::test]
async fn test_fallback_with_reason() {
    let handler = create_test_handler();
    handler
        .living_graph()
        .register_connection("src", "dst")
        .await;

    let params = Some(json!({
        "from": "src",
        "to": "dst",
        "reason": "manual_test"
    }));

    let result = handler.fallback(&params).await.unwrap();
    assert!(result.get("status").is_some());
    assert_eq!(result["from"], "src");
    assert_eq!(result["to"], "dst");
}

#[tokio::test]
async fn test_fallback_default_reason() {
    let handler = create_test_handler();
    handler.living_graph().register_connection("a", "b").await;

    let params = Some(json!({ "from": "a", "to": "b" }));
    let result = handler.fallback(&params).await.unwrap();
    assert_eq!(result["status"], "degraded");
}

#[tokio::test]
async fn test_start_monitoring() {
    let handler = create_test_handler();
    let result = handler.start_monitoring().await.unwrap();
    assert_eq!(result["status"], "started");
    assert!(result["message"].as_str().unwrap().contains("started"));
}

#[tokio::test]
async fn test_stop_monitoring() {
    let handler = create_test_handler();
    let result = handler.stop_monitoring().await.unwrap();
    assert_eq!(result["status"], "stopped");
    assert!(result["message"].as_str().unwrap().contains("stopped"));
}

#[tokio::test]
async fn test_escalate_registered_connection() {
    let handler = create_test_handler();
    handler
        .living_graph()
        .register_connection("client", "server")
        .await;

    let params = Some(json!({
        "from": "client",
        "to": "server"
    }));

    let result = handler.escalate(&params).await.unwrap();
    assert!(result.get("status").is_some());
    assert_eq!(result["from"], "client");
    assert_eq!(result["to"], "server");
}

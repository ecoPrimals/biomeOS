// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::handlers::lifecycle::LifecycleHandler;
use serde_json::json;

#[tokio::test]
async fn status_healthy_count_tracks_correctly() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "p1",
            "socket_path": "/tmp/p1.sock",
            "pid": 1
        })))
        .await
        .expect("register");

    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 1);
    assert_eq!(status["healthy"], 0);

    handler
        .apoptosis(&Some(json!({"name": "p1"})))
        .await
        .expect("apoptosis");

    let status = handler.status().await.expect("status");
    assert_eq!(status["healthy"], 0);
}

#[tokio::test]
async fn incubating_state_details_include_timeout() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "incubating",
            "socket_path": "/tmp/inc.sock",
            "pid": 1
        })))
        .await
        .expect("register");

    let status = handler.status().await.expect("status");
    let primals = status["primals"].as_array().expect("primals");
    let p = primals
        .iter()
        .find(|x| x["name"] == "incubating")
        .expect("primal");
    assert_eq!(p["state"], "incubating");
    let details = &p["details"];
    assert!(details.get("started_at").is_some());
    assert!(details.get("timeout_ms").is_some());
}

#[tokio::test]
async fn dead_state_details_include_reason() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "to-die",
            "socket_path": "/tmp/die.sock",
            "pid": 1
        })))
        .await
        .expect("register");

    handler
        .apoptosis(&Some(json!({"name": "to-die", "reason": "user_request"})))
        .await
        .expect("apoptosis");

    let status = handler.status().await.expect("status");
    let primals = status["primals"].as_array().expect("primals");
    let p = primals
        .iter()
        .find(|x| x["name"] == "to-die")
        .expect("primal");
    assert_eq!(p["state"], "dead");
    let details = &p["details"];
    assert!(details.get("since").is_some());
    assert!(details.get("reason").is_some());
}

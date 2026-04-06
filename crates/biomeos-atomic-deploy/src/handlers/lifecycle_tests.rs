// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project
//
// Sibling tests for `lifecycle.rs` (handler API surface).

#![expect(clippy::unwrap_used, reason = "test")]

use serde_json::json;

use super::lifecycle::LifecycleHandler;

#[tokio::test]
async fn status_empty_reports_zero_healthy() {
    let handler = LifecycleHandler::new("sibling-lc-fam");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
    assert_eq!(status["healthy"], 0);
    assert!(status["primals"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn shutdown_all_on_empty_manager_completes() {
    let handler = LifecycleHandler::new("sibling-lc-shut");
    let out = handler.shutdown_all().await.expect("shutdown_all");
    assert_eq!(out["shutdown"], "complete");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
}

#[tokio::test]
async fn resurrect_rejects_missing_name_field() {
    let handler = LifecycleHandler::new("sibling-lc-res");
    let err = handler
        .resurrect(&Some(json!({})))
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn apoptosis_rejects_missing_name_field() {
    let handler = LifecycleHandler::new("sibling-lc-apo");
    let err = handler
        .apoptosis(&Some(json!({ "reason": "user_request" })))
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn get_success_includes_family_and_metrics_block() {
    let handler = LifecycleHandler::new("sibling-lc-get");
    handler
        .register(&Some(json!({
            "name": "p-sibling",
            "socket_path": "/tmp/p-sibling.sock",
            "pid": 7
        })))
        .await
        .expect("register");
    let g = handler
        .get(&Some(json!({ "name": "p-sibling" })))
        .await
        .expect("get");
    assert_eq!(g["family_id"], "sibling-lc-get");
    assert!(g.get("metrics").is_some());
    assert_eq!(g["state"], "incubating");
}

#[tokio::test]
async fn status_increments_count_after_register() {
    let handler = LifecycleHandler::new("sibling-lc-st");
    handler
        .register(&Some(json!({
            "name": "only-one",
            "socket_path": "/tmp/only-one.sock"
        })))
        .await
        .expect("register");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 1);
    let primals = status["primals"].as_array().unwrap();
    assert_eq!(primals[0]["name"], "only-one");
}

#[tokio::test]
async fn apoptosis_default_reason_string_when_omitted() {
    let handler = LifecycleHandler::new("sibling-lc-reason");
    handler
        .register(&Some(json!({
            "name": "r1",
            "socket_path": "/tmp/r1.sock",
            "pid": 1
        })))
        .await
        .expect("register");
    let out = handler
        .apoptosis(&Some(json!({ "name": "r1" })))
        .await
        .expect("apoptosis");
    assert_eq!(out["reason"], "user_request");
}

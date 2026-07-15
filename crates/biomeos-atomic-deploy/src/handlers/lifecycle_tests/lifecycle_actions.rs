// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::handlers::lifecycle::LifecycleHandler;
use serde_json::json;

#[tokio::test]
async fn resurrect_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .resurrect(&None)
        .await
        .expect_err("resurrect with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn resurrect_nonexistent_returns_error_body() {
    let handler = LifecycleHandler::new("test-family");
    let result = handler
        .resurrect(&Some(json!({"name": "ghost"})))
        .await
        .expect("resurrect returns Ok");
    assert!(result.get("error").is_some());
    assert!(result["error"].as_str().unwrap().contains("ghost"));
}

#[tokio::test]
async fn apoptosis_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .apoptosis(&None)
        .await
        .expect_err("apoptosis with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn apoptosis_user_request_transitions_to_dead() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "victim",
            "socket_path": "/tmp/victim.sock",
            "pid": 9999
        })))
        .await
        .expect("register");

    let result = handler
        .apoptosis(&Some(json!({"name": "victim"})))
        .await
        .expect("apoptosis");
    assert_eq!(result["initiated"], "victim");
    assert_eq!(result["reason"], "user_request");
    assert_eq!(result["state"], "apoptosis");

    let status = handler.status().await.expect("status");
    let primals = status["primals"].as_array().expect("primals");
    let victim = primals
        .iter()
        .find(|p| p["name"] == "victim")
        .expect("victim");
    assert_eq!(victim["state"], "dead");
}

#[tokio::test]
async fn apoptosis_all_reasons_map_correctly() {
    let reasons = [
        ("ecosystem_health", "ecosystem_health"),
        ("resource_pressure", "resource_pressure"),
        ("system_shutdown", "system_shutdown"),
        ("unknown_reason", "unknown_reason"),
    ];

    for (reason_param, expected_reason) in reasons {
        let handler = LifecycleHandler::new("test-family");
        let name = format!("primal-{reason_param}");
        handler
            .register(&Some(json!({
                "name": name,
                "socket_path": format!("/tmp/{}.sock", name),
                "pid": 1
            })))
            .await
            .expect("register");

        let result = handler
            .apoptosis(&Some(json!({
                "name": name,
                "reason": reason_param
            })))
            .await
            .expect("apoptosis");
        assert_eq!(
            result["reason"].as_str(),
            Some(expected_reason),
            "reason {reason_param} should map to {expected_reason}"
        );
    }
}

#[tokio::test]
async fn shutdown_all_kills_everything() {
    let handler = LifecycleHandler::new("test-family");
    for name in &["a", "b", "c"] {
        handler
            .register(&Some(json!({
                "name": name,
                "socket_path": format!("/tmp/{}.sock", name),
                "pid": 1
            })))
            .await
            .expect("register");
    }

    let result = handler.shutdown_all().await.expect("shutdown_all");
    assert_eq!(result["shutdown"], "complete");
    assert!(result["message"].as_str().unwrap().contains("All primals"));

    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 3);
    let primals = status["primals"].as_array().expect("primals");
    for p in primals {
        assert_eq!(p["state"], "dead");
    }
}

#[tokio::test]
async fn resurrect_registered_primal_succeeds() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "resurrect-me",
            "socket_path": "/tmp/resurrect-me.sock",
            "pid": 1234
        })))
        .await
        .expect("register");

    let result = handler
        .resurrect(&Some(json!({"name": "resurrect-me"})))
        .await
        .expect("resurrect");
    assert_eq!(result["requested"], "resurrect-me");
    assert!(result["message"].as_str().unwrap().contains("Resurrection"));
}

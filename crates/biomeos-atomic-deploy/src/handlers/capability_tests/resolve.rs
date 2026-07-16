// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use tempfile::tempdir;

use super::{handler_with_registration, make_handler};

#[tokio::test]
async fn test_resolve_missing_params() {
    let handler = make_handler();
    assert!(handler.resolve(&None).await.is_err());
}

#[tokio::test]
async fn test_resolve_missing_capability_field() {
    let handler = make_handler();
    let params = Some(json!({}));
    let err = handler.resolve(&params).await;
    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("capability") || msg.contains("domain") || msg.contains("Missing"),
        "expected missing field error, got: {msg}"
    );
}

#[tokio::test]
async fn test_resolve_unregistered_capability() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent_xyz" }));
    assert!(handler.resolve(&params).await.is_err());
}

#[tokio::test]
async fn test_resolve_registered_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.resolve(&params).await.unwrap();
    assert_eq!(result["resolved"], true);
    assert_eq!(result["capability"], "crypto");
    assert_eq!(result["primal"], "beardog");
    assert!(result["provider_count"].as_u64().unwrap() >= 1);
    assert!(result["endpoint"].is_string());
}

#[tokio::test]
async fn test_resolve_uses_domain_alias() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "domain": "crypto" }));
    let result = handler.resolve(&params).await.unwrap();
    assert_eq!(result["resolved"], true);
    assert_eq!(result["capability"], "crypto");
}

#[tokio::test]
async fn test_resolve_logs_metrics() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    handler.resolve(&params).await.unwrap();

    let metrics = handler.get_metrics().await.unwrap();
    assert_eq!(
        metrics["total_requests"].as_u64().unwrap(),
        1,
        "resolve should log a routing metric"
    );
    let m0 = &metrics["metrics"].as_array().unwrap()[0];
    assert_eq!(m0["method"], "capability.resolve");
    assert_eq!(m0["success"], true);
    assert_eq!(m0["capability"], "crypto");
}

#[tokio::test]
async fn test_resolve_logs_failure_metric() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent" }));
    let _ = handler.resolve(&params).await;

    let metrics = handler.get_metrics().await.unwrap();
    assert_eq!(metrics["total_requests"].as_u64().unwrap(), 1);
    let m0 = &metrics["metrics"].as_array().unwrap()[0];
    assert_eq!(m0["success"], false);
    assert!(m0["error"].is_string());
}

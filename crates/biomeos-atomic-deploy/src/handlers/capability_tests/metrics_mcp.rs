// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde_json::json;

use super::{handler_with_registration, make_handler};

#[tokio::test]
async fn test_metrics_empty() {
    let handler = make_handler();
    let result = handler.get_metrics().await.unwrap();
    assert_eq!(result["total_requests"], 0);
    assert!(result["metrics"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_discover_registered_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.discover(&params).await.unwrap();
    assert_eq!(result["capability"], "crypto");
    let primals = result["primals"].as_array().unwrap();
    assert!(!primals.is_empty());
    assert_eq!(primals[0]["name"], "beardog");
}

#[tokio::test]
async fn test_discover_missing_params() {
    let handler = make_handler();
    let result = handler.discover(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_route_missing_params() {
    let handler = make_handler();
    let result = handler.route(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_route_missing_method() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.route(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_unregistered_capability() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent_capability_xyz" }));
    let result = handler.discover(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not registered") || err.contains("Capability") || err.contains("not found"),
        "expected capability error, got: {err}"
    );
}

#[tokio::test]
async fn test_mcp_tools_list_empty() {
    let handler = make_handler();
    let result = handler.mcp_tools_list().await.unwrap();
    assert_eq!(result["tool_count"], 0);
    assert!(result["tools"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_mcp_tools_list_after_register() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock",
        "semantic_mappings": {
            "sha256": "crypto.hash",
            "sign": "crypto.sign"
        }
    }));
    handler.register(&params).await.unwrap();

    let result = handler.mcp_tools_list().await.unwrap();
    let tool_count = result["tool_count"].as_u64().unwrap();
    assert!(
        tool_count >= 2,
        "expected at least 2 tools, got {tool_count}"
    );
}

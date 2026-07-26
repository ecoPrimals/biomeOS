// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde_json::json;

use super::{handler_with_registration, make_handler};

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
async fn test_discover_missing_capability_field() {
    let handler = make_handler();
    let params = Some(json!({}));
    let err = handler.discover(&params).await.unwrap_err();
    assert!(err.to_string().contains("capability") || err.to_string().contains("Missing"));
}

#[tokio::test]
async fn test_route_missing_capability_field() {
    let handler = make_handler();
    let params = Some(json!({ "method": "x", "params": {} }));
    assert!(handler.route(&params).await.is_err());
}

#[tokio::test]
async fn test_discover_uses_domain_alias_instead_of_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "domain": "crypto" }));
    let result = handler.discover(&params).await.expect("domain alias");
    assert_eq!(result["capability"], "crypto");
}
#[tokio::test]
async fn test_discover_missing_capability_field_errors() {
    let handler = make_handler();
    let err = handler.discover(&Some(json!({}))).await;
    assert!(err.is_err());
}

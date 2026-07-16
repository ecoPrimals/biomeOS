// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;
use tempfile::tempdir;

use super::{handler_with_registration, make_handler};

#[tokio::test]
async fn test_register_basic() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "http",
        "primal": "songbird",
        "socket": "/tmp/songbird.sock",
        "source": "unit_test"
    }));
    let result = handler.register(&params).await.unwrap();
    assert_eq!(result["success"], true);
    assert_eq!(result["capability"], "http");
    assert_eq!(result["primal"], "songbird");
}

#[tokio::test]
async fn test_register_missing_params() {
    let handler = make_handler();
    let result = handler.register(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_missing_capability() {
    let handler = make_handler();
    let params = Some(json!({
        "primal": "beardog",
        "socket": "/tmp/test.sock"
    }));
    let result = handler.register(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_missing_primal() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "socket": "/tmp/test.sock"
    }));
    let result = handler.register(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_missing_socket() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog"
    }));
    let result = handler.register(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_register_with_semantic_mappings() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock",
        "semantic_mappings": {
            "sha256": "crypto.blake3_hash",
            "sign": "crypto.sign"
        }
    }));
    let result = handler.register(&params).await.unwrap();
    assert_eq!(result["success"], true);

    let translations_result = handler.list_translations().await.unwrap();
    assert!(translations_result["count"].as_u64().unwrap() >= 2);
}

#[tokio::test]
async fn test_register_default_source() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock"
    }));
    let result = handler.register(&params).await.unwrap();
    assert_eq!(result["success"], true);
}
#[tokio::test]
async fn test_register_missing_params_for_route_register_alias() {
    let handler = make_handler();
    assert!(handler.register_route(&None).await.is_err());
}

#[tokio::test]
async fn test_discover_missing_capability_field_errors() {
    let handler = make_handler();
    let err = handler.discover(&Some(json!({}))).await;
    assert!(err.is_err());
}

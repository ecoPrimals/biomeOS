// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability handler tests - extracted to keep capability.rs under 1000 lines

use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;

use super::capability::CapabilityHandler;

fn make_handler() -> CapabilityHandler {
    let router = Arc::new(NeuralRouter::new("test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    CapabilityHandler::new(router, registry)
}

async fn handler_with_registration() -> CapabilityHandler {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog-test.sock",
        "source": "test"
    }));
    handler.register(&params).await.unwrap();
    handler
}

#[tokio::test]
async fn test_capability_handler_creation() {
    let handler = make_handler();
    let result = handler.list().await.unwrap();
    assert!(result["capabilities"].as_array().unwrap().is_empty());
    assert_eq!(result["count"], 0);
}

#[tokio::test]
async fn test_list_empty() {
    let handler = make_handler();
    let result = handler.list().await.unwrap();
    assert_eq!(result["count"], 0);
    assert!(result["capabilities"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_list_after_register() {
    let handler = handler_with_registration().await;
    let result = handler.list().await.unwrap();
    assert_eq!(result["count"], 1);
    let caps = result["capabilities"].as_array().unwrap();
    assert!(caps.iter().any(|c| c.as_str() == Some("crypto")));
}

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
async fn test_providers_empty() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "nonexistent" }));
    let result = handler.providers(&params).await.unwrap();
    assert_eq!(result["count"], 0);
}

#[tokio::test]
async fn test_providers_after_register() {
    let handler = handler_with_registration().await;
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.providers(&params).await.unwrap();
    assert_eq!(result["count"], 1);
    let providers = result["providers"].as_array().unwrap();
    assert_eq!(providers[0]["primal"], "beardog");
}

#[tokio::test]
async fn test_providers_missing_params() {
    let handler = make_handler();
    let result = handler.providers(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metrics_empty() {
    let handler = make_handler();
    let result = handler.get_metrics().await.unwrap();
    assert_eq!(result["total_requests"], 0);
    assert!(result["metrics"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_discover_translations_empty() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.discover_translations(&params).await.unwrap();
    assert_eq!(result["count"], 0);
    assert_eq!(result["capability"], "crypto");
}

#[tokio::test]
async fn test_discover_translations_after_register() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/beardog.sock",
        "semantic_mappings": {
            "sha256": "crypto.blake3_hash"
        }
    }));
    handler.register(&params).await.unwrap();

    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.discover_translations(&params).await.unwrap();
    assert_eq!(result["capability"], "crypto");
}

#[tokio::test]
async fn test_discover_translations_missing_params() {
    let handler = make_handler();
    let result = handler.discover_translations(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_list_translations_empty() {
    let handler = make_handler();
    let result = handler.list_translations().await.unwrap();
    assert_eq!(result["count"], 0);
}

#[tokio::test]
async fn test_list_translations_after_register() {
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
    handler.register(&params).await.unwrap();

    let result = handler.list_translations().await.unwrap();
    let count = result["count"].as_u64().unwrap();
    assert!(count >= 2, "Expected at least 2 translations, got {count}");

    let translations = result["translations"].as_array().unwrap();
    let semantics: Vec<&str> = translations
        .iter()
        .filter_map(|t| t["semantic"].as_str())
        .collect();
    assert!(semantics.contains(&"crypto.sha256"));
    assert!(semantics.contains(&"crypto.sign"));
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
async fn test_call_missing_params() {
    let handler = make_handler();
    let result = handler.call(&None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_missing_capability() {
    let handler = make_handler();
    let params = Some(json!({ "operation": "sha256" }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_missing_operation() {
    let handler = make_handler();
    let params = Some(json!({ "capability": "crypto" }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_call_dotted_capability() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto.sha256",
        "args": { "data": "test" }
    }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.contains("Missing 'operation'"));
}

#[tokio::test]
async fn test_call_params_alias_for_args() {
    let handler = handler_with_registration().await;
    let params = Some(json!({
        "capability": "crypto",
        "operation": "sha256",
        "params": { "data": "test" }
    }));
    let result = handler.call(&params).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(!err.contains("Missing"));
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
async fn test_multiple_capabilities() {
    let handler = make_handler();

    handler
        .register(&Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock"
        })))
        .await
        .unwrap();

    handler
        .register(&Some(json!({
            "capability": "http",
            "primal": "songbird",
            "socket": "/tmp/songbird.sock"
        })))
        .await
        .unwrap();

    let result = handler.list().await.unwrap();
    assert_eq!(result["count"], 2);
}

#[tokio::test]
async fn test_multiple_providers_same_capability() {
    let handler = make_handler();

    handler
        .register(&Some(json!({
            "capability": "compute",
            "primal": "toadstool-tower",
            "socket": "/tmp/toadstool-tower.sock"
        })))
        .await
        .unwrap();

    handler
        .register(&Some(json!({
            "capability": "compute",
            "primal": "toadstool-gate2",
            "socket": "/tmp/toadstool-gate2.sock"
        })))
        .await
        .unwrap();

    let params = Some(json!({ "capability": "compute" }));
    let result = handler.providers(&params).await.unwrap();
    assert_eq!(result["count"], 2);
}

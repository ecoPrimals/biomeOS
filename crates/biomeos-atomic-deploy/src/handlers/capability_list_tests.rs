// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `list`, `providers`, and translation discovery/listing tests.

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test assertions")]

use serde_json::json;

use crate::handlers::capability_tests::{handler_with_registration, make_handler};

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

#[tokio::test]
async fn test_list_includes_cost_estimates() {
    let handler = handler_with_registration().await;
    let result = handler.list().await.unwrap();
    let details = result["details"].as_array().unwrap();
    if !details.is_empty() {
        let first = &details[0];
        assert!(first.get("cost_estimates").is_some());
        assert!(first.get("operation_dependencies").is_some());
        assert!(first.get("locality").is_some());
    }
}

#[tokio::test]
async fn test_register_semantic_mappings_non_object_skipped() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/x.sock",
        "semantic_mappings": []
    }));
    handler.register(&params).await.unwrap();
    let tr = handler.list_translations().await.unwrap();
    assert_eq!(tr["count"], 0);
}

#[tokio::test]
async fn test_register_semantic_mappings_ignores_non_string_values() {
    let handler = make_handler();
    let params = Some(json!({
        "capability": "crypto",
        "primal": "beardog",
        "socket": "/tmp/x.sock",
        "semantic_mappings": { "op1": 123, "op2": "real.method" }
    }));
    handler.register(&params).await.unwrap();
    let tr = handler.list_translations().await.unwrap();
    assert_eq!(tr["count"], 1);
}

#[tokio::test]
async fn test_list_details_compute_locality_and_costs() {
    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "compute",
            "primal": "t",
            "socket": "/tmp/c.sock"
        })))
        .await
        .unwrap();
    handler
        .register(&Some(json!({
            "capability": "relay",
            "primal": "r",
            "socket": "/tmp/r.sock"
        })))
        .await
        .unwrap();
    let list = handler.list().await.unwrap();
    let details = list["details"].as_array().unwrap();
    let compute = details
        .iter()
        .find(|d| d["capability"] == "compute")
        .expect("compute entry");
    assert_eq!(compute["locality"], "local");
    let relay = details
        .iter()
        .find(|d| d["capability"] == "relay")
        .expect("relay");
    assert_eq!(relay["locality"], "mesh");
}

#[tokio::test]
async fn test_discover_translations_missing_capability_field() {
    let handler = make_handler();
    let err = handler.discover_translations(&Some(json!({}))).await;
    assert!(err.is_err());
}

#[tokio::test]
async fn test_list_includes_shader_domain_metadata() {
    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "shader",
            "primal": "gpu",
            "socket": "/tmp/shader.sock"
        })))
        .await
        .unwrap();
    let list = handler.list().await.unwrap();
    let details = list["details"].as_array().unwrap();
    let shader = details
        .iter()
        .find(|d| d["capability"] == "shader")
        .expect("shader entry");
    assert_eq!(shader["locality"], "local");
    assert_eq!(shader["provider_count"], 1);
    assert!(shader["cost_estimates"].is_array());
}

#[tokio::test]
async fn test_list_includes_stun_locality_mesh() {
    let handler = make_handler();
    handler
        .register(&Some(json!({
            "capability": "stun",
            "primal": "s",
            "socket": "/tmp/stun.sock"
        })))
        .await
        .unwrap();
    let list = handler.list().await.unwrap();
    let details = list["details"].as_array().unwrap();
    let stun = details
        .iter()
        .find(|d| d["capability"] == "stun")
        .expect("stun");
    assert_eq!(stun["locality"], "mesh");
}

#[tokio::test]
async fn test_providers_missing_capability_field_errors() {
    let handler = make_handler();
    let err = handler.providers(&Some(json!({}))).await;
    assert!(err.is_err());
}

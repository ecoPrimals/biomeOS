// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{crypto_derive_seed, health_check, lineage_verify};
use super::common::{test_context, test_node_with_config};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_crypto_derive_seed_fallback() {
    let node = test_node_with_config("derive1", {
        let mut c = HashMap::new();
        c.insert("source".to_string(), json!("tower"));
        c
    });
    let ctx = test_context();

    // No security socket configured → should use deterministic fallback
    let result = crypto_derive_seed(&node, &ctx).await.unwrap();
    assert_eq!(result["method"], "deterministic_fallback");
    assert_eq!(result["derived_from"], "tower");
    assert!(result["seed"].as_str().unwrap().contains("test-family"));
}

#[tokio::test]
async fn test_crypto_derive_seed_default_source() {
    let node = test_node_with_config("derive2", HashMap::new());
    let ctx = test_context();

    let result = crypto_derive_seed(&node, &ctx).await.unwrap();
    assert_eq!(result["derived_from"], "family"); // default
}

#[tokio::test]
async fn test_lineage_verify_no_provider() {
    let node = test_node_with_config("verify1", {
        let mut c = HashMap::new();
        c.insert("primal_name".to_string(), json!("beardog"));
        c
    });
    let ctx = test_context();

    let result = lineage_verify(&node, &ctx).await.unwrap();
    assert_eq!(result["verified"], false);
    assert_eq!(result["method"], "no_security_provider");
}

#[tokio::test]
async fn test_lineage_verify_missing_primal_name() {
    let node = test_node_with_config("verify2", HashMap::new());
    let ctx = test_context();

    let result = lineage_verify(&node, &ctx).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("primal_name"));
}

#[tokio::test]
async fn test_health_check_missing_primal_name() {
    let node = test_node_with_config("hc1", HashMap::new());
    let ctx = test_context();

    let result = health_check(&node, &ctx).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("primal_name"));
}

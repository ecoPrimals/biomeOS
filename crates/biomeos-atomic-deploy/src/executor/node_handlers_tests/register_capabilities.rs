// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::register_capabilities;
use super::common::{test_context, test_node_with_capabilities, test_node_with_config};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_register_capabilities_with_caps() {
    let node = test_node_with_capabilities(
        "reg1",
        {
            let mut c = HashMap::new();
            c.insert("primal_name".to_string(), json!("beardog"));
            c
        },
        vec!["crypto.encrypt".to_string(), "crypto.decrypt".to_string()],
    );
    let ctx = test_context();

    let result = register_capabilities(&node, &ctx).await.unwrap();
    assert_eq!(result["primal"], "beardog");
    assert_eq!(result["count"], 2);
    let registered = result["registered"].as_array().unwrap();
    assert_eq!(registered.len(), 2);
    assert!(
        registered
            .iter()
            .any(|v| v.as_str() == Some("crypto.encrypt"))
    );
    assert!(
        registered
            .iter()
            .any(|v| v.as_str() == Some("crypto.decrypt"))
    );
}

#[tokio::test]
async fn test_register_capabilities_empty_caps() {
    let node = test_node_with_capabilities(
        "reg2",
        {
            let mut c = HashMap::new();
            c.insert("primal_name".to_string(), json!("songbird"));
            c
        },
        vec![],
    );
    let ctx = test_context();

    let result = register_capabilities(&node, &ctx).await.unwrap();
    assert_eq!(result["primal"], "songbird");
    assert_eq!(result["count"], 0);
    assert!(result["registered"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_register_capabilities_default_primal_name() {
    let node = test_node_with_capabilities("reg3", HashMap::new(), vec!["mesh".to_string()]);
    let ctx = test_context();

    let result = register_capabilities(&node, &ctx).await.unwrap();
    assert_eq!(result["primal"], "unknown");
    assert_eq!(result["count"], 1);
}

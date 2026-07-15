// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{test_context, test_node_with_config};
use super::super::{log_error, log_info, log_warn};
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_log_info() {
    let node = test_node_with_config("log1", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Hello from biomeOS"));
        c
    });
    let ctx = test_context();

    let result = log_info(&node, &ctx).await.unwrap();
    assert_eq!(result["level"], "info");
    assert_eq!(result["message"], "Hello from biomeOS");
}

#[tokio::test]
async fn test_log_info_with_env_substitution() {
    let node = test_node_with_config("log2", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Family: ${FAMILY_ID}"));
        c
    });
    let ctx = test_context();

    let result = log_info(&node, &ctx).await.unwrap();
    assert_eq!(result["message"], "Family: test-family");
}

#[tokio::test]
async fn test_log_info_no_message() {
    let node = test_node_with_config("log3", HashMap::new());
    let ctx = test_context();

    let result = log_info(&node, &ctx).await.unwrap();
    assert_eq!(result["message"], "(no message)");
}

#[tokio::test]
async fn test_log_warn() {
    let node = test_node_with_config("warn1", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Something concerning"));
        c
    });
    let ctx = test_context();

    let result = log_warn(&node, &ctx).await.unwrap();
    assert_eq!(result["level"], "warn");
    assert_eq!(result["message"], "Something concerning");
}

#[tokio::test]
async fn test_log_error() {
    let node = test_node_with_config("err1", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Critical failure"));
        c
    });
    let ctx = test_context();

    let result = log_error(&node, &ctx).await.unwrap();
    assert_eq!(result["level"], "error");
    assert_eq!(result["message"], "Critical failure");
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::discover_capability_provider;
use super::common::{test_context, test_context_with_env};
use std::collections::HashMap;

#[tokio::test]
async fn test_discover_capability_provider_via_socket_env() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sock_path = temp_dir.path().join("beardog.sock");
    // Create a regular file (not a real socket, but metadata check uses exists)
    std::fs::write(&sock_path, b"").unwrap();

    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        sock_path.to_string_lossy().to_string(),
    );
    let ctx = test_context_with_env(env);

    let result = discover_capability_provider(&ctx, "security").await;
    assert_eq!(result, Some(sock_path.to_string_lossy().to_string()));
}

#[tokio::test]
async fn test_discover_capability_provider_via_endpoint_env() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SECURITY_ENDPOINT".to_string(),
        "http://localhost:8080".to_string(),
    );
    let ctx = test_context_with_env(env);

    let result = discover_capability_provider(&ctx, "security").await;
    assert_eq!(result, Some("http://localhost:8080".to_string()));
}

#[tokio::test]
async fn test_discover_capability_provider_none_found() {
    let ctx = test_context();

    let result = discover_capability_provider(&ctx, "unknown_capability").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_capability_provider_socket_not_exists() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        "/nonexistent/beardog.sock".to_string(),
    );
    let ctx = test_context_with_env(env);

    // Socket file doesn't exist, should fall through to endpoint check
    let result = discover_capability_provider(&ctx, "security").await;
    assert!(result.is_none());
}

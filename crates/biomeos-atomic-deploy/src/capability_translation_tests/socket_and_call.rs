// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for capability translation registry.
//!
//! Extracted from capability_translation module to keep main module under 1000 LOC.

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use crate::capability_translation::{
    CapabilityTranslation, CapabilityTranslationRegistry, RegistryStats, resolve_primal_socket,
    resolve_primal_socket_with,
};
use std::collections::HashMap;

fn test_resolve_primal_socket_env_override() {
    let unique_primal = "testprimal_env_override";
    let socket = resolve_primal_socket_with(
        unique_primal,
        "test-family",
        Some("/custom/unique-test.sock"),
    );
    assert_eq!(socket, "/custom/unique-test.sock");
}

#[test]
fn test_resolve_primal_socket_fallback() {
    let unique_primal = "testprimal_fallback";

    let socket = resolve_primal_socket(unique_primal, "test-family");

    assert!(
        socket.contains(unique_primal),
        "Socket should contain primal name"
    );
    assert!(
        socket.contains("test-family"),
        "Socket should contain family ID"
    );
    assert!(socket.ends_with(".sock"), "Socket should end with .sock");
}

#[test]
fn test_resolve_primal_socket_different_primals() {
    // Unique names avoid collision with real SONGBIRD_SOCKET / NESTGATE_SOCKET in the environment.
    let a = resolve_primal_socket_with("testprimal_sock_a", "fam1", None);
    let b = resolve_primal_socket_with("testprimal_sock_b", "fam1", None);

    assert_ne!(a, b);

    assert!(a.contains("testprimal_sock_a"));
    assert!(b.contains("testprimal_sock_b"));
}

#[test]
fn test_registry_default_impl() {
    let registry = CapabilityTranslationRegistry::default();

    assert_eq!(registry.stats().total_translations, 0);
}

#[test]
fn test_translation_with_param_mappings() {
    let mut registry = CapabilityTranslationRegistry::new();

    let mut param_mappings = HashMap::new();
    param_mappings.insert("private_key".to_string(), "our_secret".to_string());
    param_mappings.insert("public_key".to_string(), "their_public".to_string());

    registry.register_translation(
        "crypto.ecdh_derive",
        "beardog",
        "x25519_derive_secret",
        "/tmp/beardog.sock",
        Some(param_mappings),
    );

    let translation = registry.get_translation("crypto.ecdh_derive").unwrap();
    assert_eq!(
        translation.param_mappings.get("private_key"),
        Some(&"our_secret".to_string())
    );
    assert_eq!(
        translation.param_mappings.get("public_key"),
        Some(&"their_public".to_string())
    );
}

#[test]
fn test_get_translation_unknown_capability() {
    let registry = CapabilityTranslationRegistry::new();
    assert!(registry.get_translation("nonexistent.capability").is_none());
    assert!(!registry.has_capability("nonexistent.capability"));
}

#[tokio::test]
async fn test_call_capability_no_provider() {
    let registry = CapabilityTranslationRegistry::new();

    let result = registry
        .call_capability("unknown.capability", serde_json::json!({}))
        .await;

    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("No provider for capability")
    );
}

#[tokio::test]
async fn test_call_capability_socket_connection_fails() {
    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation(
        "test.fake_call",
        "fake_primal",
        "fake_method",
        "/nonexistent/path/does-not-exist-12345.sock",
        None,
    );

    let result = registry
        .call_capability("test.fake_call", serde_json::json!({}))
        .await;

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Provider") || err_msg.contains("connect") || err_msg.contains("socket"),
        "Expected provider/connection error, got: {err_msg}"
    );
}

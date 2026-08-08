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

#[test]
fn test_register_translation() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock",
        None,
    );

    assert!(registry.has_capability("crypto.generate_keypair"));

    let translation = registry.get_translation("crypto.generate_keypair").unwrap();
    assert_eq!(translation.semantic, "crypto.generate_keypair");
    assert_eq!(translation.provider, "beardog");
    assert_eq!(translation.actual_method, "x25519_generate_ephemeral");
    assert_eq!(translation.socket, "/tmp/beardog.sock");
}

#[test]
fn test_provider_capabilities() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock",
        None,
    );

    registry.register_translation(
        "crypto.ecdh_derive",
        "beardog",
        "x25519_derive_secret",
        "/tmp/beardog.sock",
        None,
    );

    let caps = registry.provider_capabilities("beardog");
    assert_eq!(caps.len(), 2);
    assert!(caps.contains(&"crypto.generate_keypair".to_string()));
    assert!(caps.contains(&"crypto.ecdh_derive".to_string()));
}

#[test]
fn test_list_all() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock",
        None,
    );

    registry.register_translation(
        "http.request",
        "songbird",
        "http_request",
        "/tmp/songbird.sock",
        None,
    );

    let all = registry.list_all();
    assert_eq!(all.len(), 2);
}

#[test]
fn test_stats() {
    let mut registry = CapabilityTranslationRegistry::new();

    registry.register_translation(
        "crypto.generate_keypair",
        "beardog",
        "x25519_generate_ephemeral",
        "/tmp/beardog.sock",
        None,
    );

    registry.register_translation(
        "http.request",
        "songbird",
        "http_request",
        "/tmp/songbird.sock",
        None,
    );

    let stats = registry.stats();
    assert_eq!(stats.total_translations, 2);
    assert_eq!(stats.total_providers, 2);
    assert_eq!(stats.capabilities_by_provider["beardog"], 1);
    assert_eq!(stats.capabilities_by_provider["songbird"], 1);
}

fn find_capability_registry_config() -> Option<std::path::PathBuf> {
    let mut dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    loop {
        let candidate = dir.join("config/capability_registry.toml");
        if candidate.exists() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[test]
fn test_load_from_capability_registry_toml() {
    let config_path = match find_capability_registry_config() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: config/capability_registry.toml not found");
            return;
        }
    };

    let mut registry = CapabilityTranslationRegistry::new();
    let count = registry
        .load_from_config(&config_path, "default", |provider, _family_id| {
            format!("/tmp/{provider}.sock")
        })
        .expect("should load capability_registry.toml");

    assert!(
        count > 0,
        "Should load at least some translations from config"
    );

    // Verify new translations exist
    assert!(
        registry.has_capability("compute.dispatch.submit"),
        "Should have compute.dispatch.submit"
    );
    assert!(
        registry.has_capability("secrets.store"),
        "Should have secrets.store"
    );
    assert!(
        registry.has_capability("model.register"),
        "Should have model.register"
    );
    assert!(
        registry.has_capability("relay.authorize"),
        "Should have relay.authorize"
    );
    assert!(
        registry.has_capability("hardware.observe"),
        "Should have hardware.observe"
    );
}

#[test]
fn test_load_defaults() {
    let mut registry = CapabilityTranslationRegistry::new();

    let count = registry.load_defaults("default");

    assert!(count > 0, "Should load at least some translations");

    assert!(
        registry.has_capability("beacon.generate"),
        "Should have beacon.generate"
    );
    assert!(
        registry.has_capability("crypto.encrypt"),
        "Should have crypto.encrypt"
    );

    assert!(
        registry.has_capability("network.beacon_exchange"),
        "Should have network.beacon_exchange"
    );

    assert!(
        registry.has_capability("storage.put"),
        "Should have storage.put"
    );

    assert!(registry.has_capability("ai.query"), "Should have ai.query");

    let beardog_caps = registry.provider_capabilities("beardog");
    assert!(!beardog_caps.is_empty(), "BearDog should have capabilities");
    assert!(
        beardog_caps.contains(&"beacon.generate".to_string()),
        "BearDog should provide beacon.generate"
    );
}

#[test]
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
#[test]
fn test_capability_translation_struct() {
    let mut param_mappings = HashMap::new();
    param_mappings.insert("a".to_string(), "b".to_string());

    let translation = CapabilityTranslation {
        semantic: "test.semantic".to_string(),
        provider: "beardog".to_string(),
        actual_method: "actual_method".to_string(),
        socket: "/tmp/beardog.sock".to_string(),
        ribocipher: false,
        param_mappings: param_mappings.clone(),
        metadata: HashMap::new(),
    };

    assert_eq!(translation.semantic, "test.semantic");
    assert_eq!(translation.provider, "beardog");
    assert_eq!(translation.param_mappings.get("a"), Some(&"b".to_string()));
}

#[test]
fn test_capability_translation_serde() {
    let translation = CapabilityTranslation {
        semantic: "crypto.encrypt".to_string(),
        provider: "beardog".to_string(),
        actual_method: "chacha20_encrypt".to_string(),
        socket: "/tmp/b.sock".to_string(),
        ribocipher: false,
        param_mappings: HashMap::new(),
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&translation).expect("serialize");
    assert!(json.contains("crypto.encrypt"));
    let parsed: CapabilityTranslation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.semantic, translation.semantic);
}

#[test]
fn test_registry_stats_struct() {
    let mut registry = CapabilityTranslationRegistry::new();
    registry.register_translation("a", "p1", "m1", "/tmp/1.sock", None);
    registry.register_translation("b", "p1", "m2", "/tmp/1.sock", None);
    registry.register_translation("c", "p2", "m3", "/tmp/2.sock", None);

    let stats = registry.stats();
    assert_eq!(stats.total_translations, 3);
    assert_eq!(stats.total_providers, 2);
    assert_eq!(stats.capabilities_by_provider["p1"], 2);
    assert_eq!(stats.capabilities_by_provider["p2"], 1);
}

#[test]
fn test_registry_stats_serialization() {
    let stats = RegistryStats {
        total_translations: 10,
        total_providers: 3,
        capabilities_by_provider: [("a".to_string(), 5), ("b".to_string(), 3)]
            .into_iter()
            .collect(),
    };
    let json = serde_json::to_string(&stats).expect("serialize");
    assert!(json.contains("10"));
    assert!(json.contains('3'));
}

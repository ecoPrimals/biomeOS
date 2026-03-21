// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for capability translation registry.
//!
//! Extracted from capability_translation module to keep main module under 1000 LOC.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::capability_translation::{
    CapabilityTranslation, CapabilityTranslationRegistry, RegistryStats, resolve_primal_socket,
};
use biomeos_test_utils::{TestEnvGuard, remove_test_env, set_test_env};
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
        .load_from_config(&config_path, |provider, _family_id| {
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

    let count = registry.load_defaults();

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
    let env_var = format!("{}_SOCKET", unique_primal.to_uppercase());

    set_test_env(&env_var, "/custom/unique-test.sock");

    let socket = resolve_primal_socket(unique_primal, "test-family");
    assert_eq!(socket, "/custom/unique-test.sock");

    remove_test_env(&env_var);
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
    remove_test_env("SONGBIRD_SOCKET");
    remove_test_env("NESTGATE_SOCKET");

    let songbird = resolve_primal_socket("songbird", "fam1");
    let nestgate = resolve_primal_socket("nestgate", "fam1");

    assert_ne!(songbird, nestgate);

    assert!(songbird.contains("songbird"));
    assert!(nestgate.contains("nestgate"));
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

#[test]
fn test_capability_translation_struct() {
    let mut param_mappings = HashMap::new();
    param_mappings.insert("a".to_string(), "b".to_string());

    let translation = CapabilityTranslation {
        semantic: "test.semantic".to_string(),
        provider: "beardog".to_string(),
        actual_method: "actual_method".to_string(),
        socket: "/tmp/beardog.sock".to_string(),
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

/// Verify that capability_registry.toml providers are all known primal names.
///
/// Absorbed from primalSpring v0.3.0's `capabilities_match_registry_toml` pattern.
/// Prevents config/code drift: if the TOML references a primal, that primal
/// must exist in `biomeos_types::primal_names`.
#[test]
fn capabilities_match_registry_toml() {
    let config_path = match find_capability_registry_config() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: config/capability_registry.toml not found");
            return;
        }
    };

    let mut registry = CapabilityTranslationRegistry::new();
    let count = registry
        .load_from_config(&config_path, |provider, _family_id| {
            format!("/tmp/{provider}.sock")
        })
        .expect("should load capability_registry.toml");

    assert!(count > 0, "Registry should have translations");

    let all = registry.list_all();
    let providers: std::collections::HashSet<&str> =
        all.iter().map(|t| t.provider.as_str()).collect();

    for provider in &providers {
        if *provider == "*" {
            continue; // wildcard = every primal implements this capability
        }
        assert!(
            biomeos_types::primal_names::is_known_primal(provider),
            "Provider '{provider}' in capability_registry.toml is not a known primal. \
             Add it to biomeos_types::primal_names or fix the TOML."
        );
    }
}

/// Verify every known primal has at least one translation in the TOML.
///
/// This ensures that if we add a primal to the code, we also add
/// its capabilities to the registry config.
#[test]
fn all_core_primals_have_capabilities_in_toml() {
    use biomeos_types::primal_names;

    let config_path = match find_capability_registry_config() {
        Some(p) => p,
        None => {
            eprintln!("Skipping: config/capability_registry.toml not found");
            return;
        }
    };

    let mut registry = CapabilityTranslationRegistry::new();
    registry
        .load_from_config(&config_path, |provider, _family_id| {
            format!("/tmp/{provider}.sock")
        })
        .expect("should load capability_registry.toml");

    let all = registry.list_all();
    let providers: std::collections::HashSet<&str> =
        all.iter().map(|t| t.provider.as_str()).collect();

    let core_primals = [
        primal_names::BEARDOG,
        primal_names::SONGBIRD,
        primal_names::TOADSTOOL,
        primal_names::NESTGATE,
        primal_names::SQUIRREL,
    ];

    for primal in &core_primals {
        assert!(
            providers.contains(primal),
            "Core primal '{primal}' has no capabilities in capability_registry.toml. \
             Add its translations to the config."
        );
    }
}

/// `BIOMEOS_*_PROVIDER=Ok(value)` path in [`defaults::load_defaults_into`](crate::capability_translation::defaults::load_defaults_into).
#[test]
#[serial_test::serial]
fn test_load_defaults_compute_provider_env_override() {
    let _g = TestEnvGuard::set("BIOMEOS_COMPUTE_PROVIDER", "songbird");
    let mut registry = CapabilityTranslationRegistry::new();
    registry.load_defaults();
    let t = registry
        .get_translation("compute.execute")
        .expect("compute.execute should be registered");
    assert_eq!(t.provider, "songbird");
}

/// Strict discovery with unset provider env still resolves defaults via domain tuple (warn path).
#[test]
#[serial_test::serial]
fn test_load_defaults_strict_discovery_unset_providers_use_domain_defaults() {
    let _strict = TestEnvGuard::set("BIOMEOS_STRICT_DISCOVERY", "1");
    let _sec = TestEnvGuard::remove("BIOMEOS_SECURITY_PROVIDER");
    let _net = TestEnvGuard::remove("BIOMEOS_NETWORK_PROVIDER");
    let _stor = TestEnvGuard::remove("BIOMEOS_STORAGE_PROVIDER");
    let _comp = TestEnvGuard::remove("BIOMEOS_COMPUTE_PROVIDER");
    let _ai = TestEnvGuard::remove("BIOMEOS_AI_PROVIDER");

    let mut registry = CapabilityTranslationRegistry::new();
    let count = registry.load_defaults();
    assert!(
        count > 10,
        "defaults should still register domain translations when strict and env unset"
    );
    assert!(registry.has_capability("crypto.encrypt"));
}

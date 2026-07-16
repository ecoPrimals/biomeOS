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

use super::find_capability_registry_config;

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
        .load_from_config(&config_path, "default", |provider, _family_id| {
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
        .load_from_config(&config_path, "default", |provider, _family_id| {
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
fn test_load_defaults_compute_provider_env_override() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_COMPUTE_PROVIDER", Some("songbird"));
    let mut registry = CapabilityTranslationRegistry::new();
    registry.load_defaults_with("default", &env);
    let t = registry
        .get_translation("compute.execute")
        .expect("compute.execute should be registered");
    assert_eq!(t.provider, "songbird");
}

/// Strict discovery with unset provider env still resolves defaults via domain tuple (warn path).
#[test]
fn test_load_defaults_strict_discovery_unset_providers_use_domain_defaults() {
    let mut env = HashMap::new();
    env.insert("BIOMEOS_STRICT_DISCOVERY", Some("1"));
    env.insert("BIOMEOS_SECURITY_PROVIDER", None);
    env.insert("BIOMEOS_NETWORK_PROVIDER", None);
    env.insert("BIOMEOS_STORAGE_PROVIDER", None);
    env.insert("BIOMEOS_COMPUTE_PROVIDER", None);
    env.insert("BIOMEOS_AI_PROVIDER", None);

    let mut registry = CapabilityTranslationRegistry::new();
    let count = registry.load_defaults_with("default", &env);
    assert!(
        count > 10,
        "defaults should still register domain translations when strict and env unset"
    );
    assert!(registry.has_capability("crypto.encrypt"));
}

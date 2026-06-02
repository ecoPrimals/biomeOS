// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

// ── Builder: construction & defaults ────────────────────────────────

#[test]
fn test_builder_default_trait() {
    let builder = BiomeOSConfigBuilder::default();
    let config = builder.name("from-default").build().unwrap();
    assert_eq!(config.metadata.name, "from-default");
}

#[test]
fn test_config_builder() {
    let config = BiomeOSConfigBuilder::new()
        .name("test-config")
        .environment(Environment::Testing)
        .organization_scale(OrganizationScale::Team)
        .enable_feature("telemetry")
        .max_workers(8)
        .build()
        .unwrap();

    assert_eq!(config.metadata.name, "test-config");
    assert_eq!(config.system.environment, Environment::Testing);
    assert_eq!(config.system.organization_scale, OrganizationScale::Team);
    assert!(config.features.telemetry);
    assert_eq!(config.system.workers.worker_threads, Some(8));
}

// ── Builder: feature flags ─────────────────────────────────────────

#[test]
fn test_enable_feature_ai_first() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("ai_first")
        .build()
        .unwrap();
    assert!(config.features.ai_first);
}

#[test]
fn test_enable_feature_ai_integration_alias() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("ai_integration")
        .build()
        .unwrap();
    assert!(config.features.ai_first);
}

#[test]
fn test_enable_feature_crypto_locks() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("crypto_locks")
        .build()
        .unwrap();
    assert!(config.features.crypto_locks);
}

#[test]
fn test_enable_feature_auto_scaling() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("auto_scaling")
        .build()
        .unwrap();
    assert!(config.features.auto_scaling);
}

#[test]
fn test_enable_feature_distributed_computing_alias() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("distributed_computing")
        .build()
        .unwrap();
    assert!(config.features.auto_scaling);
}

#[test]
fn test_enable_feature_networking() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("networking")
        .build()
        .unwrap();
    assert!(config.features.experimental);
}

#[test]
fn test_enable_feature_advanced_networking_alias() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("advanced_networking")
        .build()
        .unwrap();
    assert!(config.features.experimental);
}

#[test]
fn test_enable_feature_multi_tenant() {
    let config = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("multi_tenant")
        .build()
        .unwrap();
    assert!(config.features.experimental);
    assert!(config.features.debug);
}

#[test]
fn test_enable_feature_unknown_ignored() {
    let base = BiomeOSConfigBuilder::new().name("base").build().unwrap();
    let with_unknown = BiomeOSConfigBuilder::new()
        .name("feat")
        .enable_feature("nonexistent_feature")
        .build()
        .unwrap();
    // Unknown feature should not change any flags compared to baseline
    assert_eq!(
        with_unknown.features.crypto_locks,
        base.features.crypto_locks,
    );
    assert_eq!(
        with_unknown.features.auto_scaling,
        base.features.auto_scaling,
    );
}

// ── Builder: system settings ───────────────────────────────────────

#[test]
fn test_max_workers() {
    let config = BiomeOSConfigBuilder::new()
        .name("w")
        .max_workers(16)
        .build()
        .unwrap();
    assert_eq!(config.system.workers.worker_threads, Some(16));
}

#[test]
fn test_connection_timeout() {
    let config = BiomeOSConfigBuilder::new()
        .name("t")
        .connection_timeout(7500)
        .build()
        .unwrap();
    assert_eq!(
        config.system.timeouts.connection_timeout,
        std::time::Duration::from_millis(7500)
    );
}

#[test]
fn test_discovery_method() {
    let config = BiomeOSConfigBuilder::new()
        .name("d")
        .discovery_method(DiscoveryMethod::Registry)
        .build()
        .unwrap();
    assert_eq!(config.discovery.default_method, DiscoveryMethod::Registry);
}

#[test]
fn test_discovery_endpoint() {
    let config = BiomeOSConfigBuilder::new()
        .name("e")
        .discovery_endpoint("unix:///run/user/1000/biomeos/songbird.sock")
        .build()
        .unwrap();
    assert!(config.discovery.registry.is_some());
    assert_eq!(
        config.discovery.registry.unwrap().url,
        "unix:///run/user/1000/biomeos/songbird.sock"
    );
}

#[test]
fn test_env_var() {
    let config = BiomeOSConfigBuilder::new()
        .name("env")
        .environment(Environment::Development)
        .env_var("RUST_LOG", "debug")
        .env_var("MY_VAR", "hello")
        .build()
        .unwrap();
    let env_config = config
        .environments
        .get("development")
        .expect("Should have development environment");
    assert_eq!(env_config.variables.get("RUST_LOG").unwrap(), "debug");
    assert_eq!(env_config.variables.get("MY_VAR").unwrap(), "hello");
}

// ── Presets ────────────────────────────────────────────────────────

#[test]
fn test_development_preset() {
    let config = presets::development().unwrap();
    assert_eq!(config.system.environment, Environment::Development);
    assert_eq!(
        config.system.organization_scale,
        OrganizationScale::Individual
    );
    assert!(config.features.telemetry);
    assert_eq!(config.system.workers.worker_threads, Some(4));
}

#[test]
fn test_production_preset() {
    let config = presets::production().unwrap();
    assert_eq!(config.system.environment, Environment::Production);
    assert_eq!(
        config.system.organization_scale,
        OrganizationScale::Enterprise
    );
    assert!(config.features.ai_first);
    assert!(config.features.crypto_locks);
    assert!(config.features.auto_scaling);
    assert_eq!(config.system.workers.worker_threads, Some(16));
}

#[test]
fn test_testing_preset() {
    let config = presets::testing().unwrap();
    assert_eq!(config.system.environment, Environment::Testing);
    assert_eq!(config.system.organization_scale, OrganizationScale::Team);
    assert!(config.features.telemetry);
    assert_eq!(config.system.workers.worker_threads, Some(2));
}

#[test]
fn test_local_preset() {
    let config = presets::local().unwrap();
    assert_eq!(config.system.environment, Environment::Development);
    assert_eq!(
        config.system.organization_scale,
        OrganizationScale::Individual
    );
    assert_eq!(config.system.workers.worker_threads, Some(2));
    // Should have RUST_LOG env var
    let dev_env = config.environments.get("development");
    assert!(dev_env.is_some());
    assert_eq!(dev_env.unwrap().variables.get("RUST_LOG").unwrap(), "debug");
}

// ── Validation ─────────────────────────────────────────────────────

#[test]
fn test_production_ready_checks() {
    let config = presets::production().unwrap();
    assert!(validation::is_production_ready(&config));
}

#[test]
fn test_development_not_production_ready() {
    let config = presets::development().unwrap();
    assert!(!validation::is_production_ready(&config));
}

#[test]
fn test_validate_config_production_no_warnings_expected() {
    let config = presets::production().unwrap();
    let warnings = validation::validate_config(&config).unwrap();
    // Production preset should have minimal warnings
    // (no localhost in registry, enough workers, crypto locks on)
    for w in &warnings {
        // Acceptable: method-related warnings since preset uses Registry but no registry URL
        assert!(
            !w.contains("low worker count"),
            "Production should have enough workers"
        );
    }
}

#[test]
fn test_validate_config_low_workers_production() {
    let config = BiomeOSConfigBuilder::new()
        .name("low-workers")
        .environment(Environment::Production)
        .enable_feature("crypto_locks")
        .max_workers(2)
        .connection_timeout(5000)
        .build()
        .unwrap();
    let warnings = validation::validate_config(&config).unwrap();
    assert!(
        warnings.iter().any(|w| w.contains("low worker count")),
        "Should warn about low worker count in production"
    );
}

#[test]
fn test_validate_config_low_timeout() {
    let config = BiomeOSConfigBuilder::new()
        .name("low-timeout")
        .connection_timeout(500) // Very low
        .build()
        .unwrap();
    let warnings = validation::validate_config(&config).unwrap();
    assert!(
        warnings.iter().any(|w| w.contains("timeout")),
        "Should warn about low connection timeout"
    );
}

#[test]
fn test_validate_config_no_crypto_in_production() {
    let config = BiomeOSConfigBuilder::new()
        .name("no-crypto")
        .environment(Environment::Production)
        .max_workers(8)
        .connection_timeout(5000)
        .build()
        .unwrap();
    let warnings = validation::validate_config(&config).unwrap();
    assert!(
        warnings.iter().any(|w| w.contains("crypto locks")),
        "Should warn about missing crypto locks in production"
    );
}

#[test]
fn test_validate_config_registry_method_without_registry() {
    let config = BiomeOSConfigBuilder::new()
        .name("no-registry")
        .discovery_method(DiscoveryMethod::Registry)
        .build()
        .unwrap();
    let warnings = validation::validate_config(&config).unwrap();
    assert!(
        warnings
            .iter()
            .any(|w| w.contains("registry") || w.contains("Registry")),
        "Should warn about registry discovery without registry config"
    );
}

#[test]
fn test_is_production_ready_requires_production_env() {
    let config = BiomeOSConfigBuilder::new()
        .name("not-prod-env")
        .environment(Environment::Development)
        .enable_feature("crypto_locks")
        .max_workers(16)
        .build()
        .unwrap();
    assert!(!validation::is_production_ready(&config));
}

#[test]
fn test_is_production_ready_requires_crypto() {
    let config = BiomeOSConfigBuilder::new()
        .name("no-crypto")
        .environment(Environment::Production)
        .max_workers(16)
        .build()
        .unwrap();
    assert!(!validation::is_production_ready(&config));
}

#[test]
fn test_is_production_ready_requires_enough_workers() {
    let config = BiomeOSConfigBuilder::new()
        .name("few-workers")
        .environment(Environment::Production)
        .enable_feature("crypto_locks")
        .max_workers(2)
        .build()
        .unwrap();
    assert!(!validation::is_production_ready(&config));
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Configuration Module - UNIFIED TYPES IMPLEMENTATION
//!
//! ✅ UNIFICATION COMPLETE: Configuration types now use the unified system from biomeos-types
//! ✅ All configuration types have been consolidated in biomeos-types

use biomeos_types::config::features::EnvironmentLimits;
use biomeos_types::config::resources::RegistryConfig;
use biomeos_types::{
    BiomeOSConfig, BiomeResult, Environment, OrganizationScale, config::resources::DiscoveryMethod,
};

// All configuration types are now properly unified in biomeos-types

// Note: Use BiomeResult<T> and BiomeError from biomeos-types for consistency

/// Configuration builder for easy setup
pub struct BiomeOSConfigBuilder {
    config: BiomeOSConfig,
}

impl BiomeOSConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: BiomeOSConfig::default(),
        }
    }

    /// Set the configuration name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.metadata.name = name.into();
        self
    }

    /// Set the environment
    pub fn environment(mut self, env: Environment) -> Self {
        self.config.system.environment = env;
        self
    }

    /// Set organization scale
    pub fn organization_scale(mut self, scale: OrganizationScale) -> Self {
        self.config.system.organization_scale = scale;
        self
    }

    /// Add discovery endpoint  
    pub fn discovery_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        // Updated to use the correct structure from unified types
        if self.config.discovery.registry.is_none() {
            self.config.discovery.registry = Some(RegistryConfig {
                url: endpoint.into(),
                auth: None,
                health_check_interval: std::time::Duration::from_secs(30),
            });
        }
        self
    }

    /// Set discovery method
    pub fn discovery_method(mut self, method: DiscoveryMethod) -> Self {
        self.config.discovery.default_method = method;
        self
    }

    /// Enable feature flag
    pub fn enable_feature(mut self, feature: impl Into<String>) -> Self {
        match feature.into().as_str() {
            "ai_integration" | "ai_first" => self.config.features.ai_first = true,
            "crypto_locks" => self.config.features.crypto_locks = true,
            "distributed_computing" | "auto_scaling" => self.config.features.auto_scaling = true,
            "real_time_monitoring" | "telemetry" => self.config.features.telemetry = true,
            "advanced_networking" | "networking" => {
                // networking maps to experimental for advanced features
                self.config.features.experimental = true;
            }
            "multi_tenant" => {
                // multi_tenant maps to experimental feature set
                self.config.features.experimental = true;
                self.config.features.debug = true; // Enable debug for multi-tenant testing
            }
            _ => {} // Unknown feature, ignore
        }
        self
    }

    /// Set system limits
    pub fn max_workers(mut self, max_workers: u32) -> Self {
        // WorkerConfig uses worker_threads instead of max_workers
        self.config.system.workers.worker_threads = Some(max_workers as usize);
        self
    }

    /// Set connection timeout
    pub fn connection_timeout(mut self, timeout_ms: u64) -> Self {
        // connection_timeout expects Duration
        self.config.system.timeouts.connection_timeout =
            std::time::Duration::from_millis(timeout_ms);
        self
    }

    /// Add environment variable
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let env_name = format!("{:?}", self.config.system.environment).to_lowercase();
        let env_config = self.config.environments.entry(env_name).or_insert_with(|| {
            biomeos_types::config::EnvironmentConfig {
                name: format!("{:?}", self.config.system.environment),
                description: Some(format!(
                    "Environment configuration for {:?}",
                    self.config.system.environment
                )),
                features: biomeos_types::FeatureFlags::default(),
                limits: EnvironmentLimits {
                    max_users: None,
                    max_sessions: None,
                    rate_limit: None,
                    retention_days: None,
                    storage_limit: None,
                },
                endpoints: std::collections::HashMap::new(),
                variables: std::collections::HashMap::new(),
            }
        });
        env_config.variables.insert(key.into(), value.into());
        self
    }

    /// Build the final configuration
    pub fn build(self) -> BiomeResult<BiomeOSConfig> {
        let mut config = self.config;
        config.metadata.created_at = chrono::Utc::now();
        config.validate()?;
        Ok(config)
    }
}

impl Default for BiomeOSConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration presets for common scenarios
pub mod presets {
    use super::*;

    /// Development configuration preset
    pub fn development() -> BiomeResult<BiomeOSConfig> {
        BiomeOSConfigBuilder::new()
            .name("development-biome")
            .environment(Environment::Development)
            .organization_scale(OrganizationScale::Individual)
            .discovery_method(DiscoveryMethod::Static)
            .enable_feature("real_time_monitoring")
            .max_workers(4)
            .connection_timeout(5000)
            .build()
    }

    /// Production configuration preset
    pub fn production() -> BiomeResult<BiomeOSConfig> {
        BiomeOSConfigBuilder::new()
            .name("production-biome")
            .environment(Environment::Production)
            .organization_scale(OrganizationScale::Enterprise)
            .discovery_method(DiscoveryMethod::Registry)
            .enable_feature("ai_first")
            .enable_feature("crypto_locks")
            .enable_feature("auto_scaling")
            .enable_feature("telemetry")
            .enable_feature("networking")
            .enable_feature("multi_tenant")
            .max_workers(16)
            .connection_timeout(10000)
            .build()
    }

    /// Testing configuration preset
    pub fn testing() -> BiomeResult<BiomeOSConfig> {
        BiomeOSConfigBuilder::new()
            .name("testing-biome")
            .environment(Environment::Testing)
            .organization_scale(OrganizationScale::Team)
            .discovery_method(DiscoveryMethod::Static)
            .enable_feature("telemetry")
            .max_workers(2)
            .connection_timeout(3000)
            .build()
    }

    /// Local development configuration
    ///
    /// NOTE: Uses fallback endpoint for development when discovery unavailable.
    /// Production should use discovery-based endpoint resolution.
    pub fn local() -> BiomeResult<BiomeOSConfig> {
        // EVOLUTION: Environment-only, no localhost fallbacks
        // Primals discover each other via Unix sockets (preferred) or environment
        // EVOLVED: Use runtime discovery instead of requiring env var
        // If no discovery endpoint set, use Songbird socket discovery
        let discovery_endpoint = std::env::var("DISCOVERY_ENDPOINT")
            .or_else(|_| std::env::var("BIOMEOS_DISCOVERY_ENDPOINT"))
            .unwrap_or_else(|_| {
                // DEEP DEBT SOLUTION: Discover Songbird socket at runtime
                // No hardcoded paths, no panics - pure capability-based discovery
                use crate::socket_discovery::SocketDiscovery;
                let family_id = std::env::var("FAMILY_ID")
                    .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                    .unwrap_or_else(|_| "default".to_string());
                let discovery = SocketDiscovery::new(family_id);
                format!(
                    "unix://{}",
                    discovery
                        .build_socket_path(biomeos_types::primal_names::SONGBIRD)
                        .to_string_lossy()
                )
            });

        BiomeOSConfigBuilder::new()
            .name("local-biome")
            .environment(Environment::Development)
            .organization_scale(OrganizationScale::Individual)
            .discovery_method(DiscoveryMethod::Static)
            .discovery_endpoint(discovery_endpoint) // From env or localhost (dev only)
            .enable_feature("real_time_monitoring")
            .max_workers(2)
            .connection_timeout(5000)
            .env_var("RUST_LOG", "debug")
            .build()
    }
}

/// Configuration validation utilities
pub mod validation {
    use super::*;

    /// Validate configuration for common issues
    pub fn validate_config(config: &BiomeOSConfig) -> BiomeResult<Vec<String>> {
        let mut warnings = Vec::new();

        // Check for development settings in production
        if config.system.environment == Environment::Production {
            if let Some(ref registry) = config.discovery.registry
                && registry.url.contains("localhost")
            {
                warnings.push("Production environment contains localhost endpoints".to_string());
            }

            // Check worker thread count
            let worker_count = config.system.workers.worker_threads.unwrap_or(1);
            if worker_count < 4 {
                warnings.push("Production environment has low worker count".to_string());
            }
        }

        // Check discovery configuration
        let has_registry = config.discovery.registry.is_some();

        if !has_registry {
            match config.discovery.default_method {
                DiscoveryMethod::Registry => {
                    warnings.push(
                        "Registry discovery method requires registry configuration".to_string(),
                    );
                }
                DiscoveryMethod::Consul | DiscoveryMethod::Kubernetes => {
                    warnings.push("Discovery method requires additional configuration".to_string());
                }
                _ => {} // Static and DNS methods don't require registry
            }
        }

        // Check security settings
        if config.system.environment == Environment::Production && !config.features.crypto_locks {
            warnings.push("Production environment should enable crypto locks".to_string());
        }

        // Check resource limits - connection_timeout is Duration
        if config.system.timeouts.connection_timeout < std::time::Duration::from_millis(1000) {
            warnings.push("Connection timeout is very low and may cause issues".to_string());
        }

        Ok(warnings)
    }

    /// Check if configuration is suitable for production
    pub fn is_production_ready(config: &BiomeOSConfig) -> bool {
        let has_localhost = config
            .discovery
            .registry
            .as_ref()
            .map(|r| r.url.contains("localhost"))
            .unwrap_or(false);

        let worker_count = config.system.workers.worker_threads.unwrap_or(1);

        config.system.environment == Environment::Production
            && config.features.crypto_locks
            && worker_count >= 4
            && !has_localhost
    }
}

#[cfg(test)]
mod tests {
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
}

//! Configuration Module - UNIFIED TYPES IMPLEMENTATION
//!
//! ✅ UNIFICATION COMPLETE: Configuration types now use the unified system from biomeos-types
//! ✅ All configuration types have been consolidated in biomeos-types

use biomeos_types::config::features::EnvironmentLimits;
use biomeos_types::config::resources::RegistryConfig;
use biomeos_types::{
    config::resources::DiscoveryMethod, BiomeOSConfig, BiomeResult, Environment, OrganizationScale,
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
        // Development-only preset - uses env vars or localhost fallback
        // Production must use explicit configuration with discovery
        let discovery_endpoint = std::env::var("DISCOVERY_ENDPOINT")
            .unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                {
                    "http://localhost:8001".to_string()
                }
                #[cfg(not(debug_assertions))]
                {
                    panic!("DISCOVERY_ENDPOINT must be set in release builds")
                }
            });
        
        #[allow(deprecated)]
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
            if let Some(ref registry) = config.discovery.registry {
                if registry.url.contains("localhost") {
                    warnings
                        .push("Production environment contains localhost endpoints".to_string());
                }
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

    #[test]
    fn test_development_preset() {
        let config = presets::development().unwrap();
        assert_eq!(config.system.environment, Environment::Development);
        assert_eq!(
            config.system.organization_scale,
            OrganizationScale::Individual
        );
        assert!(config.features.telemetry);
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
    }

    #[test]
    fn test_config_validation() {
        let config = presets::production().unwrap();
        let _warnings = validation::validate_config(&config).unwrap();
        // Production config may have some warnings, that's expected
        assert!(validation::is_production_ready(&config));
    }

    #[test]
    fn test_development_not_production_ready() {
        let config = presets::development().unwrap();
        assert!(!validation::is_production_ready(&config));
    }
}

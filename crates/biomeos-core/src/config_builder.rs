// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` Configuration Builder - REWRITTEN FOR UNIFIED TYPES
//!
//! ✅ MIGRATION COMPLETE: Now uses unified types from biomeos-types
//!
//! Provides a flexible builder pattern for creating `BiomeOS` configurations
//! with customizable discovery settings and deployment-specific values.

// Import unified types from biomeos-types
use biomeos_types::constants::ports;
use biomeos_types::{
    BiomeOSConfig, Environment, OrganizationScale,
    config::{
        TlsVersion,
        features::UITheme,
        network::TlsConfig,
        resources::{DiscoveryMethod, DnsConfig, RegistryAuth, RegistryConfig},
        security::{AuthMethod, DataAtRestConfig, DataInTransitConfig, EncryptionAlgorithm},
    },
};
use std::time::Duration;
use tracing::warn;

/// Builder for creating flexible `BiomeOS` configurations using unified types
#[derive(Debug, Clone)]
pub struct BiomeOSConfigBuilder {
    /// Base configuration to build upon
    config: BiomeOSConfig,
}

impl Default for BiomeOSConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeOSConfigBuilder {
    /// Create a new configuration builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: BiomeOSConfig::default(),
        }
    }

    /// Create a builder from an existing configuration
    #[must_use]
    pub const fn from_config(config: BiomeOSConfig) -> Self {
        Self { config }
    }

    /// Configure for local development
    #[must_use]
    pub fn for_local_development() -> Self {
        let mut builder = Self::new();
        builder.config.system.environment = Environment::Development;

        // EVOLUTION: Environment-only configuration, no hardcoded fallbacks
        // Unix socket is preferred for IPC. HTTP bridge is temporary for PetalTongue transition.
        //
        // For local development, set:
        //   export BIOMEOS_BIND_ADDRESS="127.0.0.1"  # If HTTP bridge needed
        //   export BIOMEOS_UNIX_SOCKET="$XDG_RUNTIME_DIR/biomeos/biomeos.sock"  # Primary IPC
        //
        // Deep Debt Principle: Fail fast with clear guidance instead of silent hardcoded fallbacks.
        builder.config.network.bind_address =
            std::env::var(biomeos_types::env_config::vars::BIND_ADDRESS).unwrap_or_else(|_| {
                warn!("BIOMEOS_BIND_ADDRESS not set. Unix socket preferred for IPC.");
                warn!("For HTTP bridge: export BIOMEOS_BIND_ADDRESS=127.0.0.1");
                biomeos_types::constants::endpoints::DEFAULT_LOCALHOST.to_string()
            });

        builder.config.network.port = std::env::var(biomeos_types::env_config::vars::PORT)
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(ports::HTTP_BRIDGE);

        // Configure DNS discovery for local dev
        builder.config.discovery.methods = vec![DiscoveryMethod::Dns];

        builder
    }

    /// Configure for production deployment
    #[must_use]
    pub fn for_production() -> Self {
        let mut builder = Self::new();
        builder.config.system.environment = Environment::Production;
        builder.config.system.organization_scale = OrganizationScale::Enterprise;
        builder.config.network.bind_address =
            biomeos_types::constants::endpoints::production_bind_address();

        // Enable multiple discovery methods for production
        builder.config.discovery.methods = vec![
            DiscoveryMethod::Registry,
            DiscoveryMethod::Dns,
            DiscoveryMethod::Consul,
        ];

        builder
    }

    /// Configure for testing environment (test-only, not compiled into production builds).
    #[cfg(test)]
    #[must_use]
    pub fn for_testing() -> Self {
        let mut builder = Self::new();
        builder.config.system.environment = Environment::Testing;

        // EVOLUTION: Test configuration prefers Unix sockets over network
        // Tests should use Unix sockets for isolation and speed.
        //
        // For network testing (if absolutely necessary), set:
        //   export BIOMEOS_TEST_BIND="127.0.0.1"
        //   export BIOMEOS_TEST_PORT=8083
        //
        // Deep Debt Principle: Prefer Unix sockets for tests, use network only when needed.
        builder.config.network.bind_address =
            std::env::var(biomeos_types::env_config::vars::TEST_BIND).unwrap_or_else(|_| {
                warn!("BIOMEOS_TEST_BIND not set. Using Unix sockets for test isolation.");
                warn!("For network tests: export BIOMEOS_TEST_BIND=127.0.0.1");
                biomeos_types::constants::endpoints::DEFAULT_LOCALHOST.to_string()
            });

        builder.config.network.port = std::env::var(biomeos_types::env_config::vars::TEST_PORT)
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(ports::TEST_DEFAULT);

        // Use static discovery for testing
        builder.config.discovery.methods = vec![DiscoveryMethod::Dns];

        builder
    }

    /// Configure for registry-based discovery
    #[must_use]
    pub fn for_registry_discovery(registry_endpoint: &str) -> Self {
        let mut builder = Self::for_production();

        // Configure registry discovery
        builder.config.discovery.methods = vec![DiscoveryMethod::Registry];
        builder.config.discovery.registry = Some(RegistryConfig {
            url: registry_endpoint.to_string(),
            auth: None,
            health_check_interval: std::time::Duration::from_secs(30),
        });

        builder
    }

    /// Set the system environment
    #[must_use]
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.config.system.environment = environment;
        self
    }

    /// Set the organization scale
    #[must_use]
    pub const fn with_organization_scale(mut self, scale: OrganizationScale) -> Self {
        self.config.system.organization_scale = scale;
        self
    }

    /// Set the network bind address
    #[must_use]
    pub fn with_bind_address(mut self, address: &str) -> Self {
        self.config.network.bind_address = address.to_string();
        self
    }

    /// Set the network port
    #[must_use]
    pub const fn with_port(mut self, port: u16) -> Self {
        self.config.network.port = port;
        self
    }

    /// Set discovery methods
    #[must_use]
    pub fn with_discovery_methods(mut self, methods: Vec<DiscoveryMethod>) -> Self {
        self.config.discovery.methods = methods;
        self
    }

    /// Add a discovery method
    #[must_use]
    pub fn add_discovery_method(mut self, method: DiscoveryMethod) -> Self {
        // Since DiscoveryMethod doesn't implement PartialEq, we'll just add it
        // The unified config system will handle deduplication if needed
        self.config.discovery.methods.push(method);
        self
    }

    /// Configure registry discovery
    #[must_use]
    pub fn with_registry_discovery(mut self, url: &str, auth: Option<(String, String)>) -> Self {
        self.config
            .discovery
            .methods
            .push(DiscoveryMethod::Registry);
        self.config.discovery.registry = Some(RegistryConfig {
            url: url.to_string(),
            auth: auth.map(|(username, password)| RegistryAuth { username, password }),
            health_check_interval: std::time::Duration::from_secs(30),
        });
        self
    }

    /// Configure DNS discovery
    #[must_use]
    pub fn with_dns_discovery(self, servers: Vec<String>) -> Self {
        self.with_dns_discovery_domain(servers, String::new())
    }

    /// Configure DNS discovery with an explicit domain
    #[must_use]
    pub fn with_dns_discovery_domain(mut self, servers: Vec<String>, domain: String) -> Self {
        self.config.discovery.dns = Some(DnsConfig {
            servers,
            domain,
            timeout: Duration::from_secs(5),
        });
        self
    }

    /// Configure timeouts
    #[must_use]
    pub const fn with_timeouts(
        mut self,
        default_request: std::time::Duration,
        connection: std::time::Duration,
        discovery: std::time::Duration,
    ) -> Self {
        self.config.system.timeouts.default_request_timeout = default_request;
        self.config.system.timeouts.connection_timeout = connection;
        self.config.system.timeouts.discovery_timeout = discovery;
        self
    }

    /// Set data directory
    #[must_use]
    pub fn with_data_dir(mut self, path: &str) -> Self {
        self.config.system.data_dir = std::path::PathBuf::from(path);
        self
    }

    /// Set config directory
    #[must_use]
    pub fn with_config_dir(mut self, path: &str) -> Self {
        self.config.system.config_dir = std::path::PathBuf::from(path);
        self
    }

    /// Enable security features
    #[must_use]
    pub fn with_security_enabled(mut self, enabled: bool) -> Self {
        if enabled {
            // Enable authentication with API key as default
            self.config.security.authentication.default_method = AuthMethod::ApiKey;
            self.config.security.authentication.methods = vec![AuthMethod::ApiKey];

            // Enable encryption at rest and in transit
            self.config.security.encryption.at_rest = DataAtRestConfig {
                enabled: true,
                algorithm: EncryptionAlgorithm::AES256GCM,
                key_rotation_interval: Duration::from_secs(86400 * 30), // 30 days
            };
            self.config.security.encryption.in_transit = DataInTransitConfig {
                enabled: true,
                min_tls_version: "1.3".to_string(),
                cipher_suites: vec![],
            };
        } else {
            // Disable authentication
            self.config.security.authentication.default_method = AuthMethod::None;
            self.config.security.authentication.methods = vec![AuthMethod::None];

            // Disable encryption
            self.config.security.encryption.at_rest = DataAtRestConfig {
                enabled: false,
                algorithm: EncryptionAlgorithm::AES256GCM,
                key_rotation_interval: Duration::from_secs(86400 * 30), // 30 days
            };
            self.config.security.encryption.in_transit = DataInTransitConfig {
                enabled: false,
                min_tls_version: "1.2".to_string(),
                cipher_suites: vec![],
            };
        }

        self
    }

    /// Configure TLS
    #[must_use]
    pub fn with_tls(mut self, cert_file: &str, key_file: &str) -> Self {
        self.config.network.tls = Some(TlsConfig {
            enabled: true,
            cert_file: Some(std::path::PathBuf::from(cert_file)),
            key_file: Some(std::path::PathBuf::from(key_file)),
            ca_file: None,
            min_version: TlsVersion::V1_2,
            cipher_suites: vec![],
            verify_client: false,
        });
        self
    }

    /// Enable observability features
    #[must_use]
    pub const fn with_observability(mut self, enable_metrics: bool, enable_tracing: bool) -> Self {
        self.config.observability.metrics.enabled = enable_metrics;
        self.config.observability.tracing.enabled = enable_tracing;
        self
    }

    /// Enable UI dashboard
    #[must_use]
    pub const fn with_ui_enabled(mut self, enabled: bool) -> Self {
        self.config.ui.enabled = enabled;
        self
    }

    /// Set UI theme
    #[must_use]
    pub fn with_ui_theme(mut self, theme: UITheme) -> Self {
        self.config.ui.theme = theme;
        self
    }

    /// Set UI language
    #[must_use]
    pub fn with_ui_language(mut self, language: &str) -> Self {
        self.config.ui.language = language.to_string();
        self
    }

    /// Enable feature flag
    pub fn with_feature(self, _feature: &str, _enabled: bool) -> Self {
        // This is a simplified feature flag system - in reality, you'd want
        // to define specific feature flags in the FeatureFlags struct
        // Extensions field is not available in unified BiomeOSConfig
        warn!("Feature flags should be configured through the FeatureFlags struct, not extensions");
        self
    }

    /// Build the final `BiomeOS` configuration
    #[must_use]
    pub fn build(self) -> BiomeOSConfig {
        self.config
    }

    /// Get a reference to the current configuration
    #[must_use]
    pub const fn config(&self) -> &BiomeOSConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration for advanced customization
    pub const fn config_mut(&mut self) -> &mut BiomeOSConfig {
        &mut self.config
    }
}

/// Quick configuration factory functions for common use cases
impl BiomeOSConfigBuilder {
    /// Create configuration for standard local development with primals
    #[must_use]
    pub fn standard_development() -> Self {
        Self::for_local_development()
            .with_discovery_methods(vec![DiscoveryMethod::Dns])
            .with_ui_enabled(true)
            .with_ui_theme(UITheme::Dark)
            .with_observability(true, true)
    }

    /// Create configuration for distributed deployment
    #[must_use]
    pub fn distributed_deployment() -> Self {
        Self::for_production()
            .with_organization_scale(OrganizationScale::Enterprise)
            .with_security_enabled(true)
            .with_discovery_methods(vec![DiscoveryMethod::Consul, DiscoveryMethod::Kubernetes])
    }

    /// Create configuration for development with all features enabled
    #[must_use]
    pub fn development_full() -> Self {
        Self::for_local_development()
            .with_security_enabled(true)
            .with_ui_enabled(true)
            .with_ui_theme(UITheme::Dark)
            .with_ui_language("en")
            .with_observability(true, true)
            .with_timeouts(
                Duration::from_secs(30),
                Duration::from_secs(10),
                Duration::from_secs(10),
            )
            .with_feature("ai_assistance", true)
            .with_feature("advanced_monitoring", true)
    }
}

#[cfg(test)]
#[path = "config_builder_tests.rs"]
mod tests;

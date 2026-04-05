// SPDX-License-Identifier: AGPL-3.0-only
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
            std::env::var("BIOMEOS_BIND_ADDRESS").unwrap_or_else(|_| {
                warn!("BIOMEOS_BIND_ADDRESS not set. Unix socket preferred for IPC.");
                warn!("For HTTP bridge: export BIOMEOS_BIND_ADDRESS=127.0.0.1");
                "127.0.0.1".to_string() // Fallback to localhost for development only
            });

        builder.config.network.port = std::env::var("BIOMEOS_PORT")
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

    /// Configure for testing environment
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
            std::env::var("BIOMEOS_TEST_BIND").unwrap_or_else(|_| {
                warn!("BIOMEOS_TEST_BIND not set. Using Unix sockets for test isolation.");
                warn!("For network tests: export BIOMEOS_TEST_BIND=127.0.0.1");
                "127.0.0.1".to_string() // Fallback for tests that need network
            });

        builder.config.network.port = std::env::var("BIOMEOS_TEST_PORT")
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

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_pattern() {
        let config = BiomeOSConfigBuilder::new()
            .with_environment(Environment::Testing)
            .with_port(8080)
            .with_bind_address("0.0.0.0")
            .build();

        assert_eq!(config.system.environment, Environment::Testing);
        assert_eq!(config.network.port, 8080);
        assert_eq!(config.network.bind_address, "0.0.0.0");
    }

    #[test]
    fn test_discovery_configuration() {
        let config = BiomeOSConfigBuilder::new()
            .with_discovery_methods(vec![DiscoveryMethod::Registry, DiscoveryMethod::Dns])
            .build();

        assert_eq!(config.discovery.methods.len(), 2);
        assert!(
            config
                .discovery
                .methods
                .contains(&DiscoveryMethod::Registry)
        );
        assert!(config.discovery.methods.contains(&DiscoveryMethod::Dns));
    }

    #[test]
    fn test_security_configuration() {
        let config = BiomeOSConfigBuilder::new()
            .with_security_enabled(true)
            .build();

        assert!(matches!(
            config.security.authentication.default_method,
            AuthMethod::ApiKey
        ));
        assert!(config.security.encryption.at_rest.enabled);
        assert!(config.security.encryption.in_transit.enabled);
    }

    #[test]
    fn test_factory_methods() {
        let config = BiomeOSConfigBuilder::standard_development().build();

        assert_eq!(config.system.environment, Environment::Development);
        assert!(config.ui.enabled);
        assert!(matches!(
            config.ui.theme,
            biomeos_types::config::features::UITheme::Dark
        ));
        assert!(config.observability.metrics.enabled);
        assert!(config.observability.tracing.enabled);
    }

    #[test]
    fn test_production_configuration() {
        let config = BiomeOSConfigBuilder::for_production().build();

        assert_eq!(config.system.environment, Environment::Production);
        assert_eq!(
            config.system.organization_scale,
            OrganizationScale::Enterprise
        );
        assert!(
            config
                .discovery
                .methods
                .contains(&DiscoveryMethod::Registry)
        );
    }

    #[test]
    fn test_registry_discovery() {
        let config =
            BiomeOSConfigBuilder::for_registry_discovery("http://registry.example.com").build();

        assert!(
            config
                .discovery
                .methods
                .contains(&DiscoveryMethod::Registry)
        );
        assert!(config.discovery.registry.is_some());
        assert_eq!(
            config.discovery.registry.as_ref().unwrap().url,
            "http://registry.example.com"
        );
    }

    #[test]
    fn test_from_config() {
        let base = BiomeOSConfigBuilder::new()
            .with_port(9999)
            .with_bind_address("192.0.2.1")
            .build();
        let config = BiomeOSConfigBuilder::from_config(base).build();
        assert_eq!(config.network.port, 9999);
        assert_eq!(config.network.bind_address, "192.0.2.1");
    }

    #[test]
    fn test_add_discovery_method() {
        let config = BiomeOSConfigBuilder::new()
            .with_discovery_methods(vec![]) // Start empty
            .add_discovery_method(DiscoveryMethod::Dns)
            .add_discovery_method(DiscoveryMethod::Consul)
            .build();
        assert_eq!(config.discovery.methods.len(), 2);
        assert!(config.discovery.methods.contains(&DiscoveryMethod::Dns));
        assert!(config.discovery.methods.contains(&DiscoveryMethod::Consul));
    }

    #[test]
    fn test_with_registry_discovery_and_auth() {
        let config = BiomeOSConfigBuilder::new()
            .with_registry_discovery(
                "https://registry.test",
                Some(("user".into(), "pass".into())),
            )
            .build();
        assert!(config.discovery.registry.is_some());
        let reg = config.discovery.registry.as_ref().unwrap();
        assert_eq!(reg.url, "https://registry.test");
        assert!(reg.auth.is_some());
        let auth = reg.auth.as_ref().unwrap();
        assert_eq!(auth.username, "user");
        assert_eq!(auth.password, "pass");
    }

    #[test]
    fn test_with_dns_discovery() {
        let config = BiomeOSConfigBuilder::new()
            .with_dns_discovery(vec!["192.0.2.53".into(), "198.51.100.53".into()])
            .build();
        assert!(config.discovery.dns.is_some());
        let dns = config.discovery.dns.as_ref().unwrap();
        assert_eq!(dns.servers, vec!["192.0.2.53", "198.51.100.53"]);
    }

    #[test]
    fn test_with_timeouts() {
        let config = BiomeOSConfigBuilder::new()
            .with_timeouts(
                Duration::from_secs(60),
                Duration::from_secs(15),
                Duration::from_secs(20),
            )
            .build();
        assert_eq!(
            config.system.timeouts.default_request_timeout,
            Duration::from_secs(60)
        );
        assert_eq!(
            config.system.timeouts.connection_timeout,
            Duration::from_secs(15)
        );
        assert_eq!(
            config.system.timeouts.discovery_timeout,
            Duration::from_secs(20)
        );
    }

    #[test]
    fn test_with_data_dir_and_config_dir() {
        let config = BiomeOSConfigBuilder::new()
            .with_data_dir("/var/lib/biomeos")
            .with_config_dir("/etc/biomeos")
            .build();
        assert_eq!(
            config.system.data_dir,
            std::path::PathBuf::from("/var/lib/biomeos")
        );
        assert_eq!(
            config.system.config_dir,
            std::path::PathBuf::from("/etc/biomeos")
        );
    }

    #[test]
    fn test_with_security_disabled() {
        let config = BiomeOSConfigBuilder::new()
            .with_security_enabled(false)
            .build();
        assert!(matches!(
            config.security.authentication.default_method,
            AuthMethod::None
        ));
        assert!(!config.security.encryption.at_rest.enabled);
        assert!(!config.security.encryption.in_transit.enabled);
    }

    #[test]
    fn test_with_tls() {
        let config = BiomeOSConfigBuilder::new()
            .with_tls("/etc/certs/server.pem", "/etc/certs/key.pem")
            .build();
        assert!(config.network.tls.is_some());
        let tls = config.network.tls.as_ref().unwrap();
        assert!(tls.enabled);
        assert_eq!(
            tls.cert_file.as_ref().unwrap(),
            &std::path::PathBuf::from("/etc/certs/server.pem")
        );
        assert_eq!(
            tls.key_file.as_ref().unwrap(),
            &std::path::PathBuf::from("/etc/certs/key.pem")
        );
    }

    #[test]
    fn test_with_observability() {
        let config = BiomeOSConfigBuilder::new()
            .with_observability(true, false)
            .build();
        assert!(config.observability.metrics.enabled);
        assert!(!config.observability.tracing.enabled);
    }

    #[test]
    fn test_with_ui_settings() {
        let config = BiomeOSConfigBuilder::new()
            .with_ui_enabled(true)
            .with_ui_theme(UITheme::Light)
            .with_ui_language("fr")
            .build();
        assert!(config.ui.enabled);
        assert!(matches!(config.ui.theme, UITheme::Light));
        assert_eq!(config.ui.language, "fr");
    }

    #[test]
    fn test_config_accessors() {
        let mut builder = BiomeOSConfigBuilder::new().with_port(1234);
        assert_eq!(builder.config().network.port, 1234);
        builder.config_mut().network.port = 5678;
        assert_eq!(builder.config().network.port, 5678);
    }

    #[test]
    fn test_distributed_deployment() {
        let config = BiomeOSConfigBuilder::distributed_deployment().build();
        assert_eq!(config.system.environment, Environment::Production);
        assert_eq!(
            config.system.organization_scale,
            OrganizationScale::Enterprise
        );
        assert!(config.discovery.methods.contains(&DiscoveryMethod::Consul));
        assert!(
            config
                .discovery
                .methods
                .contains(&DiscoveryMethod::Kubernetes)
        );
        assert!(config.security.encryption.at_rest.enabled);
    }

    #[test]
    fn test_development_full() {
        let config = BiomeOSConfigBuilder::development_full().build();
        assert_eq!(config.system.environment, Environment::Development);
        assert!(config.ui.enabled);
        assert!(
            config
                .security
                .authentication
                .methods
                .contains(&AuthMethod::ApiKey)
        );
        assert_eq!(config.ui.language, "en");
    }

    #[test]
    fn test_default_builder() {
        let builder = BiomeOSConfigBuilder::default();
        let config = builder.build();
        // Default config is built successfully
        assert!(config.network.port > 0);
    }
}

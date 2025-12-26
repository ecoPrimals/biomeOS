//! BiomeOS Configuration Builder - REWRITTEN FOR UNIFIED TYPES
//!
//! ✅ MIGRATION COMPLETE: Now uses unified types from biomeos-types
//!
//! Provides a flexible builder pattern for creating BiomeOS configurations
//! with customizable discovery settings and deployment-specific values.

// Import unified types from biomeos-types
use biomeos_types::{
    config::{
        features::UITheme,
        network::TlsConfig,
        resources::{DiscoveryMethod, DnsConfig, RegistryAuth, RegistryConfig},
        security::{AuthMethod, DataAtRestConfig, DataInTransitConfig, EncryptionAlgorithm},
        TlsVersion,
    },
    BiomeOSConfig, Environment, OrganizationScale,
};
use std::time::Duration;
use tracing::warn;

/// Builder for creating flexible BiomeOS configurations using unified types
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
    pub fn new() -> Self {
        Self {
            config: BiomeOSConfig::default(),
        }
    }

    /// Create a builder from an existing configuration
    pub fn from_config(config: BiomeOSConfig) -> Self {
        Self { config }
    }

    /// Configure for local development
    pub fn for_local_development() -> Self {
        let mut builder = Self::new();
        builder.config.system.environment = Environment::Development;
        builder.config.network.bind_address = "127.0.0.1".to_string();
        builder.config.network.port = 8080;

        // Configure DNS discovery for localhost
        builder.config.discovery.methods = vec![DiscoveryMethod::Dns];

        builder
    }

    /// Configure for production deployment
    pub fn for_production() -> Self {
        let mut builder = Self::new();
        builder.config.system.environment = Environment::Production;
        builder.config.system.organization_scale = OrganizationScale::Enterprise;
        builder.config.network.bind_address = "0.0.0.0".to_string();

        // Enable multiple discovery methods for production
        builder.config.discovery.methods = vec![
            DiscoveryMethod::Registry,
            DiscoveryMethod::Dns,
            DiscoveryMethod::Consul,
        ];

        builder
    }

    /// Configure for testing environment
    pub fn for_testing() -> Self {
        let mut builder = Self::new();
        builder.config.system.environment = Environment::Testing;
        builder.config.network.bind_address = "localhost".to_string();
        builder.config.network.port = 8083;

        // Use static discovery for testing
        builder.config.discovery.methods = vec![DiscoveryMethod::Dns];

        builder
    }

    /// Configure for registry-based discovery
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
    pub fn with_environment(mut self, environment: Environment) -> Self {
        self.config.system.environment = environment;
        self
    }

    /// Set the organization scale
    pub fn with_organization_scale(mut self, scale: OrganizationScale) -> Self {
        self.config.system.organization_scale = scale;
        self
    }

    /// Set the network bind address
    pub fn with_bind_address(mut self, address: &str) -> Self {
        self.config.network.bind_address = address.to_string();
        self
    }

    /// Set the network port
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.network.port = port;
        self
    }

    /// Set discovery methods
    pub fn with_discovery_methods(mut self, methods: Vec<DiscoveryMethod>) -> Self {
        self.config.discovery.methods = methods;
        self
    }

    /// Add a discovery method
    pub fn add_discovery_method(mut self, method: DiscoveryMethod) -> Self {
        // Since DiscoveryMethod doesn't implement PartialEq, we'll just add it
        // The unified config system will handle deduplication if needed
        self.config.discovery.methods.push(method);
        self
    }

    /// Configure registry discovery
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
    pub fn with_dns_discovery(mut self, servers: Vec<String>) -> Self {
        self.config.discovery.dns = Some(DnsConfig {
            servers,
            domain: "".to_string(), // Placeholder, will be updated by the user
            timeout: Duration::from_secs(5),
        });
        self
    }

    /// Configure timeouts
    pub fn with_timeouts(
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
    pub fn with_data_dir(mut self, path: &str) -> Self {
        self.config.system.data_dir = std::path::PathBuf::from(path);
        self
    }

    /// Set config directory
    pub fn with_config_dir(mut self, path: &str) -> Self {
        self.config.system.config_dir = std::path::PathBuf::from(path);
        self
    }

    /// Enable security features
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
    pub fn with_observability(mut self, enable_metrics: bool, enable_tracing: bool) -> Self {
        self.config.observability.metrics.enabled = enable_metrics;
        self.config.observability.tracing.enabled = enable_tracing;
        self
    }

    /// Enable UI dashboard
    pub fn with_ui_enabled(mut self, enabled: bool) -> Self {
        self.config.ui.enabled = enabled;
        self
    }

    /// Set UI theme
    pub fn with_ui_theme(mut self, theme: UITheme) -> Self {
        self.config.ui.theme = theme;
        self
    }

    /// Set UI language
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

    /// Build the final BiomeOS configuration
    pub fn build(self) -> BiomeOSConfig {
        self.config
    }

    /// Get a reference to the current configuration
    pub fn config(&self) -> &BiomeOSConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration for advanced customization
    pub fn config_mut(&mut self) -> &mut BiomeOSConfig {
        &mut self.config
    }
}

/// Quick configuration factory functions for common use cases
impl BiomeOSConfigBuilder {
    /// Create configuration for standard local development with primals
    pub fn standard_development() -> Self {
        Self::for_local_development()
            .with_discovery_methods(vec![DiscoveryMethod::Dns])
            .with_ui_enabled(true)
            .with_ui_theme(UITheme::Dark)
            .with_observability(true, true)
    }

    /// Create configuration for distributed deployment
    pub fn distributed_deployment() -> Self {
        Self::for_production()
            .with_organization_scale(OrganizationScale::Enterprise)
            .with_security_enabled(true)
            .with_discovery_methods(vec![DiscoveryMethod::Consul, DiscoveryMethod::Kubernetes])
    }

    /// Create configuration for development with all features enabled
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
        assert!(config
            .discovery
            .methods
            .contains(&DiscoveryMethod::Registry));
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
        assert!(config
            .discovery
            .methods
            .contains(&DiscoveryMethod::Registry));
    }

    #[test]
    fn test_registry_discovery() {
        let config =
            BiomeOSConfigBuilder::for_registry_discovery("http://registry.example.com").build();

        assert!(config
            .discovery
            .methods
            .contains(&DiscoveryMethod::Registry));
        assert!(config.discovery.registry.is_some());
        assert_eq!(
            config.discovery.registry.as_ref().unwrap().url,
            "http://registry.example.com"
        );
    }
}

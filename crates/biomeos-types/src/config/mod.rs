// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unified Configuration System
//!
//! This module provides comprehensive configuration management for BiomeOS,
//! supporting all system components with proper validation and inheritance.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::{BiomeError, BiomeResult};

// Sub-modules for organized configuration
pub mod features;
pub mod network;
pub mod observability;
pub mod resources;
pub mod security;
pub mod system;

// Re-export main configuration types - using what actually exists
pub use features::{EnvironmentConfig, FeatureFlags, UIConfig};
pub use network::{HttpConfig, NetworkConfig, TlsConfig, TlsVersion};
pub use observability::{AlertingConfig, LoggingConfig, ObservabilityConfig, TracingConfig};
pub use resources::{
    CpuConfig, DiscoveryConfig, HealthMonitoringConfig, MemoryConfig, ResourceConfig,
    ResourceLimits,
};
pub use security::{AuthenticationConfig, AuthorizationConfig, EncryptionConfig, SecurityConfig};
pub use system::{
    Environment, OrganizationScale, SystemConfig, SystemLimits, TimeoutConfig, WorkerConfig,
};

#[cfg(test)]
mod mod_tests;

/// Universal biomeOS Configuration
///
/// This is the master configuration structure that unifies all configuration
/// aspects of the biomeOS ecosystem. It provides a single source of truth
/// for all system, network, security, and operational configurations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BiomeOSConfig {
    /// Configuration metadata
    #[serde(default)]
    pub metadata: ConfigMetadata,

    /// System-level configuration
    #[serde(default)]
    pub system: SystemConfig,

    /// Network configuration
    #[serde(default)]
    pub network: NetworkConfig,

    /// Security configuration  
    #[serde(default)]
    pub security: SecurityConfig,

    /// Resource management configuration
    #[serde(default)]
    pub resources: ResourceConfig,

    /// Service discovery configuration
    #[serde(default)]
    pub discovery: DiscoveryConfig,

    /// Health monitoring configuration
    #[serde(default)]
    pub health: HealthMonitoringConfig,

    /// Observability configuration
    #[serde(default)]
    pub observability: ObservabilityConfig,

    /// UI configuration
    #[serde(default)]
    pub ui: UIConfig,

    /// Environment-specific configurations
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,

    /// Feature flags
    #[serde(default)]
    pub features: FeatureFlags,
}
// BiomeOSConfig Default derived via #[derive(Default)]

/// Configuration metadata
///
/// Provides versioning, authoring, and lifecycle information for configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Configuration version using semantic versioning
    #[serde(default = "ConfigMetadata::default_version")]
    pub version: String,

    /// Configuration name/identifier  
    #[serde(default = "ConfigMetadata::default_name")]
    pub name: String,

    /// Configuration description
    #[serde(default)]
    pub description: Option<String>,

    /// Configuration author
    #[serde(default)]
    pub author: Option<String>,

    /// When this configuration was created
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,

    /// When this configuration was last modified
    #[serde(default = "Utc::now")]
    pub modified_at: DateTime<Utc>,

    /// Configuration tags for organization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Custom metadata fields
    #[serde(default)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl ConfigMetadata {
    fn default_version() -> String {
        "1.0.0".to_string()
    }

    fn default_name() -> String {
        "default-biome-config".to_string()
    }
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            version: "1.0.0".to_string(),
            name: "default-biome-config".to_string(),
            description: None,
            author: None,
            created_at: now,
            modified_at: now,
            tags: vec![],
            custom: HashMap::new(),
        }
    }
}

impl BiomeOSConfig {
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<std::path::Path>) -> BiomeResult<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            BiomeError::config_error(format!("Failed to read config file: {e}"), None::<String>)
        })?;

        let config: Self = serde_yaml::from_str(&content).map_err(|e| {
            BiomeError::config_error(format!("Failed to parse config file: {e}"), None::<String>)
        })?;

        config.validate()?;
        Ok(config)
    }

    /// Save configuration to file
    pub fn to_file(&self, path: impl AsRef<std::path::Path>) -> BiomeResult<()> {
        let content = serde_yaml::to_string(self).map_err(|e| {
            BiomeError::config_error(format!("Failed to serialize config: {e}"), None::<String>)
        })?;

        std::fs::write(path, content).map_err(|e| {
            BiomeError::config_error(format!("Failed to write config file: {e}"), None::<String>)
        })?;

        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();

        // Apply environment variable overrides
        if let Ok(port) = std::env::var("BIOMEOS_PORT") {
            if let Ok(port_num) = port.parse::<u16>() {
                config.network.port = port_num;
            }
        }

        if let Ok(bind_addr) = std::env::var("BIOMEOS_BIND_ADDRESS") {
            config.network.bind_address = bind_addr;
        }

        if let Ok(log_level) = std::env::var("BIOMEOS_LOG_LEVEL") {
            config.observability.logging.level = match log_level.to_lowercase().as_str() {
                "trace" => observability::LogLevel::Trace,
                "debug" => observability::LogLevel::Debug,
                "info" => observability::LogLevel::Info,
                "warn" => observability::LogLevel::Warn,
                "error" => observability::LogLevel::Error,
                "off" => observability::LogLevel::Off,
                _ => observability::LogLevel::Info,
            };
        }

        if let Ok(debug) = std::env::var("BIOMEOS_DEBUG") {
            config.features.debug = debug.to_lowercase() == "true";
        }

        if let Ok(experimental) = std::env::var("BIOMEOS_EXPERIMENTAL") {
            config.features.experimental = experimental.to_lowercase() == "true";
        }

        config
    }

    /// Validate the configuration
    pub fn validate(&self) -> BiomeResult<()> {
        // Validate port ranges
        if self.network.port == 0 {
            return Err(BiomeError::config_error(
                "Port cannot be 0",
                Some("network.port"),
            ));
        }

        // Validate timeouts
        if self.system.timeouts.default_request_timeout.as_secs() == 0 {
            return Err(BiomeError::config_error(
                "Request timeout cannot be 0",
                Some("system.timeouts.default_request_timeout"),
            ));
        }

        // Validate resource limits
        if self.system.limits.max_connections == 0 {
            return Err(BiomeError::config_error(
                "Max connections cannot be 0",
                Some("system.limits.max_connections"),
            ));
        }

        // Validate observability settings
        if self.observability.metrics.enabled && self.observability.metrics.interval.as_secs() == 0
        {
            return Err(BiomeError::config_error(
                "Metrics interval cannot be 0",
                Some("observability.metrics.interval"),
            ));
        }

        // Validate security settings
        if self.security.session.timeout.as_secs() == 0 {
            return Err(BiomeError::config_error(
                "Session timeout cannot be 0",
                Some("security.session.timeout"),
            ));
        }

        Ok(())
    }

    /// Merge with another configuration (other takes precedence)
    pub fn merge(&mut self, other: BiomeOSConfig) -> BiomeResult<()> {
        // Update metadata
        self.metadata.modified_at = Utc::now();
        self.metadata.version = other.metadata.version;

        // Merge configurations
        self.system = other.system;
        self.network = other.network;
        self.security = other.security;
        self.resources = other.resources;
        self.discovery = other.discovery;
        self.health = other.health;
        self.observability = other.observability;
        self.ui = other.ui;
        self.features = other.features;

        // Merge environments
        for (name, env) in other.environments {
            self.environments.insert(name, env);
        }

        // Merge extensions
        for (key, value) in other.metadata.custom {
            self.metadata.custom.insert(key, value);
        }

        self.validate()?;
        Ok(())
    }

    /// Get environment-specific configuration
    ///
    /// Returns the resolved config. Environment variables from the config are
    /// stored in `metadata.custom` under `env.{KEY}` keys rather than mutating
    /// the global process environment (which is not thread-safe). Callers that
    /// spawn child processes should pass these via `Command::env()`.
    pub fn for_environment(&self, env: &str) -> BiomeResult<Self> {
        let mut config = self.clone();

        if let Some(env_config) = self.environments.get(env) {
            config.features = env_config.features.clone();

            // Store env variables in config metadata for callers to propagate
            // via Command::env() rather than mutating global process state.
            for (key, value) in &env_config.variables {
                config.metadata.custom.insert(
                    format!("env.{key}"),
                    serde_json::Value::String(value.clone()),
                );
            }

            for (service, endpoint) in &env_config.endpoints {
                config.metadata.custom.insert(
                    format!("{service}_endpoint"),
                    serde_json::Value::String(endpoint.clone()),
                );
            }

            config.metadata.modified_at = Utc::now();
        }

        Ok(config)
    }

    /// Create a configuration builder
    pub fn builder() -> BiomeOSConfigBuilder {
        BiomeOSConfigBuilder::new()
    }
}

/// Configuration builder for fluent configuration creation
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

    /// Set configuration name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.metadata.name = name.into();
        self
    }

    /// Set configuration version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.config.metadata.version = version.into();
        self
    }

    /// Set system environment
    pub fn environment(mut self, env: Environment) -> Self {
        self.config.system.environment = env;
        self
    }

    /// Set network port
    pub fn port(mut self, port: u16) -> Self {
        self.config.network.port = port;
        self
    }

    /// Set bind address
    pub fn bind_address(mut self, addr: impl Into<String>) -> Self {
        self.config.network.bind_address = addr.into();
        self
    }

    /// Enable debug mode
    pub fn debug(mut self, debug: bool) -> Self {
        self.config.features.debug = debug;
        self
    }

    /// Enable experimental features
    pub fn experimental(mut self, experimental: bool) -> Self {
        self.config.features.experimental = experimental;
        self
    }

    /// Set log level
    pub fn log_level(mut self, level: observability::LogLevel) -> Self {
        self.config.observability.logging.level = level;
        self
    }

    /// Add environment configuration
    pub fn add_environment(
        mut self,
        name: impl Into<String>,
        env_config: EnvironmentConfig,
    ) -> Self {
        self.config.environments.insert(name.into(), env_config);
        self
    }

    /// Add custom extension
    pub fn add_extension(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.metadata.custom.insert(key.into(), value);
        self
    }

    /// Build the configuration
    pub fn build(mut self) -> BiomeResult<BiomeOSConfig> {
        self.config.metadata.modified_at = Utc::now();
        self.config.validate()?;
        Ok(self.config)
    }
}

impl Default for BiomeOSConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

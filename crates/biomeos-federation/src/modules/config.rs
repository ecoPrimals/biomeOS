//! Federation Configuration Management - UNIFIED CONFIG IMPLEMENTATION
//!
//! ✅ UNIFICATION COMPLETE: Now uses the unified configuration system from biomeos-types
//! 
//! This module handles loading, validation, and management of federation configuration
//! using the comprehensive BiomeOSConfig system.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, warn};

// Use unified configuration types from biomeos-types
pub use biomeos_types::{
    BiomeOSConfig,
    SystemConfig,
    Environment,
    OrganizationScale,
    ResourceConfig,
    SecurityConfig,
    NetworkConfig,
    DiscoveryConfig,
    HealthMonitoringConfig,
    ObservabilityConfig,
    UIConfig,
    FeatureFlags,
    config::EnvironmentConfig,
    config::ConfigMetadata,
    BiomeError,
    BiomeResult,
};

/// Federation-specific configuration extensions
/// 
/// This extends the unified BiomeOSConfig with federation-specific settings
/// while maintaining compatibility with the unified system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationConfig {
    /// Base BiomeOS configuration
    pub base: BiomeOSConfig,
    
    /// Federation-specific settings
    pub federation: FederationSettings,
    
    /// Tower deployment configuration
    pub tower: TowerConfig,
    
    /// Manifest management settings
    pub manifests: ManifestSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationSettings {
    /// Federation name
    pub name: String,
    
    /// Federation domain
    pub domain: String,
    
    /// Federation port (uses unified network config as base)
    pub port: u16,
    
    /// SSL enabled flag
    pub ssl_enabled: bool,
    
    /// Federation-specific feature flags
    pub features: FederationFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerConfig {
    /// Deployment path
    pub deployment_path: PathBuf,
    
    /// Backup path
    pub backup_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSettings {
    /// Template directories
    pub template_dirs: Vec<PathBuf>,
    
    /// Custom manifest directory
    pub custom_dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationFeatures {
    /// Enable distributed deployment
    pub distributed_deployment: bool,
    
    /// Enable manifest caching
    pub manifest_caching: bool,
    
    /// Enable federation discovery
    pub federation_discovery: bool,
}

impl Default for FederationConfig {
    fn default() -> Self {
        // Create base unified configuration
        let mut base_config = BiomeOSConfig::default();
        
        // Configure federation-specific defaults
        base_config.system.name = "tower-federation".to_string();
        base_config.system.environment = Environment::Production;
        base_config.system.organization_scale = OrganizationScale::Enterprise;
        
        // Configure network settings for federation
        base_config.network.http_port = 8443;
        base_config.network.https_enabled = true;
        base_config.network.bind_address = "0.0.0.0".to_string();
        
        // Configure resource limits for federation workloads
        base_config.resources.max_cpu_cores = Some(16);
        base_config.resources.max_memory_gb = Some(64);
        base_config.resources.max_storage_gb = Some(1000);
        
        // Configure security for federation
        base_config.security.tls_enabled = true;
        base_config.security.require_authentication = true;
        
        // Configure discovery for federation coordination
        base_config.discovery.enable_auto_discovery = true;
        base_config.discovery.discovery_interval_seconds = 30;
        
        // Configure health monitoring
        base_config.health.enable_health_checks = true;
        base_config.health.health_check_interval_seconds = 10;
        
        Self {
            base: base_config,
            federation: FederationSettings {
                name: "tower-federation".to_string(),
                domain: "federation.local".to_string(),
                port: 8443,
                ssl_enabled: true,
                features: FederationFeatures {
                    distributed_deployment: true,
                    manifest_caching: true,
                    federation_discovery: true,
                },
            },
            tower: TowerConfig {
                deployment_path: PathBuf::from("/opt/ecoprimal/deployments"),
                backup_path: PathBuf::from("/opt/ecoprimal/backups"),
            },
            manifests: ManifestSettings {
                template_dirs: vec![
                    PathBuf::from("/etc/ecoprimal/manifests/templates"),
                    PathBuf::from("/opt/ecoprimal/manifests/templates"),
                ],
                custom_dir: Some(PathBuf::from("/etc/ecoprimal/manifests/custom")),
            },
        }
    }
}

impl FederationConfig {
    /// Create a new federation config with custom base config
    pub fn with_base_config(base: BiomeOSConfig) -> Self {
        Self {
            base,
            ..Default::default()
        }
    }
    
    /// Get the effective HTTP port (federation port or base config port)
    pub fn effective_http_port(&self) -> u16 {
        if self.federation.port != 8443 {
            self.federation.port
        } else {
            self.base.network.http_port
        }
    }
    
    /// Check if SSL is enabled (federation setting or base config)
    pub fn is_ssl_enabled(&self) -> bool {
        self.federation.ssl_enabled || self.base.network.https_enabled
    }
    
    /// Get effective resource limits combining federation and base config
    pub fn effective_resource_limits(&self) -> &ResourceConfig {
        &self.base.resources
    }
    
    /// Convert to BiomeError for unified error handling
    pub fn to_biome_result<T>(result: Result<T>) -> BiomeResult<T> {
        result.map_err(|e| BiomeError::config_error(e.to_string(), Some("federation_config")))
    }
}

/// Load configuration from file with fallback to defaults
pub fn load_config(config_path: &PathBuf) -> BiomeResult<FederationConfig> {
    if config_path.exists() {
        let content = fs::read_to_string(config_path)
            .map_err(|e| BiomeError::config_error(
                format!("Failed to read config file: {}", config_path.display()),
                Some(format!("file: {}", config_path.display())),
            ))?;
        
        toml::from_str(&content)
            .map_err(|e| BiomeError::config_error(
                format!("Failed to parse config file: {}", e),
                Some(format!("file: {}", config_path.display())),
            ))
    } else {
        warn!("Config file not found at {}, using defaults", config_path.display());
        Ok(FederationConfig::default())
    }
}

/// Validate configuration settings using unified validation
pub fn validate_config(config: &FederationConfig) -> BiomeResult<()> {
    // Validate base configuration first
    validate_base_config(&config.base)?;
    
    // Validate federation-specific settings
    validate_federation_settings(&config.federation)?;
    
    // Validate tower configuration
    validate_tower_config(&config.tower)?;
    
    // Validate manifest settings
    validate_manifest_settings(&config.manifests)?;
    
    Ok(())
}

/// Validate base BiomeOS configuration
fn validate_base_config(config: &BiomeOSConfig) -> BiomeResult<()> {
    // Validate system configuration
    if config.system.name.is_empty() {
        return Err(BiomeError::config_error(
            "System name cannot be empty".to_string(),
            Some("system.name".to_string()),
        ));
    }
    
    // Validate resource limits
    if let Some(max_cpu) = config.resources.max_cpu_cores {
        if max_cpu == 0 {
            return Err(BiomeError::config_error(
                "max_cpu_cores must be greater than 0".to_string(),
                Some("resources.max_cpu_cores".to_string()),
            ));
        }
    }
    
    if let Some(max_memory) = config.resources.max_memory_gb {
        if max_memory == 0 {
            return Err(BiomeError::config_error(
                "max_memory_gb must be greater than 0".to_string(),
                Some("resources.max_memory_gb".to_string()),
            ));
        }
    }
    
    // Validate network configuration
    if config.network.http_port == 0 {
        return Err(BiomeError::config_error(
            "HTTP port must be greater than 0".to_string(),
            Some("network.http_port".to_string()),
        ));
    }
    
    Ok(())
}

/// Validate federation-specific settings
fn validate_federation_settings(config: &FederationSettings) -> BiomeResult<()> {
    if config.name.is_empty() {
        return Err(BiomeError::config_error(
            "Federation name cannot be empty".to_string(),
            Some("federation.name".to_string()),
        ));
    }
    
    if config.domain.is_empty() {
        return Err(BiomeError::config_error(
            "Federation domain cannot be empty".to_string(),
            Some("federation.domain".to_string()),
        ));
    }
    
    if config.port == 0 {
        return Err(BiomeError::config_error(
            "Federation port must be greater than 0".to_string(),
            Some("federation.port".to_string()),
        ));
    }
    
    Ok(())
}

/// Validate tower configuration
fn validate_tower_config(config: &TowerConfig) -> BiomeResult<()> {
    // Check if directories exist or can be created
    if let Err(e) = std::fs::create_dir_all(&config.deployment_path) {
        return Err(BiomeError::config_error(
            format!("Cannot create deployment directory: {}", e),
            Some(format!("tower.deployment_path: {}", config.deployment_path.display())),
        ));
    }
    
    if let Err(e) = std::fs::create_dir_all(&config.backup_path) {
        return Err(BiomeError::config_error(
            format!("Cannot create backup directory: {}", e),
            Some(format!("tower.backup_path: {}", config.backup_path.display())),
        ));
    }
    
    Ok(())
}

/// Validate manifest settings
fn validate_manifest_settings(config: &ManifestSettings) -> BiomeResult<()> {
    // Validate template directories exist
    for dir in &config.template_dirs {
        if !dir.exists() {
            warn!("Template directory does not exist: {}", dir.display());
        }
    }

    if let Some(custom_dir) = &config.custom_dir {
        if !custom_dir.exists() {
            warn!("Custom manifest directory does not exist: {}", custom_dir.display());
        }
    }
    
    Ok(())
}

/// Configuration builder for easy setup
pub struct FederationConfigBuilder {
    config: FederationConfig,
}

impl FederationConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: FederationConfig::default(),
        }
    }
    
    /// Set federation name
    pub fn with_federation_name(mut self, name: impl Into<String>) -> Self {
        self.config.federation.name = name.into();
        self
    }
    
    /// Set federation domain
    pub fn with_domain(mut self, domain: impl Into<String>) -> Self {
        self.config.federation.domain = domain.into();
        self
    }
    
    /// Set federation port
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.federation.port = port;
        self
    }
    
    /// Enable/disable SSL
    pub fn with_ssl(mut self, enabled: bool) -> Self {
        self.config.federation.ssl_enabled = enabled;
        self.config.base.network.https_enabled = enabled;
        self
    }
    
    /// Set deployment path
    pub fn with_deployment_path(mut self, path: PathBuf) -> Self {
        self.config.tower.deployment_path = path;
        self
    }
    
    /// Set backup path
    pub fn with_backup_path(mut self, path: PathBuf) -> Self {
        self.config.tower.backup_path = path;
        self
    }
    
    /// Set environment
    pub fn with_environment(mut self, env: Environment) -> Self {
        self.config.base.system.environment = env;
        self
    }
    
    /// Set organization scale
    pub fn with_organization_scale(mut self, scale: OrganizationScale) -> Self {
        self.config.base.system.organization_scale = scale;
        self
    }
    
    /// Build the configuration
    pub fn build(self) -> FederationConfig {
        self.config
    }
    
    /// Build and validate the configuration
    pub fn build_and_validate(self) -> BiomeResult<FederationConfig> {
        let config = self.build();
        validate_config(&config)?;
        Ok(config)
    }
}

impl Default for FederationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = FederationConfig::default();
        assert_eq!(config.federation.name, "tower-federation");
        assert_eq!(config.federation.port, 8443);
        assert!(config.federation.ssl_enabled);
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_config_builder() {
        let config = FederationConfigBuilder::new()
            .with_federation_name("test-federation")
            .with_port(9000)
            .with_ssl(false)
            .build();
        
        assert_eq!(config.federation.name, "test-federation");
        assert_eq!(config.federation.port, 9000);
        assert!(!config.federation.ssl_enabled);
    }

    #[test]
    fn test_effective_port() {
        let config = FederationConfig::default();
        assert_eq!(config.effective_http_port(), 8443);
        
        let mut config = FederationConfig::default();
        config.federation.port = 9000;
        assert_eq!(config.effective_http_port(), 9000);
    }

    #[test]
    fn test_config_validation() {
        let mut config = FederationConfig::default();
        assert!(validate_config(&config).is_ok());
        
        // Test invalid federation name
        config.federation.name = "".to_string();
        assert!(validate_config(&config).is_err());
    }
    
    #[test]
    fn test_config_file_loading() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("test-config.toml");
        
        let config_content = r#"
[base.system]
name = "test-federation"
environment = "Development"

[federation]
name = "test-federation"
domain = "test.local"
port = 9000
ssl_enabled = false

[tower]
deployment_path = "/tmp/deployments"
backup_path = "/tmp/backups"
"#;
        
        std::fs::write(&config_path, config_content).unwrap();
        
        let loaded_config = load_config(&config_path).unwrap();
        assert_eq!(loaded_config.federation.name, "test-federation");
        assert_eq!(loaded_config.federation.port, 9000);
        assert!(!loaded_config.federation.ssl_enabled);
    }
} 
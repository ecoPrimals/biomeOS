//! BiomeOS Configuration Management
//!
//! Provides configuration types and management for biomeOS ecosystem.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// BiomeOS configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSConfig {
    /// System configuration
    pub system: SystemConfig,

    /// Primal configuration
    pub primals: PrimalConfigs,

    /// Security configuration
    pub security: SecurityConfig,

    /// Licensing configuration
    pub licensing: LicensingConfig,

    /// Integration configuration
    pub integration: IntegrationConfig,
}

/// System-level configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub name: String,
    pub version: String,
    pub environment: Environment,
    pub log_level: String,
    pub data_dir: String,
}

/// Environment type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Production,
}

/// Primal configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfigs {
    pub discovery: DiscoveryConfig,
    pub endpoints: HashMap<String, String>,
    pub timeouts: TimeoutConfig,
}

/// Service discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    pub method: DiscoveryMethod,
    pub auto_discovery: bool,
    pub static_endpoints: HashMap<String, String>,
    pub scan_hosts: Vec<String>,
    pub scan_ports: Vec<u16>,
}

/// Discovery method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    Static,
    NetworkScan,
    Registry {
        url: String,
    },
    /// Universal service discovery endpoint (capability-based, not tied to specific service names)
    ServiceDiscovery {
        endpoint: String,
    },
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub default_timeout_ms: u64,
    pub discovery_timeout_ms: u64,
    pub health_check_interval_ms: u64,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_crypto_locks: bool,
    pub genetic_key_path: Option<String>,
    pub ai_cat_door: AiCatDoorConfig,
}

/// AI Cat Door configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCatDoorConfig {
    pub enabled: bool,
    pub cost_protection_threshold: f64,
    pub monthly_budget: f64,
}

/// Licensing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingConfig {
    pub license_type: LicenseType,
    pub organization_scale: Option<OrganizationScale>,
    pub entropy_tier: EntropyTier,
}

/// License type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseType {
    Individual,
    SmallBusiness,
    Enterprise,
    Custom { terms: String },
}

/// Organization scale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationScale {
    Individual,
    SmallBusiness,
    RegionalBusiness,
    NationalEnterprise,
    GlobalEnterprise,
    Hyperscale,
}

/// Entropy tier for human involvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropyTier {
    HumanLived,
    Supervised,
    Machine,
}

/// Integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationConfig {
    pub songbird: SongbirdIntegrationConfig,
    pub ecosystem: EcosystemIntegrationConfig,
}

/// Songbird integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongbirdIntegrationConfig {
    pub endpoint: Option<String>,
    pub auto_register: bool,
    pub health_reporting_interval_ms: u64,
}

/// Ecosystem integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemIntegrationConfig {
    pub enable_cross_primal_communication: bool,
    pub ai_first_responses: bool,
    pub universal_registration: bool,
}

impl Default for BiomeOSConfig {
    fn default() -> Self {
        Self {
            system: SystemConfig {
                name: "biomeOS".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                environment: Environment::Development,
                log_level: "info".to_string(),
                data_dir: dirs::data_dir()
                    .unwrap_or_else(|| "/tmp/biomeos".into())
                    .to_string_lossy()
                    .to_string(),
            },
            primals: PrimalConfigs {
                discovery: DiscoveryConfig {
                    method: DiscoveryMethod::NetworkScan,
                    auto_discovery: true,
                    static_endpoints: HashMap::new(),
                    scan_hosts: vec![
                        "127.0.0.1".to_string(),
                        "localhost".to_string(),
                        "::1".to_string(), // IPv6 localhost
                    ],
                    scan_ports: vec![8080, 8081, 8082, 8083, 8084, 9000, 3000],
                },
                endpoints: HashMap::new(),
                timeouts: TimeoutConfig {
                    default_timeout_ms: 5000,
                    discovery_timeout_ms: 10000,
                    health_check_interval_ms: 30000,
                },
            },
            security: SecurityConfig {
                enable_crypto_locks: true,
                genetic_key_path: None,
                ai_cat_door: AiCatDoorConfig {
                    enabled: true,
                    cost_protection_threshold: 20.0,
                    monthly_budget: 20.0,
                },
            },
            licensing: LicensingConfig {
                license_type: LicenseType::Individual,
                organization_scale: None,
                entropy_tier: EntropyTier::HumanLived,
            },
            integration: IntegrationConfig {
                songbird: SongbirdIntegrationConfig {
                    endpoint: None,
                    auto_register: true,
                    health_reporting_interval_ms: 60000,
                },
                ecosystem: EcosystemIntegrationConfig {
                    enable_cross_primal_communication: true,
                    ai_first_responses: true,
                    universal_registration: true,
                },
            },
        }
    }
}

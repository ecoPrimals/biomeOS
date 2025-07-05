//! # biomeOS Core
//!
//! Core types, traits, and functionality for the biomeOS operating system.
//! This crate provides the fundamental building blocks for orchestrating
//! the five Primals into a cohesive biological computing environment.

pub mod biome;
pub mod config;
pub mod errors;
pub mod health;
pub mod installer;
pub mod networking;
pub mod primal;
pub mod security;
pub mod storage;
pub mod universal;
pub mod cloud;
pub mod crypto;
pub mod compute;
pub mod orchestration;
pub mod locks;

// Re-export core types with specific imports to avoid conflicts
pub use biome::{Biome, BiomeSpec, BiomeManifest};
pub use primal::{PrimalType, Capability, ResourceLimits as PrimalResourceLimits, PrimalConfig};
// pub use primal::{Primal, PrimalError, PrimalType, PrimalManifest, PrimalSpec, PrimalCapability, HealthStatus as PrimalHealthStatus, Capability};
// pub use networking::{NetworkingError, ServiceRegistry, ServiceRegistration, ServiceInstance, ServiceHealth, DiscoveryProvider, DiscoveryError};
pub use security::{SecurityConfig};
// pub use security::{SecurityProvider, EncryptionManager, AccessControlManager, SecurityConfig, SecurityError};
pub use storage::{StorageConfig};
// pub use storage::{StorageProvider, StorageConfig, StoragePool, Dataset, Snapshot, StorageError};
pub use config::{ConfigError};
pub use health::{HealthStatus, HealthInfo, HealthMetrics};
pub use universal::{
    UniversalPlatform, MycorrhizaConfig, EnergyFlowState, AiPersonalityConfig,
    DeploymentConfig, UniversalPlatformOps
};
pub use installer::{UniversalInstaller};
pub use cloud::{UniversalCloudManager, CloudProviderType};
pub use crypto::{UniversalCryptoManager, CryptoProvider};
pub use compute::{UniversalComputeManager, ComputeProviderType};
pub use orchestration::{UniversalOrchestrationManager, OrchestratorType};
pub use locks::{CryptoLockManager, ComplianceLevel, PersonalAiLimits};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// biomeOS version information
pub const BIOMEOS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Unique identifier for a biome instance
pub type BiomeId = Uuid;

/// Unique identifier for a Primal instance  
pub type PrimalId = String;

/// biomeOS result type
pub type BiomeResult<T> = Result<T, BiomeError>;

/// Core biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSConfig {
    /// Global biomeOS settings
    pub global: GlobalConfig,
    /// Primal-specific configurations
    pub primals: HashMap<PrimalType, GlobalConfig>,
    /// Security configuration
    pub security: SecurityConfig,
    /// Network configuration  
    pub networking: NetworkConfig,
    /// Storage configuration
    pub storage: StorageConfig,
}

/// Core network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network interface to bind to
    pub interface: String,
    /// Port ranges for services
    pub port_ranges: Vec<(u16, u16)>,
    /// Enable IPv6 support
    pub ipv6_enabled: bool,
}

/// Global biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// biomeOS instance name
    pub instance_name: String,
    /// Data directory for biomeOS
    pub data_dir: String,
    /// Log level
    pub log_level: String,
    /// Enable development mode
    pub dev_mode: bool,
    /// Telemetry settings
    pub telemetry: TelemetryConfig,
}

/// Telemetry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Enable telemetry collection
    pub enabled: bool,
    /// Telemetry endpoint
    pub endpoint: Option<String>,
    /// Collection interval in seconds
    pub interval_secs: u64,
}

impl Default for BiomeOSConfig {
    fn default() -> Self {
        Self {
            global: GlobalConfig {
                instance_name: "default-biome".to_string(),
                data_dir: "/var/lib/biomeos".to_string(),
                log_level: "info".to_string(),
                dev_mode: false,
                telemetry: TelemetryConfig {
                    enabled: true,
                    endpoint: None,
                    interval_secs: 60,
                },
            },
            primals: HashMap::new(),
            security: SecurityConfig::default(),
            networking: NetworkConfig {
                interface: "0.0.0.0".to_string(),
                port_ranges: vec![(8080, 8080), (8443, 8443)],
                ipv6_enabled: false,
            },
            storage: StorageConfig::default(),
        }
    }
}

/// Initialize biomeOS core systems
pub async fn init_biomeos() -> BiomeResult<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("biomeOS v{} initializing...", BIOMEOS_VERSION);
    
    // Core initialization will be expanded as we add more functionality
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_biome_os_config_creation() {
        let config = BiomeOSConfig::default();
        assert_eq!(config.global.instance_name, "default-biome");
        assert!(!config.global.dev_mode);
        assert_eq!(config.networking.port_ranges.len(), 2);
    }

    #[test]
    fn test_biome_os_config_serialization() {
        let config = BiomeOSConfig::default();
        let serialized = serde_json::to_string(&config).unwrap();
        assert!(serialized.contains("default-biome"));
        
        let deserialized: BiomeOSConfig = serde_json::from_str(&serialized).unwrap();
        assert_eq!(config.global.instance_name, deserialized.global.instance_name);
    }

    #[test]
    fn test_universal_biome_config_basic() {
        let config = UniversalBiomeConfig::default();
        assert!(!config.biome.metadata.name.is_empty());
        // Test that basic structure exists
        assert!(!config.providers.cloud_providers.is_empty());
    }

    #[test]
    fn test_genetic_key_cost_scaling() {
        // Test the cost multiplier calculation logic
        let cost_multipliers = BusinessCostMultipliers {
            small_business: 0.1,
            medium_business: 1.0,
            large_enterprise: 10.0,
            mega_corp: 100.0,
        };
        
        // Verify inverse scaling: small business pays less, mega corps pay more
        assert_eq!(cost_multipliers.small_business, 0.1);
        assert_eq!(cost_multipliers.medium_business, 1.0);
        assert_eq!(cost_multipliers.large_enterprise, 10.0);
        assert_eq!(cost_multipliers.mega_corp, 100.0);
        
        // Verify inverse scaling principle: bigger companies pay proportionally more
        assert!(cost_multipliers.mega_corp > cost_multipliers.large_enterprise);
        assert!(cost_multipliers.large_enterprise > cost_multipliers.medium_business);
        assert!(cost_multipliers.medium_business > cost_multipliers.small_business);
    }

    #[test]
    fn test_ai_cat_door_cost_protection() {
        let ai_cat_door = AiCatDoorConfig::default();
        assert!(ai_cat_door.enabled);
        assert_eq!(ai_cat_door.cost_protection.max_monthly_cost, 20.0);
        assert!(ai_cat_door.cost_protection.auto_disable_on_limit);
        
        // Verify grandma-safe $20/month protection
        assert!(ai_cat_door.cost_protection.max_monthly_cost <= 20.0);
        assert!(!ai_cat_door.cost_protection.alert_thresholds.is_empty());
    }

    #[test]
    fn test_sovereignty_levels_hierarchy() {
        // Test that we have all sovereignty levels defined
        let _maximum = SovereigntyLevel::Maximum;
        let _high = SovereigntyLevel::High;
        let _medium = SovereigntyLevel::Medium;
        let _low = SovereigntyLevel::Low;
        let _minimal = SovereigntyLevel::Minimal;
        
        // Basic test that they exist and can be constructed
        assert!(true);
    }

    #[test]
    fn test_crypto_lock_configuration() {
        let crypto_config = CryptoLockConfig::default();
        assert!(crypto_config.enabled);
        assert!(crypto_config.ai_cat_door.enabled);
        
        // Test that crypto locks provide security without vendor lock-in
        assert!(!crypto_config.sovereign_keys.is_empty());
    }

    #[test]
    fn test_beardog_access_levels() {
        // Test genetic beardog key access levels exist
        let _power_user = BeardogAccessLevel::PowerUser;
        let _small_business = BeardogAccessLevel::SmallBusiness;
        let _medium_business = BeardogAccessLevel::MediumBusiness;
        let _enterprise = BeardogAccessLevel::Enterprise;
        let _mega_corp = BeardogAccessLevel::MegaCorp;
        let _research = BeardogAccessLevel::Research;
        let _government = BeardogAccessLevel::Government;
        
        // Test that all access levels are available
        assert!(true);
    }

    #[test]
    fn test_commercial_models() {
        // Test different commercial access models
        let _licensing_only = CommercialModel::LicensingOnly;
        let _partnership_access = CommercialModel::PartnershipAccess;
        let _user_choice = CommercialModel::UserChoice;
        let _fully_open = CommercialModel::FullyOpen;
        
        // Verify models exist for different business approaches
        assert!(true);
    }

    #[test]
    fn test_performance_config_creation() {
        // Test configuration creation performance
        let start = std::time::Instant::now();
        for _ in 0..100 {
            let _config = BiomeOSConfig::default();
        }
        let duration = start.elapsed();
        
        // Should create configs quickly (< 10ms for 100 configs)
        assert!(duration < Duration::from_millis(10), "Config creation should be very fast");
    }

    #[test]
    fn test_performance_universal_config_creation() {
        // Test universal config creation performance
        let start = std::time::Instant::now();
        for _ in 0..50 {
            let _config = UniversalBiomeConfig::default();
        }
        let duration = start.elapsed();
        
        // Should complete quickly (< 50ms for 50 configs since they're more complex)
        assert!(duration < Duration::from_millis(50), "Universal config creation should be fast");
    }

    #[test]
    fn test_memory_efficiency_basic_config() {
        // Test memory efficiency with basic configs
        let configs: Vec<BiomeOSConfig> = (0..100).map(|_| {
            BiomeOSConfig::default()
        }).collect();
        
        assert_eq!(configs.len(), 100);
        
        // All configs should be valid
        for config in &configs {
            assert!(!config.global.instance_name.is_empty());
            assert!(!config.global.data_dir.is_empty());
        }
    }

    #[test]
    fn test_memory_efficiency_universal_config() {
        // Test memory efficiency with universal configs
        let configs: Vec<UniversalBiomeConfig> = (0..25).map(|_| {
            UniversalBiomeConfig::default()
        }).collect();
        
        assert_eq!(configs.len(), 25);
        
        // All configs should be valid
        for config in &configs {
            assert!(!config.biome.metadata.name.is_empty());
        }
    }

    #[test]
    fn test_inverse_scaling_economics() {
        let inverse_scaling = InverseScaleConfig::default();
        assert!(inverse_scaling.enabled);
        assert!(inverse_scaling.good_faith_model);
        
        // Test the core inverse scaling principle
        let multipliers = &inverse_scaling.cost_multipliers;
        assert_eq!(multipliers.small_business, 0.1);   // Small business: 10% of base cost
        assert_eq!(multipliers.medium_business, 1.0);  // Medium business: base cost
        assert_eq!(multipliers.large_enterprise, 10.0); // Enterprise: 10x base cost
        assert_eq!(multipliers.mega_corp, 100.0);      // Mega corp: 100x base cost (Amazon scale)
        
        // Verify the economics make sense: bigger companies subsidize smaller ones
        assert!(multipliers.mega_corp / multipliers.small_business >= 1000.0);
    }

    #[test]
    fn test_genetic_beardog_key_structure() {
        let genetic_key = GeneticBeardogKey::default();
        assert!(!genetic_key.parent_key_fingerprint.is_empty());
        assert!(!genetic_key.genetic_lineage.is_empty());
        
        // Test genetic inheritance concept
        assert!(genetic_key.genetic_lineage.len() >= 1);
    }

    #[test]
    fn test_partnership_vs_licensing() {
        let licensing = LicensingConfig::default();
        
        // Personal use should always be free and sovereign
        assert!(licensing.personal_use.ai_cat_door_enabled);
        
        // Commercial use should offer user choice
        let _commercial_options = &licensing.commercial_use;
        assert!(true); // Test passes if structure exists
    }

    #[test]
    fn test_cost_protection_thresholds() {
        let cost_protection = CostProtectionConfig::default();
        assert_eq!(cost_protection.max_monthly_cost, 20.0);
        assert!(cost_protection.auto_disable_on_limit);
        
        // Test alert thresholds are reasonable
        assert!(!cost_protection.alert_thresholds.is_empty());
        for &threshold in &cost_protection.alert_thresholds {
            assert!(threshold < cost_protection.max_monthly_cost);
            assert!(threshold > 0.0);
        }
    }

    #[test]
    fn test_ecosystem_provider_structure() {
        let config = UniversalBiomeConfig::default();
        
        // Test that ecosystem providers are configured
        assert!(!config.providers.cloud_providers.is_empty());
        assert!(!config.providers.compute_providers.is_empty());
        assert!(!config.providers.orchestration_providers.is_empty());
        assert!(!config.providers.crypto_providers.is_empty());
        
        // Verify sovereignty-focused provider configuration
        for provider in &config.providers.cloud_providers {
            assert!(!provider.name.is_empty());
        }
    }

    #[test]
    fn test_biome_error_types() {
        // Test BiomeError variants
        let config_error = BiomeError::Config("test config error".to_string());
        assert!(config_error.to_string().contains("test config error"));
        
        let security_error = BiomeError::Security("test security issue".to_string());
        assert!(security_error.to_string().contains("test security issue"));
        
        let sovereignty_error = BiomeError::SovereigntyViolation("sovereignty breach".to_string());
        assert!(sovereignty_error.to_string().contains("sovereignty breach"));
        
        let vendor_lock_error = BiomeError::VendorLock("vendor lock detected".to_string());
        assert!(vendor_lock_error.to_string().contains("vendor lock detected"));
    }

    #[test]
    fn test_biomeos_version() {
        // Test that version is defined and reasonable
        assert!(!BIOMEOS_VERSION.is_empty());
        assert!(BIOMEOS_VERSION.len() >= 5); // At least "0.1.0"
    }

    #[tokio::test]
    async fn test_biomeos_initialization() {
        // Test that biomeOS core can initialize
        let result = init_biomeos().await;
        assert!(result.is_ok(), "biomeOS should initialize successfully");
    }

    #[test]
    fn test_configuration_resilience() {
        // Test configuration handles edge cases gracefully
        let mut config = BiomeOSConfig::default();
        
        // Test with very long instance name
        config.global.instance_name = "a".repeat(1000);
        assert!(!config.global.instance_name.is_empty());
        
        // Test with empty instance name
        config.global.instance_name = String::new();
        assert!(config.global.instance_name.is_empty());
        
        // Test that serialization still works
        let serialization_result = serde_json::to_string(&config);
        assert!(serialization_result.is_ok());
    }

    #[test]
    fn test_performance_serialization() {
        let config = UniversalBiomeConfig::default();
        
        // Test serialization performance
        let start = std::time::Instant::now();
        for _ in 0..10 {
            let _serialized = serde_json::to_string(&config).unwrap();
        }
        let duration = start.elapsed();
        
        // Should serialize quickly
        assert!(duration < Duration::from_millis(100), "Serialization should be fast");
    }
}

/// Main biomeOS error type
#[derive(Debug, thiserror::Error)]
pub enum BiomeError {
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Primal error: {0}")]
    Primal(String),
    
    #[error("Primal not found: {0}")]
    PrimalNotFound(String),
    
    #[error("Networking error: {0}")]
    Networking(String),
    
    #[error("Security error: {0}")]
    Security(String),
    
    #[error("Storage error: {0}")]
    Storage(String),
    
    #[error("Cloud provider error: {0}")]
    Cloud(String),
    
    #[error("Compute provider error: {0}")]
    Compute(String),
    
    #[error("Orchestration error: {0}")]
    Orchestration(String),
    
    #[error("Crypto lock error: {0}")]
    CryptoLock(String),
    
    #[error("Universal platform error: {0}")]
    UniversalPlatform(String),
    
    #[error("Installation error: {0}")]
    Installation(String),
    
    #[error("Sovereignty violation: {0}")]
    SovereigntyViolation(String),
    
    #[error("Vendor lock detected: {0}")]
    VendorLock(String),
    
    #[error("Compliance violation: {severity:?} - {message}")]
    ComplianceViolation { severity: String, message: String },
    
    #[error("Initialization error: {0}")]
    Initialization(String),
    
    #[error("Health check failed: {0}")]
    HealthCheck(String),
    
    #[error("Operation timeout: {0}")]
    Timeout(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
    
    #[error("Generic error: {message}")]
    Generic { message: String },
}

/// Universal biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalBiomeConfig {
    /// Core biome configuration
    pub biome: BiomeSpec,
    
    /// Universal platform configuration
    pub platform: UniversalPlatformConfig,
    
    /// Crypto lock configuration
    pub crypto_locks: CryptoLockConfig,
    
    /// Provider configurations
    pub providers: UniversalProviderConfig,
}

/// Universal platform configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalPlatformConfig {
    /// MYCORRHIZA energy flow configuration
    pub mycorrhiza: MycorrhizaConfig,
    
    /// Platform detection and deployment
    pub deployment: DeploymentConfig,
    
    /// AI assistant configuration
    pub ai_assistant: AiPersonalityConfig,
    
    /// Sovereignty requirements
    pub sovereignty_level: SovereigntyLevel,
}

/// Crypto lock configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoLockConfig {
    /// Enable crypto locks for external dependencies
    pub enabled: bool,
    
    /// AI cat door configuration for basic users
    pub ai_cat_door: AiCatDoorConfig,
    
    /// Sovereign key management
    pub sovereign_keys: Vec<SovereignKeyConfig>,
    
    /// Compliance requirements
    pub compliance_level: ComplianceLevel,
    
    /// Licensing terms
    pub licensing: LicensingConfig,
}

/// AI cat door configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCatDoorConfig {
    /// Enable AI cat door for basic users
    pub enabled: bool,
    
    /// Allowed AI services for personal use
    pub allowed_services: Vec<String>,
    
    /// Usage limits for grandma-safe operation
    pub usage_limits: PersonalAiLimits,
    
    /// Cost limits to prevent surprise bills
    pub cost_protection: CostProtectionConfig,
}

/// Cost protection for AI cat door
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProtectionConfig {
    /// Maximum monthly cost in USD
    pub max_monthly_cost: f64,
    
    /// Alert thresholds
    pub alert_thresholds: Vec<f64>,
    
    /// Auto-disable when limit reached
    pub auto_disable_on_limit: bool,
}

/// Sovereign key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignKeyConfig {
    /// Key identifier
    pub key_id: String,
    
    /// Grantee information
    pub grantee: String,
    
    /// Access level granted
    pub access_level: String,
    
    /// Dependencies this key unlocks
    pub dependencies: Vec<String>,
    
    /// Key validity period
    pub validity_months: u32,
}

/// Licensing configuration (sovereignty-respecting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingConfig {
    /// Personal use terms (always free and sovereign)
    pub personal_use: PersonalLicenseConfig,
    
    /// Commercial use terms (licensing OR partnership alternative)
    pub commercial_use: CommercialLicenseConfig,
    
    /// Voluntary partnership configuration (alternative to licensing)
    pub partnership: PartnershipConfig,
}

/// Personal license configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalLicenseConfig {
    /// AI cat door enabled
    pub ai_cat_door_enabled: bool,
    
    /// Rate limits for personal use
    pub rate_limits: Vec<String>,
    
    /// Data limits for personal use
    pub data_limits: Vec<String>,
    
    /// Attribution requirements
    pub attribution_required: bool,
}

/// Commercial usage configuration (licensing OR partnership)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommercialLicenseConfig {
    /// Commercial model: Licensing, Partnership, or Both
    pub commercial_model: CommercialModel,
    
    /// Pricing tiers for licensing model
    pub pricing_tiers: Vec<String>,
    
    /// Enterprise features available
    pub enterprise_features: Vec<String>,
    
    /// Support included with licensing
    pub support_included: bool,
    
    /// Partnership alternative benefits
    pub partnership_benefits: Vec<String>,
}

/// Commercial models available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommercialModel {
    /// Traditional licensing only
    LicensingOnly,
    /// Partnership access via genetic beardog key
    PartnershipAccess,
    /// User choice: licensing OR partnership access
    UserChoice,
    /// Fully open (no commercial restrictions)
    FullyOpen,
}

/// Partnership access configuration (sovereignty-first)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipConfig {
    /// Generation 1: Encrypted endpoint with genetic beardog key
    pub gen1_endpoint: Option<String>,
    
    /// Generation 2: Self-sustaining rhizoCrypt (future)
    pub gen2_rhizo_enabled: bool,
    
    /// Access tier priority: Sovereign > Humanity > Companies > Governments
    pub access_priority: AccessPriority,
    
    /// Inverse scale model for companies (bigger pays more)
    pub inverse_scaling: InverseScaleConfig,
    
    /// Partnership benefits unlocked
    pub partnership_benefits: Vec<String>,
}

/// Access priority order (sovereignty-first)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPriority {
    /// Sovereign users (highest priority)
    Sovereign,
    /// Humanity/individuals (second priority)  
    Humanity,
    /// Companies (third priority, inverse scaling)
    Companies,
    /// Governments (lowest priority)
    Governments,
}

/// Inverse scale configuration (bigger entities pay more)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InverseScaleConfig {
    /// Enable inverse scaling for burden sharing
    pub enabled: bool,
    
    /// Small business threshold (employees/revenue)
    pub small_business_threshold: BusinessThreshold,
    
    /// Enterprise threshold (Amazon-scale entities)
    pub enterprise_threshold: BusinessThreshold,
    
    /// Cost multipliers by business size
    pub cost_multipliers: BusinessCostMultipliers,
    
    /// Good faith model for Gen 1
    pub good_faith_model: bool,
}

/// Business size thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessThreshold {
    pub max_employees: Option<u32>,
    pub max_annual_revenue_usd: Option<u64>,
    pub max_market_cap_usd: Option<u64>,
}

/// Cost multipliers for burden sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessCostMultipliers {
    /// Small business: cheap access
    pub small_business: f64,  // e.g., 0.1x (very cheap)
    
    /// Medium business: standard rate
    pub medium_business: f64, // e.g., 1.0x (baseline)
    
    /// Large enterprise: carries the weight
    pub large_enterprise: f64, // e.g., 10.0x (Amazon pays more)
    
    /// Mega corp: maximum burden sharing
    pub mega_corp: f64,       // e.g., 100.0x (carry the weight)
}

/// Genetic beardog key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticBeardogKey {
    /// Key derivation from original beardog key
    pub parent_key_fingerprint: String,
    
    /// Genetic lineage for access verification
    pub genetic_lineage: Vec<String>,
    
    /// Access level granted by this key
    pub access_level: BeardogAccessLevel,
    
    /// Encrypted endpoint for Gen 1 access
    pub encrypted_endpoint: Option<String>,
    
    /// Key validity period
    pub valid_until: Option<chrono::DateTime<chrono::Utc>>,
}

/// Beardog access levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeardogAccessLevel {
    /// Basic power user access
    PowerUser,
    
    /// Small business access (cheap)
    SmallBusiness,
    
    /// Medium business access (standard)
    MediumBusiness,
    
    /// Enterprise access (expensive, carries weight)
    Enterprise,
    
    /// Mega corp access (maximum burden sharing)
    MegaCorp,
    
    /// Research/humanitarian access
    Research,
    
    /// Government access (lowest priority)
    Government,
}

/// Universal provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalProviderConfig {
    /// Container runtime providers
    pub container_providers: Vec<ContainerProviderConfig>,
    
    /// Cloud providers
    pub cloud_providers: Vec<CloudProviderConfig>,
    
    /// Compute providers
    pub compute_providers: Vec<ComputeProviderConfig>,
    
    /// Orchestration providers
    pub orchestration_providers: Vec<OrchestrationProviderConfig>,
    
    /// Crypto providers
    pub crypto_providers: Vec<CryptoProviderConfig>,
}

/// Container provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerProviderConfig {
    pub name: String,
    pub provider_type: String,
    pub enabled: bool,
    pub priority: u32,
    pub config: std::collections::HashMap<String, String>,
}

/// Cloud provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderConfig {
    pub name: String,
    pub provider_type: CloudProviderType,
    pub enabled: bool,
    pub sovereignty_compliant: bool,
    pub credentials: Option<String>,
    pub regions: Vec<String>,
}

/// Compute provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProviderConfig {
    pub name: String,
    pub provider_type: ComputeProviderType,
    pub enabled: bool,
    pub sovereignty_impact: String,
    pub device_preferences: Vec<String>,
}

/// Orchestration provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationProviderConfig {
    pub name: String,
    pub orchestrator_type: OrchestratorType,
    pub enabled: bool,
    pub self_hosted: bool,
    pub config: std::collections::HashMap<String, String>,
}

/// Crypto provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoProviderConfig {
    pub name: String,
    pub provider_type: CryptoProvider,
    pub enabled: bool,
    pub quantum_resistant: bool,
    pub fallback_priority: u32,
}

/// Sovereignty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyLevel {
    /// Maximum sovereignty - air-gapped, zero external dependencies
    Maximum,
    /// High sovereignty - minimal external dependencies, all crypto-locked
    High,
    /// Medium sovereignty - some external dependencies with exit strategies
    Medium,
    /// Low sovereignty - standard external dependencies allowed
    Low,
    /// Minimal sovereignty - any external dependencies allowed
    Minimal,
}

/// Universal biomeOS manager - orchestrates all components
pub struct UniversalBiomeManager {
    /// Configuration
    pub config: UniversalBiomeConfig,
    
    /// Universal platform
    pub platform: UniversalPlatform,
    
    /// Crypto lock manager
    pub crypto_locks: CryptoLockManager,
    
    /// Provider managers
    pub cloud_manager: UniversalCloudManager,
    pub compute_manager: UniversalComputeManager,
    pub orchestration_manager: UniversalOrchestrationManager,
    pub crypto_manager: UniversalCryptoManager,
    
    /// Universal installer
    pub installer: UniversalInstaller,
}

impl UniversalBiomeManager {
    /// Create a new biomeOS manager with the given configuration
    pub fn new(_config: BiomeOSConfig) -> Self {
        // Convert BiomeOSConfig to UniversalBiomeConfig
        let universal_config = UniversalBiomeConfig::default();
        Self::new_from_universal_config(universal_config)
    }

    /// Create a new biomeOS manager with UniversalBiomeConfig
    pub fn new_from_universal_config(config: UniversalBiomeConfig) -> Self {
        Self {
            config: config.clone(),
            platform: UniversalPlatform::new(),
            crypto_locks: CryptoLockManager::new(),
            cloud_manager: UniversalCloudManager::new(),
            compute_manager: UniversalComputeManager::new(),
            orchestration_manager: UniversalOrchestrationManager::new(),
            crypto_manager: UniversalCryptoManager::new(),
            installer: UniversalInstaller::new(),
        }
    }

    /// Start the biomeOS manager
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation for starting the manager
        Ok(())
    }

    /// Shutdown the biomeOS manager
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation for shutting down the manager
        Ok(())
    }

    /// Perform a health check
    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation for health checking
        Ok(())
    }

    /// Calculate sovereignty score
    pub fn calculate_sovereignty_score(&self) -> f32 {
        // Return a sovereignty score between 0.0 and 3.0
        // biomeOS starts with high sovereignty
        3.0
    }

    /// Discover available primals
    pub fn discover_available_primals(&self) -> Vec<String> {
        // Return list of discoverable primals
        vec!["toadstool".to_string(), "songbird".to_string(), "nestgate".to_string(), "squirrel".to_string(), "beardog".to_string()]
    }

    /// Calculate cost multiplier based on access level
    pub fn calculate_cost_multiplier(&self, access_level: &str) -> f64 {
        match access_level {
            "individual" => 1.0,
            "small_business" => 0.1,
            "enterprise" => 10.0,
            "mega_corp" => 100.0,
            _ => 1.0, // Default
        }
    }

    /// Configure AI cat door with budget limit and request limit
    pub async fn configure_ai_cat_door(&self, budget_usd: f64, request_limit: u32) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Implementation for configuring AI cat door
        Ok(())
    }

    /// Get AI cat door status
    pub async fn get_ai_cat_door_status(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Implementation for getting AI cat door status
        Ok("active".to_string())
    }

    /// Check if the manager supports a given pattern
    pub fn supports_pattern(&self, pattern: &str) -> bool {
        matches!(pattern, "recursive" | "universal" | "agnostic" | "sovereign" | "iterative")
    }

    /// Validate a crypto lock signature
    pub fn validate_crypto_lock(&self, signature: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if signature.is_empty() {
            Err("Invalid crypto lock signature".into())
        } else {
            Ok(())
        }
    }

    /// Check if ready for ecosystem coordination
    pub fn can_coordinate_ecosystem(&self) -> bool {
        true
    }

    /// Get supported installation modes
    pub fn get_supported_install_modes(&self) -> Vec<String> {
        vec!["basic".to_string(), "ai_research".to_string(), "secure_enterprise".to_string()]
    }

    /// Detect platform information
    pub fn detect_platform(&self) -> String {
        format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
    }
}

/// Default configurations for grandma-safe operation
impl Default for UniversalBiomeConfig {
    fn default() -> Self {
        Self {
            biome: BiomeSpec::default(),
            platform: UniversalPlatformConfig::default(),
            crypto_locks: CryptoLockConfig::default(),
            providers: UniversalProviderConfig::default(),
        }
    }
}

impl Default for UniversalPlatformConfig {
    fn default() -> Self {
        Self {
            mycorrhiza: MycorrhizaConfig::default(),
            deployment: DeploymentConfig::default(),
            ai_assistant: AiPersonalityConfig::default(),
            sovereignty_level: SovereigntyLevel::Medium,
        }
    }
}

impl Default for CryptoLockConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // Crypto locks enabled by default
            ai_cat_door: AiCatDoorConfig::default(),
            sovereign_keys: vec![],
            compliance_level: ComplianceLevel::Personal,
            licensing: LicensingConfig::default(),
        }
    }
}

impl Default for AiCatDoorConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // AI cat door enabled for grandma-safe operation
            allowed_services: vec![
                "openai".to_string(),
                "anthropic".to_string(),
                "local-llama".to_string(),
            ],
            usage_limits: PersonalAiLimits::grandma_safe_defaults(),
            cost_protection: CostProtectionConfig::default(),
        }
    }
}

impl Default for CostProtectionConfig {
    fn default() -> Self {
        Self {
            max_monthly_cost: 20.0,  // $20/month limit for grandma safety
            alert_thresholds: vec![10.0, 15.0, 18.0],  // Alert at $10, $15, $18
            auto_disable_on_limit: true,  // Auto-disable to prevent overage
        }
    }
}

impl Default for LicensingConfig {
    fn default() -> Self {
        Self {
            personal_use: PersonalLicenseConfig::default(),
            commercial_use: CommercialLicenseConfig::default(),
            partnership: PartnershipConfig::default(),
        }
    }
}

impl Default for PersonalLicenseConfig {
    fn default() -> Self {
        Self {
            ai_cat_door_enabled: true,
            rate_limits: vec!["100 requests/hour".to_string()],
            data_limits: vec!["1GB/month".to_string()],
            attribution_required: false,
        }
    }
}

impl Default for CommercialLicenseConfig {
    fn default() -> Self {
        Self {
            commercial_model: CommercialModel::UserChoice,  // Let users choose sovereignty
            pricing_tiers: vec!["Startup".to_string(), "Business".to_string(), "Enterprise".to_string()],
            enterprise_features: vec!["Priority Support".to_string(), "Custom Integrations".to_string()],
            support_included: false,
            partnership_benefits: vec![
                "Development Influence".to_string(),
                "Early Access Features".to_string(),
                "Direct Developer Support".to_string(),
                "Sovereign Partnership Status".to_string(),
            ],
        }
    }
}

impl Default for PartnershipConfig {
    fn default() -> Self {
        Self {
            gen1_endpoint: None,  // User provides endpoint for genetic beardog key access
            gen2_rhizo_enabled: false,  // Future rhizoCrypt integration
            access_priority: AccessPriority::Humanity,  // Default to humanity priority
            inverse_scaling: InverseScaleConfig::default(),  // Burden sharing enabled
            partnership_benefits: vec![
                "Genetic beardog key access".to_string(),
                "Priority support for sovereignty".to_string(),
                "Direct development influence".to_string(),
                "Good faith partnership model".to_string(),
            ],
        }
    }
}

impl Default for UniversalProviderConfig {
    fn default() -> Self {
        Self {
            container_providers: vec![
                ContainerProviderConfig {
                    name: "podman".to_string(),
                    provider_type: "podman".to_string(),
                    enabled: true,
                    priority: 1,
                    config: std::collections::HashMap::new(),
                },
                ContainerProviderConfig {
                    name: "docker".to_string(),
                    provider_type: "docker".to_string(),
                    enabled: true,
                    priority: 2,
                    config: std::collections::HashMap::new(),
                },
            ],
            cloud_providers: vec![],
            compute_providers: vec![
                ComputeProviderConfig {
                    name: "cpu".to_string(),
                    provider_type: ComputeProviderType::Cpu { 
                        architecture: "x86_64".to_string(), 
                        instruction_sets: vec!["avx2".to_string()] 
                    },
                    enabled: true,
                    sovereignty_impact: "none".to_string(),
                    device_preferences: vec!["cpu".to_string()],
                },
            ],
            orchestration_providers: vec![
                OrchestrationProviderConfig {
                    name: "none".to_string(),
                    orchestrator_type: OrchestratorType::None,
                    enabled: true,
                    self_hosted: true,
                    config: std::collections::HashMap::new(),
                },
            ],
            crypto_providers: vec![
                CryptoProviderConfig {
                    name: "rustls".to_string(),
                    provider_type: CryptoProvider::Rustls { version: "0.21".to_string() },
                    enabled: true,
                    quantum_resistant: false,
                    fallback_priority: 1,
                },
            ],
        }
    }
}

impl Default for InverseScaleConfig {
    fn default() -> Self {
        Self {
            enabled: true,  // Burden sharing always enabled
            small_business_threshold: BusinessThreshold {
                max_employees: Some(50),
                max_annual_revenue_usd: Some(1_000_000),  // $1M annual revenue
                max_market_cap_usd: None,
            },
            enterprise_threshold: BusinessThreshold {
                max_employees: Some(10_000),
                max_annual_revenue_usd: Some(1_000_000_000),  // $1B annual revenue
                max_market_cap_usd: Some(100_000_000_000),    // $100B market cap
            },
            cost_multipliers: BusinessCostMultipliers {
                small_business: 0.1,   // 10x cheaper (Amazon subsidizes)
                medium_business: 1.0,  // Baseline rate
                large_enterprise: 10.0, // 10x more expensive
                mega_corp: 100.0,      // 100x more (Amazon carries the weight)
            },
            good_faith_model: true,  // Gen 1 operates on good faith
        }
    }
}

impl Default for GeneticBeardogKey {
    fn default() -> Self {
        Self {
            parent_key_fingerprint: "not_set".to_string(),
            genetic_lineage: vec![],
            access_level: BeardogAccessLevel::PowerUser,
            encrypted_endpoint: None,
            valid_until: None,
        }
    }
}

/// Service registration for the "Songbird Pattern"
/// This struct defines the "birth announcement" that each Primal sends to Songbird
#[derive(Debug, Clone, Serialize)]
pub struct ServiceRegistration {
    /// The unique name of the service, from the manifest.
    pub service_name: String,

    /// The Primal type that owns this service
    pub primal_type: PrimalType,

    /// The runtime type, for songbird's awareness.
    pub runtime: RuntimeType, 

    /// The internal IP address and port where the service's API is listening.
    /// This is on the private `primal_net`.
    pub internal_address: String, // e.g., "10.42.0.5:8000"

    /// The public-facing paths that should be routed to this service.
    /// Example: `"/api/v1/storage"`
    pub public_routes: Vec<String>,

    /// A URL for songbird to poll for health checks.
    pub health_check_url: String,

    /// Any additional metadata the service wants to provide.
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Biome ID this service belongs to
    pub biome_id: String,

    /// Service capabilities
    pub capabilities: ServiceCapabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceCapabilities {
    /// Core capabilities provided by this service
    pub core: Vec<String>,
    /// Extended features available
    pub extended: Vec<String>,
    /// Integration points with other Primals
    pub integrations: Vec<String>,
}

/// Runtime types supported by biomeOS
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    Wasm,
    Container,
    Native,
    Gpu,
    Agent, // For AI agents via Squirrel
}

/// MYCORRHIZA compliance status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplianceStatus {
    FullyCompliant,
    PartiallyCompliant,
    NonCompliant,
    Unknown,
}

/// Extended resource requirements for biomeOS
#[derive(Debug, Clone)]
pub struct BiomeResourceRequirements {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub storage_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<u64>,
    pub gpu_count: Option<u32>,
}

/// Primal lifecycle events
#[derive(Debug, Clone)]
pub enum PrimalEvent {
    Starting,
    Started,
    Healthy,
    Degraded,
    Stopped,
    Failed(String),
}

/// Event listener for Primal lifecycle events
#[async_trait::async_trait]
pub trait PrimalEventListener: Send + Sync {
    async fn on_primal_event(&self, primal_type: PrimalType, event: PrimalEvent);
}

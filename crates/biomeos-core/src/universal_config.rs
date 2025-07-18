//! Universal biomeOS Configuration Types
//!
//! This module contains the advanced configuration structures for universal biomeOS,
//! including crypto locks, sovereignty settings, and provider configurations.

use crate::{
    biome::BiomeSpec,
    universal::{
        energy_flow::{AiPersonalityConfig, MycorrhizaConfig},
        platform_detection::DeploymentConfig,
    },
    locks::ai_cat_door::PersonalAiLimits,
    cloud::CloudProviderType,
    compute::ComputeProviderType,
    crypto::CryptoProvider,
    locks::ComplianceLevel,
    orchestration::OrchestratorType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Universal biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    /// Enable AI cat door
    pub enabled: bool,

    /// Monthly budget limit in USD
    pub monthly_budget_usd: f64,

    /// Request limit per month
    pub monthly_request_limit: u32,

    /// Allowed AI models
    pub allowed_models: Vec<String>,

    /// Auto-approval for requests under threshold
    pub auto_approval_threshold_usd: f64,

    /// Notification settings
    pub notifications: AiNotificationConfig,
}

/// AI notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiNotificationConfig {
    /// Email notifications
    pub email_enabled: bool,

    /// SMS notifications
    pub sms_enabled: bool,

    /// Budget threshold for notifications (percentage)
    pub budget_threshold: f64,
}

/// Sovereign key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignKeyConfig {
    /// Key identifier
    pub key_id: String,

    /// Key type
    pub key_type: String,

    /// Key material (encrypted)
    pub key_material: String,

    /// Key usage permissions
    pub permissions: Vec<String>,
}

/// Licensing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingConfig {
    /// License type
    pub license_type: LicenseType,

    /// Commercial license terms
    pub commercial_terms: Option<CommercialLicenseConfig>,

    /// Personal license terms
    pub personal_terms: Option<PersonalLicenseConfig>,
}

/// License type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseType {
    Personal,
    Commercial,
    Enterprise,
}

/// Commercial license configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommercialLicenseConfig {
    /// Commercial model
    pub model: CommercialModel,

    /// Cost multipliers
    pub cost_multipliers: BusinessCostMultipliers,

    /// Partnership configuration
    pub partnerships: Vec<PartnershipConfig>,
}

/// Personal license configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalLicenseConfig {
    /// AI limits
    pub ai_limits: PersonalAiLimits,

    /// Cost protection
    pub cost_protection: CostProtectionConfig,
}

/// Commercial model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommercialModel {
    PayPerUse,
    Monthly,
    Annual,
    Enterprise,
}

/// Business cost multipliers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessCostMultipliers {
    /// Small business multiplier
    pub small_business: f64,

    /// Enterprise multiplier
    pub enterprise: f64,

    /// Mega corporation multiplier
    pub mega_corp: f64,

    /// Business thresholds
    pub thresholds: BusinessThreshold,
}

/// Business threshold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessThreshold {
    /// Revenue threshold for small business
    pub small_business_revenue: f64,

    /// Revenue threshold for enterprise
    pub enterprise_revenue: f64,

    /// Revenue threshold for mega corp
    pub mega_corp_revenue: f64,
}

/// Partnership configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipConfig {
    /// Partner name
    pub partner_name: String,

    /// Access priority
    pub access_priority: AccessPriority,

    /// BearDog access level
    pub beardog_access: BeardogAccessLevel,

    /// Genetic BearDog key
    pub genetic_key: Option<GeneticBeardogKey>,
}

/// Access priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// BearDog access level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeardogAccessLevel {
    Basic,
    Standard,
    Premium,
    Enterprise,
}

/// Genetic BearDog key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticBeardogKey {
    /// Key sequence
    pub sequence: String,

    /// Key generation
    pub generation: u32,

    /// Key fitness
    pub fitness: f64,
}

/// Cost protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostProtectionConfig {
    /// Inverse scale configuration
    pub inverse_scale: InverseScaleConfig,

    /// Maximum monthly cost
    pub max_monthly_cost: f64,

    /// Warning threshold
    pub warning_threshold: f64,
}

/// Inverse scale configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InverseScaleConfig {
    /// Enable inverse scaling
    pub enabled: bool,

    /// Scale factor
    pub scale_factor: f64,

    /// Minimum cost
    pub min_cost: f64,
}

/// Sovereignty level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyLevel {
    /// Closed system - completely sovereign
    Closed,

    /// Private open - trusted partners only
    PrivateOpen,

    /// Commercial open - pay-to-play
    CommercialOpen,

    /// Public open - open to all
    PublicOpen,
}

/// Universal provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalProviderConfig {
    /// Cloud provider configurations
    pub cloud: Vec<CloudProviderConfig>,

    /// Compute provider configurations
    pub compute: Vec<ComputeProviderConfig>,

    /// Storage provider configurations
    pub storage: Vec<StorageProviderConfig>,

    /// Orchestration provider configurations
    pub orchestration: Vec<OrchestrationProviderConfig>,

    /// Crypto provider configurations
    pub crypto: Vec<CryptoProviderConfig>,

    /// Container provider configurations
    pub container: Vec<ContainerProviderConfig>,
}

/// Cloud provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderConfig {
    /// Provider type
    pub provider_type: CloudProviderType,

    /// Provider configuration
    pub config: HashMap<String, String>,
}

/// Compute provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProviderConfig {
    /// Provider type
    pub provider_type: ComputeProviderType,

    /// Provider configuration
    pub config: HashMap<String, String>,
}

/// Storage provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProviderConfig {
    /// Provider type
    pub provider_type: String,

    /// Provider configuration
    pub config: HashMap<String, String>,
}

/// Orchestration provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationProviderConfig {
    /// Orchestrator type
    pub orchestrator_type: OrchestratorType,

    /// Provider configuration
    pub config: HashMap<String, String>,
}

/// Crypto provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoProviderConfig {
    /// Provider type
    pub provider_type: CryptoProvider,

    /// Provider configuration
    pub config: HashMap<String, String>,
}

/// Container provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerProviderConfig {
    /// Provider type
    pub provider_type: String,

    /// Provider configuration
    pub config: HashMap<String, String>,
}

impl Default for UniversalPlatformConfig {
    fn default() -> Self {
        Self {
            mycorrhiza: MycorrhizaConfig::default(),
            deployment: DeploymentConfig::default(),
            ai_assistant: AiPersonalityConfig::default(),
            sovereignty_level: SovereigntyLevel::Closed,
        }
    }
}

impl Default for CryptoLockConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            ai_cat_door: AiCatDoorConfig::default(),
            sovereign_keys: Vec::new(),
            compliance_level: ComplianceLevel::High,
            licensing: LicensingConfig::default(),
        }
    }
}

impl Default for AiCatDoorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            monthly_budget_usd: 50.0,
            monthly_request_limit: 1000,
            allowed_models: vec!["gpt-3.5-turbo".to_string()],
            auto_approval_threshold_usd: 5.0,
            notifications: AiNotificationConfig::default(),
        }
    }
}

impl Default for AiNotificationConfig {
    fn default() -> Self {
        Self {
            email_enabled: true,
            sms_enabled: false,
            budget_threshold: 0.8,
        }
    }
}

impl Default for LicensingConfig {
    fn default() -> Self {
        Self {
            license_type: LicenseType::Personal,
            commercial_terms: None,
            personal_terms: Some(PersonalLicenseConfig::default()),
        }
    }
}

impl Default for PersonalLicenseConfig {
    fn default() -> Self {
        Self {
            ai_limits: PersonalAiLimits::default(),
            cost_protection: CostProtectionConfig::default(),
        }
    }
}

impl Default for CostProtectionConfig {
    fn default() -> Self {
        Self {
            inverse_scale: InverseScaleConfig::default(),
            max_monthly_cost: 100.0,
            warning_threshold: 80.0,
        }
    }
}

impl Default for InverseScaleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            scale_factor: 0.1,
            min_cost: 0.01,
        }
    }
}

impl Default for UniversalProviderConfig {
    fn default() -> Self {
        Self {
            cloud: Vec::new(),
            compute: Vec::new(),
            storage: Vec::new(),
            orchestration: vec![OrchestrationProviderConfig {
                orchestrator_type: OrchestratorType::None,
                config: HashMap::new(),
            }],
            crypto: Vec::new(),
            container: Vec::new(),
        }
    }
}

//! Universal biomeOS Configuration Types
//!
//! This module contains the advanced configuration structures for universal biomeOS,
//! including crypto locks, sovereignty settings, and provider configurations.

use crate::{
    AiPersonalityConfig, BiomeSpec, CloudProviderType, ComplianceLevel, ComputeProviderType,
    CryptoProvider, DeploymentConfig, MycorrhizaConfig, OrchestratorType, PersonalAiLimits,
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub small_business: f64, // e.g., 0.1x (very cheap)

    /// Medium business: standard rate
    pub medium_business: f64, // e.g., 1.0x (baseline)

    /// Large enterprise: carries the weight
    pub large_enterprise: f64, // e.g., 10.0x (Amazon pays more)

    /// Mega corp: maximum burden sharing
    pub mega_corp: f64, // e.g., 100.0x (carry the weight)
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
    pub config: HashMap<String, String>,
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
    pub config: HashMap<String, String>,
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

// Default implementations for grandma-safe operation

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
            enabled: true, // Crypto locks enabled by default
            ai_cat_door: AiCatDoorConfig::default(),
            sovereign_keys: vec![SovereignKeyConfig {
                key_id: "default-sovereign-key".to_string(),
                grantee: "individual-user".to_string(),
                access_level: "personal".to_string(),
                dependencies: vec!["ai-services".to_string()],
                validity_months: 12,
            }],
            compliance_level: ComplianceLevel::Personal,
            licensing: LicensingConfig::default(),
        }
    }
}

impl Default for AiCatDoorConfig {
    fn default() -> Self {
        Self {
            enabled: true, // AI cat door enabled for grandma-safe operation
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
            max_monthly_cost: 20.0,                   // $20/month limit for grandma safety
            alert_thresholds: vec![10.0, 15.0, 18.0], // Alert at $10, $15, $18
            auto_disable_on_limit: true,              // Auto-disable to prevent overage
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
            commercial_model: CommercialModel::UserChoice, // Let users choose sovereignty
            pricing_tiers: vec![
                "Startup".to_string(),
                "Business".to_string(),
                "Enterprise".to_string(),
            ],
            enterprise_features: vec![
                "Priority Support".to_string(),
                "Custom Integrations".to_string(),
            ],
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
            gen1_endpoint: None, // User provides endpoint for genetic beardog key access
            gen2_rhizo_enabled: false, // Future rhizoCrypt integration
            access_priority: AccessPriority::Humanity, // Default to humanity priority
            inverse_scaling: InverseScaleConfig::default(), // Burden sharing enabled
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
                    config: HashMap::new(),
                },
                ContainerProviderConfig {
                    name: "docker".to_string(),
                    provider_type: "docker".to_string(),
                    enabled: true,
                    priority: 2,
                    config: HashMap::new(),
                },
            ],
            cloud_providers: vec![CloudProviderConfig {
                name: "local".to_string(),
                provider_type: CloudProviderType::SelfHosted,
                enabled: true,
                sovereignty_compliant: true,
                credentials: None,
                regions: vec!["local".to_string()],
            }],
            compute_providers: vec![ComputeProviderConfig {
                name: "cpu".to_string(),
                provider_type: ComputeProviderType::Cpu {
                    architecture: "x86_64".to_string(),
                    instruction_sets: vec!["avx2".to_string()],
                },
                enabled: true,
                sovereignty_impact: "none".to_string(),
                device_preferences: vec!["cpu".to_string()],
            }],
            orchestration_providers: vec![OrchestrationProviderConfig {
                name: "none".to_string(),
                orchestrator_type: OrchestratorType::None,
                enabled: true,
                self_hosted: true,
                config: HashMap::new(),
            }],
            crypto_providers: vec![CryptoProviderConfig {
                name: "rustls".to_string(),
                provider_type: CryptoProvider::Rustls {
                    version: "0.21".to_string(),
                },
                enabled: true,
                quantum_resistant: false,
                fallback_priority: 1,
            }],
        }
    }
}

impl Default for InverseScaleConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Burden sharing always enabled
            small_business_threshold: BusinessThreshold {
                max_employees: Some(50),
                max_annual_revenue_usd: Some(1_000_000), // $1M annual revenue
                max_market_cap_usd: None,
            },
            enterprise_threshold: BusinessThreshold {
                max_employees: Some(10_000),
                max_annual_revenue_usd: Some(1_000_000_000), // $1B annual revenue
                max_market_cap_usd: Some(100_000_000_000),   // $100B market cap
            },
            cost_multipliers: BusinessCostMultipliers {
                small_business: 0.1,    // 10x cheaper (Amazon subsidizes)
                medium_business: 1.0,   // Baseline rate
                large_enterprise: 10.0, // 10x more expensive
                mega_corp: 100.0,       // 100x more (Amazon carries the weight)
            },
            good_faith_model: true, // Gen 1 operates on good faith
        }
    }
}

impl Default for GeneticBeardogKey {
    fn default() -> Self {
        Self {
            parent_key_fingerprint: "genesis-key-fingerprint".to_string(),
            genetic_lineage: vec!["genesis".to_string(), "first-generation".to_string()],
            access_level: BeardogAccessLevel::PowerUser,
            encrypted_endpoint: None,
            valid_until: None,
        }
    }
}

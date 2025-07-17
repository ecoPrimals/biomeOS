//! MYCORRHIZA Energy Flow Management
//!
//! This module implements the MYCORRHIZA energy flow management system that protects
//! biomeOS ecosystems by controlling external access and maintaining sovereignty.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// MYCORRHIZA Energy Flow Management
/// The universal energy flow management system that protects biomeOS ecosystems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MycorrhizaConfig {
    /// Current system energy state
    pub system_state: EnergyFlowState,
    /// Personal AI configuration (always available)
    pub personal_ai: PersonalAiConfig,
    /// Trust-based external access settings
    pub trusted_externals: TrustedExternalsConfig,
    /// Commercial access settings
    pub commercial_access: CommercialAccessConfig,
    /// Security enforcement settings
    pub enforcement: EnforcementConfig,
}

/// Energy flow states for MYCORRHIZA
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnergyFlowState {
    /// Closed system - completely sovereign (default for grandma safety)
    Closed,
    /// Private open - trust-based external access
    PrivateOpen,
    /// Commercial open - pay-to-play enterprise integrations
    CommercialOpen,
}

/// Personal AI configuration (always available in all states)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAiConfig {
    /// Enable personal AI cat door
    pub enabled: bool,
    /// Local models available
    pub local_models: Vec<String>,
    /// Personal API keys for external AI (encrypted)
    pub api_keys: HashMap<String, String>,
    /// AI assistant personality/behavior
    pub personality: AiPersonalityConfig,
}

/// AI assistant personality configuration for grandma-safe interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPersonalityConfig {
    /// Helpfulness level (high for grandma-safe)
    pub helpfulness: f64,
    /// Technical complexity of explanations (low for grandma-safe)
    pub technical_complexity: f64,
    /// Proactive assistance (high for grandma-safe)
    pub proactiveness: f64,
    /// Safety warnings verbosity (high for grandma-safe)
    pub safety_verbosity: f64,
}

/// Trusted externals configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedExternalsConfig {
    /// Enable trusted external access
    pub enabled: bool,
    /// Crypto keys granted on good faith
    pub grants: Vec<TrustedGrant>,
    /// Relationship-based access controls
    pub relationships: HashMap<String, RelationshipLevel>,
}

/// Trusted grant for external access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustedGrant {
    /// Grant identifier
    pub id: String,
    /// Granted to (person/org)
    pub granted_to: String,
    /// Crypto key for access
    pub crypto_key: String,
    /// Access level granted
    pub access_level: AccessLevel,
    /// Grant expiration
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Relationship levels for trust-based access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipLevel {
    /// Family member
    Family,
    /// Close friend
    CloseFriend,
    /// Research collaborator
    ResearchPartner,
    /// Professional colleague
    Professional,
    /// Community member
    Community,
}

/// Access levels for trusted grants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Read-only access
    ReadOnly,
    /// Limited write access
    Limited,
    /// Full access
    Full,
    /// Administrative access
    Admin,
}

/// Commercial access configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommercialAccessConfig {
    /// Enable commercial access
    pub enabled: bool,
    /// Licensed cloud providers
    pub licensed_providers: Vec<CommercialProvider>,
    /// Revenue sharing configuration
    pub revenue_config: RevenueConfig,
}

/// Commercial provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommercialProvider {
    /// Provider name (AWS, GCP, Azure, etc.)
    pub name: String,
    /// License key
    pub license_key: String,
    /// Access level purchased
    pub access_level: CommercialAccessLevel,
    /// License expiration
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Commercial access levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommercialAccessLevel {
    /// Basic integration access
    Basic,
    /// Standard enterprise features
    Standard,
    /// Premium enterprise features
    Premium,
    /// Full enterprise access
    Enterprise,
}

/// Revenue configuration for commercial access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueConfig {
    /// Revenue sharing percentage for biomeOS development
    pub revenue_share_percent: f64,
    /// Recipient wallet for revenue sharing
    pub recipient_wallet: String,
    /// Minimum payment threshold
    pub minimum_payment: f64,
}

/// Security enforcement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnforcementConfig {
    /// Deep packet inspection
    pub deep_packet_inspection: bool,
    /// API signature detection
    pub api_signature_detection: bool,
    /// Behavioral analysis for anomalous patterns
    pub behavioral_analysis: bool,
    /// ML-based detection for unknown external APIs
    pub ml_detection: bool,
    /// Threat response strategy
    pub threat_response: ThreatResponse,
}

/// Threat response strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatResponse {
    /// Block threats and preserve evidence
    BlockAndPreserve,
    /// Warn about threats but allow
    WarnAllow,
    /// Log threats silently
    LogOnly,
    /// Block threats without logging
    BlockSilent,
}

impl Default for MycorrhizaConfig {
    fn default() -> Self {
        Self {
            system_state: EnergyFlowState::Closed,
            personal_ai: PersonalAiConfig {
                enabled: true,
                local_models: vec![
                    "llama3.2-3b-instruct".to_string(),
                    "qwen2.5-7b-instruct".to_string(),
                ],
                api_keys: HashMap::new(),
                personality: AiPersonalityConfig::default(),
            },
            trusted_externals: TrustedExternalsConfig {
                enabled: false,
                grants: Vec::new(),
                relationships: HashMap::new(),
            },
            commercial_access: CommercialAccessConfig {
                enabled: false,
                licensed_providers: Vec::new(),
                revenue_config: RevenueConfig {
                    revenue_share_percent: 30.0,
                    recipient_wallet: "biomeos-development".to_string(),
                    minimum_payment: 10.0,
                },
            },
            enforcement: EnforcementConfig {
                deep_packet_inspection: true,
                api_signature_detection: true,
                behavioral_analysis: true,
                ml_detection: true,
                threat_response: ThreatResponse::BlockAndPreserve,
            },
        }
    }
}

impl Default for AiPersonalityConfig {
    fn default() -> Self {
        Self {
            helpfulness: 0.9,
            technical_complexity: 0.3,
            proactiveness: 0.8,
            safety_verbosity: 0.9,
        }
    }
}

/// Security level enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SecurityLevel {
    Low,
    Standard,
    High,
    Critical,
}

use crate::crypto::{PrivateKey, PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dependency ID type
pub type DependencyId = String;

/// Violation severity
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Recommendation priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Implementation effort
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImplementationEffort {
    Trivial,
    Low,
    Medium,
    High,
    Extreme,
}

/// External dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExternalDependency {
    pub id: DependencyId,
    pub name: String,
    pub dependency_type: DependencyType,
    pub vendor: String,
    pub access_requirements: AccessRequirements,
    pub sovereignty_impact: SovereigntyImpact,
    pub licensing: LicensingInfo,
    pub api_signatures: Vec<ApiSignature>,
    pub alternatives: Vec<AlternativeDependency>,
}

/// Dependency types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyType {
    /// Cloud providers
    CloudProvider { services: Vec<String> },
    /// Container orchestrators
    Orchestrator { api_version: String },
    /// Databases
    Database { engine: String, version: String },
    /// Message queues
    MessageQueue { protocol: String },
    /// Monitoring systems
    Monitoring { metrics_format: String },
    /// AI/ML services
    AiService { model_types: Vec<String> },
    /// Crypto libraries
    CryptoLibrary { algorithms: Vec<String> },
    /// Package registries
    PackageRegistry { package_types: Vec<String> },
    /// CDN/Storage
    ContentDelivery { protocols: Vec<String> },
    /// Authentication providers
    AuthProvider { protocols: Vec<String> },
    /// Custom dependency
    Custom {
        category: String,
        description: String,
    },
}

/// Access requirements for dependencies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessRequirements {
    pub crypto_lock_required: bool,
    pub sovereign_key_required: bool,
    pub compliance_level: ComplianceLevel,
    pub usage_restrictions: Vec<UsageRestriction>,
    pub cat_door_allowed: bool, // AI cat door exception
}

/// Compliance levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ComplianceLevel {
    Standard,
    /// No restrictions - open source friendly
    Open,
    /// Personal use only
    Personal,
    /// Research/academic use
    Research,
    /// Commercial license required
    Commercial,
    /// Enterprise license required
    Enterprise,
    High,
    /// Sovereign key required
    Sovereign,
    /// Prohibited for commercial use
    NonCommercial,
}

/// Usage restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageRestriction {
    /// Maximum requests per time period
    RateLimit { requests_per_hour: u32 },
    /// Maximum data transfer
    DataLimit { mb_per_month: u64 },
    /// Geographic restrictions
    GeoRestriction { allowed_countries: Vec<String> },
    /// Time-based restrictions
    TimeRestriction { allowed_hours: Vec<u8> },
    /// Feature restrictions
    FeatureRestriction { disabled_features: Vec<String> },
    /// Custom restriction
    Custom {
        restriction_type: String,
        parameters: HashMap<String, String>,
    },
}

/// Access context for decision making
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessContext {
    pub user_type: UserType,
    pub usage_pattern: UsagePattern,
    pub geographic_location: Option<String>,
    pub current_usage: CurrentUsage,
    pub biome_configuration: BiomeConfiguration,
}

/// User types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserType {
    /// Individual user (grandma-safe cat door allowed)
    Individual { verified: bool },
    /// Research institution
    Research { institution: String, verified: bool },
    /// Commercial entity
    Commercial {
        company: String,
        revenue_tier: RevenueTier,
    },
    /// Government entity
    Government {
        country: String,
        security_clearance: Option<String>,
    },
    /// Non-profit organization
    NonProfit {
        organization: String,
        verified: bool,
    },
}

/// Revenue tiers for commercial users
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RevenueTier {
    Startup,    // < $1M ARR
    SmallBiz,   // $1M - $10M ARR
    MidMarket,  // $10M - $100M ARR
    Enterprise, // > $100M ARR
    High,
}

/// Usage patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsagePattern {
    pub usage_type: UsageType,
    pub scale: UsageScale,
    pub frequency: UsageFrequency,
    pub data_sensitivity: DataSensitivity,
    pub commercial_purpose: bool,
    pub revenue_generating: bool,
}

/// Usage types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageType {
    Personal,
    Educational,
    Research,
    Development,
    Testing,
    Production,
    Commercial,
    Internal,
}

/// Usage scale
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageScale {
    Small,
    Individual,
    Team,
    Department,
    Organization,
    PublicService,
}

/// Usage frequency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UsageFrequency {
    Monthly,
    Occasional,
    Regular,
    Heavy,
    Continuous,
}

/// Data sensitivity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataSensitivity {
    Public,
    Internal,
    Confidential,
    Restricted,
    TopSecret,
}

/// Current usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CurrentUsage {
    pub daily_requests: u32,
    pub monthly_data_gb: f64,
    pub concurrent_connections: u32,
    pub peak_usage_time: chrono::DateTime<chrono::Utc>,
    pub cost_current_month: f64,
}

/// biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BiomeConfiguration {
    pub energy_flow_state: EnergyFlowState,
    pub sovereignty_level: SovereigntyLevel,
    pub ai_cat_door_enabled: bool,
    pub compliance_frameworks: Vec<String>,
    pub geographic_restrictions: Vec<String>,
}

/// Energy flow states (from universal.rs)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EnergyFlowState {
    Closed,
    PrivateOpen,
    CommercialOpen,
}

/// Sovereignty levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SovereigntyLevel {
    Maximum, // Air-gapped, zero external deps
    High,    // Minimal external deps, all crypto-locked
    Medium,  // Some external deps with exit strategies
    Low,     // Standard external deps allowed
    Minimal, // Any external deps allowed
}

/// Access decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AccessDecision {
    pub decision: AccessVerdict,
    pub reasoning: String,
    pub conditions: Vec<AccessCondition>,
    pub alternatives_suggested: Vec<AlternativeDependency>,
    pub compliance_notes: Vec<String>,
}

/// Access verdict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessVerdict {
    /// Full access granted
    Granted,
    /// Access granted with conditions
    ConditionalGrant,
    /// Access denied
    Denied,
    /// Requires manual review
    RequiresReview,
    /// Deferred pending compliance
    Deferred {
        review_date: chrono::DateTime<chrono::Utc>,
    },
}

/// Access conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AccessCondition {
    /// Rate limiting
    RateLimit { max_requests_per_hour: u32 },
    /// Data limits
    DataLimit { max_gb_per_month: u64 },
    /// Time-based access
    TimeRestriction { allowed_hours: Vec<u8> },
    /// Geo restrictions
    GeoRestriction { blocked_countries: Vec<String> },
    /// Monitoring required
    MonitoringRequired { audit_level: AuditLevel },
    /// Fallback required
    FallbackRequired { fallback_provider: String },
    /// Payment required
    PaymentRequired { amount: f64, currency: String },
    /// Key upgrade required
    KeyUpgradeRequired { target_tier: String },
}

/// Audit levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditLevel {
    Basic,
    Detailed,
    Forensic,
}

/// API signature for detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiSignature {
    pub service_name: String,
    pub api_patterns: Vec<ApiPattern>,
    pub headers: Vec<HeaderPattern>,
    pub auth_patterns: Vec<AuthPattern>,
    pub payload_patterns: Vec<PayloadPattern>,
}

/// API patterns for detection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiPattern {
    UrlPattern {
        regex: String,
    },
    DomainPattern {
        domains: Vec<String>,
    },
    PathPattern {
        paths: Vec<String>,
    },
    Custom {
        pattern_type: String,
        pattern: String,
    },
}

/// Header patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HeaderPattern {
    Contains { key: String, value: String },
    StartsWith { key: String, prefix: String },
    Exists { key: String },
    Custom { pattern: String },
}

/// Authentication patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthPattern {
    BearerToken,
    ApiKey { header_name: String },
    BasicAuth,
    OAuth2,
    AwsSignatureV4,
    CustomAuth { auth_type: String },
}

/// Payload patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PayloadPattern {
    JsonSchema { schema: String },
    XmlSchema { schema: String },
    ProtobufSchema { schema: String },
    Custom { format: String, pattern: String },
}

/// Alternative dependencies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlternativeDependency {
    pub name: String,
    pub vendor: String,
    pub compatibility_score: f64,
    pub migration_effort: MigrationDifficulty,
    pub sovereignty_improvement: f64,
    pub cost_comparison: CostComparison,
}

/// Migration difficulty
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MigrationDifficulty {
    Trivial,  // Drop-in replacement available
    Easy,     // Some configuration changes needed
    Moderate, // Significant development effort
    Hard,     // Major re-architecture needed
    Extreme,  // Complete system redesign
}

/// Cost comparison
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CostComparison {
    Cheaper { savings_percent: f64 },
    MoreExpensive { increase_percent: f64 },
    Similar { difference_percent: f64 },
    Unknown,
}

/// Licensing information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LicensingInfo {
    pub license_type: LicenseType,
    pub commercial_terms: Option<CommercialTerms>,
    pub attribution_required: bool,
    pub source_disclosure_required: bool,
    pub patent_grant: bool,
    pub copyleft_requirements: Vec<String>,
}

/// License types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LicenseType {
    /// Open source licenses
    OpenSource { license: OpenSourceLicense },
    /// Proprietary with free tier
    FreeTier { limitations: Vec<String> },
    /// Commercial only
    Commercial { pricing_model: PricingModel },
    /// Dual license (open source + commercial)
    Dual {
        oss_license: OpenSourceLicense,
        commercial_terms: CommercialTerms,
    },
    /// Custom license terms
    Custom { license_text: String },
}

/// Open source licenses
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OpenSourceLicense {
    Mit,
    Apache2,
    Gpl3,
    Agpl3,
    Lgpl3,
    Bsd3Clause,
    Mpl2,
    Unlicense,
    PublicDomain,
    Custom { name: String, osi_approved: bool },
}

/// Pricing models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PricingModel {
    PerUser,
    PerRequest,
    PerGB,
    PerCPUHour,
    Subscription,
    OneTime,
    RevenueBased,
    Custom { model: String },
}

/// Commercial terms
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommercialTerms {
    pub base_price: f64,
    pub currency: String,
    pub pricing_model: PricingModel,
    pub volume_discounts: Vec<VolumeDiscount>,
    pub support_included: bool,
    pub sla_guarantees: Vec<String>,
}

/// Volume discounts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VolumeDiscount {
    pub threshold: u64,
    pub discount_percent: f64,
}

/// Sovereignty impact assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SovereigntyImpact {
    pub impact_level: SovereigntyImpactLevel,
    pub data_residency_requirements: Vec<String>,
    pub vendor_lock_risk: VendorLockRisk,
    pub exit_strategy: ExitStrategy,
    pub alternatives_available: bool,
}

/// Sovereignty impact levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SovereigntyImpactLevel {
    /// No impact on sovereignty
    None,
    /// Minimal impact - easily replaceable
    Minimal,
    /// Moderate impact - some effort to replace
    Moderate,
    /// High impact - significant effort to replace
    High,
    /// Critical impact - very difficult to replace
    Critical,
}

/// Vendor lock risk assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VendorLockRisk {
    pub risk_level: RiskLevel,
    pub lock_factors: Vec<LockFactor>,
    pub migration_difficulty: MigrationDifficulty,
    pub cost_to_exit: Option<f64>,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Lock-in factors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LockFactor {
    ProprietaryApi,
    ProprietaryDataFormat,
    NetworkEffects,
    HighSwitchingCosts,
    IntegratedEcosystem,
    ContractualObligations,
    TechnicalDebt,
}

/// Exit strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExitStrategy {
    pub data_portability: DataPortability,
    pub code_portability: CodePortability,
    pub estimated_migration_time_weeks: u32,
    pub migration_checklist: Vec<String>,
}

/// Data portability assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataPortability {
    FullyPortable,
    MostlyPortable { limitations: Vec<String> },
    PartiallyPortable { requires_conversion: Vec<String> },
    NotPortable { locked_data_types: Vec<String> },
}

/// Code portability assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CodePortability {
    FullyPortable,
    MostlyPortable {
        vendor_specific_apis: Vec<String>,
    },
    PartiallyPortable {
        major_refactoring_needed: Vec<String>,
    },
    NotPortable {
        tightly_coupled_systems: Vec<String>,
    },
}

/// Report period
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReportPeriod {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
}

/// Usage metrics
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UsageMetrics {
    pub dependency_id: DependencyId,
    pub reporting_period: ReportPeriod,
    pub request_count: u64,
    pub data_transferred_gb: f64,
    pub peak_concurrency: u32,
    pub error_rate: f64,
    pub average_response_time_ms: f64,
    pub cost_incurred: f64,
    pub quota_utilization: f64,
}

/// Sovereign key specification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SovereignKeySpec {
    pub grantee: String,
    pub grantee_type: GranteeType,
    pub access_level: SovereignAccessLevel,
    pub dependencies: Vec<DependencyId>,
    pub validity_period: ValidityPeriod,
    pub restrictions: Vec<KeyRestriction>,
}

/// Grantee types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GranteeType {
    Individual {
        email: String,
        verified: bool,
    },
    Organization {
        name: String,
        tax_id: String,
    },
    OpenSourceProject {
        repository: String,
        maintainers: Vec<String>,
    },
    ResearchInstitution {
        name: String,
        country: String,
    },
}

/// Sovereign access levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SovereignAccessLevel {
    /// Read-only access
    ReadOnly,
    /// Limited write access
    Limited,
    /// Full access
    Full,
    /// Administrative access
    Admin,
    /// Custom access level
    Custom { permissions: Vec<String> },
}

/// Validity period
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidityPeriod {
    /// Specific duration
    Duration { months: u32 },
    /// Until specific date
    UntilDate { date: chrono::DateTime<chrono::Utc> },
    /// Indefinite (with annual review)
    Indefinite,
    /// One-time use
    OneTime,
}

/// Key restrictions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyRestriction {
    IpWhitelist {
        ips: Vec<String>,
    },
    GeoRestriction {
        allowed_countries: Vec<String>,
    },
    UsageQuota {
        max_requests_per_month: u64,
    },
    DataQuota {
        max_gb_per_month: u64,
    },
    TimeRestriction {
        allowed_hours: Vec<u8>,
    },
    CustomRestriction {
        restriction: String,
        parameters: HashMap<String, String>,
    },
}

/// Sovereign key
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SovereignKey {
    pub key_id: String,
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
    pub spec: SovereignKeySpec,
    pub signature: Signature,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub status: KeyStatus,
}

/// Key status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum KeyStatus {
    Active,
    Suspended { reason: String },
    Revoked { reason: String },
    Expired,
    PendingActivation,
}

impl Default for UsagePattern {
    fn default() -> Self {
        Self {
            frequency: UsageFrequency::Monthly,
            scale: UsageScale::Small,
            data_sensitivity: DataSensitivity::Public,
            commercial_purpose: false,
            revenue_generating: false,
            usage_type: UsageType::Development,
        }
    }
}


impl Default for AccessRequirements {
    fn default() -> Self {
        Self {
            compliance_level: ComplianceLevel::Standard,
            crypto_lock_required: false,
            sovereign_key_required: false,
            usage_restrictions: vec![],
            cat_door_allowed: false,
        }
    }
}


impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::CloudProvider { .. } => write!(f, "cloud_provider"),
            DependencyType::Orchestrator { .. } => write!(f, "orchestrator"),
            DependencyType::Database { .. } => write!(f, "database"),
            DependencyType::MessageQueue { .. } => write!(f, "message_queue"),
            DependencyType::Monitoring { .. } => write!(f, "monitoring"),
            DependencyType::AiService { .. } => write!(f, "ai_service"),
            DependencyType::CryptoLibrary { .. } => write!(f, "crypto_library"),
            DependencyType::PackageRegistry { .. } => write!(f, "package_registry"),
            DependencyType::ContentDelivery { .. } => write!(f, "content_delivery"),
            DependencyType::AuthProvider { .. } => write!(f, "auth_provider"),
            DependencyType::Custom { .. } => write!(f, "custom"),
        }
    }
}


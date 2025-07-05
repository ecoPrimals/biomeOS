use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::BiomeResult;
use crate::crypto::{KeyAlgorithm, PrivateKey, PublicKey, Signature};

/// Crypto Lock System - gates external dependencies while preserving AI cat door
#[async_trait]
pub trait CryptoLockInterface {
    /// Validate access to external dependency
    async fn validate_access(&self, dependency: &ExternalDependency, context: &AccessContext) -> BiomeResult<AccessDecision>;
    
    /// Register new external dependency
    async fn register_dependency(&self, dependency: &ExternalDependency) -> BiomeResult<DependencyId>;
    
    /// Grant sovereign access key
    async fn grant_sovereign_key(&self, spec: &SovereignKeySpec) -> BiomeResult<SovereignKey>;
    
    /// Revoke sovereign access key
    async fn revoke_sovereign_key(&self, key_id: &str) -> BiomeResult<()>;
    
    /// Check compliance status
    async fn check_compliance(&self, usage_pattern: &UsagePattern) -> BiomeResult<ComplianceStatus>;
    
    /// Generate compliance report
    async fn generate_compliance_report(&self) -> BiomeResult<ComplianceReport>;
    
    /// Update licensing terms
    async fn update_licensing(&self, terms: &LicensingTerms) -> BiomeResult<()>;
    
    /// Monitor dependency usage
    async fn monitor_usage(&self, dependency_id: &DependencyId) -> BiomeResult<UsageMetrics>;
}

/// External dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Custom { category: String, description: String },
}

/// Access requirements for dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessRequirements {
    pub crypto_lock_required: bool,
    pub sovereign_key_required: bool,
    pub compliance_level: ComplianceLevel,
    pub usage_restrictions: Vec<UsageRestriction>,
    pub cat_door_allowed: bool,  // AI cat door exception
}

/// Compliance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceLevel {
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
    /// Sovereign key required
    Sovereign,
    /// Prohibited for commercial use
    NonCommercial,
}

/// Usage restrictions
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Custom { restriction_type: String, parameters: HashMap<String, String> },
}

/// Sovereignty impact assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyImpact {
    pub impact_level: SovereigntyImpactLevel,
    pub data_residency_requirements: Vec<String>,
    pub vendor_lock_risk: VendorLockRisk,
    pub exit_strategy: ExitStrategy,
    pub alternatives_available: bool,
}

/// Sovereignty impact levels
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorLockRisk {
    pub risk_level: RiskLevel,
    pub lock_factors: Vec<LockFactor>,
    pub migration_difficulty: MigrationDifficulty,
    pub cost_to_exit: Option<f64>,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Lock-in factors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockFactor {
    ProprietaryApi,
    ProprietaryDataFormat,
    NetworkEffects,
    HighSwitchingCosts,
    IntegratedEcosystem,
    ContractualObligations,
    TechnicalDebt,
}

/// Migration difficulty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationDifficulty {
    Trivial,    // Drop-in replacement available
    Easy,       // Some configuration changes needed
    Moderate,   // Significant development effort
    Hard,       // Major re-architecture needed
    Extreme,    // Complete system redesign
}

/// Exit strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategy {
    pub data_portability: DataPortability,
    pub code_portability: CodePortability,
    pub estimated_migration_time_weeks: u32,
    pub migration_checklist: Vec<String>,
}

/// Data portability assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataPortability {
    FullyPortable,
    MostlyPortable { limitations: Vec<String> },
    PartiallyPortable { requires_conversion: Vec<String> },
    NotPortable { locked_data_types: Vec<String> },
}

/// Code portability assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CodePortability {
    FullyPortable,
    MostlyPortable { vendor_specific_apis: Vec<String> },
    PartiallyPortable { major_refactoring_needed: Vec<String> },
    NotPortable { tightly_coupled_systems: Vec<String> },
}

/// Licensing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingInfo {
    pub license_type: LicenseType,
    pub commercial_terms: Option<CommercialTerms>,
    pub attribution_required: bool,
    pub source_disclosure_required: bool,
    pub patent_grant: bool,
    pub copyleft_requirements: Vec<String>,
}

/// License types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseType {
    /// Open source licenses
    OpenSource { license: OpenSourceLicense },
    /// Proprietary with free tier
    FreeTier { limitations: Vec<String> },
    /// Commercial only
    Commercial { pricing_model: PricingModel },
    /// Dual license (open source + commercial)
    Dual { oss_license: OpenSourceLicense, commercial_terms: CommercialTerms },
    /// Custom license terms
    Custom { license_text: String },
}

/// Open source licenses
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommercialTerms {
    pub base_price: f64,
    pub currency: String,
    pub pricing_model: PricingModel,
    pub volume_discounts: Vec<VolumeDiscount>,
    pub support_included: bool,
    pub sla_guarantees: Vec<String>,
}

/// Volume discounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDiscount {
    pub threshold: u64,
    pub discount_percent: f64,
}

/// API signature for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiSignature {
    pub service_name: String,
    pub api_patterns: Vec<ApiPattern>,
    pub headers: Vec<HeaderPattern>,
    pub auth_patterns: Vec<AuthPattern>,
    pub payload_patterns: Vec<PayloadPattern>,
}

/// API patterns for detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiPattern {
    UrlPattern { regex: String },
    DomainPattern { domains: Vec<String> },
    PathPattern { paths: Vec<String> },
    Custom { pattern_type: String, pattern: String },
}

/// Header patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HeaderPattern {
    Contains { key: String, value: String },
    StartsWith { key: String, prefix: String },
    Exists { key: String },
    Custom { pattern: String },
}

/// Authentication patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthPattern {
    BearerToken,
    ApiKey { header_name: String },
    BasicAuth,
    OAuth2,
    AwsSignatureV4,
    CustomAuth { auth_type: String },
}

/// Payload patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayloadPattern {
    JsonSchema { schema: String },
    XmlSchema { schema: String },
    ProtobufSchema { schema: String },
    Custom { format: String, pattern: String },
}

/// Alternative dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeDependency {
    pub name: String,
    pub vendor: String,
    pub compatibility_score: f64,
    pub migration_effort: MigrationDifficulty,
    pub sovereignty_improvement: f64,
    pub cost_comparison: CostComparison,
}

/// Cost comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostComparison {
    Cheaper { savings_percent: f64 },
    MoreExpensive { increase_percent: f64 },
    Similar { difference_percent: f64 },
    Unknown,
}

/// Access context for decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessContext {
    pub user_type: UserType,
    pub usage_pattern: UsagePattern,
    pub geographic_location: Option<String>,
    pub current_usage: CurrentUsage,
    pub biome_configuration: BiomeConfiguration,
}

/// User types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserType {
    /// Individual user (grandma-safe cat door allowed)
    Individual { verified: bool },
    /// Research institution
    Research { institution: String, verified: bool },
    /// Commercial entity
    Commercial { company: String, revenue_tier: RevenueTier },
    /// Government entity
    Government { country: String, security_clearance: Option<String> },
    /// Non-profit organization
    NonProfit { organization: String, verified: bool },
}

/// Revenue tiers for commercial users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevenueTier {
    Startup,     // < $1M ARR
    SmallBiz,    // $1M - $10M ARR
    MidMarket,   // $10M - $100M ARR
    Enterprise,  // > $100M ARR
}

/// Usage patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePattern {
    pub usage_type: UsageType,
    pub scale: UsageScale,
    pub frequency: UsageFrequency,
    pub data_sensitivity: DataSensitivity,
    pub commercial_purpose: bool,
    pub revenue_generating: bool,
}

/// Usage types
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageScale {
    Individual,
    Team,
    Department,
    Organization,
    PublicService,
}

/// Usage frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageFrequency {
    Occasional,
    Regular,
    Heavy,
    Continuous,
}

/// Data sensitivity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataSensitivity {
    Public,
    Internal,
    Confidential,
    Restricted,
    TopSecret,
}

/// Current usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentUsage {
    pub daily_requests: u32,
    pub monthly_data_gb: f64,
    pub concurrent_connections: u32,
    pub peak_usage_time: chrono::DateTime<chrono::Utc>,
    pub cost_current_month: f64,
}

/// biomeOS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeConfiguration {
    pub energy_flow_state: EnergyFlowState,
    pub sovereignty_level: SovereigntyLevel,
    pub ai_cat_door_enabled: bool,
    pub compliance_frameworks: Vec<String>,
    pub geographic_restrictions: Vec<String>,
}

/// Energy flow states (from universal.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnergyFlowState {
    Closed,
    PrivateOpen,
    CommercialOpen,
}

/// Sovereignty levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyLevel {
    Maximum,    // Air-gapped, zero external deps
    High,       // Minimal external deps, all crypto-locked
    Medium,     // Some external deps with exit strategies
    Low,        // Standard external deps allowed
    Minimal,    // Any external deps allowed
}

/// Access decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessDecision {
    pub decision: AccessVerdict,
    pub reasoning: String,
    pub conditions: Vec<AccessCondition>,
    pub alternatives_suggested: Vec<AlternativeDependency>,
    pub compliance_notes: Vec<String>,
}

/// Access verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    Deferred { review_date: chrono::DateTime<chrono::Utc> },
}

/// Access conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLevel {
    Basic,
    Detailed,
    Forensic,
}

/// Dependency ID type
pub type DependencyId = String;

/// Sovereign key specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereignKeySpec {
    pub grantee: String,
    pub grantee_type: GranteeType,
    pub access_level: SovereignAccessLevel,
    pub dependencies: Vec<DependencyId>,
    pub validity_period: ValidityPeriod,
    pub restrictions: Vec<KeyRestriction>,
}

/// Grantee types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GranteeType {
    Individual { email: String, verified: bool },
    Organization { name: String, tax_id: String },
    OpenSourceProject { repository: String, maintainers: Vec<String> },
    ResearchInstitution { name: String, country: String },
}

/// Sovereign access levels
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyRestriction {
    IpWhitelist { ips: Vec<String> },
    GeoRestriction { allowed_countries: Vec<String> },
    UsageQuota { max_requests_per_month: u64 },
    DataQuota { max_gb_per_month: u64 },
    TimeRestriction { allowed_hours: Vec<u8> },
    CustomRestriction { restriction: String, parameters: HashMap<String, String> },
}

/// Sovereign key
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyStatus {
    Active,
    Suspended { reason: String },
    Revoked { reason: String },
    Expired,
    PendingActivation,
}

/// Compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub overall_status: OverallComplianceStatus,
    pub dependency_compliance: HashMap<DependencyId, DependencyCompliance>,
    pub violations: Vec<ComplianceViolation>,
    pub recommendations: Vec<ComplianceRecommendation>,
}

/// Overall compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OverallComplianceStatus {
    Compliant,
    NonCompliant { severity: ViolationSeverity },
    Warning { issues: Vec<String> },
    Unknown,
}

/// Violation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Dependency compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyCompliance {
    pub compliant: bool,
    pub license_compliance: LicenseCompliance,
    pub usage_compliance: UsageCompliance,
    pub sovereignty_compliance: SovereigntyCompliance,
}

/// License compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LicenseCompliance {
    Compliant,
    NonCompliant { violations: Vec<String> },
    Unknown,
}

/// Usage compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UsageCompliance {
    WithinLimits,
    ExceedsLimits { overages: Vec<String> },
    Unauthorized { violations: Vec<String> },
}

/// Sovereignty compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyCompliance {
    FullySovereign,
    PartiallySovereign { dependencies: Vec<String> },
    NonSovereign { vendor_locks: Vec<String> },
}

/// Compliance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceViolation {
    pub violation_id: String,
    pub violation_type: ViolationType,
    pub severity: ViolationSeverity,
    pub dependency_id: DependencyId,
    pub description: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub resolution_required: bool,
    pub suggested_actions: Vec<String>,
}

/// Types of security/sovereignty violations
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ViolationType {
    /// Data sovereignty violation
    DataSovereignty,
    /// Vendor lock-in detected
    VendorLock,
    /// Excessive external dependencies
    ExcessiveDependencies,
    /// Unencrypted data transmission
    UnencryptedTransmission,
    /// Missing exit strategy
    NoExitStrategy,
    /// Cost limit exceeded
    CostLimitExceeded,
    /// Geographic restriction violation
    GeographicViolation,
    /// AI usage policy violation
    AiPolicyViolation,
}

/// Compliance recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRecommendation {
    pub recommendation_id: String,
    pub priority: RecommendationPriority,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub effort: ImplementationEffort,
    pub actions: Vec<String>,
}

/// Recommendation priority
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Recommendation category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationCategory {
    Licensing,
    Security,
    Performance,
    Cost,
    Sovereignty,
    Compliance,
}

/// Implementation effort
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImplementationEffort {
    Trivial,
    Low,
    Medium,
    High,
    Extreme,
}

/// Compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub report_id: String,
    pub generated_at: chrono::DateTime<chrono::Utc>,
    pub report_period: ReportPeriod,
    pub overall_status: OverallComplianceStatus,
    pub summary: ComplianceSummary,
    pub detailed_findings: Vec<DetailedFinding>,
    pub recommendations: Vec<ComplianceRecommendation>,
    pub cost_analysis: CostAnalysis,
    pub sovereignty_analysis: SovereigntyAnalysis,
}

/// Report period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportPeriod {
    pub start_date: chrono::DateTime<chrono::Utc>,
    pub end_date: chrono::DateTime<chrono::Utc>,
}

/// Compliance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total_dependencies: u32,
    pub compliant_dependencies: u32,
    pub non_compliant_dependencies: u32,
    pub total_violations: u32,
    pub critical_violations: u32,
    pub sovereignty_score: f64,
}

/// Detailed finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedFinding {
    pub finding_id: String,
    pub dependency_id: DependencyId,
    pub finding_type: FindingType,
    pub severity: ViolationSeverity,
    pub description: String,
    pub evidence: Vec<String>,
    pub impact_assessment: String,
    pub recommended_actions: Vec<String>,
}

/// Finding types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FindingType {
    LicenseIssue,
    SecurityRisk,
    SovereigntyRisk,
    CostOverrun,
    PerformanceIssue,
    ComplianceGap,
}

/// Cost analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    pub total_cost: f64,
    pub currency: String,
    pub cost_breakdown: Vec<CostBreakdown>,
    pub cost_trends: Vec<CostTrend>,
    pub optimization_opportunities: Vec<CostOptimization>,
}

/// Cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub dependency_id: DependencyId,
    pub cost: f64,
    pub percentage_of_total: f64,
    pub cost_type: CostType,
}

/// Cost types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CostType {
    Licensing,
    Usage,
    Support,
    Migration,
    Compliance,
}

/// Cost trend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostTrend {
    pub period: String,
    pub cost: f64,
    pub change_percent: f64,
}

/// Cost optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOptimization {
    pub optimization_id: String,
    pub title: String,
    pub description: String,
    pub potential_savings: f64,
    pub implementation_effort: ImplementationEffort,
    pub risk_level: RiskLevel,
}

/// Sovereignty analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyAnalysis {
    pub sovereignty_score: f64,
    pub vendor_lock_risks: Vec<VendorLockRisk>,
    pub data_residency_issues: Vec<DataResidencyIssue>,
    pub exit_strategies: Vec<ExitStrategyAssessment>,
    pub independence_roadmap: Vec<IndependenceStep>,
}

/// Data residency issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataResidencyIssue {
    pub issue_id: String,
    pub dependency_id: DependencyId,
    pub data_location: String,
    pub required_location: String,
    pub severity: ViolationSeverity,
    pub mitigation_options: Vec<String>,
}

/// Exit strategy assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategyAssessment {
    pub dependency_id: DependencyId,
    pub exit_difficulty: MigrationDifficulty,
    pub estimated_cost: f64,
    pub estimated_time_weeks: u32,
    pub major_blockers: Vec<String>,
    pub recommended_alternatives: Vec<AlternativeDependency>,
}

/// Independence step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndependenceStep {
    pub step_id: String,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_effort: ImplementationEffort,
    pub dependencies: Vec<String>,
    pub success_criteria: Vec<String>,
}

/// Licensing terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicensingTerms {
    pub terms_version: String,
    pub effective_date: chrono::DateTime<chrono::Utc>,
    pub personal_use_terms: PersonalUseTerms,
    pub commercial_use_terms: CommercialUseTerms,
    pub compliance_requirements: Vec<ComplianceRequirement>,
}

/// Personal use terms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalUseTerms {
    pub ai_cat_door_enabled: bool,
    pub rate_limits: Vec<RateLimit>,
    pub data_limits: Vec<DataLimit>,
    pub feature_restrictions: Vec<String>,
    pub attribution_required: bool,
}

/// Commercial use terms (licensing OR partnership)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommercialUseTerms {
    pub commercial_model: CommercialModel,
    pub pricing_tiers: Vec<PricingTier>,
    pub partnership: Option<PartnershipContribution>,
    pub enterprise_features: Vec<String>,
    pub support_included: bool,
}

/// Commercial models available
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommercialModel {
    /// Traditional licensing only
    LicensingOnly,
    /// Partnership contribution only (voluntary)
    PartnershipOnly,
    /// User choice: licensing OR partnership
    UserChoice,
    /// Fully open (no commercial restrictions)
    FullyOpen,
}

/// Rate limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub resource: String,
    pub limit: u64,
    pub period: String,
}

/// Data limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataLimit {
    pub data_type: String,
    pub limit_gb: u64,
    pub period: String,
}

/// Pricing tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingTier {
    pub tier_name: String,
    pub price: f64,
    pub currency: String,
    pub billing_period: String,
    pub included_features: Vec<String>,
    pub usage_limits: Vec<UsageLimit>,
}

/// Usage limit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimit {
    pub metric: String,
    pub limit: u64,
    pub overage_cost: Option<f64>,
}

/// Partnership contribution (voluntary, sovereign)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartnershipContribution {
    pub enabled: bool,
    pub percentage: Option<f64>,
    pub minimum_threshold: Option<f64>,
    pub payment_frequency: String,
    pub sovereign_wallet: Option<String>,  // sweetgrass/rhizoCrypt
    pub benefits: Vec<String>,
}

/// Compliance requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub requirement_id: String,
    pub title: String,
    pub description: String,
    pub mandatory: bool,
    pub verification_method: String,
}

/// Usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Crypto Lock Manager - implements the actual crypto lock system
pub struct CryptoLockManager {
    pub dependencies: HashMap<DependencyId, ExternalDependency>,
    pub sovereign_keys: HashMap<String, SovereignKey>,
    pub licensing_terms: LicensingTerms,
    pub usage_monitor: UsageMonitor,
    pub compliance_engine: ComplianceEngine,
    pub ai_cat_door: AiCatDoor,
}

/// Usage monitor for tracking dependency usage
pub struct UsageMonitor {
    pub active_sessions: HashMap<String, ActiveSession>,
    pub usage_history: Vec<UsageRecord>,
    pub rate_limiters: HashMap<DependencyId, RateLimiter>,
}

/// Active session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub session_id: String,
    pub dependency_id: DependencyId,
    pub user_context: AccessContext,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub requests_made: u32,
    pub data_transferred_mb: f64,
}

/// Usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageRecord {
    pub record_id: String,
    pub dependency_id: DependencyId,
    pub user_context: AccessContext,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub operation: String,
    pub success: bool,
    pub response_time_ms: u64,
    pub data_size_bytes: u64,
    pub cost: Option<f64>,
}

/// Rate limiter
pub struct RateLimiter {
    pub dependency_id: DependencyId,
    pub limits: Vec<RateLimit>,
    pub current_usage: HashMap<String, u64>,
    pub reset_times: HashMap<String, chrono::DateTime<chrono::Utc>>,
}

/// Compliance engine
pub struct ComplianceEngine {
    pub rules: Vec<ComplianceRule>,
    pub violation_handlers: HashMap<ViolationType, Box<dyn ViolationHandler>>,
    pub audit_log: Vec<AuditRecord>,
}

/// Compliance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub rule_id: String,
    pub rule_type: ComplianceRuleType,
    pub condition: RuleCondition,
    pub action: RuleAction,
    pub enabled: bool,
}

/// Compliance rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceRuleType {
    LicenseCheck,
    UsageLimit,
    SovereigntyCheck,
    SecurityCheck,
    DataResidency,
    Custom { rule_type: String },
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    Always,
    UsageExceeds { metric: String, threshold: u64 },
    LicenseType { license_types: Vec<LicenseType> },
    UserType { user_types: Vec<UserType> },
    GeographicLocation { countries: Vec<String> },
    TimeWindow { start_hour: u8, end_hour: u8 },
    Custom { condition: String, parameters: HashMap<String, String> },
}

/// Rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Deny,
    RequireApproval,
    ApplyRestrictions { restrictions: Vec<UsageRestriction> },
    SendAlert { recipients: Vec<String> },
    LogViolation,
    Custom { action: String, parameters: HashMap<String, String> },
}

/// Violation handler
pub trait ViolationHandler {
    fn handle_violation(&self, violation: &ComplianceViolation) -> BiomeResult<ViolationResponse>;
}

/// Violation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationResponse {
    pub action_taken: String,
    pub remediation_steps: Vec<String>,
    pub escalation_required: bool,
    pub auto_resolved: bool,
}

/// Audit record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    pub record_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub event_type: AuditEventType,
    pub actor: String,
    pub target: String,
    pub action: String,
    pub result: AuditResult,
    pub metadata: HashMap<String, String>,
}

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEventType {
    AccessRequest,
    AccessGranted,
    AccessDenied,
    KeyGenerated,
    KeyRevoked,
    ViolationDetected,
    ComplianceCheck,
    ConfigurationChange,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure { reason: String },
    Warning { message: String },
}

/// AI Cat Door - special allowance for personal AI access
pub struct AiCatDoor {
    pub enabled: bool,
    pub allowed_ai_services: Vec<AiServiceConfig>,
    pub personal_api_keys: HashMap<String, String>,
    pub usage_limits: PersonalAiLimits,
}

/// AI service configuration for cat door
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiServiceConfig {
    pub service_name: String,
    pub api_endpoint: String,
    pub service_type: AiServiceType,
    pub max_requests_per_day: u32,
    pub max_tokens_per_request: u32,
    pub cost_limit_per_month: f64,
}

/// AI service types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiServiceType {
    TextGeneration,
    ImageGeneration,
    CodeGeneration,
    Translation,
    Summarization,
    QuestionAnswering,
    Custom { service_type: String },
}

/// Personal AI usage limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalAiLimits {
    pub daily_request_limit: u32,
    pub monthly_token_limit: u64,
    pub monthly_cost_limit: f64,
    pub concurrent_request_limit: u32,
}

/// Configuration for compliance system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Compliance frameworks to enforce
    pub frameworks: Vec<String>,
    
    /// Reporting requirements
    pub reporting: ReportingConfig,
    
    /// Violation handling
    pub violation_handlers: HashMap<ViolationType, String>,
    
    /// Audit settings
    pub audit: AuditConfig,
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Enable compliance reporting
    pub enabled: bool,
    
    /// Reporting frequency
    pub frequency: ReportingFrequency,
    
    /// Report recipients
    pub recipients: Vec<String>,
    
    /// Report format
    pub format: ReportFormat,
}

/// Reporting frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportingFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
}

/// Report format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Csv,
    Html,
    Pdf,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,
    
    /// Audit log retention days
    pub retention_days: u32,
    
    /// Log file path
    pub log_path: String,
    
    /// Events to audit
    pub events: Vec<AuditEvent>,
}

/// Audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    DependencyAccess,
    ViolationDetected,
    ConfigurationChange,
    KeyRotation,
    ExitStrategyTriggered,
}

impl CryptoLockManager {
    /// Create new crypto lock manager with sovereignty-first defaults
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            sovereign_keys: HashMap::new(),
            licensing_terms: LicensingTerms::default(),
            usage_monitor: UsageMonitor::new(),
            compliance_engine: ComplianceEngine::new(),
            ai_cat_door: AiCatDoor::default(),
        }
    }
    
    /// Initialize with grandma-safe AI cat door
    pub fn initialize_with_cat_door(&mut self, ai_services: Vec<AiServiceConfig>) {
        self.ai_cat_door.enabled = true;
        self.ai_cat_door.allowed_ai_services = ai_services;
        self.ai_cat_door.usage_limits = PersonalAiLimits::grandma_safe_defaults();
    }
}

impl Default for LicensingTerms {
    fn default() -> Self {
        Self {
            terms_version: "1.0".to_string(),
            effective_date: chrono::Utc::now(),
            personal_use_terms: PersonalUseTerms::default(),
            commercial_use_terms: CommercialUseTerms::default(),
            compliance_requirements: vec![],
        }
    }
}

impl Default for PersonalUseTerms {
    fn default() -> Self {
        Self {
            ai_cat_door_enabled: true,  // Grandma-safe default
            rate_limits: vec![],
            data_limits: vec![],
            feature_restrictions: vec![],
            attribution_required: false,
        }
    }
}

impl Default for CommercialUseTerms {
    fn default() -> Self {
        Self {
            commercial_model: CommercialModel::UserChoice,  // Sovereignty-respecting choice
            pricing_tiers: vec![],
            partnership: None,  // Opt-in, not extractive
            enterprise_features: vec![],
            support_included: false,
        }
    }
}

impl UsageMonitor {
    pub fn new() -> Self {
        Self {
            active_sessions: HashMap::new(),
            usage_history: vec![],
            rate_limiters: HashMap::new(),
        }
    }
}

impl ComplianceEngine {
    pub fn new() -> Self {
        Self {
            rules: vec![],
            violation_handlers: HashMap::new(),
            audit_log: vec![],
        }
    }
}

impl Default for AiCatDoor {
    fn default() -> Self {
        Self {
            enabled: true,  // Grandma-safe default
            allowed_ai_services: vec![],
            personal_api_keys: HashMap::new(),
            usage_limits: PersonalAiLimits::grandma_safe_defaults(),
        }
    }
}

impl PersonalAiLimits {
    pub fn grandma_safe_defaults() -> Self {
        Self {
            daily_request_limit: 100,        // Reasonable daily limit
            monthly_token_limit: 1_000_000,  // Generous but not unlimited
            monthly_cost_limit: 20.0,        // $20/month limit
            concurrent_request_limit: 3,     // Prevent abuse
        }
    }
} 
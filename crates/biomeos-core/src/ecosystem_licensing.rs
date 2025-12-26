//! Ecosystem Licensing Coordination
//!
//! Implements cross-project licensing strategy and market-based access control
//! as defined in handOff/ECOPRIMALS_LICENSING_COORDINATION.md

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// EcoPrimals license context shared across all projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcoPrimalsLicenseContext {
    /// Organization classification
    pub organization_classification: OrganizationScale,

    /// Entropy profile for human involvement
    pub entropy_profile: EntropyProfile,

    /// Current license status
    pub license_status: LicenseStatus,

    /// Access levels across different projects
    pub project_access_map: HashMap<String, AccessLevel>,

    /// Pricing context
    pub pricing_context: PricingContext,

    /// Context creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Organization scale for progressive pricing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrganizationScale {
    Individual,
    SmallBusiness,
    RegionalBusiness,
    NationalEnterprise,
    GlobalEnterprise,
    Hyperscale,
}

/// Entropy profile tracking human involvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyProfile {
    pub entropy_tier: EntropyTier,
    pub human_involvement_score: f64,
    pub automation_percentage: f64,
    pub last_human_interaction: Option<chrono::DateTime<chrono::Utc>>,
    pub entropy_evidence: Vec<EntropyEvidence>,
}

/// Entropy tiers based on human involvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropyTier {
    /// High human involvement, lived experience
    HumanLived,

    /// Human supervised automation
    Supervised,

    /// Pure machine automation
    Machine,
}

/// Evidence of human entropy in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyEvidence {
    pub evidence_type: EntropyEvidenceType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub description: String,
    pub confidence_score: f64,
}

/// Types of entropy evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntropyEvidenceType {
    HumanInput,
    ManualOverride,
    CreativeDecision,
    ProblemSolving,
    AdaptiveResponse,
    UnplannedAction,
}

/// License status across ecosystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseStatus {
    pub is_valid: bool,
    pub license_type: EcosystemLicenseType,
    pub active_projects: Vec<String>,
    pub violations: Vec<LicenseViolation>,
    pub compliance_score: f64,
}

/// Ecosystem license types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EcosystemLicenseType {
    Individual {
        verification_method: IndividualVerification,
    },
    Corporate {
        license_agreement_id: String,
        payment_status: PaymentStatus,
    },
    AGPL3Compliant {
        source_disclosure_url: String,
        compliance_verified: bool,
    },
    Custom {
        terms: String,
        agreement_id: String,
    },
}

/// Individual verification methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndividualVerification {
    GoodFaith,
    EncryptedConnection,
    HumanVerified,
    InstitutionalVouched,
}

/// Payment status for corporate licenses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Current,
    Overdue,
    Suspended,
    Cancelled,
}

/// License violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub severity: ViolationSeverity,
    pub resolved: bool,
}

/// Types of license violations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationType {
    CommercialUseWithoutLicense,
    SourceDisclosureRequired,
    ExcessiveAutomation,
    UnauthorizedDistribution,
    ComplianceFailure,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Warning,
    Minor,
    Major,
    Critical,
}

/// Access levels for different projects/features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Full access to all features
    Full {
        external_adapters: bool,
        cross_project_integration: bool,
        enterprise_features: bool,
        restrictions: Option<Vec<String>>,
    },

    /// Limited access with restrictions
    Limited {
        allowed_features: Vec<String>,
        restrictions: Vec<String>,
        usage_limits: UsageLimits,
    },

    /// Blocked access
    Blocked {
        reason: String,
        appeal_process: Option<String>,
    },
}

/// Usage limits for limited access
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimits {
    pub requests_per_hour: Option<u64>,
    pub compute_hours_per_month: Option<f64>,
    pub storage_gb_limit: Option<f64>,
    pub concurrent_operations: Option<u32>,
}

/// Pricing context for ecosystem-wide billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PricingContext {
    pub base_monthly_cost: f64,
    pub ecosystem_discount: f64,
    pub automation_tax: f64,
    pub volume_discounts: Vec<VolumeDiscount>,
    pub current_month_usage: HashMap<String, f64>,
}

/// Volume discount tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeDiscount {
    pub threshold: f64,
    pub discount_percentage: f64,
    pub applies_to: Vec<String>,
}

/// EcoPrimals integration trait for license coordination
#[async_trait]
pub trait EcoPrimalsIntegration: Send + Sync {
    /// Update license context from ecosystem coordinator
    async fn update_license_context(&self, context: &EcoPrimalsLicenseContext) -> Result<()>;

    /// Verify access for a specific operation
    async fn verify_access(&self, operation: &str) -> Result<AccessLevel>;

    /// Report usage for billing/compliance
    async fn report_usage(&self, usage: UsageReport) -> Result<()>;

    /// Get current license context
    async fn get_license_context(&self) -> Result<EcoPrimalsLicenseContext>;
}

/// Usage report for ecosystem billing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageReport {
    pub project_name: String,
    pub reporting_period: ReportingPeriod,
    pub resource_usage: HashMap<String, f64>,
    pub feature_usage: HashMap<String, u64>,
    pub external_adapter_usage: HashMap<String, u64>,
    pub estimated_cost: f64,
}

/// Reporting period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingPeriod {
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub end_time: chrono::DateTime<chrono::Utc>,
    pub period_type: PeriodType,
}

/// Period types for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeriodType {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

/// Calculate ecosystem pricing based on organization scale and entropy
pub fn calculate_ecosystem_pricing(
    organization_scale: OrganizationScale,
    active_projects: &[String],
    entropy_tier: EntropyTier,
) -> f64 {
    let base_monthly_cost = match organization_scale {
        OrganizationScale::Individual => 0.0, // Always free for individuals
        OrganizationScale::SmallBusiness => 50.0 * active_projects.len() as f64,
        OrganizationScale::RegionalBusiness => 200.0 * active_projects.len() as f64,
        OrganizationScale::NationalEnterprise => 1000.0 * active_projects.len() as f64,
        OrganizationScale::GlobalEnterprise => 5000.0 * active_projects.len() as f64,
        OrganizationScale::Hyperscale => 25000.0 * active_projects.len() as f64,
    };

    // Cross-project discount for ecosystem adoption
    let ecosystem_discount = if active_projects.len() > 2 { 0.8 } else { 1.0 };

    // Automation tax based on entropy tier
    let automation_tax = match entropy_tier {
        EntropyTier::HumanLived => 1.0, // No tax for human involvement
        EntropyTier::Supervised => 1.3, // 30% tax for supervised automation
        EntropyTier::Machine => 2.0,    // 100% tax for pure automation
    };

    base_monthly_cost * ecosystem_discount * automation_tax
}

/// Detect organization scale from usage patterns
pub fn detect_organization_scale(context: &OrganizationContext) -> OrganizationScale {
    match (context.employee_count, context.revenue_annual_usd) {
        (Some(employees), _) if employees >= 50000 => OrganizationScale::Hyperscale,
        (Some(employees), _) if employees >= 5000 => OrganizationScale::GlobalEnterprise,
        (Some(employees), _) if employees >= 500 => OrganizationScale::NationalEnterprise,
        (Some(employees), _) if employees >= 50 => OrganizationScale::RegionalBusiness,
        (Some(employees), _) if employees >= 5 => OrganizationScale::SmallBusiness,
        (_, Some(revenue)) if revenue >= 1_000_000_000.0 => OrganizationScale::GlobalEnterprise,
        (_, Some(revenue)) if revenue >= 100_000_000.0 => OrganizationScale::NationalEnterprise,
        (_, Some(revenue)) if revenue >= 10_000_000.0 => OrganizationScale::RegionalBusiness,
        (_, Some(revenue)) if revenue >= 1_000_000.0 => OrganizationScale::SmallBusiness,
        _ => OrganizationScale::Individual,
    }
}

/// Organization context for scale detection
#[derive(Debug, Clone)]
pub struct OrganizationContext {
    pub employee_count: Option<u32>,
    pub revenue_annual_usd: Option<f64>,
    pub external_customers: bool,
    pub infrastructure_scale: InfrastructureScale,
}

/// Infrastructure scale indicators
#[derive(Debug, Clone)]
pub enum InfrastructureScale {
    Personal,
    SmallTeam,
    Department,
    Enterprise,
    Global,
}

/// Default implementation of EcoPrimals integration
pub struct DefaultEcoPrimalsIntegration {
    context: std::sync::RwLock<Option<EcoPrimalsLicenseContext>>,
    project_name: String,
}

impl DefaultEcoPrimalsIntegration {
    pub fn new(project_name: String) -> Self {
        Self {
            context: std::sync::RwLock::new(None),
            project_name,
        }
    }
}

#[async_trait]
impl EcoPrimalsIntegration for DefaultEcoPrimalsIntegration {
    async fn update_license_context(&self, context: &EcoPrimalsLicenseContext) -> Result<()> {
        let mut guard = self
            .context
            .write()
            .map_err(|e| anyhow::anyhow!("Failed to acquire licensing context lock: {}", e))?;
        *guard = Some(context.clone());

        tracing::info!(
            "Updated license context for project: {} (org scale: {:?})",
            self.project_name,
            context.organization_classification
        );

        Ok(())
    }

    async fn verify_access(&self, operation: &str) -> Result<AccessLevel> {
        let context_guard = self
            .context
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire license context read lock: {}", e))?;

        if let Some(context) = context_guard.as_ref() {
            // Individual users always get full access
            if matches!(
                context.organization_classification,
                OrganizationScale::Individual
            ) {
                return Ok(AccessLevel::Full {
                    external_adapters: true,
                    cross_project_integration: true,
                    enterprise_features: true,
                    restrictions: None,
                });
            }

            // Check project-specific access
            if let Some(access_level) = context.project_access_map.get(&self.project_name) {
                Ok(access_level.clone())
            } else {
                // Default to limited access if not explicitly configured
                Ok(AccessLevel::Limited {
                    allowed_features: vec![operation.to_string()],
                    restrictions: vec!["Corporate license required for full access".to_string()],
                    usage_limits: UsageLimits {
                        requests_per_hour: Some(100),
                        compute_hours_per_month: Some(10.0),
                        storage_gb_limit: Some(5.0),
                        concurrent_operations: Some(2),
                    },
                })
            }
        } else {
            // No context available - assume individual access
            Ok(AccessLevel::Full {
                external_adapters: true,
                cross_project_integration: true,
                enterprise_features: true,
                restrictions: None,
            })
        }
    }

    async fn report_usage(&self, usage: UsageReport) -> Result<()> {
        tracing::info!(
            "Reporting usage for project {}: {:?}",
            usage.project_name,
            usage.resource_usage
        );

        // Report usage to ecosystem coordinator via HTTP
        // ECOSYSTEM_COORDINATOR_URL must be set via environment or discovery
        // No hardcoded fallback - fail fast if not configured
        let coordinator_url = std::env::var("ECOSYSTEM_COORDINATOR_URL")
            .unwrap_or_else(|_| {
                tracing::warn!("ECOSYSTEM_COORDINATOR_URL not set, licensing will not work");
                "http://localhost:8080".to_string() // Last resort for local dev
            });

        let client = reqwest::Client::new();
        let usage_report = serde_json::json!({
            "project_name": usage.project_name,
            "resource_usage": usage.resource_usage,
            "reporting_time": chrono::Utc::now(),
            "license_context": self.get_license_context().await.ok()
        });

        match client
            .post(format!("{}/api/v1/usage-report", coordinator_url))
            .json(&usage_report)
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                tracing::info!("Successfully reported usage to ecosystem coordinator");
            }
            Ok(response) => {
                tracing::warn!("Usage reporting returned status: {}", response.status());
            }
            Err(e) => {
                tracing::warn!("Failed to report usage to ecosystem coordinator: {}", e);
            }
        }
        Ok(())
    }

    async fn get_license_context(&self) -> Result<EcoPrimalsLicenseContext> {
        let context_guard = self
            .context
            .read()
            .map_err(|e| anyhow::anyhow!("Failed to acquire license context read lock: {}", e))?;

        context_guard
            .as_ref()
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("No license context available"))
    }
}

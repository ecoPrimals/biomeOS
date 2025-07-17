use super::compliance::{
    ComplianceReport, ComplianceStatus, ComplianceViolation, ViolationResponse,
};
use super::licensing::LicensingTerms;
use super::types::*;
use crate::BiomeResult;
use async_trait::async_trait;

/// Crypto Lock System - gates external dependencies while preserving AI cat door
#[async_trait]
pub trait CryptoLockInterface {
    /// Validate access to external dependency
    async fn validate_access(
        &self,
        dependency: &ExternalDependency,
        context: &AccessContext,
    ) -> BiomeResult<AccessDecision>;

    /// Register new external dependency
    async fn register_dependency(
        &self,
        dependency: &ExternalDependency,
    ) -> BiomeResult<DependencyId>;

    /// Grant sovereign access key
    async fn grant_sovereign_key(&self, spec: &SovereignKeySpec) -> BiomeResult<SovereignKey>;

    /// Revoke sovereign access key
    async fn revoke_sovereign_key(&self, key_id: &str) -> BiomeResult<()>;

    /// Check compliance status
    async fn check_compliance(&self, usage_pattern: &UsagePattern)
        -> BiomeResult<ComplianceStatus>;

    /// Generate compliance report
    async fn generate_compliance_report(&self) -> BiomeResult<ComplianceReport>;

    /// Update licensing terms
    async fn update_licensing(&self, terms: &LicensingTerms) -> BiomeResult<()>;

    /// Monitor dependency usage
    async fn monitor_usage(&self, dependency_id: &DependencyId) -> BiomeResult<UsageMetrics>;
}

/// Violation handler
pub trait ViolationHandler: Send + Sync {
    fn handle_violation(&self, violation: &ComplianceViolation) -> BiomeResult<ViolationResponse>;
}

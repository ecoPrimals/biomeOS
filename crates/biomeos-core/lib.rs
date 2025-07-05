//! # biomeOS Core
//!
//! Universal biological computing platform that allows any system to participate
//! as a "Primal" through standardized interfaces. Completely agnostic to specific
//! implementations while maintaining biological metaphors and sovereign architecture.

use std::collections::HashMap;
use std::time::Duration;

/// Core trait that all Primals must implement
pub trait Primal {
    /// Unique identifier for this Primal type (completely agnostic)
    fn primal_type(&self) -> String;
    
    /// Capabilities this Primal provides to the biome
    fn capabilities(&self) -> Vec<Capability>;
    
    /// Current health and operational status
    fn health_status(&self) -> HealthStatus;
    
    /// Resource requirements for optimal operation
    fn resource_requirements(&self) -> ResourceRequirements;
    
    /// MYCORRHIZA compliance and energy flow management
    fn mycorrhiza_compliance(&self) -> ComplianceStatus;
    
    /// External access requests that need MYCORRHIZA approval
    fn external_access_requests(&self) -> Vec<ExternalAccessRequest>;
    
    /// Enforce energy flow state changes from MYCORRHIZA
    fn enforce_energy_flow_state(&mut self, state: EnergyFlowState) -> Result<(), PrimalError>;
    
    /// Internal communication with other Primals (always free)
    fn internal_communicate(&self, message: PrimalMessage) -> Result<PrimalResponse, PrimalError>;
}

/// MYCORRHIZA compliance status for Primals
#[derive(Debug, Clone, PartialEq)]
pub enum ComplianceStatus {
    Compliant {
        external_access_locked: bool,
        internal_communication_free: bool,
        personal_ai_accessible: bool,
    },
    NonCompliant {
        violations: Vec<ComplianceViolation>,
        risk_level: RiskLevel,
    },
    Transitioning {
        from_state: EnergyFlowState,
        to_state: EnergyFlowState,
        progress: f32,
    },
}

/// Energy flow states managed by MYCORRHIZA
#[derive(Debug, Clone, PartialEq)]
pub enum EnergyFlowState {
    /// Default sovereign state - locked to externals, AI cat door open
    Closed {
        personal_ai_enabled: bool,
        local_models: Vec<String>,
        api_providers: Vec<String>,
    },
    
    /// Trust-based selective opening
    PrivateOpen {
        personal_ai_enabled: bool,
        trusted_grants: Vec<TrustedGrant>,
        monitoring_enabled: bool,
    },
    
    /// Commercial integrations for enterprises
    CommercialOpen {
        personal_ai_enabled: bool,
        licensed_providers: Vec<LicensedProvider>,
        monthly_budget: Option<u64>,
    },
}

/// External access request that needs MYCORRHIZA approval
#[derive(Debug, Clone)]
pub struct ExternalAccessRequest {
    pub request_id: String,
    pub primal_source: String,
    pub target_service: String,
    pub access_type: AccessType,
    pub justification: String,
    pub data_sensitivity: DataSensitivity,
    pub duration: Option<Duration>,
}

/// Trust-based access grant for private open systems
#[derive(Debug, Clone)]
pub struct TrustedGrant {
    pub recipient: String,
    pub crypto_key: String,
    pub scope: Vec<String>,
    pub granted_by: String,
    pub expires: chrono::DateTime<chrono::Utc>,
    pub revocable: bool,
}

/// Commercial license for enterprise access
#[derive(Debug, Clone)]
pub struct LicensedProvider {
    pub provider: String,
    pub license_key: String,
    pub payment_status: PaymentStatus,
    pub access_scope: Vec<String>,
    pub monthly_fee: u64,
    pub contract_expires: chrono::DateTime<chrono::Utc>,
}

/// Types of external access requests
#[derive(Debug, Clone, PartialEq)]
pub enum AccessType {
    ApiCall,
    DataTransfer,
    ComputeOffload,
    NetworkConnection,
    ServiceIntegration,
}

/// Data sensitivity levels for access control
#[derive(Debug, Clone, PartialEq)]
pub enum DataSensitivity {
    Public,
    Internal,
    Confidential,
    Sovereign,
}

/// Payment status for commercial licenses
#[derive(Debug, Clone, PartialEq)]
pub enum PaymentStatus {
    Active,
    Pending,
    Overdue,
    Suspended,
    Cancelled,
}

/// Compliance violations detected by MYCORRHIZA
#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    pub violation_type: ViolationType,
    pub description: String,
    pub detected_at: chrono::DateTime<chrono::Utc>,
    pub evidence: Vec<String>,
    pub risk_level: RiskLevel,
}

/// Types of compliance violations
#[derive(Debug, Clone, PartialEq)]
pub enum ViolationType {
    UnauthorizedExternalAccess,
    ApiBypass,
    DataExfiltration,
    ProtocolViolation,
    LicenseViolation,
}

/// Risk levels for violations and non-compliance
#[derive(Debug, Clone, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
} 
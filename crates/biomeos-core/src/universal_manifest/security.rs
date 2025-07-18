//! Security Requirements Module
//!
//! This module defines security requirements for biomes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// Authentication requirements
    pub authentication: AuthenticationSpec,
    /// Authorization requirements
    pub authorization: AuthorizationSpec,
    /// Encryption requirements
    pub encryption: EncryptionSpec,
    /// Compliance requirements
    pub compliance: ComplianceSpec,
    /// Audit requirements
    pub audit: AuditSpec,
    /// Backup requirements
    pub backup: BackupSpec,
    /// Key management requirements
    pub key_management: KeyManagementSpec,
}

/// Authentication specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationSpec {
    /// Authentication method
    pub method: String,
    /// Authentication configuration
    pub config: HashMap<String, String>,
    /// Enable multi-factor authentication
    pub mfa_enabled: bool,
}

/// Authorization specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationSpec {
    /// Authorization method
    pub method: String,
    /// Authorization configuration
    pub config: HashMap<String, String>,
    /// Role-based access control
    pub rbac_enabled: bool,
}

/// Encryption specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSpec {
    /// Encryption algorithm
    pub algorithm: String,
    /// Key size
    pub key_size: u32,
    /// Enable encryption at rest
    pub at_rest: bool,
    /// Enable encryption in transit
    pub in_transit: bool,
}

/// Compliance specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSpec {
    /// Compliance frameworks
    pub frameworks: Vec<String>,
    /// Compliance requirements
    pub requirements: HashMap<String, String>,
    /// Enable compliance monitoring
    pub monitoring_enabled: bool,
}

/// Audit specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSpec {
    /// Enable audit logging
    pub enabled: bool,
    /// Audit log level
    pub level: String,
    /// Audit storage
    pub storage: AuditStorage,
}

/// Audit storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, String>,
}

/// Backup specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupSpec {
    /// Enable backups
    pub enabled: bool,
    /// Backup schedule
    pub schedule: String,
    /// Backup storage
    pub storage: BackupStorage,
}

/// Backup storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStorage {
    /// Storage type
    pub storage_type: String,
    /// Storage configuration
    pub config: HashMap<String, String>,
}

/// Key management specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementSpec {
    /// Key management service
    pub service: String,
    /// Key configuration
    pub config: HashMap<String, String>,
    /// Enable key rotation
    pub rotation_enabled: bool,
}

/// Network security specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSecuritySpec {
    /// Enable network security
    pub enabled: bool,
    /// Security policies
    pub policies: Vec<SecurityPolicy>,
}

/// Security policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    /// Policy name
    pub name: String,
    /// Policy rules
    pub rules: Vec<SecurityRule>,
}

/// Security rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    /// Rule name
    pub name: String,
    /// Rule action
    pub action: String,
    /// Rule conditions
    pub conditions: HashMap<String, String>,
} 
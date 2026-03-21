// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Security Specifications for Manifests
//!
//! This module contains all security-related types including BiomeSecuritySpec,
//! SecurityPolicySpec, AccessControlSpec, RBAC, encryption, and audit configurations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Biome security specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeSecuritySpec {
    /// Security policies
    pub policies: Vec<SecurityPolicySpec>,

    /// Access control
    pub access_control: Option<AccessControlSpec>,

    /// Encryption settings
    pub encryption: Option<EncryptionSpec>,

    /// Audit configuration
    pub audit: Option<AuditSpec>,
}

/// Security policy specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicySpec {
    /// Policy name
    pub name: String,

    /// Policy type
    pub policy_type: SecurityPolicyType,

    /// Policy rules
    pub rules: Vec<SecurityRuleSpec>,
}

/// Security policy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityPolicyType {
    /// Network-level security policy
    NetworkPolicy,
    /// Pod security policy (PSP)
    PodSecurityPolicy,
    /// Custom security policy
    CustomPolicy,
}

/// Security rule specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRuleSpec {
    /// Rule name
    pub name: String,

    /// Rule action
    pub action: SecurityAction,

    /// Rule conditions
    pub conditions: Vec<SecurityConditionSpec>,
}

/// Security actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Audit the action (log only)
    Audit,
}

/// Security condition specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConditionSpec {
    /// Field to check
    pub field: String,

    /// Condition operator
    pub operator: ConditionOperator,

    /// Values to match
    pub values: Vec<String>,
}

/// Condition operators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionOperator {
    /// Equals comparison
    Equals,
    /// Not-equals comparison
    NotEquals,
    /// Value is in the set
    In,
    /// Value is not in the set
    NotIn,
    /// Key exists
    Exists,
    /// Key does not exist
    DoesNotExist,
    /// Greater than comparison
    GreaterThan,
    /// Less than comparison
    LessThan,
}

/// Access control specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlSpec {
    /// RBAC configuration
    pub rbac: Option<RbacSpec>,

    /// ABAC configuration
    pub abac: Option<AbacSpec>,

    /// Service accounts
    pub service_accounts: Vec<ServiceAccountSpec>,
}

/// RBAC specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacSpec {
    /// Roles
    pub roles: Vec<RoleSpec>,

    /// Role bindings
    pub role_bindings: Vec<RoleBindingSpec>,
}

/// Role specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleSpec {
    /// Role name
    pub name: String,

    /// Permissions
    pub permissions: Vec<PermissionSpec>,
}

/// Permission specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSpec {
    /// Resource
    pub resource: String,

    /// Actions
    pub actions: Vec<String>,

    /// Resource names
    pub resource_names: Vec<String>,
}

/// Role binding specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleBindingSpec {
    /// Binding name
    pub name: String,

    /// Role reference
    pub role_ref: RoleRef,

    /// Subjects
    pub subjects: Vec<SubjectSpec>,
}

/// Role reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleRef {
    /// Role kind
    pub kind: RoleKind,

    /// Role name
    pub name: String,
}

/// Role kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoleKind {
    /// Namespace-scoped role
    Role,
    /// Cluster-scoped role
    ClusterRole,
}

/// Subject specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectSpec {
    /// Subject kind
    pub kind: SubjectKind,

    /// Subject name
    pub name: String,

    /// Namespace
    pub namespace: Option<String>,
}

/// Subject kinds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubjectKind {
    /// User subject
    User,
    /// Group subject
    Group,
    /// Service account subject
    ServiceAccount,
}

/// ABAC specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbacSpec {
    /// Policy file path
    pub policy_file: String,

    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

/// Service account specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceAccountSpec {
    /// Account name
    pub name: String,

    /// Annotations
    pub annotations: HashMap<String, String>,

    /// Secrets
    pub secrets: Vec<String>,
}

/// Encryption specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSpec {
    /// Encryption at rest
    pub at_rest: Option<EncryptionAtRestSpec>,

    /// Encryption in transit
    pub in_transit: Option<EncryptionInTransitSpec>,
}

/// Encryption at rest specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionAtRestSpec {
    /// Provider
    pub provider: String,

    /// Key management
    pub key_management: KeyManagementSpec,
}

/// Encryption in transit specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionInTransitSpec {
    /// TLS configuration
    pub tls: TlsSpec,

    /// mTLS configuration
    pub mtls: Option<MutualTlsSpec>,
}

/// Key management specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementSpec {
    /// Provider
    pub provider: String,

    /// Key ID
    pub key_id: String,

    /// Rotation policy
    pub rotation_policy: Option<KeyRotationPolicy>,
}

/// Key rotation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotationPolicy {
    /// Rotation interval in days
    pub interval_days: u32,

    /// Automatic rotation
    pub auto_rotate: bool,
}

/// TLS specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSpec {
    /// Certificate source
    pub cert_source: CertificateSource,

    /// Minimum TLS version
    pub min_version: TlsVersion,

    /// Cipher suites
    pub cipher_suites: Vec<String>,
}

/// Certificate sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateSource {
    /// Self-signed certificate
    SelfSigned,
    /// Certificate from cert-manager
    CertManager {
        /// Issuer name
        issuer: String,
    },
    /// External certificate from secret
    External {
        /// Secret name containing the certificate
        secret_name: String,
    },
}

/// TLS versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TlsVersion {
    /// TLS version 1.2
    #[serde(rename = "1.2")]
    V1_2,
    /// TLS version 1.3
    #[serde(rename = "1.3")]
    V1_3,
}

/// Mutual TLS specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutualTlsSpec {
    /// Client certificate verification
    pub client_cert_verification: ClientCertVerification,

    /// Trusted CA bundle
    pub ca_bundle: Option<String>,
}

/// Client certificate verification modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientCertVerification {
    /// Client certificate is required
    Required,
    /// Client certificate is optional
    Optional,
    /// Client certificate verification is disabled
    Disabled,
}

/// Audit specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSpec {
    /// Enable audit logging
    pub enabled: bool,

    /// Audit log file path
    pub log_file: Option<String>,

    /// Audit webhook
    pub webhook: Option<AuditWebhookSpec>,

    /// Log format
    pub format: AuditLogFormat,

    /// Retention policy
    pub retention: Option<AuditRetentionPolicy>,
}

/// Audit webhook specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditWebhookSpec {
    /// Webhook URL
    pub url: String,

    /// Client certificate
    pub client_cert: Option<String>,

    /// Client key
    pub client_key: Option<String>,

    /// CA certificate
    pub ca_cert: Option<String>,
}

/// Audit log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditLogFormat {
    /// JSON formatted logs
    Json,
    /// Legacy text format
    Legacy,
}

/// Audit retention policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRetentionPolicy {
    /// Maximum log age in days
    pub max_age_days: u32,

    /// Maximum log size in MB
    pub max_size_mb: u32,

    /// Maximum number of backup files
    pub max_backups: u32,
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Service Security Configurations
//!
//! This module contains security-related types including authentication,
//! authorization, encryption, and secrets management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSecurity {
    /// Security context
    pub security_context: SecurityContext,

    /// Authentication configuration
    pub authentication: Option<AuthenticationConfig>,

    /// Authorization configuration
    pub authorization: Option<AuthorizationConfig>,

    /// Encryption configuration
    pub encryption: EncryptionConfig,

    /// Secrets configuration
    pub secrets: SecretsConfig,
}

/// Security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User ID to run as
    pub run_as_user: Option<u32>,

    /// Group ID to run as
    pub run_as_group: Option<u32>,

    /// Run as non-root user
    pub run_as_non_root: bool,

    /// Read-only root filesystem
    pub read_only_root_fs: bool,

    /// Allow privilege escalation
    pub allow_privilege_escalation: bool,

    /// Security capabilities
    pub capabilities: SecurityCapabilities,

    /// SELinux options
    pub selinux: Option<SeLinuxOptions>,

    /// AppArmor profile
    pub apparmor_profile: Option<String>,

    /// Seccomp profile
    pub seccomp_profile: Option<SeccompProfile>,
}

/// Security capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityCapabilities {
    /// Capabilities to add
    pub add: Vec<String>,

    /// Capabilities to drop
    pub drop: Vec<String>,
}

/// SELinux options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeLinuxOptions {
    /// SELinux user
    pub user: Option<String>,

    /// SELinux role
    pub role: Option<String>,

    /// SELinux type
    #[serde(rename = "type")]
    pub selinux_type: Option<String>,

    /// SELinux level
    pub level: Option<String>,
}

/// Seccomp profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeccompProfile {
    /// Runtime default profile
    RuntimeDefault,

    /// Unconfined (no restrictions)
    Unconfined,

    /// Localhost profile
    Localhost(String),
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Authentication methods
    pub methods: Vec<AuthMethod>,

    /// Default authentication method
    pub default_method: Option<String>,

    /// Multi-factor authentication
    pub mfa: Option<MfaConfig>,

    /// Token configuration
    pub tokens: Option<TokenConfig>,

    /// JWT configuration
    pub jwt: Option<JwtConfig>,

    /// OAuth2 configuration
    pub oauth2: Option<OAuth2Config>,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// Username/password authentication
    Basic,

    /// API key authentication
    ApiKey,

    /// JWT token authentication
    Jwt,

    /// OAuth2 authentication
    OAuth2,

    /// LDAP authentication
    Ldap,

    /// SAML authentication
    Saml,

    /// Certificate-based authentication
    Certificate,

    /// Custom authentication method
    Custom(String),
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    /// MFA enabled
    pub enabled: bool,

    /// Required MFA methods
    pub required_methods: Vec<MfaMethod>,

    /// Optional MFA methods
    pub optional_methods: Vec<MfaMethod>,

    /// MFA timeout (seconds)
    pub timeout: u32,
}

/// MFA methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod {
    /// Time-based OTP
    Totp,

    /// SMS verification
    Sms,

    /// Email verification
    Email,

    /// Hardware token
    Hardware,

    /// Biometric authentication
    Biometric,

    /// Custom MFA method
    Custom(String),
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Token lifetime (seconds)
    pub lifetime: u32,

    /// Token refresh enabled
    pub refresh_enabled: bool,

    /// Token algorithm
    pub algorithm: String,

    /// Token secret
    pub secret: String,
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret
    pub secret: String,

    /// JWT algorithm
    pub algorithm: String,

    /// JWT audience
    pub audience: Vec<String>,

    /// JWT issuer
    pub issuer: String,

    /// Token lifetime
    pub lifetime: u32,
}

/// OAuth2 configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuth2Config {
    /// Client ID
    pub client_id: String,

    /// Client secret
    pub client_secret: String,

    /// Authorization URL
    pub auth_url: String,

    /// Token URL
    pub token_url: String,

    /// Scopes
    pub scopes: Vec<String>,
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Authorization model
    pub model: AuthzModel,

    /// Policies
    pub policies: Vec<AuthzPolicy>,

    /// Default action
    pub default_action: AuthzAction,
}

/// Authorization models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthzModel {
    /// Role-based access control
    Rbac,

    /// Attribute-based access control
    Abac,

    /// Relationship-based access control
    ReBAC,

    /// Custom model
    Custom(String),
}

/// Authorization policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzPolicy {
    /// Policy name
    pub name: String,

    /// Policy rules
    pub rules: Vec<AuthzRule>,

    /// Policy effect
    pub effect: AuthzEffect,
}

/// Authorization rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthzRule {
    /// Rule name
    pub name: String,

    /// Resource patterns
    pub resources: Vec<String>,

    /// Actions
    pub actions: Vec<String>,

    /// Conditions
    pub conditions: HashMap<String, serde_json::Value>,
}

/// Authorization effects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthzEffect {
    /// Allow the request
    Allow,
    /// Deny the request
    Deny,
}

/// Authorization actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthzAction {
    /// Allow the action
    Allow,
    /// Deny the action
    Deny,
    /// Audit the action (log only)
    Audit,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption at rest
    pub at_rest: bool,

    /// Encryption in transit
    pub in_transit: bool,

    /// Encryption algorithm
    pub algorithm: String,

    /// Key management
    pub key_management: KeyManagement,
}

/// Key management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagement {
    /// Key provider
    pub provider: KeyProvider,

    /// Key rotation
    pub rotation: KeyRotation,
}

/// Key providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyProvider {
    /// Generated keys
    Generated,

    /// External key management service
    External {
        /// Provider name
        provider: String,
        /// Configuration
        config: HashMap<String, String>,
    },

    /// HSM-based keys
    Hsm {
        /// HSM configuration
        config: HashMap<String, String>,
    },

    /// Vault-based keys
    Vault {
        /// Vault address
        address: String,
        /// Vault path
        path: String,
    },
}

/// Key rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRotation {
    /// Rotation enabled
    pub enabled: bool,

    /// Rotation interval (seconds)
    pub interval: u32,

    /// Automatic rotation
    pub automatic: bool,
}

/// Secrets configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsConfig {
    /// Secrets provider
    pub provider: SecretsProvider,

    /// Secret references
    pub secrets: Vec<SecretReference>,
}

/// Secrets providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecretsProvider {
    /// Kubernetes secrets
    Kubernetes,

    /// HashiCorp Vault
    Vault {
        /// Vault address
        address: String,
        /// Vault token
        token: String,
    },

    /// AWS Secrets Manager
    AwsSecretsManager {
        /// AWS region
        region: String,
    },

    /// Azure Key Vault
    AzureKeyVault {
        /// Vault URL
        vault_url: String,
    },

    /// Google Secret Manager
    GoogleSecretManager {
        /// Project ID
        project_id: String,
    },

    /// External secrets service
    External {
        /// Service URL
        url: String,
        /// Authentication
        auth: HashMap<String, String>,
    },
}

/// Secret reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretReference {
    /// Secret name
    pub name: String,

    /// Secret key
    pub key: String,

    /// Environment variable name
    pub env_var: Option<String>,

    /// File mount path
    pub file_path: Option<String>,

    /// Secret is optional
    pub optional: bool,
}

/// Network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Policy name
    pub name: String,

    /// Policy type
    pub policy_type: NetworkPolicyType,

    /// Pod selector
    pub pod_selector: HashMap<String, String>,

    /// Ingress rules
    pub ingress: Vec<NetworkPolicyRule>,

    /// Egress rules
    pub egress: Vec<NetworkPolicyRule>,
}

/// Network policy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyType {
    /// Ingress (incoming) traffic policy
    Ingress,
    /// Egress (outgoing) traffic policy
    Egress,
    /// Both ingress and egress
    Both,
}

/// Network policy rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyRule {
    /// From/to peers
    pub peers: Vec<NetworkPolicyPeer>,

    /// Allowed ports
    pub ports: Vec<NetworkPolicyPort>,
}

/// Network policy peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyPeer {
    /// Pod selector
    pub pod_selector: Option<HashMap<String, String>>,

    /// Namespace selector
    pub namespace_selector: Option<HashMap<String, String>>,

    /// IP block
    pub ip_block: Option<NetworkPolicyIPBlock>,
}

/// Network policy IP block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyIPBlock {
    /// CIDR block
    pub cidr: String,

    /// Except blocks
    pub except: Vec<String>,
}

/// Network policy port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicyPort {
    /// Port number or name
    pub port: NetworkPolicyPortValue,

    /// Protocol
    pub protocol: NetworkPolicyProtocol,

    /// End port (for port ranges)
    pub end_port: Option<u16>,
}

/// Network policy port value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyPortValue {
    /// Port number
    Number(u16),
    /// Port name
    Name(String),
}

/// Network policy protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyProtocol {
    /// TCP protocol
    TCP,
    /// UDP protocol
    UDP,
    /// SCTP protocol
    SCTP,
}

/// Network policy action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyAction {
    /// Allow traffic
    Allow,
    /// Deny traffic
    Deny,
    /// Log traffic
    Log,
}

/// Default implementation for ServiceSecurity
impl Default for ServiceSecurity {
    fn default() -> Self {
        Self {
            security_context: SecurityContext {
                run_as_user: Some(1000),
                run_as_group: Some(1000),
                run_as_non_root: true,
                read_only_root_fs: true,
                allow_privilege_escalation: false,
                capabilities: SecurityCapabilities {
                    add: vec![],
                    drop: vec!["ALL".to_string()],
                },
                selinux: None,
                apparmor_profile: None,
                seccomp_profile: Some(SeccompProfile::RuntimeDefault),
            },
            authentication: None,
            authorization: None,
            encryption: EncryptionConfig {
                at_rest: false,
                in_transit: true,
                algorithm: "AES-256-GCM".to_string(),
                key_management: KeyManagement {
                    provider: KeyProvider::Generated,
                    rotation: KeyRotation {
                        enabled: false,
                        interval: 86400, // 24 hours
                        automatic: false,
                    },
                },
            },
            secrets: SecretsConfig {
                provider: SecretsProvider::Kubernetes,
                secrets: vec![],
            },
        }
    }
}

#[cfg(test)]
#[path = "security_tests.rs"]
mod tests;

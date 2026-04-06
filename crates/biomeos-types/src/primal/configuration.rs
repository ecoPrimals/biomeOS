// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal Configuration System
//!
//! This module contains configuration types for primals including
//! network configuration, security settings, and dependency management.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use super::capabilities::PrimalCapability;
use super::core::{PrimalMetadata, PrimalType, ResourceRequirements};

/// Primal configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalConfiguration {
    /// Unique configuration identifier
    pub id: Uuid,

    /// Primal type information
    pub primal_type: PrimalType,

    /// Configuration parameters (structured, not just `HashMap`)
    pub configuration: ConfigurationParameters,

    /// Dependencies on other primals
    pub dependencies: Vec<PrimalDependency>,

    /// Capabilities this primal provides
    pub capabilities: Vec<PrimalCapability>,

    /// Resource requirements and limits
    pub resources: ResourceRequirements,

    /// Network configuration
    pub networking: NetworkConfiguration,

    /// Security configuration
    pub security: SecurityConfiguration,

    /// Metadata and tags
    pub metadata: PrimalMetadata,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

/// Structured configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationParameters {
    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Configuration files and their contents
    pub config_files: HashMap<String, String>,

    /// Feature flags
    pub features: HashMap<String, bool>,

    /// Structured parameters by category
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Primal dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDependency {
    /// Dependency name or identifier
    pub name: String,

    /// Required primal type
    pub primal_type: PrimalType,

    /// Required capabilities
    pub required_capabilities: Vec<String>,

    /// Version constraints
    pub version_constraint: String,

    /// Whether this dependency is optional
    pub optional: bool,
}

/// Network configuration for primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfiguration {
    /// Ports to expose
    pub ports: Vec<PortConfiguration>,

    /// Ingress configuration
    pub ingress: Option<IngressConfiguration>,

    /// Network policies
    pub policies: Vec<NetworkPolicy>,
}

/// Port configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortConfiguration {
    /// Port name/identifier
    pub name: String,

    /// Port number
    pub port: u16,

    /// Protocol (TCP, UDP, HTTP, HTTPS)
    pub protocol: String,

    /// Whether to expose externally
    pub expose: bool,

    /// Load balancing configuration
    pub load_balancing: Option<LoadBalancingConfig>,
}

/// Ingress configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngressConfiguration {
    /// Hostname for ingress
    pub hostname: String,

    /// Path prefix
    pub path_prefix: String,

    /// TLS configuration
    pub tls_enabled: bool,

    /// TLS certificate configuration
    pub tls_config: Option<TlsConfiguration>,
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfiguration {
    /// Certificate source
    pub cert_source: CertificateSource,

    /// Minimum TLS version
    pub min_version: String,

    /// Cipher suites
    pub cipher_suites: Vec<String>,
}

/// Certificate source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificateSource {
    /// Let's Encrypt automatic certificate
    LetsEncrypt {
        /// Domains to obtain certificates for
        domains: Vec<String>,
    },

    /// Provided certificate files
    Files {
        /// Path to certificate file
        cert_path: String,
        /// Path to private key file
        key_path: String,
    },

    /// Certificate from secret store
    Secret {
        /// Secret name containing the certificate
        secret_name: String,
    },
}

/// Network policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Policy name
    pub name: String,

    /// Policy type
    pub policy_type: NetworkPolicyType,

    /// Rules
    pub rules: Vec<NetworkRule>,
}

/// Network policy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkPolicyType {
    /// Inbound traffic rules
    Ingress,
    /// Outbound traffic rules
    Egress,
    /// Both inbound and outbound
    Both,
}

/// Network rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRule {
    /// Source/destination
    pub target: NetworkTarget,

    /// Allowed ports
    pub ports: Vec<u16>,

    /// Allowed protocols
    pub protocols: Vec<String>,
}

/// Network target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkTarget {
    /// Any source/destination
    Any,

    /// Specific CIDR block
    Cidr(String),

    /// Specific primal
    Primal(String),

    /// Primal category
    Category(String),
}

/// Load balancing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancingConfig {
    /// Load balancing algorithm
    pub algorithm: LoadBalancingAlgorithm,

    /// Health check configuration
    pub health_check: Option<HealthCheckConfig>,

    /// Session stickiness
    pub sticky_sessions: bool,
}

/// Load balancing algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingAlgorithm {
    /// Round-robin distribution
    RoundRobin,
    /// Least active connections
    LeastConnections,
    /// Weighted round-robin
    WeightedRoundRobin {
        /// Per-target weights
        weights: HashMap<String, u32>,
    },
    /// Hash by client IP
    IpHash,
    /// Random selection
    Random,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check path
    pub path: String,

    /// Check interval in seconds
    pub interval_secs: u32,

    /// Timeout in seconds
    pub timeout_secs: u32,

    /// Healthy threshold
    pub healthy_threshold: u32,

    /// Unhealthy threshold
    pub unhealthy_threshold: u32,
}

/// Security configuration for primals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfiguration {
    /// Authentication requirements
    pub authentication: AuthenticationConfig,

    /// Authorization policies
    pub authorization: AuthorizationConfig,

    /// Encryption settings
    pub encryption: EncryptionConfig,

    /// Audit configuration
    pub audit: AuditConfig,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Authentication method
    pub method: AuthenticationMethod,

    /// Token configuration
    pub token_config: Option<TokenConfig>,

    /// Multi-factor authentication
    pub mfa_required: bool,
}

/// Authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthenticationMethod {
    /// No authentication required
    None,
    /// API key authentication
    ApiKey,
    /// Bearer token authentication
    BearerToken,
    /// Mutual TLS (client certificate)
    MutualTls,
    /// OAuth 2.0 authentication
    OAuth2,
    /// Custom authentication handler
    Custom {
        /// Custom method identifier
        method: String,
    },
}

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    /// Token lifetime in seconds
    pub lifetime_secs: u32,

    /// Refresh token support
    pub refresh_enabled: bool,

    /// Token signing key
    pub signing_key: Option<String>,
}

/// Authorization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationConfig {
    /// Authorization model
    pub model: AuthorizationModel,

    /// Policies
    pub policies: Vec<AuthorizationPolicy>,
}

/// Authorization models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthorizationModel {
    /// No authorization
    None,
    /// Role-Based Access Control
    Rbac,
    /// Attribute-Based Access Control
    Abac,
    /// Custom authorization model
    Custom {
        /// Model identifier
        model: String,
    },
}

/// Authorization policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationPolicy {
    /// Policy name
    pub name: String,

    /// Resources this policy applies to
    pub resources: Vec<String>,

    /// Actions allowed
    pub actions: Vec<String>,

    /// Conditions
    pub conditions: HashMap<String, serde_json::Value>,
}

/// Encryption configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    /// Encryption at rest
    pub at_rest: bool,

    /// Encryption in transit
    pub in_transit: bool,

    /// Key management
    pub key_management: KeyManagementConfig,
}

/// Key management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagementConfig {
    /// Key source
    pub key_source: KeySource,

    /// Key rotation period in days
    pub rotation_period_days: Option<u32>,
}

/// Key sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeySource {
    /// Auto-generated keys
    Generated,
    /// Keys provided from a file
    Provided {
        /// Path to the key file
        key_path: String,
    },
    /// Keys from a Hardware Security Module
    Hsm {
        /// HSM configuration parameters
        hsm_config: HashMap<String, String>,
    },
    /// Keys from `HashiCorp` Vault
    Vault {
        /// Vault secret path
        vault_path: String,
    },
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    /// Enable audit logging
    pub enabled: bool,

    /// Audit log destination
    pub destination: AuditDestination,

    /// Events to audit
    pub events: Vec<AuditEvent>,
}

/// Audit destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditDestination {
    /// File-based audit log
    File {
        /// Log file path
        path: String,
    },
    /// Syslog destination
    Syslog {
        /// Syslog server address
        server: String,
    },
    /// Database destination
    Database {
        /// Database connection string
        connection: String,
    },
    /// External service destination
    External {
        /// External endpoint URL
        endpoint: String,
    },
}

/// Audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// Authentication events
    Authentication,
    /// Authorization events
    Authorization,
    /// Configuration change events
    Configuration,
    /// Data access events
    DataAccess,
    /// Administrative actions
    Administrative,
    /// All event types
    All,
}

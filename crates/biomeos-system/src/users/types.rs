//! User management types and data structures
//!
//! This module contains all the type definitions for user management,
//! including user accounts, sessions, permissions, and BearDog integration.

use biomeos_core::{BeardogAccessLevel, GeneticBeardogKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// User manager for biomeOS with BearDog integration
pub struct UserManager {
    /// Configuration
    pub config: UserConfig,
    /// User accounts
    pub users: RwLock<HashMap<String, User>>,
    /// User sessions (managed by BearDog)
    pub sessions: RwLock<HashMap<String, UserSession>>,
    /// User groups
    pub groups: RwLock<HashMap<String, UserGroup>>,
    /// BearDog security provider integration
    pub beardog_provider: Option<BeardogSecurityProvider>,
}

/// BearDog security provider interface
#[derive(Debug, Clone)]
pub struct BeardogSecurityProvider {
    /// BearDog endpoint for user authentication
    pub auth_endpoint: String,
    /// BearDog endpoint for key management
    pub key_management_endpoint: String,
    /// BearDog endpoint for secret storage
    pub secret_storage_endpoint: String,
    /// BearDog service token for system operations
    pub service_token: String,
    /// Enable genetic key support
    pub genetic_keys_enabled: bool,
    /// Enable HSM integration
    pub hsm_integration_enabled: bool,
}

/// User configuration with BearDog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    /// User database path
    pub user_db_path: PathBuf,
    /// Home directory base
    pub home_dir_base: PathBuf,
    /// Default shell
    pub default_shell: PathBuf,
    /// Default user group
    pub default_group: String,
    /// BearDog integration configuration
    pub beardog_config: BeardogIntegrationConfig,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Enable guest account
    pub enable_guest: bool,
    /// Enable sudo access
    pub enable_sudo: bool,
    /// Enable genetic BearDog keys
    pub enable_genetic_keys: bool,
}

/// BearDog integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogIntegrationConfig {
    /// BearDog service endpoint
    pub endpoint: String,
    /// Authentication method for BearDog
    pub auth_method: BeardogAuthMethod,
    /// Enable key management through BearDog
    pub key_management_enabled: bool,
    /// Enable secret storage through BearDog
    pub secret_storage_enabled: bool,
    /// Enable audit logging through BearDog
    pub audit_logging_enabled: bool,
    /// Enable HSM integration
    pub hsm_integration_enabled: bool,
    /// BearDog compliance mode
    pub compliance_mode: String, // "standard", "fips140", "soc2", "gdpr"
    /// Security level for user operations
    pub security_level: String, // "public", "internal", "confidential", "secret"
}

/// BearDog authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BeardogAuthMethod {
    /// Mutual TLS authentication
    MutualTLS {
        cert_path: PathBuf,
        key_path: PathBuf,
        ca_path: PathBuf,
    },
    /// API key authentication
    ApiKey { key_reference: String },
    /// Service account authentication
    ServiceAccount {
        account_id: String,
        private_key_reference: String,
    },
    /// Genetic BearDog key authentication
    GeneticKey {
        parent_key_fingerprint: String,
        genetic_lineage: Vec<String>,
    },
}

/// User account with BearDog security integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User ID
    pub id: u32,
    /// Username
    pub username: String,
    /// Full name
    pub full_name: Option<String>,
    /// Email address
    pub email: Option<String>,
    /// BearDog key reference (replaces password_hash)
    pub beardog_key_reference: String,
    /// Genetic BearDog key (if enabled)
    pub genetic_key: Option<GeneticBeardogKey>,
    /// BearDog access level
    pub access_level: BeardogAccessLevel,
    /// User home directory
    pub home_dir: PathBuf,
    /// Default shell
    pub shell: PathBuf,
    /// Primary group ID
    pub primary_group: u32,
    /// Additional groups
    pub groups: Vec<String>,
    /// User status
    pub status: UserStatus,
    /// Account creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last login time
    pub last_login: Option<chrono::DateTime<chrono::Utc>>,
    /// Account expiry
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    /// User permissions
    pub permissions: Vec<Permission>,
    /// SSH public keys (stored in BearDog)
    pub ssh_key_references: Vec<String>,
    /// API keys (stored in BearDog)
    pub api_key_references: Vec<String>,
    /// User metadata
    pub metadata: HashMap<String, String>,
}

/// User status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserStatus {
    /// User account is active
    Active,
    /// User account is disabled
    Disabled,
    /// User account is locked
    Locked,
    /// User account is expired
    Expired,
    /// User account is pending activation
    Pending,
    /// User account is pending BearDog key generation
    PendingKeyGeneration,
}

/// User permission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// Permission name
    pub name: String,
    /// Permission description
    pub description: String,
    /// Permission scope
    pub scope: PermissionScope,
    /// Permission level
    pub level: PermissionLevel,
    /// BearDog authorization reference
    pub beardog_auth_reference: Option<String>,
}

/// Permission scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionScope {
    /// System-wide permission
    System,
    /// User-specific permission
    User,
    /// Group-specific permission
    Group,
    /// Resource-specific permission
    Resource { resource: String },
    /// Primal-specific permission
    Primal { primal: String },
    /// Biome-specific permission
    Biome { biome: String },
}

/// Permission level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionLevel {
    /// Read-only access
    Read,
    /// Read-write access
    Write,
    /// Execute access
    Execute,
    /// Administrative access
    Admin,
    /// Full access
    Full,
    /// BearDog-secured access
    BeardogSecured,
}

/// User session with BearDog security context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    /// Session ID
    pub id: String,
    /// User ID
    pub user_id: u32,
    /// Username
    pub username: String,
    /// BearDog security context
    pub beardog_context: BeardogSecurityContext,
    /// Session start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Session last activity
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Session expiry time
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Session IP address
    pub ip_address: Option<String>,
    /// Session user agent
    pub user_agent: Option<String>,
    /// Session status
    pub status: SessionStatus,
    /// BearDog audit trail reference
    pub audit_trail_reference: String,
    /// Session metadata
    pub metadata: HashMap<String, String>,
}

/// BearDog security context for sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogSecurityContext {
    /// BearDog authentication token
    pub auth_token: String,
    /// Security level for this session
    pub security_level: String,
    /// Authorized operations
    pub authorized_operations: Vec<String>,
    /// Key access grants
    pub key_access_grants: Vec<String>,
    /// Secret access grants
    pub secret_access_grants: Vec<String>,
    /// Threat assessment score
    pub threat_assessment_score: f64,
    /// Compliance status
    pub compliance_status: String,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session is expired
    Expired,
    /// Session is revoked
    Revoked,
    /// Session is terminated
    Terminated,
    /// Session is blocked by BearDog
    BeardogBlocked,
    /// Session is under threat assessment
    ThreatAssessment,
}

/// User group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGroup {
    /// Group ID
    pub id: u32,
    /// Group name
    pub name: String,
    /// Group description
    pub description: Option<String>,
    /// Group members
    pub members: Vec<String>,
    /// Group permissions
    pub permissions: Vec<Permission>,
    /// BearDog group policy reference
    pub beardog_policy_reference: Option<String>,
    /// Group metadata
    pub metadata: HashMap<String, String>,
}

/// User authentication request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAuthRequest {
    /// Username
    pub username: String,
    /// Authentication method
    pub auth_method: UserAuthMethod,
    /// Client IP address
    pub client_ip: Option<String>,
    /// Client user agent
    pub client_user_agent: Option<String>,
    /// Request metadata
    pub metadata: HashMap<String, String>,
}

/// User authentication methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserAuthMethod {
    /// Password authentication (validated through BearDog)
    Password { password: String },
    /// SSH key authentication (validated through BearDog)
    SshKey {
        public_key: String,
        signature: String,
    },
    /// API key authentication (validated through BearDog)
    ApiKey { key: String },
    /// Genetic BearDog key authentication
    GeneticKey { key: GeneticBeardogKey },
    /// Biometric authentication (processed through BearDog)
    Biometric {
        biometric_data: String,
        biometric_type: String,
    },
}

/// BearDog key management operations
#[derive(Debug, Clone)]
pub struct BeardogKeyManagement {
    /// BearDog provider reference
    pub provider: Option<BeardogSecurityProvider>,
}

/// Key types for BearDog integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    /// Authentication key
    Authentication,
    /// Encryption key
    Encryption,
    /// Signing key
    Signing,
    /// SSH key pair
    SshKeyPair,
    /// API key
    ApiKey,
    /// Genetic BearDog key
    GeneticKey,
} 
// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Security Configuration
//!
//! This module contains security-related configuration types including
//! authentication, authorization, encryption, audit, session, and CSRF configuration.

mod authorization;
mod crypto;
mod tls;

pub use authorization::{
    AbacConfig, AbacRule, AuthorizationAction, AuthorizationConfig, AuthorizationPolicy, RbacConfig,
};
pub use crypto::{
    DataAtRestConfig, EncryptionAlgorithm, EncryptionConfig, KeyDerivationAlgorithm,
    KeyDerivationConfig, KeyManagementConfig, KeyStorageBackend,
};
pub use tls::{DataInTransitConfig, HstsConfig};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

/// Security configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Authentication configuration
    #[serde(default)]
    pub authentication: AuthenticationConfig,

    /// Authorization configuration
    #[serde(default)]
    pub authorization: AuthorizationConfig,

    /// Encryption configuration
    #[serde(default)]
    pub encryption: EncryptionConfig,

    /// Audit configuration
    #[serde(default)]
    pub audit: AuditConfig,

    /// Session configuration
    #[serde(default)]
    pub session: SessionConfig,

    /// CSRF protection
    #[serde(default)]
    pub csrf: CsrfConfig,

    /// Security headers
    #[serde(default)]
    pub headers: SecurityHeaders,
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthenticationConfig {
    /// Default authentication method
    pub default_method: AuthMethod,

    /// Available authentication methods
    pub methods: Vec<AuthMethod>,

    /// JWT configuration
    pub jwt: Option<JwtConfig>,

    /// OAuth configuration
    pub oauth: Option<OAuthConfig>,

    /// API key configuration
    pub api_key: Option<ApiKeyConfig>,

    /// Multi-factor authentication
    pub mfa: Option<MfaConfig>,
}

/// Authentication methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// API key authentication
    ApiKey,
    /// Bearer token authentication
    Bearer,
    /// HTTP Basic authentication
    Basic,
    /// JSON Web Token
    Jwt,
    /// OAuth 2.0
    OAuth2,
    /// SAML-based SSO
    Saml,
    /// LDAP directory authentication
    Ldap,
    /// Custom authentication handler
    Custom(String),
}

/// JWT configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// JWT secret key
    pub secret: String,

    /// Token expiration time
    pub expiration: Duration,

    /// Refresh token expiration
    pub refresh_expiration: Duration,

    /// JWT algorithm
    pub algorithm: JwtAlgorithm,

    /// JWT issuer
    pub issuer: Option<String>,

    /// JWT audience
    pub audience: Option<Vec<String>>,
}

/// JWT algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JwtAlgorithm {
    /// HMAC-SHA256
    HS256,
    /// HMAC-SHA384
    HS384,
    /// HMAC-SHA512
    HS512,
    /// RSA-SHA256
    RS256,
    /// RSA-SHA384
    RS384,
    /// RSA-SHA512
    RS512,
    /// ECDSA-SHA256
    ES256,
    /// ECDSA-SHA384
    ES384,
    /// ECDSA-SHA512
    ES512,
}

/// OAuth configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    /// OAuth providers
    pub providers: HashMap<String, OAuthProvider>,

    /// Default redirect URI
    pub redirect_uri: String,

    /// OAuth scopes
    pub scopes: Vec<String>,
}

/// OAuth provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    /// Client ID
    pub client_id: String,

    /// Client secret
    pub client_secret: String,

    /// Authorization URL
    pub auth_url: String,

    /// Token URL
    pub token_url: String,

    /// User info URL
    pub user_info_url: Option<String>,

    /// Additional scopes
    pub scopes: Vec<String>,
}

/// API key configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    /// API key header name
    pub header_name: String,

    /// API key query parameter name
    pub query_param: Option<String>,

    /// API key validation method
    pub validation: ApiKeyValidation,

    /// API key expiration
    pub expiration: Option<Duration>,
}

/// API key validation methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeyValidation {
    /// Validate against a database
    Database,
    /// Validate against a static list of keys
    StaticList(Vec<String>),
    /// Validate via an external service
    External(String),
}

/// Multi-factor authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MfaConfig {
    /// Enable MFA
    pub enabled: bool,

    /// MFA methods
    pub methods: Vec<MfaMethod>,

    /// TOTP configuration
    pub totp: Option<TotpConfig>,

    /// SMS configuration
    pub sms: Option<SmsConfig>,

    /// Email configuration
    pub email: Option<EmailMfaConfig>,
}

/// MFA methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MfaMethod {
    /// Time-based one-time password
    Totp,
    /// SMS verification code
    Sms,
    /// Email verification code
    Email,
    /// Hardware security key (e.g., `YubiKey`)
    Hardware,
    /// Backup recovery codes
    Backup,
}

/// TOTP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TotpConfig {
    /// TOTP issuer name
    pub issuer: String,

    /// TOTP secret length
    pub secret_length: usize,

    /// TOTP time step
    pub time_step: Duration,

    /// TOTP digits
    pub digits: u32,
}

/// SMS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsConfig {
    /// SMS provider
    pub provider: String,

    /// SMS API key
    pub api_key: String,

    /// SMS sender ID
    pub sender_id: String,
}

/// Email MFA configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMfaConfig {
    /// Email provider
    pub provider: String,

    /// Email API key
    pub api_key: String,

    /// From email address
    pub from_email: String,
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

    /// Audit log format
    pub format: AuditFormat,

    /// Log rotation configuration
    pub rotation: LogRotationConfig,
}

/// Audit destinations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditDestination {
    /// Write audit logs to a file
    File(PathBuf),
    /// Forward to syslog
    Syslog(SyslogConfig),
    /// Store in a database
    Database(DatabaseConfig),
    /// Send to an external service
    External(String),
}

/// Syslog configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyslogConfig {
    /// Syslog server
    pub server: String,

    /// Syslog port
    pub port: u16,

    /// Syslog facility
    pub facility: String,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,

    /// Database table
    pub table: String,

    /// Connection pool size
    pub pool_size: Option<usize>,
}

/// Audit events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// Authentication events (login, logout, failures)
    Authentication,
    /// Authorization events (access granted/denied)
    Authorization,
    /// Data access events (reads, writes)
    DataAccess,
    /// Configuration change events
    Configuration,
    /// Administrative actions
    Administrative,
    /// Security-related events
    Security,
    /// Error events
    Error,
    /// Audit all event types
    All,
}

/// Audit log formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditFormat {
    /// JSON format
    Json,
    /// Common Event Format
    Cef,
    /// Syslog format
    Syslog,
    /// Custom format template
    Custom(String),
}

/// Log rotation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotationConfig {
    /// Max file size before rotation
    pub max_size: usize,

    /// Max number of files to keep
    pub max_files: usize,

    /// Compress rotated files
    pub compress: bool,
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session timeout
    pub timeout: Duration,

    /// Session storage
    pub storage: SessionStorage,

    /// Session cookie configuration
    pub cookie: CookieConfig,

    /// Enable session fixation protection
    pub fixation_protection: bool,
}

/// Session storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStorage {
    /// In-memory storage (not persistent)
    Memory,
    /// File-based storage
    File(PathBuf),
    /// Database-backed storage
    Database(DatabaseConfig),
    /// Redis-backed storage
    Redis(RedisConfig),
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL
    pub url: String,

    /// Redis key prefix
    pub prefix: String,

    /// Connection pool size
    pub pool_size: Option<usize>,
}

/// Cookie configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieConfig {
    /// Cookie name
    pub name: String,

    /// Cookie domain
    pub domain: Option<String>,

    /// Cookie path
    pub path: String,

    /// Cookie secure flag
    pub secure: bool,

    /// Cookie HTTP-only flag
    pub http_only: bool,

    /// Cookie `SameSite` attribute
    pub same_site: SameSite,
}

/// `SameSite` cookie attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SameSite {
    /// Strict same-site policy
    Strict,
    /// Lax same-site policy (default)
    Lax,
    /// No same-site restriction
    None,
}

/// CSRF configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CsrfConfig {
    /// Enable CSRF protection
    pub enabled: bool,

    /// CSRF token header name
    pub header_name: String,

    /// CSRF token parameter name
    pub param_name: String,

    /// CSRF token cookie name
    pub cookie_name: String,

    /// CSRF token length
    pub token_length: usize,
}

/// Security headers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeaders {
    /// Content Security Policy
    pub csp: Option<String>,

    /// X-Frame-Options
    pub frame_options: Option<String>,

    /// X-Content-Type-Options
    pub content_type_options: bool,

    /// X-XSS-Protection
    pub xss_protection: bool,

    /// Strict-Transport-Security
    pub hsts: Option<HstsConfig>,

    /// Referrer-Policy
    pub referrer_policy: Option<String>,
}

impl Default for AuthenticationConfig {
    fn default() -> Self {
        Self {
            default_method: AuthMethod::None,
            methods: vec![AuthMethod::None],
            jwt: None,
            oauth: None,
            api_key: None,
            mfa: None,
        }
    }
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            destination: AuditDestination::File(PathBuf::from("./logs/audit.log")),
            events: vec![AuditEvent::All],
            format: AuditFormat::Json,
            rotation: LogRotationConfig::default(),
        }
    }
}

impl Default for LogRotationConfig {
    fn default() -> Self {
        Self {
            max_size: 100 * 1024 * 1024, // 100MB
            max_files: 10,
            compress: true,
        }
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30 * 60), // 30 minutes
            storage: SessionStorage::Memory,
            cookie: CookieConfig::default(),
            fixation_protection: true,
        }
    }
}

impl Default for CookieConfig {
    fn default() -> Self {
        Self {
            name: "session".to_string(),
            domain: None,
            path: "/".to_string(),
            secure: false,
            http_only: true,
            same_site: SameSite::Lax,
        }
    }
}

impl Default for CsrfConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            header_name: "X-CSRF-Token".to_string(),
            param_name: "_csrf".to_string(),
            cookie_name: "csrf".to_string(),
            token_length: 32,
        }
    }
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            csp: None,
            frame_options: Some("DENY".to_string()),
            content_type_options: true,
            xss_protection: true,
            hsts: None,
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

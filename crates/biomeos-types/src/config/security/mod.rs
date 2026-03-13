// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    /// Hardware security key (e.g., YubiKey)
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

    /// Cookie SameSite attribute
    pub same_site: SameSite,
}

/// SameSite cookie attribute
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
mod tests {
    use super::*;

    // ═══════════════════════════════════════════════════════════════════════
    // Default Implementations
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(!config.audit.enabled);
        assert!(!config.csrf.enabled);
        assert_eq!(config.authentication.default_method, AuthMethod::None);
    }

    #[test]
    fn test_authentication_config_default() {
        let config = AuthenticationConfig::default();
        assert_eq!(config.default_method, AuthMethod::None);
        assert_eq!(config.methods.len(), 1);
        assert!(config.jwt.is_none());
        assert!(config.oauth.is_none());
        assert!(config.api_key.is_none());
        assert!(config.mfa.is_none());
    }

    #[test]
    fn test_audit_config_default() {
        let config = AuditConfig::default();
        assert!(!config.enabled);
        assert!(matches!(config.format, AuditFormat::Json));
        assert!(!config.events.is_empty());
    }

    #[test]
    fn test_log_rotation_default() {
        let config = LogRotationConfig::default();
        assert_eq!(config.max_size, 100 * 1024 * 1024);
        assert_eq!(config.max_files, 10);
        assert!(config.compress);
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30 * 60));
        assert!(matches!(config.storage, SessionStorage::Memory));
        assert!(config.fixation_protection);
    }

    #[test]
    fn test_cookie_config_default() {
        let config = CookieConfig::default();
        assert_eq!(config.name, "session");
        assert!(config.domain.is_none());
        assert_eq!(config.path, "/");
        assert!(!config.secure);
        assert!(config.http_only);
        assert!(matches!(config.same_site, SameSite::Lax));
    }

    #[test]
    fn test_csrf_config_default() {
        let config = CsrfConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.header_name, "X-CSRF-Token");
        assert_eq!(config.param_name, "_csrf");
        assert_eq!(config.cookie_name, "csrf");
        assert_eq!(config.token_length, 32);
    }

    #[test]
    fn test_security_headers_default() {
        let config = SecurityHeaders::default();
        assert!(config.csp.is_none());
        assert_eq!(config.frame_options.as_deref(), Some("DENY"));
        assert!(config.content_type_options);
        assert!(config.xss_protection);
        assert!(config.hsts.is_none());
        assert!(config.referrer_policy.is_some());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Serialization Roundtrip
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_security_config_serialization() {
        let config = SecurityConfig::default();
        let json = serde_json::to_string(&config).expect("serialize");
        let deserialized: SecurityConfig = serde_json::from_str(&json).expect("deserialize");
        assert!(!deserialized.csrf.enabled);
    }

    #[test]
    fn test_auth_method_serialization() {
        for method in [
            AuthMethod::None,
            AuthMethod::ApiKey,
            AuthMethod::Bearer,
            AuthMethod::Basic,
            AuthMethod::Jwt,
            AuthMethod::OAuth2,
            AuthMethod::Saml,
            AuthMethod::Ldap,
            AuthMethod::Custom("kerberos".to_string()),
        ] {
            let json = serde_json::to_string(&method).expect("serialize");
            let _: AuthMethod = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_jwt_algorithm_serialization() {
        for alg in [
            JwtAlgorithm::HS256,
            JwtAlgorithm::HS384,
            JwtAlgorithm::HS512,
            JwtAlgorithm::RS256,
            JwtAlgorithm::RS384,
            JwtAlgorithm::RS512,
            JwtAlgorithm::ES256,
            JwtAlgorithm::ES384,
            JwtAlgorithm::ES512,
        ] {
            let json = serde_json::to_string(&alg).expect("serialize");
            let _: JwtAlgorithm = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_audit_event_serialization() {
        for event in [
            AuditEvent::Authentication,
            AuditEvent::Authorization,
            AuditEvent::DataAccess,
            AuditEvent::Configuration,
            AuditEvent::Administrative,
            AuditEvent::Security,
            AuditEvent::Error,
            AuditEvent::All,
        ] {
            let json = serde_json::to_string(&event).expect("serialize");
            let _: AuditEvent = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_audit_format_serialization() {
        for format in [
            AuditFormat::Json,
            AuditFormat::Cef,
            AuditFormat::Syslog,
            AuditFormat::Custom("%{timestamp} %{event}".to_string()),
        ] {
            let json = serde_json::to_string(&format).expect("serialize");
            let _: AuditFormat = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_session_storage_serialization() {
        for storage in [
            SessionStorage::Memory,
            SessionStorage::File(PathBuf::from("/tmp/sessions")),
        ] {
            let json = serde_json::to_string(&storage).expect("serialize");
            let _: SessionStorage = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_same_site_serialization() {
        for same_site in [SameSite::Strict, SameSite::Lax, SameSite::None] {
            let json = serde_json::to_string(&same_site).expect("serialize");
            let _: SameSite = serde_json::from_str(&json).expect("deserialize");
        }
    }

    #[test]
    fn test_mfa_method_serialization() {
        for method in [
            MfaMethod::Totp,
            MfaMethod::Sms,
            MfaMethod::Email,
            MfaMethod::Hardware,
            MfaMethod::Backup,
        ] {
            let json = serde_json::to_string(&method).expect("serialize");
            let _: MfaMethod = serde_json::from_str(&json).expect("deserialize");
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Complex Configuration Types
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_jwt_config_creation() {
        let config = JwtConfig {
            secret: "super-secret-key".to_string(),
            expiration: Duration::from_secs(3600),
            refresh_expiration: Duration::from_secs(86400),
            algorithm: JwtAlgorithm::HS256,
            issuer: Some("biomeOS".to_string()),
            audience: Some(vec!["api".to_string()]),
        };
        assert_eq!(config.secret, "super-secret-key");
        assert!(config.issuer.is_some());
    }

    #[test]
    fn test_oauth_config_with_providers() {
        let mut providers = HashMap::new();
        providers.insert(
            "github".to_string(),
            OAuthProvider {
                client_id: "client123".to_string(),
                client_secret: "secret456".to_string(),
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                user_info_url: Some("https://api.github.com/user".to_string()),
                scopes: vec!["read:user".to_string()],
            },
        );

        let config = OAuthConfig {
            providers,
            redirect_uri: "https://app.local/callback".to_string(),
            scopes: vec!["openid".to_string(), "profile".to_string()],
        };

        assert!(config.providers.contains_key("github"));
        assert_eq!(config.scopes.len(), 2);
    }

    #[test]
    fn test_api_key_config_variants() {
        // Database validation
        let db_config = ApiKeyConfig {
            header_name: "X-API-Key".to_string(),
            query_param: Some("api_key".to_string()),
            validation: ApiKeyValidation::Database,
            expiration: Some(Duration::from_secs(86400)),
        };
        assert!(matches!(db_config.validation, ApiKeyValidation::Database));

        // Static list validation
        let static_config = ApiKeyConfig {
            header_name: "Authorization".to_string(),
            query_param: None,
            validation: ApiKeyValidation::StaticList(vec!["key1".to_string(), "key2".to_string()]),
            expiration: None,
        };
        if let ApiKeyValidation::StaticList(keys) = &static_config.validation {
            assert_eq!(keys.len(), 2);
        }

        // External validation
        let ext_config = ApiKeyConfig {
            header_name: "X-API-Key".to_string(),
            query_param: None,
            validation: ApiKeyValidation::External("https://auth.local/validate".to_string()),
            expiration: None,
        };
        assert!(matches!(
            ext_config.validation,
            ApiKeyValidation::External(_)
        ));
    }

    #[test]
    fn test_totp_config_creation() {
        let config = TotpConfig {
            issuer: "BiomeOS".to_string(),
            secret_length: 32,
            time_step: Duration::from_secs(30),
            digits: 6,
        };
        assert_eq!(config.issuer, "BiomeOS");
        assert_eq!(config.digits, 6);
    }

    #[test]
    fn test_syslog_config_creation() {
        let config = SyslogConfig {
            server: "syslog.local".to_string(),
            port: 514,
            facility: "local0".to_string(),
        };
        assert_eq!(config.port, 514);
    }

    #[test]
    fn test_redis_config_creation() {
        let config = RedisConfig {
            url: "redis://localhost:6379".to_string(),
            prefix: "biomeos:session:".to_string(),
            pool_size: Some(10),
        };
        assert!(config.url.starts_with("redis://"));
    }

    #[test]
    fn test_mfa_config_creation() {
        let config = MfaConfig {
            enabled: true,
            methods: vec![MfaMethod::Totp, MfaMethod::Backup],
            totp: Some(TotpConfig {
                issuer: "Test".to_string(),
                secret_length: 32,
                time_step: Duration::from_secs(30),
                digits: 6,
            }),
            sms: None,
            email: None,
        };
        assert!(config.enabled);
        assert_eq!(config.methods.len(), 2);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Debug Formatting
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_auth_method_debug() {
        let method = AuthMethod::Custom("kerberos".to_string());
        let debug = format!("{:?}", method);
        assert!(debug.contains("kerberos"));
    }

    #[test]
    fn test_audit_destination_debug() {
        let dest = AuditDestination::File(PathBuf::from("/var/log/audit.log"));
        let debug = format!("{:?}", dest);
        assert!(debug.contains("audit.log"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Clone
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_security_config_clone() {
        let original = SecurityConfig::default();
        let cloned = original.clone();
        assert!(!cloned.csrf.enabled);
    }

    #[test]
    fn test_complex_config_clone() {
        let original = JwtConfig {
            secret: "secret".to_string(),
            expiration: Duration::from_secs(3600),
            refresh_expiration: Duration::from_secs(86400),
            algorithm: JwtAlgorithm::RS256,
            issuer: Some("test".to_string()),
            audience: Some(vec!["api".to_string()]),
        };
        let cloned = original.clone();
        assert_eq!(cloned.secret, "secret");
        assert!(matches!(cloned.algorithm, JwtAlgorithm::RS256));
    }
}

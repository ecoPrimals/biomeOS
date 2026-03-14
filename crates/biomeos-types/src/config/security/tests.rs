// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unit tests for security configuration types.

use super::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

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
    let debug = format!("{method:?}");
    assert!(debug.contains("kerberos"));
}

#[test]
fn test_audit_destination_debug() {
    let dest = AuditDestination::File(PathBuf::from("/var/log/audit.log"));
    let debug = format!("{dest:?}");
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

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project
//
// Test module for service/security.rs - included via #[path]

#![allow(clippy::unwrap_used)]

use super::*;
use std::collections::HashMap;

// --- Serde JSON roundtrips ---

#[test]
fn serde_roundtrip_service_security() {
    let val = ServiceSecurity::default();
    let json = serde_json::to_string(&val).unwrap();
    let restored: ServiceSecurity = serde_json::from_str(&json).unwrap();
    assert_eq!(
        val.security_context.run_as_user,
        restored.security_context.run_as_user
    );
    assert_eq!(
        val.security_context.run_as_group,
        restored.security_context.run_as_group
    );
    assert_eq!(
        val.security_context.run_as_non_root,
        restored.security_context.run_as_non_root
    );
    assert_eq!(val.encryption.algorithm, restored.encryption.algorithm);
}

#[test]
fn serde_roundtrip_security_context() {
    let val = SecurityContext {
        run_as_user: Some(1000),
        run_as_group: Some(1001),
        run_as_non_root: true,
        read_only_root_fs: false,
        allow_privilege_escalation: false,
        capabilities: SecurityCapabilities {
            add: vec!["NET_BIND_SERVICE".to_string()],
            drop: vec!["ALL".to_string()],
        },
        selinux: Some(SeLinuxOptions {
            user: Some("system_u".to_string()),
            role: Some("system_r".to_string()),
            selinux_type: Some("container_t".to_string()),
            level: Some("s0".to_string()),
        }),
        apparmor_profile: Some("custom-profile".to_string()),
        seccomp_profile: Some(SeccompProfile::Localhost("custom.json".to_string())),
    };
    let json = serde_json::to_string(&val).unwrap();
    let restored: SecurityContext = serde_json::from_str(&json).unwrap();
    assert_eq!(val.run_as_user, restored.run_as_user);
    assert_eq!(val.capabilities.add, restored.capabilities.add);
    assert_eq!(val.apparmor_profile, restored.apparmor_profile);
}

#[test]
fn serde_roundtrip_authentication_config() {
    let val = AuthenticationConfig {
        methods: vec![AuthMethod::Basic, AuthMethod::Jwt],
        default_method: Some("jwt".to_string()),
        mfa: Some(MfaConfig {
            enabled: true,
            required_methods: vec![MfaMethod::Totp],
            optional_methods: vec![MfaMethod::Email],
            timeout: 300,
        }),
        tokens: Some(TokenConfig {
            lifetime: 3600,
            refresh_enabled: true,
            algorithm: "HS256".to_string(),
            secret: "secret".to_string(),
        }),
        jwt: Some(JwtConfig {
            secret: "jwt-secret".to_string(),
            algorithm: "RS256".to_string(),
            audience: vec!["api".to_string()],
            issuer: "issuer".to_string(),
            lifetime: 3600,
        }),
        oauth2: Some(OAuth2Config {
            client_id: "client".to_string(),
            client_secret: "secret".to_string(),
            auth_url: "https://auth.example.com".to_string(),
            token_url: "https://auth.example.com/token".to_string(),
            scopes: vec!["openid".to_string()],
        }),
    };
    let json = serde_json::to_string(&val).unwrap();
    let restored: AuthenticationConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(val.methods.len(), restored.methods.len());
    assert_eq!(val.default_method, restored.default_method);
    assert_eq!(
        val.mfa.as_ref().unwrap().enabled,
        restored.mfa.as_ref().unwrap().enabled
    );
}

#[test]
fn serde_roundtrip_authorization_config() {
    let val = AuthorizationConfig {
        model: AuthzModel::Rbac,
        policies: vec![AuthzPolicy {
            name: "policy1".to_string(),
            rules: vec![AuthzRule {
                name: "rule1".to_string(),
                resources: vec!["/api/*".to_string()],
                actions: vec!["read".to_string()],
                conditions: HashMap::from([("env".to_string(), serde_json::json!("prod"))]),
            }],
            effect: AuthzEffect::Allow,
        }],
        default_action: AuthzAction::Deny,
    };
    let json = serde_json::to_string(&val).unwrap();
    let restored: AuthorizationConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(format!("{:?}", val.model), format!("{:?}", restored.model));
    assert_eq!(val.policies.len(), restored.policies.len());
    assert_eq!(
        format!("{:?}", val.default_action),
        format!("{:?}", restored.default_action)
    );
}

#[test]
fn serde_roundtrip_encryption_config() {
    let val = EncryptionConfig {
        at_rest: true,
        in_transit: true,
        algorithm: "AES-256-GCM".to_string(),
        key_management: KeyManagement {
            provider: KeyProvider::Vault {
                address: "https://vault.example.com".to_string(),
                path: "secret/data/keys".to_string(),
            },
            rotation: KeyRotation {
                enabled: true,
                interval: 86400,
                automatic: true,
            },
        },
    };
    let json = serde_json::to_string(&val).unwrap();
    let restored: EncryptionConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(val.at_rest, restored.at_rest);
    assert_eq!(val.algorithm, restored.algorithm);
}

#[test]
fn serde_roundtrip_secrets_config() {
    let val = SecretsConfig {
        provider: SecretsProvider::Vault {
            address: "https://vault.example.com".to_string(),
            token: "s.xxx".to_string(),
        },
        secrets: vec![SecretReference {
            name: "db-credentials".to_string(),
            key: "password".to_string(),
            env_var: Some("DB_PASSWORD".to_string()),
            file_path: None,
            optional: false,
        }],
    };
    let json = serde_json::to_string(&val).unwrap();
    let restored: SecretsConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(val.secrets.len(), restored.secrets.len());
    assert_eq!(val.secrets[0].name, restored.secrets[0].name);
}

#[test]
fn serde_roundtrip_network_policy() {
    let val = NetworkPolicy {
        name: "allow-api".to_string(),
        policy_type: NetworkPolicyType::Both,
        pod_selector: HashMap::from([("app".to_string(), "api".to_string())]),
        ingress: vec![NetworkPolicyRule {
            peers: vec![NetworkPolicyPeer {
                pod_selector: Some(HashMap::from([("role".to_string(), "client".to_string())])),
                namespace_selector: None,
                ip_block: None,
            }],
            ports: vec![NetworkPolicyPort {
                port: NetworkPolicyPortValue::Number(443),
                protocol: NetworkPolicyProtocol::TCP,
                end_port: None,
            }],
        }],
        egress: vec![],
    };
    let json = serde_json::to_string(&val).unwrap();
    let restored: NetworkPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(val.name, restored.name);
    assert_eq!(
        format!("{:?}", val.policy_type),
        format!("{:?}", restored.policy_type)
    );
    assert_eq!(val.ingress.len(), restored.ingress.len());
}

// --- Default implementations ---

#[test]
fn default_service_security() {
    let sec = ServiceSecurity::default();
    assert_eq!(sec.security_context.run_as_user, Some(1000));
    assert_eq!(sec.security_context.run_as_group, Some(1000));
    assert!(sec.security_context.run_as_non_root);
    assert!(sec.security_context.read_only_root_fs);
    assert!(!sec.security_context.allow_privilege_escalation);
    assert_eq!(sec.security_context.capabilities.drop, vec!["ALL"]);
    assert!(sec.authentication.is_none());
    assert!(sec.authorization.is_none());
    assert_eq!(sec.encryption.algorithm, "AES-256-GCM");
    assert!(sec.encryption.in_transit);
    assert!(!sec.encryption.at_rest);
    assert!(sec.secrets.secrets.is_empty());
}

// --- Enum variant coverage ---

#[test]
fn enum_auth_method_variants() {
    for (json, expected) in [
        (r#""Basic""#, AuthMethod::Basic),
        (r#""ApiKey""#, AuthMethod::ApiKey),
        (r#""Jwt""#, AuthMethod::Jwt),
        (r#""OAuth2""#, AuthMethod::OAuth2),
        (r#""Ldap""#, AuthMethod::Ldap),
        (r#""Saml""#, AuthMethod::Saml),
        (r#""Certificate""#, AuthMethod::Certificate),
        (
            r#"{"Custom":"custom-auth"}"#,
            AuthMethod::Custom("custom-auth".to_string()),
        ),
    ] {
        let parsed: AuthMethod = serde_json::from_str(json).unwrap();
        assert_eq!(format!("{parsed:?}"), format!("{:?}", expected));
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let reparsed: AuthMethod = serde_json::from_str(&roundtrip).unwrap();
        assert_eq!(format!("{reparsed:?}"), format!("{:?}", expected));
    }
}

#[test]
fn enum_mfa_method_variants() {
    for json in [
        r#""Totp""#,
        r#""Sms""#,
        r#""Email""#,
        r#""Hardware""#,
        r#""Biometric""#,
        r#"{"Custom":"custom-mfa"}"#,
    ] {
        let parsed: MfaMethod = serde_json::from_str(json).unwrap();
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let _reparsed: MfaMethod = serde_json::from_str(&roundtrip).unwrap();
    }
}

#[test]
fn enum_authz_model_variants() {
    for json in [
        r#""Rbac""#,
        r#""Abac""#,
        r#""ReBAC""#,
        r#"{"Custom":"custom-model"}"#,
    ] {
        let parsed: AuthzModel = serde_json::from_str(json).unwrap();
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let _reparsed: AuthzModel = serde_json::from_str(&roundtrip).unwrap();
    }
}

#[test]
fn enum_authz_effect_variants() {
    for json in [r#""Allow""#, r#""Deny""#] {
        let parsed: AuthzEffect = serde_json::from_str(json).unwrap();
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let _reparsed: AuthzEffect = serde_json::from_str(&roundtrip).unwrap();
    }
}

#[test]
fn enum_authz_action_variants() {
    for json in [r#""Allow""#, r#""Deny""#, r#""Audit""#] {
        let parsed: AuthzAction = serde_json::from_str(json).unwrap();
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let _reparsed: AuthzAction = serde_json::from_str(&roundtrip).unwrap();
    }
}

#[test]
fn enum_seccomp_profile_variants() {
    let runtime: SeccompProfile = serde_json::from_str(r#""RuntimeDefault""#).unwrap();
    assert!(matches!(runtime, SeccompProfile::RuntimeDefault));

    let unconfined: SeccompProfile = serde_json::from_str(r#""Unconfined""#).unwrap();
    assert!(matches!(unconfined, SeccompProfile::Unconfined));

    let localhost: SeccompProfile =
        serde_json::from_str(r#"{"Localhost":"profile.json"}"#).unwrap();
    if let SeccompProfile::Localhost(s) = localhost {
        assert_eq!(s, "profile.json");
    } else {
        panic!("Expected Localhost variant");
    }
}

#[test]
fn enum_key_provider_variants() {
    let generated: KeyProvider = serde_json::from_str(r#""Generated""#).unwrap();
    assert!(matches!(generated, KeyProvider::Generated));

    let ext_json = r#"{"External":{"provider":"aws","config":{"region":"us-east-1"}}}"#;
    let ext: KeyProvider = serde_json::from_str(ext_json).unwrap();
    if let KeyProvider::External { provider, config } = ext {
        assert_eq!(provider, "aws");
        assert_eq!(config.get("region").map(String::as_str), Some("us-east-1"));
    } else {
        panic!("Expected External variant");
    }

    let vault_json = r#"{"Vault":{"address":"https://vault","path":"/secret"}}"#;
    let vault: KeyProvider = serde_json::from_str(vault_json).unwrap();
    if let KeyProvider::Vault { address, path } = vault {
        assert_eq!(address, "https://vault");
        assert_eq!(path, "/secret");
    } else {
        panic!("Expected Vault variant");
    }
}

#[test]
fn enum_secrets_provider_variants() {
    let k8s: SecretsProvider = serde_json::from_str(r#""Kubernetes""#).unwrap();
    assert!(matches!(k8s, SecretsProvider::Kubernetes));

    let vault_json = r#"{"Vault":{"address":"https://vault","token":"s.xxx"}}"#;
    let vault: SecretsProvider = serde_json::from_str(vault_json).unwrap();
    if let SecretsProvider::Vault { address, token } = vault {
        assert_eq!(address, "https://vault");
        assert_eq!(token, "s.xxx");
    } else {
        panic!("Expected Vault variant");
    }

    let aws_json = r#"{"AwsSecretsManager":{"region":"us-east-1"}}"#;
    let aws: SecretsProvider = serde_json::from_str(aws_json).unwrap();
    if let SecretsProvider::AwsSecretsManager { region } = aws {
        assert_eq!(region, "us-east-1");
    } else {
        panic!("Expected AwsSecretsManager variant");
    }
}

#[test]
fn enum_network_policy_type_variants() {
    for (json, _) in [
        (r#""Ingress""#, NetworkPolicyType::Ingress),
        (r#""Egress""#, NetworkPolicyType::Egress),
        (r#""Both""#, NetworkPolicyType::Both),
    ] {
        let parsed: NetworkPolicyType = serde_json::from_str(json).unwrap();
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let _reparsed: NetworkPolicyType = serde_json::from_str(&roundtrip).unwrap();
    }
}

#[test]
fn enum_network_policy_port_value_variants() {
    let num: NetworkPolicyPortValue = serde_json::from_str(r#"{"Number":443}"#).unwrap();
    if let NetworkPolicyPortValue::Number(n) = num {
        assert_eq!(n, 443);
    } else {
        panic!("Expected Number variant");
    }

    let name: NetworkPolicyPortValue = serde_json::from_str(r#"{"Name":"https"}"#).unwrap();
    if let NetworkPolicyPortValue::Name(s) = name {
        assert_eq!(s, "https");
    } else {
        panic!("Expected Name variant");
    }
}

#[test]
fn enum_network_policy_protocol_variants() {
    for json in [r#""TCP""#, r#""UDP""#, r#""SCTP""#] {
        let parsed: NetworkPolicyProtocol = serde_json::from_str(json).unwrap();
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let _reparsed: NetworkPolicyProtocol = serde_json::from_str(&roundtrip).unwrap();
    }
}

#[test]
fn enum_network_policy_action_variants() {
    for json in [r#""Allow""#, r#""Deny""#, r#""Log""#] {
        let parsed: NetworkPolicyAction = serde_json::from_str(json).unwrap();
        let roundtrip = serde_json::to_string(&parsed).unwrap();
        let _reparsed: NetworkPolicyAction = serde_json::from_str(&roundtrip).unwrap();
    }
}

// --- SELinux type rename (serde) ---

#[test]
fn selinux_type_serde_rename() {
    let json = r#"{"user":"u","role":"r","type":"container_t","level":"s0"}"#;
    let opts: SeLinuxOptions = serde_json::from_str(json).unwrap();
    assert_eq!(opts.selinux_type, Some("container_t".to_string()));
    let serialized = serde_json::to_string(&opts).unwrap();
    assert!(serialized.contains("type"));
}

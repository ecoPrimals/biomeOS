//! User Management Tests for biomeOS
//!
//! Focused test suite for user management with BearDog integration

use biomeos_system::users::*;
use serde_json;
use std::collections::HashMap;
use std::path::PathBuf;

fn create_test_config() -> UserConfig {
    UserConfig {
        user_db_path: PathBuf::from("/tmp/test_users.db"),
        home_dir_base: PathBuf::from("/tmp/home"),
        default_shell: PathBuf::from("/bin/bash"),
        default_group: "users".to_string(),
        beardog_config: BeardogIntegrationConfig {
            endpoint: "https://beardog.test.local".to_string(),
            auth_method: BeardogAuthMethod::ApiKey {
                key_reference: "test_key".to_string(),
            },
            key_management_enabled: true,
            secret_storage_enabled: true,
            audit_logging_enabled: true,
            hsm_integration_enabled: false,
            compliance_mode: "standard".to_string(),
            security_level: "internal".to_string(),
        },
        session_timeout_seconds: 3600,
        enable_guest: false,
        enable_sudo: true,
        enable_genetic_keys: true,
    }
}

#[test]
fn test_user_config_creation() {
    let config = create_test_config();

    assert_eq!(config.default_group, "users");
    assert!(config.enable_genetic_keys);
    assert!(config.enable_sudo);
    assert!(!config.enable_guest);
    assert_eq!(config.session_timeout_seconds, 3600);
}

#[test]
fn test_beardog_integration_config() {
    let config = create_test_config();
    let beardog_config = &config.beardog_config;

    assert_eq!(beardog_config.endpoint, "https://beardog.test.local");
    assert!(beardog_config.key_management_enabled);
    assert!(beardog_config.secret_storage_enabled);
    assert!(beardog_config.audit_logging_enabled);
    assert!(!beardog_config.hsm_integration_enabled);
    assert_eq!(beardog_config.compliance_mode, "standard");
    assert_eq!(beardog_config.security_level, "internal");
}

#[test]
fn test_beardog_auth_method() {
    let config = create_test_config();

    match &config.beardog_config.auth_method {
        BeardogAuthMethod::ApiKey { key_reference } => {
            assert_eq!(key_reference, "test_key");
        }
        _ => panic!("Expected ApiKey auth method"),
    }
}

#[test]
fn test_user_manager_creation() {
    let config = create_test_config();
    let user_manager = UserManager::new(config);

    // Verify the user manager was created with the correct configuration
    assert_eq!(user_manager.config.default_group, "users");
    assert!(user_manager.config.enable_genetic_keys);
    assert!(user_manager.config.enable_sudo);
    assert!(!user_manager.config.enable_guest);
    assert!(user_manager.beardog_provider.is_some());
}

#[test]
fn test_beardog_security_provider_creation() {
    let provider = BeardogSecurityProvider {
        auth_endpoint: "https://beardog.test.local/auth".to_string(),
        key_management_endpoint: "https://beardog.test.local/keys".to_string(),
        secret_storage_endpoint: "https://beardog.test.local/secrets".to_string(),
        service_token: "test_service_token".to_string(),
        genetic_keys_enabled: true,
        hsm_integration_enabled: false,
    };

    assert_eq!(provider.auth_endpoint, "https://beardog.test.local/auth");
    assert_eq!(
        provider.key_management_endpoint,
        "https://beardog.test.local/keys"
    );
    assert_eq!(
        provider.secret_storage_endpoint,
        "https://beardog.test.local/secrets"
    );
    assert!(provider.genetic_keys_enabled);
    assert!(!provider.hsm_integration_enabled);
}

#[test]
fn test_beardog_key_management() {
    let key_mgmt = BeardogKeyManagement { provider: None };

    // For now, just test that the struct exists and can be instantiated
    assert!(key_mgmt.provider.is_none());
}

#[test]
fn test_user_status_variants() {
    // Test user status enum variants
    let active = UserStatus::Active;
    let disabled = UserStatus::Disabled;
    let locked = UserStatus::Locked;

    // Test serialization
    let serialized = serde_json::to_string(&active).unwrap();
    let deserialized: UserStatus = serde_json::from_str(&serialized).unwrap();

    // Different variants should have different discriminants
    assert_ne!(UserStatus::Active as u8, UserStatus::Disabled as u8);
    assert_ne!(UserStatus::Active as u8, UserStatus::Locked as u8);

    // Test status conversions
    assert_eq!(format!("{:?}", active), "Active");
    assert_eq!(format!("{:?}", disabled), "Disabled");
    assert_eq!(format!("{:?}", locked), "Locked");
}

#[test]
fn test_session_status_variants() {
    assert_eq!(SessionStatus::Active as u8, SessionStatus::Active as u8);
    assert_ne!(SessionStatus::Active as u8, SessionStatus::Expired as u8);
    assert_ne!(
        SessionStatus::Active as u8,
        SessionStatus::BeardogBlocked as u8
    );
    assert_ne!(
        SessionStatus::Active as u8,
        SessionStatus::ThreatAssessment as u8
    );
}

#[test]
fn test_permission_levels() {
    assert_eq!(PermissionLevel::Read as u8, PermissionLevel::Read as u8);
    assert_ne!(PermissionLevel::Read as u8, PermissionLevel::Write as u8);
    assert_ne!(PermissionLevel::Read as u8, PermissionLevel::Execute as u8);
    assert_ne!(PermissionLevel::Read as u8, PermissionLevel::Admin as u8);
}

#[test]
fn test_key_types() {
    assert_eq!(KeyType::Encryption as u8, KeyType::Encryption as u8);
    assert_ne!(KeyType::Encryption as u8, KeyType::Authentication as u8);
    assert_ne!(KeyType::Encryption as u8, KeyType::Signing as u8);
    assert_ne!(KeyType::Encryption as u8, KeyType::GeneticKey as u8);
}

#[test]
fn test_permission_scope_system() {
    let scope = PermissionScope::System;

    match scope {
        PermissionScope::System => assert!(true),
        _ => panic!("Expected System scope"),
    }
}

#[test]
fn test_permission_scope_resource() {
    let scope = PermissionScope::Resource {
        resource: "test_resource".to_string(),
    };

    match scope {
        PermissionScope::Resource { resource } => {
            assert_eq!(resource, "test_resource");
        }
        _ => panic!("Expected Resource scope"),
    }
}

#[test]
fn test_user_config_default() {
    let config = UserConfig::default();

    assert!(config.user_db_path.ends_with("users.db"));
    assert!(config.home_dir_base.ends_with("home"));
    assert_eq!(config.default_shell, PathBuf::from("/bin/bash"));
    assert_eq!(config.default_group, "users");
    assert_eq!(config.session_timeout_seconds, 3600);
    assert!(!config.enable_guest);
    assert!(config.enable_sudo);
    assert!(config.enable_genetic_keys);
}

#[test]
fn test_user_auth_method_variants() {
    // Test password authentication
    let password_auth = UserAuthMethod::Password {
        password: "secure_password".to_string(),
    };

    // Test pattern matching
    match password_auth {
        UserAuthMethod::Password { password } => {
            assert_eq!(password, "secure_password");
        }
        _ => panic!("Expected Password variant"),
    }

    // Test SSH key authentication
    let ssh_auth = UserAuthMethod::SshKey {
        public_key: "ssh-rsa AAAAB3NzaC1yc2E...".to_string(),
        signature: "sig_123".to_string(),
    };

    match ssh_auth {
        UserAuthMethod::SshKey {
            public_key,
            signature,
        } => {
            assert_eq!(public_key, "ssh-rsa AAAAB3NzaC1yc2E...");
            assert_eq!(signature, "sig_123");
        }
        _ => panic!("Expected SshKey variant"),
    }
}

#[test]
fn test_beardog_auth_method_variants() {
    // Test API key authentication
    let api_key_auth = BeardogAuthMethod::ApiKey {
        key_reference: "api_key_123".to_string(),
    };

    match api_key_auth {
        BeardogAuthMethod::ApiKey { key_reference } => {
            assert_eq!(key_reference, "api_key_123");
        }
        _ => panic!("Expected ApiKey variant"),
    }

    // Test service account authentication
    let service_account_auth = BeardogAuthMethod::ServiceAccount {
        account_id: "service_account_123".to_string(),
        private_key_reference: "private_key_123".to_string(),
    };

    match service_account_auth {
        BeardogAuthMethod::ServiceAccount {
            account_id,
            private_key_reference,
        } => {
            assert_eq!(account_id, "service_account_123");
            assert_eq!(private_key_reference, "private_key_123");
        }
        _ => panic!("Expected ServiceAccount variant"),
    }
}

#[test]
fn test_concurrent_user_manager_creation() {
    use std::thread;

    let handles: Vec<_> = (0..10)
        .map(|_| {
            thread::spawn(|| {
                let config = create_test_config();
                let user_manager = UserManager::new(config);
                assert_eq!(user_manager.config.default_group, "users");
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_key_type_variants() {
    // Test key type enum variants
    let encryption = KeyType::Encryption;
    let signing = KeyType::Signing;
    let authentication = KeyType::Authentication;

    // Test serialization
    let serialized = serde_json::to_string(&encryption).unwrap();
    let deserialized: KeyType = serde_json::from_str(&serialized).unwrap();

    // Different key types should have different discriminants
    assert_ne!(KeyType::Encryption as u8, KeyType::Signing as u8);
    assert_ne!(KeyType::Encryption as u8, KeyType::Authentication as u8);

    // Test key type conversions
    assert_eq!(format!("{:?}", encryption), "Encryption");
    assert_eq!(format!("{:?}", signing), "Signing");
    assert_eq!(format!("{:?}", authentication), "Authentication");
}

//! Simple User Management Tests for biomeOS

use biomeos_system::users::*;
use std::path::PathBuf;

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
fn test_user_manager_creation() {
    let config = UserConfig::default();
    let user_manager = UserManager::new(config);

    // Verify the user manager was created with the correct configuration
    assert_eq!(user_manager.config.default_group, "users");
    assert!(user_manager.config.enable_genetic_keys);
    assert!(user_manager.config.enable_sudo);
    assert!(!user_manager.config.enable_guest);
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
fn test_user_status_variants() {
    // Test that enum variants are distinct
    assert_eq!(UserStatus::Active as u8, UserStatus::Active as u8);
    assert_ne!(UserStatus::Active as u8, UserStatus::Disabled as u8);
    assert_ne!(UserStatus::Active as u8, UserStatus::Locked as u8);
    assert_ne!(UserStatus::Active as u8, UserStatus::Expired as u8);
}

#[test]
fn test_session_status_variants() {
    // Test that enum variants are distinct
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
    // Test that enum variants are distinct
    assert_eq!(PermissionLevel::Read as u8, PermissionLevel::Read as u8);
    assert_ne!(PermissionLevel::Read as u8, PermissionLevel::Write as u8);
    assert_ne!(PermissionLevel::Read as u8, PermissionLevel::Execute as u8);
    assert_ne!(PermissionLevel::Read as u8, PermissionLevel::Admin as u8);
}

#[test]
fn test_key_types() {
    // Test that enum variants are distinct
    assert_eq!(KeyType::Encryption as u8, KeyType::Encryption as u8);
    assert_ne!(KeyType::Encryption as u8, KeyType::Authentication as u8);
    assert_ne!(KeyType::Encryption as u8, KeyType::Signing as u8);
    assert_ne!(KeyType::Encryption as u8, KeyType::GeneticKey as u8);
}

//! Tests for user management functionality
//!
//! This module contains comprehensive tests for the user management system,
//! including BearDog integration, authentication, and configuration.

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::super::config::*;
    use std::collections::HashMap;
    use std::path::PathBuf;

    #[test]
    fn test_user_config_default() {
        let config = UserConfig::default();

        assert!(config.user_db_path.ends_with("users.db"));
        assert!(config.home_dir_base.ends_with("home"));
        assert_eq!(config.default_shell, PathBuf::from("/bin/bash"));
        assert_eq!(config.default_group, "users");
        assert_eq!(config.session_timeout_seconds, 3600);
        assert!(!config.enable_guest); // Default is false
        assert!(config.enable_sudo);
        assert!(config.enable_genetic_keys); // Default is true
    }

    #[test]
    fn test_user_manager_creation() {
        let config = UserConfig::default();
        let user_manager = UserManager::new(config);

        // Verify the user manager was created with the correct configuration
        assert_eq!(user_manager.config.default_group, "users");
        assert_eq!(user_manager.config.session_timeout_seconds, 3600);
        assert_eq!(
            user_manager.config.default_shell,
            PathBuf::from("/bin/bash")
        );

        // Verify BearDog provider is created when key management is enabled
        assert!(user_manager.beardog_provider.is_some());
    }

    #[test]
    fn test_beardog_integration_config() {
        let beardog_config = BeardogIntegrationConfig {
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
        };

        assert_eq!(beardog_config.endpoint, "https://beardog.test.local");
        assert!(beardog_config.key_management_enabled);
        assert!(beardog_config.secret_storage_enabled);
        assert!(beardog_config.audit_logging_enabled);
        assert!(!beardog_config.hsm_integration_enabled);
        assert_eq!(beardog_config.compliance_mode, "standard");
        assert_eq!(beardog_config.security_level, "internal");
    }

    #[test]
    fn test_user_auth_method_variants() {
        let password_auth = UserAuthMethod::Password {
            password: "secret".to_string(),
        };
        let ssh_key_auth = UserAuthMethod::SshKey {
            public_key: "ssh-rsa AAAAB3...".to_string(),
            signature: "signature".to_string(),
        };
        let api_key_auth = UserAuthMethod::ApiKey {
            key: "api_key_123".to_string(),
        };

        match password_auth {
            UserAuthMethod::Password { password } => assert_eq!(password, "secret"),
            _ => panic!("Expected password auth"),
        }

        match ssh_key_auth {
            UserAuthMethod::SshKey { public_key, .. } => assert!(public_key.starts_with("ssh-rsa")),
            _ => panic!("Expected SSH key auth"),
        }

        match api_key_auth {
            UserAuthMethod::ApiKey { key } => assert_eq!(key, "api_key_123"),
            _ => panic!("Expected API key auth"),
        }
    }

    #[test]
    fn test_user_session_status_transitions() {
        let active_status = SessionStatus::Active;
        let expired_status = SessionStatus::Expired;
        let revoked_status = SessionStatus::Revoked;

        assert_eq!(format!("{:?}", active_status), "Active");
        assert_eq!(format!("{:?}", expired_status), "Expired");
        assert_eq!(format!("{:?}", revoked_status), "Revoked");
    }

    #[test]
    fn test_beardog_auth_method_variants() {
        let mutual_tls = BeardogAuthMethod::MutualTLS {
            cert_path: PathBuf::from("/path/to/cert.pem"),
            key_path: PathBuf::from("/path/to/key.pem"),
            ca_path: PathBuf::from("/path/to/ca.pem"),
        };

        let api_key = BeardogAuthMethod::ApiKey {
            key_reference: "beardog_key_123".to_string(),
        };

        match mutual_tls {
            BeardogAuthMethod::MutualTLS {
                cert_path,
                key_path,
                ca_path,
            } => {
                assert!(cert_path.ends_with("cert.pem"));
                assert!(key_path.ends_with("key.pem"));
                assert!(ca_path.ends_with("ca.pem"));
            }
            _ => panic!("Expected mutual TLS auth"),
        }

        match api_key {
            BeardogAuthMethod::ApiKey { key_reference } => {
                assert_eq!(key_reference, "beardog_key_123");
            }
            _ => panic!("Expected API key auth"),
        }
    }

    #[test]
    fn test_beardog_key_management_creation() {
        let key_mgmt = BeardogKeyManagement { provider: None };

        assert!(key_mgmt.provider.is_none());
    }

    #[test]
    fn test_user_auth_request_creation() {
        let auth_request = UserAuthRequest {
            username: "testuser".to_string(),
            auth_method: UserAuthMethod::Password {
                password: "secret".to_string(),
            },
            client_ip: Some("127.0.0.1".to_string()),
            client_user_agent: Some("test-agent".to_string()),
            metadata: HashMap::new(),
        };

        assert_eq!(auth_request.username, "testuser");
        assert_eq!(auth_request.client_ip.unwrap(), "127.0.0.1");
        assert_eq!(auth_request.client_user_agent.unwrap(), "test-agent");
        assert_eq!(auth_request.metadata.len(), 0);
    }

    #[test]
    fn test_user_status_variants() {
        let active = UserStatus::Active;
        let disabled = UserStatus::Disabled;
        let locked = UserStatus::Locked;

        assert_eq!(format!("{:?}", active), "Active");
        assert_eq!(format!("{:?}", disabled), "Disabled");
        assert_eq!(format!("{:?}", locked), "Locked");
    }

    #[test]
    fn test_key_type_variants() {
        let auth_key = KeyType::Authentication;
        let enc_key = KeyType::Encryption;
        let sign_key = KeyType::Signing;

        assert_eq!(format!("{:?}", auth_key), "Authentication");
        assert_eq!(format!("{:?}", enc_key), "Encryption");
        assert_eq!(format!("{:?}", sign_key), "Signing");
    }

    #[tokio::test]
    async fn test_user_manager_async_operations() {
        let config = UserConfig::default();
        let user_manager = UserManager::new(config);

        // Test user existence check
        let exists = user_manager.user_exists("nonexistent").await.unwrap();
        assert!(!exists);

        // Test ID generation
        let user_id = user_manager.generate_user_id().await;
        assert_eq!(user_id, 1000); // Should start from 1000

        let group_id = user_manager.generate_group_id().await;
        assert_eq!(group_id, 100); // Should start from 100
    }

    #[tokio::test]
    async fn test_user_manager_get_operations() {
        let config = UserConfig::default();
        let user_manager = UserManager::new(config);

        // Test getting non-existent user
        let user = user_manager.get_user("nonexistent").await;
        assert!(user.is_none());

        // Test getting all users (should be empty initially)
        let all_users = user_manager.get_all_users().await;
        assert!(all_users.is_empty());

        // Test getting non-existent session
        let session = user_manager.get_session("nonexistent").await;
        assert!(session.is_none());
    }

    #[test]
    fn test_permission_creation() {
        let permission = Permission {
            name: "read_files".to_string(),
            description: "Read access to files".to_string(),
            scope: PermissionScope::System,
            level: PermissionLevel::Read,
            beardog_auth_reference: Some("beardog_auth_123".to_string()),
        };

        assert_eq!(permission.name, "read_files");
        assert_eq!(permission.description, "Read access to files");
        assert!(permission.beardog_auth_reference.is_some());

        match permission.scope {
            PermissionScope::System => {} // Expected
            _ => panic!("Expected system scope"),
        }

        match permission.level {
            PermissionLevel::Read => {} // Expected
            _ => panic!("Expected read level"),
        }
    }

    #[test]
    fn test_user_group_creation() {
        let group = UserGroup {
            id: 100,
            name: "developers".to_string(),
            description: Some("Developer group".to_string()),
            members: vec!["alice".to_string(), "bob".to_string()],
            permissions: vec![],
            beardog_policy_reference: Some("dev_policy".to_string()),
            metadata: HashMap::new(),
        };

        assert_eq!(group.id, 100);
        assert_eq!(group.name, "developers");
        assert_eq!(group.description.unwrap(), "Developer group");
        assert_eq!(group.members.len(), 2);
        assert!(group.members.contains(&"alice".to_string()));
        assert!(group.members.contains(&"bob".to_string()));
        assert_eq!(group.beardog_policy_reference.unwrap(), "dev_policy");
    }

    #[test]
    fn test_beardog_security_context() {
        let context = BeardogSecurityContext {
            auth_token: "token123".to_string(),
            security_level: "confidential".to_string(),
            authorized_operations: vec!["read".to_string(), "write".to_string()],
            key_access_grants: vec!["enc_key_1".to_string()],
            secret_access_grants: vec!["secret_1".to_string()],
            threat_assessment_score: 0.85,
            compliance_status: "compliant".to_string(),
        };

        assert_eq!(context.auth_token, "token123");
        assert_eq!(context.security_level, "confidential");
        assert_eq!(context.authorized_operations.len(), 2);
        assert_eq!(context.key_access_grants.len(), 1);
        assert_eq!(context.secret_access_grants.len(), 1);
        assert_eq!(context.threat_assessment_score, 0.85);
        assert_eq!(context.compliance_status, "compliant");
    }

    #[test]
    fn test_user_session_creation() {
        let session = UserSession {
            id: "session123".to_string(),
            user_id: 1001,
            username: "testuser".to_string(),
            beardog_context: BeardogSecurityContext {
                auth_token: "token123".to_string(),
                security_level: "internal".to_string(),
                authorized_operations: vec!["read".to_string()],
                key_access_grants: vec![],
                secret_access_grants: vec![],
                threat_assessment_score: 0.9,
                compliance_status: "compliant".to_string(),
            },
            start_time: chrono::Utc::now(),
            last_activity: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::hours(1),
            ip_address: Some("127.0.0.1".to_string()),
            user_agent: Some("test-agent".to_string()),
            status: SessionStatus::Active,
            audit_trail_reference: "audit123".to_string(),
            metadata: HashMap::new(),
        };

        assert_eq!(session.id, "session123");
        assert_eq!(session.user_id, 1001);
        assert_eq!(session.username, "testuser");
        assert_eq!(session.beardog_context.security_level, "internal");
        assert_eq!(session.ip_address.unwrap(), "127.0.0.1");
        assert_eq!(session.user_agent.unwrap(), "test-agent");
        assert_eq!(session.audit_trail_reference, "audit123");
        assert!(matches!(session.status, SessionStatus::Active));
    }

    #[test]
    fn test_permission_scope_variants() {
        let system_scope = PermissionScope::System;
        let user_scope = PermissionScope::User;
        let resource_scope = PermissionScope::Resource {
            resource: "database".to_string(),
        };

        assert_eq!(format!("{:?}", system_scope), "System");
        assert_eq!(format!("{:?}", user_scope), "User");

        match resource_scope {
            PermissionScope::Resource { resource } => assert_eq!(resource, "database"),
            _ => panic!("Expected resource scope"),
        }
    }

    #[test]
    fn test_permission_level_variants() {
        let read_level = PermissionLevel::Read;
        let write_level = PermissionLevel::Write;
        let admin_level = PermissionLevel::Admin;

        assert_eq!(format!("{:?}", read_level), "Read");
        assert_eq!(format!("{:?}", write_level), "Write");
        assert_eq!(format!("{:?}", admin_level), "Admin");
    }

    #[test]
    fn test_user_config_development() {
        let config = UserConfig::development();
        
        assert_eq!(config.beardog_config.endpoint, "http://localhost:8443");
        assert_eq!(config.beardog_config.compliance_mode, "standard");
        assert_eq!(config.beardog_config.security_level, "internal");
        assert!(!config.beardog_config.hsm_integration_enabled);
        assert!(config.enable_guest);
        assert_eq!(config.session_timeout_seconds, 86400);
    }

    #[test]
    fn test_user_config_production() {
        let config = UserConfig::production();
        
        assert_eq!(config.beardog_config.compliance_mode, "fips140");
        assert_eq!(config.beardog_config.security_level, "secret");
        assert!(config.beardog_config.hsm_integration_enabled);
        assert!(!config.enable_guest);
        assert_eq!(config.session_timeout_seconds, 1800);
    }

    #[test]
    fn test_user_config_testing() {
        let config = UserConfig::testing();
        
        assert_eq!(config.beardog_config.endpoint, "http://localhost:8444");
        assert_eq!(config.beardog_config.compliance_mode, "standard");
        assert_eq!(config.beardog_config.security_level, "internal");
        assert!(!config.beardog_config.hsm_integration_enabled);
        assert!(!config.beardog_config.audit_logging_enabled);
        assert!(config.enable_guest);
        assert!(!config.enable_genetic_keys);
    }

    #[test]
    fn test_user_config_builder_pattern() {
        let config = UserConfig::new()
            .with_beardog_endpoint("https://custom.beardog.local".to_string())
            .with_session_timeout(7200)
            .with_guest_enabled(true)
            .with_compliance_mode("fips140".to_string())
            .with_security_level("secret".to_string());

        assert_eq!(config.beardog_config.endpoint, "https://custom.beardog.local");
        assert_eq!(config.session_timeout_seconds, 7200);
        assert!(config.enable_guest);
        assert_eq!(config.beardog_config.compliance_mode, "fips140");
        assert_eq!(config.beardog_config.security_level, "secret");
    }
} 
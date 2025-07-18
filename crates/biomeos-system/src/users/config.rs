//! User configuration and default implementations
//!
//! This module contains configuration structures and their default implementations
//! for the user management system.

use std::path::PathBuf;
use super::types::{UserConfig, BeardogIntegrationConfig, BeardogAuthMethod};

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            user_db_path: dirs::data_dir()
                .unwrap_or_else(|| PathBuf::from("/var/lib"))
                .join("biomeos/users.db"),
            home_dir_base: PathBuf::from("/home"),
            default_shell: PathBuf::from("/bin/bash"),
            default_group: "users".to_string(),
            beardog_config: BeardogIntegrationConfig {
                endpoint: "https://beardog.biome.local:8443".to_string(),
                auth_method: BeardogAuthMethod::MutualTLS {
                    cert_path: PathBuf::from("/etc/biomeos/certs/user-manager.crt"),
                    key_path: PathBuf::from("/etc/biomeos/certs/user-manager.key"),
                    ca_path: PathBuf::from("/etc/biomeos/certs/beardog-ca.crt"),
                },
                key_management_enabled: true,
                secret_storage_enabled: true,
                audit_logging_enabled: true,
                hsm_integration_enabled: true,
                compliance_mode: "soc2".to_string(),
                security_level: "confidential".to_string(),
            },
            session_timeout_seconds: 3600, // 1 hour
            enable_guest: false,
            enable_sudo: true,
            enable_genetic_keys: true, // Enable genetic BearDog keys by default
        }
    }
}

impl UserConfig {
    /// Create a new user configuration with custom values
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a user configuration for development environment
    pub fn development() -> Self {
        let mut config = Self::default();
        config.beardog_config.endpoint = "http://localhost:8443".to_string();
        config.beardog_config.compliance_mode = "standard".to_string();
        config.beardog_config.security_level = "internal".to_string();
        config.beardog_config.hsm_integration_enabled = false;
        config.enable_guest = true;
        config.session_timeout_seconds = 86400; // 24 hours for development
        config
    }

    /// Create a user configuration for production environment
    pub fn production() -> Self {
        let mut config = Self::default();
        config.beardog_config.compliance_mode = "fips140".to_string();
        config.beardog_config.security_level = "secret".to_string();
        config.beardog_config.hsm_integration_enabled = true;
        config.enable_guest = false;
        config.session_timeout_seconds = 1800; // 30 minutes for production
        config
    }

    /// Create a user configuration for testing environment
    pub fn testing() -> Self {
        let mut config = Self::default();
        config.beardog_config.endpoint = "http://localhost:8444".to_string();
        config.beardog_config.auth_method = BeardogAuthMethod::ApiKey {
            key_reference: "test_key".to_string(),
        };
        config.beardog_config.compliance_mode = "standard".to_string();
        config.beardog_config.security_level = "internal".to_string();
        config.beardog_config.hsm_integration_enabled = false;
        config.beardog_config.audit_logging_enabled = false;
        config.enable_guest = true;
        config.enable_genetic_keys = false; // Disable genetic keys for testing
        config.session_timeout_seconds = 3600;
        config
    }

    /// Configure BearDog endpoint
    pub fn with_beardog_endpoint(mut self, endpoint: String) -> Self {
        self.beardog_config.endpoint = endpoint;
        self
    }

    /// Configure session timeout
    pub fn with_session_timeout(mut self, timeout_seconds: u64) -> Self {
        self.session_timeout_seconds = timeout_seconds;
        self
    }

    /// Configure home directory base
    pub fn with_home_dir_base(mut self, home_dir: PathBuf) -> Self {
        self.home_dir_base = home_dir;
        self
    }

    /// Configure default shell
    pub fn with_default_shell(mut self, shell: PathBuf) -> Self {
        self.default_shell = shell;
        self
    }

    /// Configure default group
    pub fn with_default_group(mut self, group: String) -> Self {
        self.default_group = group;
        self
    }

    /// Enable or disable guest account
    pub fn with_guest_enabled(mut self, enabled: bool) -> Self {
        self.enable_guest = enabled;
        self
    }

    /// Enable or disable sudo access
    pub fn with_sudo_enabled(mut self, enabled: bool) -> Self {
        self.enable_sudo = enabled;
        self
    }

    /// Enable or disable genetic keys
    pub fn with_genetic_keys_enabled(mut self, enabled: bool) -> Self {
        self.enable_genetic_keys = enabled;
        self
    }

    /// Configure BearDog compliance mode
    pub fn with_compliance_mode(mut self, mode: String) -> Self {
        self.beardog_config.compliance_mode = mode;
        self
    }

    /// Configure BearDog security level
    pub fn with_security_level(mut self, level: String) -> Self {
        self.beardog_config.security_level = level;
        self
    }

    /// Enable or disable HSM integration
    pub fn with_hsm_integration(mut self, enabled: bool) -> Self {
        self.beardog_config.hsm_integration_enabled = enabled;
        self
    }

    /// Enable or disable audit logging
    pub fn with_audit_logging(mut self, enabled: bool) -> Self {
        self.beardog_config.audit_logging_enabled = enabled;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.session_timeout_seconds == 0 {
            return Err("Session timeout cannot be zero".to_string());
        }

        if self.beardog_config.endpoint.is_empty() {
            return Err("BearDog endpoint cannot be empty".to_string());
        }

        if !self.home_dir_base.exists() {
            return Err("Home directory base does not exist".to_string());
        }

        if !self.default_shell.exists() {
            return Err("Default shell does not exist".to_string());
        }

        let valid_compliance_modes = ["standard", "fips140", "soc2", "gdpr"];
        if !valid_compliance_modes.contains(&self.beardog_config.compliance_mode.as_str()) {
            return Err(format!(
                "Invalid compliance mode: {}. Valid modes: {:?}",
                self.beardog_config.compliance_mode, valid_compliance_modes
            ));
        }

        let valid_security_levels = ["public", "internal", "confidential", "secret"];
        if !valid_security_levels.contains(&self.beardog_config.security_level.as_str()) {
            return Err(format!(
                "Invalid security level: {}. Valid levels: {:?}",
                self.beardog_config.security_level, valid_security_levels
            ));
        }

        Ok(())
    }
}

impl BeardogIntegrationConfig {
    /// Create a new BearDog integration configuration
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint,
            auth_method: BeardogAuthMethod::MutualTLS {
                cert_path: PathBuf::from("/etc/biomeos/certs/user-manager.crt"),
                key_path: PathBuf::from("/etc/biomeos/certs/user-manager.key"),
                ca_path: PathBuf::from("/etc/biomeos/certs/beardog-ca.crt"),
            },
            key_management_enabled: true,
            secret_storage_enabled: true,
            audit_logging_enabled: true,
            hsm_integration_enabled: true,
            compliance_mode: "soc2".to_string(),
            security_level: "confidential".to_string(),
        }
    }

    /// Configure with API key authentication
    pub fn with_api_key(mut self, key_reference: String) -> Self {
        self.auth_method = BeardogAuthMethod::ApiKey { key_reference };
        self
    }

    /// Configure with service account authentication
    pub fn with_service_account(mut self, account_id: String, private_key_reference: String) -> Self {
        self.auth_method = BeardogAuthMethod::ServiceAccount {
            account_id,
            private_key_reference,
        };
        self
    }

    /// Configure with genetic key authentication
    pub fn with_genetic_key(mut self, parent_key_fingerprint: String, genetic_lineage: Vec<String>) -> Self {
        self.auth_method = BeardogAuthMethod::GeneticKey {
            parent_key_fingerprint,
            genetic_lineage,
        };
        self
    }

    /// Enable or disable key management
    pub fn with_key_management(mut self, enabled: bool) -> Self {
        self.key_management_enabled = enabled;
        self
    }

    /// Enable or disable secret storage
    pub fn with_secret_storage(mut self, enabled: bool) -> Self {
        self.secret_storage_enabled = enabled;
        self
    }

    /// Enable or disable audit logging
    pub fn with_audit_logging(mut self, enabled: bool) -> Self {
        self.audit_logging_enabled = enabled;
        self
    }

    /// Enable or disable HSM integration
    pub fn with_hsm_integration(mut self, enabled: bool) -> Self {
        self.hsm_integration_enabled = enabled;
        self
    }

    /// Set compliance mode
    pub fn with_compliance_mode(mut self, mode: String) -> Self {
        self.compliance_mode = mode;
        self
    }

    /// Set security level
    pub fn with_security_level(mut self, level: String) -> Self {
        self.security_level = level;
        self
    }
} 
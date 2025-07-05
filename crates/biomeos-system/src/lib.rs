//! # biomeOS System Management
//!
//! Core system management functionality for biomeOS.
//! Handles system configuration, state management, and integration with biomeOS core.

pub mod boot;
pub mod devices;
// pub mod management; // Temporarily disabled due to missing dependencies
pub mod packages;
pub mod services;
pub mod users;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub use boot::*;
pub use devices::*;
// pub use management::*; // Temporarily disabled
pub use packages::*;
pub use services::*;
pub use users::*;

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// System name/hostname
    pub hostname: String,
    /// System data directory
    pub data_dir: PathBuf,
    /// Configuration directory
    pub config_dir: PathBuf,
    /// Log directory
    pub log_dir: PathBuf,
    /// Runtime directory
    pub runtime_dir: PathBuf,
    /// Boot configuration
    pub boot: boot::BootConfig,
    /// Service configuration
    pub services: services::SystemServicesConfig,
    /// Device configuration
    pub devices: devices::DeviceConfig,
    /// User configuration
    pub users: users::UserConfig,
    /// Package configuration
    pub packages: packages::PackageConfig,
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            hostname: "biomeos-host".to_string(),
            data_dir: PathBuf::from("/var/lib/biomeos"),
            config_dir: PathBuf::from("/etc/biomeos"),
            log_dir: PathBuf::from("/var/log/biomeos"),
            runtime_dir: PathBuf::from("/run/biomeos"),
            boot: boot::BootConfig::default(),
            services: services::SystemServicesConfig::default(),
            devices: devices::DeviceConfig::default(),
            users: users::UserConfig::default(),
            packages: packages::PackageConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_config_default() {
        let config = SystemConfig::default();
        
        assert_eq!(config.hostname, "biomeos-host");
        assert_eq!(config.data_dir, PathBuf::from("/var/lib/biomeos"));
        assert_eq!(config.config_dir, PathBuf::from("/etc/biomeos"));
        assert_eq!(config.log_dir, PathBuf::from("/var/log/biomeos"));
        assert_eq!(config.runtime_dir, PathBuf::from("/run/biomeos"));
    }

    #[test]
    fn test_system_config_custom() {
        let config = SystemConfig {
            hostname: "custom-host".to_string(),
            data_dir: PathBuf::from("/custom/data"),
            config_dir: PathBuf::from("/custom/config"),
            log_dir: PathBuf::from("/custom/log"),
            runtime_dir: PathBuf::from("/custom/run"),
            boot: boot::BootConfig::default(),
            services: services::SystemServicesConfig::default(),
            devices: devices::DeviceConfig::default(),
            users: users::UserConfig::default(),
            packages: packages::PackageConfig::default(),
        };
        
        assert_eq!(config.hostname, "custom-host");
        assert_eq!(config.data_dir, PathBuf::from("/custom/data"));
        assert_eq!(config.config_dir, PathBuf::from("/custom/config"));
        assert_eq!(config.log_dir, PathBuf::from("/custom/log"));
        assert_eq!(config.runtime_dir, PathBuf::from("/custom/run"));
    }

    #[test]
    fn test_system_config_serialization() {
        let config = SystemConfig::default();
        
        // Test that we can serialize and deserialize
        let serialized = serde_json::to_string(&config).unwrap();
        let deserialized: SystemConfig = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(config.hostname, deserialized.hostname);
        assert_eq!(config.data_dir, deserialized.data_dir);
        assert_eq!(config.config_dir, deserialized.config_dir);
        assert_eq!(config.log_dir, deserialized.log_dir);
        assert_eq!(config.runtime_dir, deserialized.runtime_dir);
    }

    #[test]
    fn test_system_config_path_validation() {
        let config = SystemConfig::default();
        
        // Test that all paths are absolute
        assert!(config.data_dir.is_absolute());
        assert!(config.config_dir.is_absolute());
        assert!(config.log_dir.is_absolute());
        assert!(config.runtime_dir.is_absolute());
    }

    #[test]
    fn test_system_config_field_access() {
        let config = SystemConfig::default();
        
        // Test that we can access nested configurations with correct field names
        assert_eq!(config.boot.sequence.len(), 4);
        assert_eq!(config.services.startup_timeout_seconds, 60);
        assert_eq!(config.devices.enable_detection, true);
        assert_eq!(config.users.session_timeout_seconds, 3600);
        assert_eq!(config.packages.update_interval_seconds, 86400);
    }

    #[test]
    fn test_system_config_hostname_validation() {
        let mut config = SystemConfig::default();
        
        // Test hostname assignment
        config.hostname = "test-biomeos".to_string();
        assert_eq!(config.hostname, "test-biomeos");
        
        // Test hostname with special characters
        config.hostname = "biomeos-01".to_string();
        assert_eq!(config.hostname, "biomeos-01");
    }

    #[test]
    fn test_system_config_directory_structure() {
        let config = SystemConfig::default();
        
        // Test that directories follow Linux FHS standards
        assert!(config.data_dir.starts_with("/var/lib"));
        assert!(config.config_dir.starts_with("/etc"));
        assert!(config.log_dir.starts_with("/var/log"));
        assert!(config.runtime_dir.starts_with("/run"));
    }

    #[test]
    fn test_system_config_clone() {
        let config = SystemConfig::default();
        let cloned = config.clone();
        
        assert_eq!(config.hostname, cloned.hostname);
        assert_eq!(config.data_dir, cloned.data_dir);
        assert_eq!(config.config_dir, cloned.config_dir);
        assert_eq!(config.log_dir, cloned.log_dir);
        assert_eq!(config.runtime_dir, cloned.runtime_dir);
    }

    #[test]
    fn test_system_config_debug_format() {
        let config = SystemConfig::default();
        let debug_output = format!("{:?}", config);
        
        assert!(debug_output.contains("hostname"));
        assert!(debug_output.contains("biomeos-host"));
        assert!(debug_output.contains("data_dir"));
        assert!(debug_output.contains("config_dir"));
        assert!(debug_output.contains("log_dir"));
        assert!(debug_output.contains("runtime_dir"));
    }

    #[test]
    fn test_system_config_boot_component() {
        let config = SystemConfig::default();
        
        // Test boot configuration
        assert_eq!(config.boot.timeout_seconds, 300);
        assert_eq!(config.boot.log_level, "info");
        assert_eq!(config.boot.enable_splash, true);
        assert!(matches!(config.boot.target, boot::BootTarget::Normal));
    }

    #[test]
    fn test_system_config_services_component() {
        let config = SystemConfig::default();
        
        // Test services configuration
        assert_eq!(config.services.startup_timeout_seconds, 60);
        assert!(matches!(config.services.restart_policy, services::RestartPolicy::OnFailure));
    }

    #[test]
    fn test_system_config_devices_component() {
        let config = SystemConfig::default();
        
        // Test devices configuration  
        assert_eq!(config.devices.enable_detection, true);
        assert_eq!(config.devices.auto_configure, true);
        
    }

    #[test]
    fn test_system_config_users_component() {
        let config = SystemConfig::default();
        
        // Test users configuration
        assert_eq!(config.users.session_timeout_seconds, 3600);
        assert_eq!(config.users.default_shell, PathBuf::from("/bin/bash"));
        assert_eq!(config.users.home_dir_base, PathBuf::from("/home"));
    }

    #[test]
    fn test_system_config_packages_component() {
        let config = SystemConfig::default();
        
        // Test packages configuration
        assert_eq!(config.packages.update_interval_seconds, 86400);
        assert_eq!(config.packages.auto_update, false);
        
    }

    #[test]
    fn test_system_config_path_components() {
        let config = SystemConfig::default();
        
        // Test that path components are correct
        assert_eq!(config.data_dir.file_name().unwrap(), "biomeos");
        assert_eq!(config.config_dir.file_name().unwrap(), "biomeos");
        assert_eq!(config.log_dir.file_name().unwrap(), "biomeos");
        assert_eq!(config.runtime_dir.file_name().unwrap(), "biomeos");
    }

    #[test]
    fn test_system_config_modification() {
        let mut config = SystemConfig::default();
        
        // Test that we can modify configuration
        config.hostname = "modified-host".to_string();
        config.data_dir = PathBuf::from("/tmp/test");
        
        assert_eq!(config.hostname, "modified-host");
        assert_eq!(config.data_dir, PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_system_config_json_compatibility() {
        let config = SystemConfig::default();
        
        // Test JSON serialization/deserialization
        let json = serde_json::to_value(&config).unwrap();
        assert!(json.is_object());
        
        let from_json: SystemConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.hostname, from_json.hostname);
    }

    #[test]
    fn test_system_config_memory_usage() {
        let config = SystemConfig::default();
        
        // Test that config doesn't consume excessive memory
        let size = std::mem::size_of_val(&config);
        assert!(size < 10000); // Should be reasonable size
    }

    #[test]
    fn test_system_config_nested_defaults() {
        let config = SystemConfig::default();
        
        // Test that nested configurations have proper defaults
        assert_eq!(config.boot.sequence.len(), 4);
        assert_eq!(config.services.services.len(), 1);
        assert_eq!(config.users.enable_guest, false);
        assert_eq!(config.packages.repositories.len(), 2);
    }

    #[test]
    fn test_system_config_component_creation() {
        // Test that we can create SystemConfig with custom components
        let custom_boot = boot::BootConfig {
            timeout_seconds: 60,
            sequence: vec![],
            log_level: "debug".to_string(),
            enable_splash: true,
            target: boot::BootTarget::Maintenance,
        };

        let config = SystemConfig {
            hostname: "test-host".to_string(),
            data_dir: PathBuf::from("/test/data"),
            config_dir: PathBuf::from("/test/config"),
            log_dir: PathBuf::from("/test/log"),
            runtime_dir: PathBuf::from("/test/run"),
            boot: custom_boot.clone(),
            services: services::SystemServicesConfig::default(),
            devices: devices::DeviceConfig::default(),
            users: users::UserConfig::default(),
            packages: packages::PackageConfig::default(),
        };

        assert_eq!(config.boot.timeout_seconds, 60);
        assert_eq!(config.boot.log_level, "debug");
        assert_eq!(config.boot.enable_splash, true);
        assert!(matches!(config.boot.target, boot::BootTarget::Maintenance));
    }
}

//! Device Management Tests for biomeOS

use biomeos_system::devices::*;
use std::collections::HashMap;

#[test]
fn test_device_config_default() {
    let config = DeviceConfig::default();

    assert!(config.enable_detection);
    assert!(config.auto_configure);
    assert!(config.driver_dir.ends_with("drivers"));
    assert!(config.config_dir.ends_with("devices"));
    assert!(config.blacklist.is_empty());
    assert!(config.whitelist.is_empty());
}

#[test]
fn test_device_config_custom() {
    let config = DeviceConfig {
        enable_detection: false,
        auto_configure: false,
        driver_dir: "/custom/drivers".into(),
        config_dir: "/custom/devices".into(),
        blacklist: vec!["bad_device".to_string()],
        whitelist: vec!["good_device".to_string()],
    };

    assert!(!config.enable_detection);
    assert!(!config.auto_configure);
    assert_eq!(config.driver_dir.to_string_lossy(), "/custom/drivers");
    assert_eq!(config.config_dir.to_string_lossy(), "/custom/devices");
    assert_eq!(config.blacklist.len(), 1);
    assert_eq!(config.whitelist.len(), 1);
}

#[test]
fn test_device_type_variants() {
    let types = vec![
        DeviceType::Cpu,
        DeviceType::Memory,
        DeviceType::Storage,
        DeviceType::Network,
        DeviceType::Gpu,
        DeviceType::Audio,
        DeviceType::Input,
        DeviceType::Display,
        DeviceType::Usb,
        DeviceType::Pci,
        DeviceType::Bluetooth,
        DeviceType::Sensor,
        DeviceType::Camera,
        DeviceType::Printer,
        DeviceType::Unknown,
    ];

    for device_type in types {
        // Each type should be serializable
        let json = serde_json::to_string(&device_type).unwrap();
        let _from_json: DeviceType = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_device_state_variants() {
    let states = vec![
        DeviceState::Available,
        DeviceState::InUse,
        DeviceState::Disabled,
        DeviceState::Failed,
        DeviceState::NotPresent,
        DeviceState::Unknown,
    ];

    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let _from_json: DeviceState = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_device_health_variants() {
    let health_states = vec![
        DeviceHealth::Healthy,
        DeviceHealth::Warning,
        DeviceHealth::Unhealthy,
        DeviceHealth::Unknown,
    ];

    for health in health_states {
        let json = serde_json::to_string(&health).unwrap();
        let _from_json: DeviceHealth = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_power_state_variants() {
    let power_states = vec![
        PowerState::On,
        PowerState::Off,
        PowerState::Sleep,
        PowerState::Hibernate,
        PowerState::Unknown,
    ];

    for power_state in power_states {
        let json = serde_json::to_string(&power_state).unwrap();
        let _from_json: PowerState = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_device_capability_variants() {
    let capabilities = vec![
        DeviceCapability::Read,
        DeviceCapability::Write,
        DeviceCapability::Execute,
        DeviceCapability::Network,
        DeviceCapability::Graphics,
        DeviceCapability::Audio,
        DeviceCapability::Video,
        DeviceCapability::Compute,
        DeviceCapability::Storage,
        DeviceCapability::Custom("test".to_string()),
    ];

    for capability in capabilities {
        let json = serde_json::to_string(&capability).unwrap();
        let _from_json: DeviceCapability = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_driver_status_variants() {
    let statuses = vec![
        DriverStatus::Loaded,
        DriverStatus::NotLoaded,
        DriverStatus::Failed,
        DriverStatus::Loading,
        DriverStatus::Unloading,
    ];

    for status in statuses {
        let json = serde_json::to_string(&status).unwrap();
        let _from_json: DriverStatus = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_device_usage_creation() {
    let usage = DeviceUsage {
        usage_percent: 75.5,
        bytes_processed: 1024000,
        operations_per_second: 100.0,
        error_count: 5,
        uptime_seconds: 3600,
    };

    assert_eq!(usage.usage_percent, 75.5);
    assert_eq!(usage.bytes_processed, 1024000);
    assert_eq!(usage.operations_per_second, 100.0);
    assert_eq!(usage.error_count, 5);
    assert_eq!(usage.uptime_seconds, 3600);
}

#[test]
fn test_device_status_creation() {
    let usage = DeviceUsage {
        usage_percent: 50.0,
        bytes_processed: 512000,
        operations_per_second: 50.0,
        error_count: 0,
        uptime_seconds: 1800,
    };

    let status = DeviceStatus {
        state: DeviceState::Available,
        health: DeviceHealth::Healthy,
        temperature: Some(45.5),
        power_state: PowerState::On,
        usage,
        last_error: None,
    };

    assert!(matches!(status.state, DeviceState::Available));
    assert!(matches!(status.health, DeviceHealth::Healthy));
    assert_eq!(status.temperature.unwrap(), 45.5);
    assert!(matches!(status.power_state, PowerState::On));
    assert_eq!(status.usage.usage_percent, 50.0);
    assert!(status.last_error.is_none());
}

#[test]
fn test_device_creation() {
    let mut properties = HashMap::new();
    properties.insert("vendor_id".to_string(), "0x1234".to_string());
    properties.insert("product_id".to_string(), "0x5678".to_string());

    let usage = DeviceUsage {
        usage_percent: 0.0,
        bytes_processed: 0,
        operations_per_second: 0.0,
        error_count: 0,
        uptime_seconds: 0,
    };

    let status = DeviceStatus {
        state: DeviceState::Available,
        health: DeviceHealth::Healthy,
        temperature: None,
        power_state: PowerState::On,
        usage,
        last_error: None,
    };

    let device = Device {
        id: "device_001".to_string(),
        name: "Test Device".to_string(),
        device_type: DeviceType::Usb,
        vendor: Some("Test Vendor".to_string()),
        model: Some("Test Model".to_string()),
        serial: Some("SN123456".to_string()),
        version: Some("1.0".to_string()),
        driver: Some("test_driver".to_string()),
        status,
        capabilities: vec![DeviceCapability::Read, DeviceCapability::Write],
        properties,
        path: Some("/dev/test_device".into()),
    };

    assert_eq!(device.id, "device_001");
    assert_eq!(device.name, "Test Device");
    assert!(matches!(device.device_type, DeviceType::Usb));
    assert_eq!(device.vendor.unwrap(), "Test Vendor");
    assert_eq!(device.model.unwrap(), "Test Model");
    assert_eq!(device.serial.unwrap(), "SN123456");
    assert_eq!(device.version.unwrap(), "1.0");
    assert_eq!(device.driver.unwrap(), "test_driver");
    assert_eq!(device.capabilities.len(), 2);
    assert_eq!(device.properties.len(), 2);
    assert!(device.path.is_some());
}

#[test]
fn test_device_driver_creation() {
    let mut config = HashMap::new();
    config.insert("param1".to_string(), "value1".to_string());
    config.insert("param2".to_string(), "value2".to_string());

    let driver = DeviceDriver {
        name: "test_driver".to_string(),
        version: "1.2.3".to_string(),
        description: "Test device driver".to_string(),
        path: "/drivers/test_driver.ko".into(),
        supported_devices: vec!["device_001".to_string(), "device_002".to_string()],
        status: DriverStatus::Loaded,
        config,
    };

    assert_eq!(driver.name, "test_driver");
    assert_eq!(driver.version, "1.2.3");
    assert_eq!(driver.description, "Test device driver");
    assert!(driver.path.ends_with("test_driver.ko"));
    assert_eq!(driver.supported_devices.len(), 2);
    assert!(matches!(driver.status, DriverStatus::Loaded));
    assert_eq!(driver.config.len(), 2);
}

#[tokio::test]
async fn test_device_manager_creation() {
    let config = DeviceConfig::default();
    let manager = DeviceManager::new(config.clone());

    assert_eq!(manager.config.enable_detection, config.enable_detection);
    assert_eq!(manager.config.auto_configure, config.auto_configure);
    assert_eq!(manager.config.driver_dir, config.driver_dir);
    assert_eq!(manager.config.config_dir, config.config_dir);
}

#[tokio::test]
async fn test_device_manager_initialization() {
    let config = DeviceConfig::default();
    let manager = DeviceManager::new(config);

    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_device_manager_get_operations() {
    let config = DeviceConfig::default();
    let manager = DeviceManager::new(config);

    // Test getting non-existent device
    let device = manager.get_device("nonexistent").await;
    assert!(device.is_none());

    // Test getting all devices (should be empty initially)
    let all_devices = manager.get_all_devices().await;
    assert!(all_devices.is_empty());

    // Test getting devices by type
    let usb_devices = manager.get_devices_by_type(DeviceType::Usb).await;
    assert!(usb_devices.is_empty());
}

#[test]
fn test_device_config_serialization() {
    let config = DeviceConfig::default();

    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let from_json: DeviceConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enable_detection, from_json.enable_detection);
    assert_eq!(config.auto_configure, from_json.auto_configure);
    assert_eq!(config.driver_dir, from_json.driver_dir);
    assert_eq!(config.config_dir, from_json.config_dir);
    assert_eq!(config.blacklist, from_json.blacklist);
    assert_eq!(config.whitelist, from_json.whitelist);
}

#[test]
fn test_device_serialization() {
    let usage = DeviceUsage {
        usage_percent: 25.0,
        bytes_processed: 256000,
        operations_per_second: 25.0,
        error_count: 1,
        uptime_seconds: 900,
    };

    let status = DeviceStatus {
        state: DeviceState::InUse,
        health: DeviceHealth::Warning,
        temperature: Some(60.0),
        power_state: PowerState::On,
        usage,
        last_error: Some("Test error".to_string()),
    };

    let device = Device {
        id: "test_device".to_string(),
        name: "Test Device".to_string(),
        device_type: DeviceType::Storage,
        vendor: Some("Test Corp".to_string()),
        model: Some("Model X".to_string()),
        serial: Some("ABC123".to_string()),
        version: Some("2.0".to_string()),
        driver: Some("storage_driver".to_string()),
        status,
        capabilities: vec![DeviceCapability::Read, DeviceCapability::Storage],
        properties: HashMap::new(),
        path: Some("/dev/sda".into()),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&device).unwrap();
    let from_json: Device = serde_json::from_str(&json).unwrap();

    assert_eq!(device.id, from_json.id);
    assert_eq!(device.name, from_json.name);
    assert_eq!(device.vendor, from_json.vendor);
    assert_eq!(device.model, from_json.model);
    assert_eq!(device.serial, from_json.serial);
    assert_eq!(device.capabilities.len(), from_json.capabilities.len());
}

#[test]
fn test_device_usage_validation() {
    let usage = DeviceUsage {
        usage_percent: 100.0,
        bytes_processed: u64::MAX,
        operations_per_second: 1000.0,
        error_count: 0,
        uptime_seconds: 86400, // 1 day
    };

    // Verify percentage is within reasonable bounds
    assert!(usage.usage_percent >= 0.0 && usage.usage_percent <= 100.0);

    // Verify other fields are non-negative
    assert!(usage.bytes_processed >= 0);
    assert!(usage.operations_per_second >= 0.0);
    assert!(usage.error_count >= 0);
    assert!(usage.uptime_seconds >= 0);
}

#[test]
fn test_device_blacklist_whitelist() {
    let config = DeviceConfig {
        enable_detection: true,
        auto_configure: true,
        driver_dir: "/drivers".into(),
        config_dir: "/config".into(),
        blacklist: vec!["bad_device1".to_string(), "bad_device2".to_string()],
        whitelist: vec!["good_device1".to_string(), "good_device2".to_string()],
    };

    assert_eq!(config.blacklist.len(), 2);
    assert_eq!(config.whitelist.len(), 2);
    assert!(config.blacklist.contains(&"bad_device1".to_string()));
    assert!(config.whitelist.contains(&"good_device1".to_string()));
}

#[test]
fn test_device_config_paths() {
    let config = DeviceConfig::default();

    // Verify paths are reasonable
    assert!(config.driver_dir.is_absolute());
    assert!(config.config_dir.is_absolute());
    assert!(config.driver_dir.ends_with("drivers"));
    assert!(config.config_dir.ends_with("devices"));
}

#[test]
fn test_device_capability_custom() {
    let custom_cap = DeviceCapability::Custom("special_feature".to_string());

    match custom_cap {
        DeviceCapability::Custom(ref name) => assert_eq!(name, "special_feature"),
        _ => panic!("Expected custom capability"),
    }

    // Test serialization of custom capability
    let json = serde_json::to_string(&custom_cap).unwrap();
    let from_json: DeviceCapability = serde_json::from_str(&json).unwrap();

    match from_json {
        DeviceCapability::Custom(ref name) => assert_eq!(name, "special_feature"),
        _ => panic!("Expected custom capability after deserialization"),
    }
}

//! System Services Tests for biomeOS

use biomeos_system::services::*;
use std::collections::HashMap;

#[test]
fn test_system_services_config_default() {
    let config = SystemServicesConfig::default();
    
    assert_eq!(config.services.len(), 1);
    assert_eq!(config.startup_timeout_seconds, 60);
    assert!(matches!(config.restart_policy, RestartPolicy::OnFailure));
    assert!(config.log_dir.ends_with("services"));
}

#[test]
fn test_restart_policy_variants() {
    let policies = vec![
        RestartPolicy::Never,
        RestartPolicy::OnFailure,
        RestartPolicy::Always,
        RestartPolicy::UnlessStopped,
    ];
    
    for policy in policies {
        let json = serde_json::to_string(&policy).unwrap();
        let _from_json: RestartPolicy = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_state_variants() {
    let states = vec![
        ServiceState::Stopped,
        ServiceState::Starting,
        ServiceState::Running,
        ServiceState::Stopping,
        ServiceState::Failed { reason: "test error".to_string() },
        ServiceState::Unknown,
    ];
    
    for state in states {
        let json = serde_json::to_string(&state).unwrap();
        let _from_json: ServiceState = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_health_variants() {
    let healths = vec![
        ServiceHealth::Healthy,
        ServiceHealth::Unhealthy,
        ServiceHealth::Unknown,
    ];
    
    for health in healths {
        let json = serde_json::to_string(&health).unwrap();
        let _from_json: ServiceHealth = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_log_level_variants() {
    let levels = vec![
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warning,
        LogLevel::Error,
    ];
    
    for level in levels {
        let json = serde_json::to_string(&level).unwrap();
        let _from_json: LogLevel = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_system_service_config_creation() {
    let mut env = HashMap::new();
    env.insert("VAR1".to_string(), "value1".to_string());
    
    let service_config = SystemServiceConfig {
        name: "test_service".to_string(),
        description: "Test service description".to_string(),
        executable: "/usr/bin/test_service".into(),
        args: vec!["--config".to_string()],
        environment: env.clone(),
        working_dir: Some("/var/lib/test".into()),
        user: Some("test_user".to_string()),
        group: Some("test_group".to_string()),
        dependencies: vec!["network.service".to_string()],
        critical: true,
        auto_start: true,
        restart_policy: RestartPolicy::Always,
    };
    
    assert_eq!(service_config.name, "test_service");
    assert_eq!(service_config.description, "Test service description");
    assert!(service_config.executable.ends_with("test_service"));
    assert_eq!(service_config.args.len(), 1);
    assert_eq!(service_config.environment.len(), 1);
    assert!(service_config.working_dir.is_some());
    assert_eq!(service_config.user.unwrap(), "test_user");
    assert_eq!(service_config.group.unwrap(), "test_group");
    assert_eq!(service_config.dependencies.len(), 1);
    assert!(service_config.critical);
    assert!(service_config.auto_start);
    assert!(matches!(service_config.restart_policy, RestartPolicy::Always));
}

#[tokio::test]
async fn test_system_services_manager_creation() {
    let config = SystemServicesConfig::default();
    let manager = SystemServicesManager::new(config.clone());
    
    assert_eq!(manager.config.services.len(), config.services.len());
    assert_eq!(manager.config.startup_timeout_seconds, config.startup_timeout_seconds);
    assert_eq!(manager.config.log_dir, config.log_dir);
}

#[tokio::test]
async fn test_system_services_manager_initialization() {
    let config = SystemServicesConfig::default();
    let manager = SystemServicesManager::new(config);
    
    let result = manager.initialize().await;
    assert!(result.is_ok());
}

#[test]
fn test_system_services_config_serialization() {
    let config = SystemServicesConfig::default();
    
    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let from_json: SystemServicesConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.services.len(), from_json.services.len());
    assert_eq!(config.startup_timeout_seconds, from_json.startup_timeout_seconds);
    assert_eq!(config.log_dir, from_json.log_dir);
}

#[test]
fn test_service_log_entry() {
    let log_entry = ServiceLogEntry {
        timestamp: chrono::Utc::now(),
        level: LogLevel::Info,
        message: "Test log message".to_string(),
    };
    
    assert_eq!(log_entry.message, "Test log message");
    assert!(matches!(log_entry.level, LogLevel::Info));
    assert!(log_entry.timestamp <= chrono::Utc::now());
}

#[test]
fn test_system_service_status() {
    let status = SystemServiceStatus {
        state: ServiceState::Running,
        health: ServiceHealth::Healthy,
        logs: vec![],
    };
    
    assert!(matches!(status.state, ServiceState::Running));
    assert!(matches!(status.health, ServiceHealth::Healthy));
    assert!(status.logs.is_empty());
}

#[test]
fn test_service_state_failed() {
    let failed_state = ServiceState::Failed { 
        reason: "Connection timeout".to_string() 
    };
    
    match failed_state {
        ServiceState::Failed { reason } => {
            assert_eq!(reason, "Connection timeout");
        }
        _ => panic!("Expected failed state"),
    }
}

#[test]
fn test_restart_policy_logic() {
    let policies = [
        RestartPolicy::Never,
        RestartPolicy::Always,
        RestartPolicy::OnFailure,
        RestartPolicy::UnlessStopped,
    ];
    
    for policy in &policies {
        match policy {
            RestartPolicy::Never => {
                assert_eq!(format!("{:?}", policy), "Never");
            }
            RestartPolicy::Always => {
                assert_eq!(format!("{:?}", policy), "Always");
            }
            RestartPolicy::OnFailure => {
                assert_eq!(format!("{:?}", policy), "OnFailure");
            }
            RestartPolicy::UnlessStopped => {
                assert_eq!(format!("{:?}", policy), "UnlessStopped");
            }
        }
    }
}

#[test]
fn test_service_environment_variables() {
    let mut env = HashMap::new();
    env.insert("HOME".to_string(), "/home/service".to_string());
    env.insert("PATH".to_string(), "/usr/bin:/bin".to_string());
    
    let config = SystemServiceConfig {
        name: "env_test".to_string(),
        description: "Environment test".to_string(),
        executable: "/usr/bin/env_test".into(),
        args: vec![],
        environment: env.clone(),
        working_dir: None,
        user: None,
        group: None,
        dependencies: vec![],
        critical: false,
        auto_start: true,
        restart_policy: RestartPolicy::OnFailure,
    };
    
    assert_eq!(config.environment.len(), 2);
    assert_eq!(config.environment.get("HOME").unwrap(), "/home/service");
    assert_eq!(config.environment.get("PATH").unwrap(), "/usr/bin:/bin");
}

#[test]
fn test_service_dependencies() {
    let config = SystemServiceConfig {
        name: "dependent_service".to_string(),
        description: "Service with dependencies".to_string(),
        executable: "/usr/bin/dependent".into(),
        args: vec![],
        environment: HashMap::new(),
        working_dir: None,
        user: None,
        group: None,
        dependencies: vec![
            "network.service".to_string(),
            "database.service".to_string(),
        ],
        critical: true,
        auto_start: true,
        restart_policy: RestartPolicy::Always,
    };
    
    assert_eq!(config.dependencies.len(), 2);
    assert!(config.dependencies.contains(&"network.service".to_string()));
    assert!(config.dependencies.contains(&"database.service".to_string()));
}

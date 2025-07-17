//! Boot Management Tests for biomeOS

use biomeos_system::boot::*;

#[test]
fn test_boot_config_default() {
    let config = BootConfig::default();

    assert_eq!(config.timeout_seconds, 300);
    assert_eq!(config.sequence.len(), 4);
    assert_eq!(config.log_level, "info");
    assert!(config.enable_splash);
    assert!(matches!(config.target, BootTarget::Normal));

    // Verify default boot sequence
    assert_eq!(config.sequence[0].name, "hardware_detection");
    assert_eq!(config.sequence[1].name, "system_services");
    assert_eq!(config.sequence[2].name, "primal_ecosystem");
    assert_eq!(config.sequence[3].name, "user_space");
}

#[test]
fn test_boot_target_variants() {
    let normal = BootTarget::Normal;
    let maintenance = BootTarget::Maintenance;
    let recovery = BootTarget::Recovery;
    let single_user = BootTarget::SingleUser;
    let emergency = BootTarget::Emergency;

    assert_eq!(format!("{:?}", normal), "Normal");
    assert_eq!(format!("{:?}", maintenance), "Maintenance");
    assert_eq!(format!("{:?}", recovery), "Recovery");
    assert_eq!(format!("{:?}", single_user), "SingleUser");
    assert_eq!(format!("{:?}", emergency), "Emergency");
}

#[tokio::test]
async fn test_boot_manager_creation() {
    let config = BootConfig::default();
    let manager = BootManager::new(config.clone());

    assert_eq!(manager.config.timeout_seconds, config.timeout_seconds);
    assert_eq!(manager.config.sequence.len(), config.sequence.len());
    assert_eq!(manager.config.log_level, config.log_level);
    assert_eq!(manager.config.enable_splash, config.enable_splash);
}

#[tokio::test]
async fn test_boot_manager_initialization() {
    let config = BootConfig::default();
    let manager = BootManager::new(config);

    let result = manager.initialize().await;
    assert!(result.is_ok());

    let state = manager.get_boot_state().await;
    assert!(matches!(state.phase, BootPhase::Initialization));
    assert!(state.messages.len() > 0);
    assert!(state.completed_steps.is_empty());
    assert!(state.failed_steps.is_empty());
}

#[tokio::test]
async fn test_boot_sequence_execution() {
    // Create a simple boot configuration for testing
    let simple_step = BootStep {
        name: "hardware_detection".to_string(),
        description: "Simple test step".to_string(),
        dependencies: vec![],
        timeout_seconds: 5,
        critical: false,
    };

    let config = BootConfig {
        timeout_seconds: 60,
        sequence: vec![simple_step],
        log_level: "info".to_string(),
        enable_splash: true,
        target: BootTarget::Normal,
    };

    let manager = BootManager::new(config);
    manager.initialize().await.unwrap();

    let result = manager.start_boot().await;
    assert!(result.is_ok());

    let final_state = manager.get_boot_state().await;
    assert!(matches!(final_state.phase, BootPhase::Complete));
    assert_eq!(final_state.completed_steps.len(), 1);
    assert!(final_state
        .completed_steps
        .contains(&"hardware_detection".to_string()));
    assert!(final_state.failed_steps.is_empty());
}

#[tokio::test]
async fn test_boot_config_serialization() {
    let config = BootConfig::default();

    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let from_json: BootConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.timeout_seconds, from_json.timeout_seconds);
    assert_eq!(config.sequence.len(), from_json.sequence.len());
    assert_eq!(config.log_level, from_json.log_level);
    assert_eq!(config.enable_splash, from_json.enable_splash);
}

#[test]
fn test_boot_step_dependency_validation() {
    let config = BootConfig::default();

    // Verify dependency chain is logical
    for step in &config.sequence {
        for dep in &step.dependencies {
            // Each dependency should exist as a step name in the sequence
            let dep_exists = config.sequence.iter().any(|s| &s.name == dep);
            assert!(
                dep_exists,
                "Dependency '{}' for step '{}' does not exist",
                dep, step.name
            );
        }
    }
}

#[test]
fn test_boot_target_enum_completeness() {
    // Ensure all boot targets are handled
    let targets = vec![
        BootTarget::Normal,
        BootTarget::Maintenance,
        BootTarget::Recovery,
        BootTarget::SingleUser,
        BootTarget::Emergency,
    ];

    for target in targets {
        // Each target should be serializable
        let json = serde_json::to_string(&target).unwrap();
        let _from_json: BootTarget = serde_json::from_str(&json).unwrap();
    }
}

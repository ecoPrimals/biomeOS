// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::config::*;

#[test]
fn test_subconfigs_have_defaults() {
    let _ = SystemConfig::default();
    let _ = NetworkConfig::default();
    let _ = SecurityConfig::default();
    let _ = ResourceConfig::default();
    let _ = DiscoveryConfig::default();
    let _ = HealthMonitoringConfig::default();
    let _ = ObservabilityConfig::default();
    let _ = UIConfig::default();
    let _ = FeatureFlags::default();
}

#[test]
fn test_system_config_defaults() {
    let sys = SystemConfig::default();
    assert_eq!(sys.name, "biomeos");
    assert_eq!(sys.environment, Environment::Development);
    assert_eq!(sys.organization_scale, OrganizationScale::Individual);
    assert_eq!(
        sys.timeouts.default_request_timeout,
        std::time::Duration::from_secs(30)
    );
    assert_eq!(sys.limits.max_connections, 1000);
}

#[test]
fn test_timeout_config_defaults() {
    let timeouts = TimeoutConfig::default();
    assert_eq!(
        timeouts.connection_timeout,
        std::time::Duration::from_secs(10)
    );
    assert_eq!(
        timeouts.health_check_timeout,
        std::time::Duration::from_secs(5)
    );
    assert_eq!(
        timeouts.shutdown_timeout,
        std::time::Duration::from_secs(30)
    );
}

#[test]
fn test_system_limits_defaults() {
    let limits = SystemLimits::default();
    assert_eq!(limits.max_connections, 1000);
    assert_eq!(limits.max_request_size, 10 * 1024 * 1024);
    assert_eq!(limits.max_upload_size, 100 * 1024 * 1024);
    assert!(limits.max_memory_usage.is_none());
    assert!(limits.max_cpu_usage.is_none());
}

#[test]
fn test_environment_variants() {
    let envs = vec![
        Environment::Development,
        Environment::Testing,
        Environment::Staging,
        Environment::Production,
        Environment::Custom("custom-env".to_string()),
    ];
    for env in envs {
        let json = serde_json::to_string(&env).expect("serialize env");
        let parsed: Environment = serde_json::from_str(&json).expect("parse env");
        assert_eq!(format!("{env:?}"), format!("{:?}", parsed));
    }
}

#[test]
fn test_organization_scale_variants() {
    let scales = vec![
        OrganizationScale::Individual,
        OrganizationScale::Team,
        OrganizationScale::Department,
        OrganizationScale::Enterprise,
        OrganizationScale::Global,
    ];
    for scale in scales {
        let json = serde_json::to_string(&scale).expect("serialize scale");
        let parsed: OrganizationScale = serde_json::from_str(&json).expect("parse scale");
        assert_eq!(format!("{scale:?}"), format!("{:?}", parsed));
    }
}

#[test]
fn test_feature_flags_default() {
    let features = FeatureFlags::default();
    let _ = features.debug;
}

#[test]
fn test_security_config_default() {
    let security = SecurityConfig::default();
    let _ = security.authentication;
    let _ = security.authorization;
    let _ = security.encryption;
}

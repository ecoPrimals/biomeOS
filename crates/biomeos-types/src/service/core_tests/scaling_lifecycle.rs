// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use crate::service::scaling::*;

#[test]
fn test_service_scaling_default_and_serde() {
    let scaling = ServiceScaling::default();
    assert!(matches!(scaling.scaling_type, ScalingType::Manual));
    assert_eq!(scaling.min_replicas, 1);
    assert_eq!(scaling.max_replicas, 1);
    let json = serde_json::to_string(&scaling).unwrap();
    let deserialized: ServiceScaling = serde_json::from_str(&json).unwrap();
    assert_eq!(scaling.min_replicas, deserialized.min_replicas);
}

#[test]
fn test_scaling_policy_and_metrics_serde() {
    let policy = ScalingPolicy {
        name: "scale-up".to_string(),
        direction: ScalingDirection::Up,
        amount: ScalingAmount::Fixed(2),
        cooldown: 60,
    };
    let json = serde_json::to_string(&policy).unwrap();
    let deserialized: ScalingPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(policy.name, deserialized.name);

    let metric = ScalingMetric {
        name: "cpu".to_string(),
        metric_type: ScalingMetricType::CpuUtilization,
        target_value: 80.0,
        current_value: Some(45.0),
    };
    let json = serde_json::to_string(&metric).unwrap();
    let deserialized: ScalingMetric = serde_json::from_str(&json).unwrap();
    assert!((metric.target_value - deserialized.target_value).abs() < f64::EPSILON);
}

#[test]
fn test_scaling_type_and_amount_serde() {
    let scaling_types = [
        ScalingType::Manual,
        ScalingType::Hpa,
        ScalingType::Vpa,
        ScalingType::Custom("custom".to_string()),
    ];
    for st in scaling_types {
        let json = serde_json::to_string(&st).unwrap();
        let _: ScalingType = serde_json::from_str(&json).unwrap();
    }
    let amounts = [ScalingAmount::Fixed(5), ScalingAmount::Percent(50)];
    for a in amounts {
        let json = serde_json::to_string(&a).unwrap();
        let _: ScalingAmount = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_configuration_default_and_serde() {
    let config = ServiceConfiguration::default();
    assert_eq!(config.sources.len(), 1);
    assert!(matches!(config.sources[0], ConfigSource::Environment));
    assert!(config.environment.is_empty());
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ServiceConfiguration = serde_json::from_str(&json).unwrap();
    assert_eq!(config.sources.len(), deserialized.sources.len());
}

#[test]
fn test_config_source_and_format_serde() {
    let sources = [
        ConfigSource::Environment,
        ConfigSource::Files,
        ConfigSource::External {
            url: "https://config.example.com".to_string(),
            auth: Some("token".to_string()),
        },
        ConfigSource::ConfigMap("my-config".to_string()),
        ConfigSource::Secret("my-secret".to_string()),
    ];
    for s in sources {
        let json = serde_json::to_string(&s).unwrap();
        let _: ConfigSource = serde_json::from_str(&json).unwrap();
    }
    let formats = [
        ConfigFormat::Json,
        ConfigFormat::Yaml,
        ConfigFormat::Toml,
        ConfigFormat::Properties,
        ConfigFormat::Ini,
        ConfigFormat::Custom("custom".to_string()),
    ];
    for f in formats {
        let json = serde_json::to_string(&f).unwrap();
        let _: ConfigFormat = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_lifecycle_default_and_serde() {
    let lifecycle = ServiceLifecycle::default();
    assert!(matches!(lifecycle.restart_policy, RestartPolicy::Always));
    assert_eq!(lifecycle.termination_grace_period, 30);
    let json = serde_json::to_string(&lifecycle).unwrap();
    let deserialized: ServiceLifecycle = serde_json::from_str(&json).unwrap();
    assert_eq!(
        lifecycle.termination_grace_period,
        deserialized.termination_grace_period
    );
}

#[test]
fn test_restart_policy_and_lifecycle_failure_action_serde() {
    let policies = [
        RestartPolicy::Always,
        RestartPolicy::OnFailure,
        RestartPolicy::Never,
        RestartPolicy::UnlessStopped,
    ];
    for p in policies {
        let json = serde_json::to_string(&p).unwrap();
        let _: RestartPolicy = serde_json::from_str(&json).unwrap();
    }
    let actions = [
        LifecycleFailureAction::Ignore,
        LifecycleFailureAction::Abort,
        LifecycleFailureAction::Retry,
    ];
    for a in actions {
        let json = serde_json::to_string(&a).unwrap();
        let _: LifecycleFailureAction = serde_json::from_str(&json).unwrap();
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_health_check_config_default() {
    let config = HealthCheckConfig::default();
    assert!(config.interval_secs > 0);
    assert!(config.timeout_secs > 0);
    assert!(config.failure_threshold > 0);
    assert!(config.success_threshold > 0);
}

#[test]
fn test_health_check_target_http() {
    let target = HealthCheckTarget::Http {
        url: "/health".to_string(),
        method: "GET".to_string(),
    };
    match target {
        HealthCheckTarget::Http { url, method } => {
            assert_eq!(url, "/health");
            assert_eq!(method, "GET");
        }
        _ => panic!("Expected Http target"),
    }
}

#[test]
fn test_health_check_target_tcp() {
    let target = HealthCheckTarget::Tcp {
        host: "localhost".to_string(),
        port: 8080,
    };
    match target {
        HealthCheckTarget::Tcp { host, port } => {
            assert_eq!(host, "localhost");
            assert_eq!(port, 8080);
        }
        _ => panic!("Expected Tcp target"),
    }
}

#[test]
fn test_metric_threshold() {
    let threshold = MetricThreshold {
        value: 80.0,
        operator: ThresholdOperator::GreaterThan,
        action: ThresholdAction::MarkDegraded,
    };
    assert!((threshold.value - 80.0).abs() < f64::EPSILON);
}

#[test]
fn test_threshold_actions() {
    let alert = ThresholdAction::TriggerAlert {
        alert_type: "pager".to_string(),
    };
    match alert {
        ThresholdAction::TriggerAlert { alert_type } => {
            assert_eq!(alert_type, "pager");
        }
        _ => panic!("Expected TriggerAlert"),
    }

    let remediate = ThresholdAction::ExecuteRemediation {
        action_id: "restart".to_string(),
    };
    match remediate {
        ThresholdAction::ExecuteRemediation { action_id } => {
            assert_eq!(action_id, "restart");
        }
        _ => panic!("Expected ExecuteRemediation"),
    }
}

#[test]
fn test_health_check_target_command_function_custom_serde() {
    for target in [
        HealthCheckTarget::Command {
            command: "/bin/true".to_string(),
            args: vec!["-v".to_string()],
        },
        HealthCheckTarget::Function {
            function: "check".to_string(),
        },
        HealthCheckTarget::Custom {
            target: "custom:probe".to_string(),
        },
    ] {
        let json = serde_json::to_string(&target).expect("serialize");
        let _: HealthCheckTarget = serde_json::from_str(&json).expect("deserialize");
    }
}

#[test]
fn test_metric_threshold_full_serde_roundtrip() {
    let mt = MetricThreshold {
        value: 0.92,
        operator: ThresholdOperator::GreaterThan,
        action: ThresholdAction::MarkCritical,
    };
    let json = serde_json::to_string(&mt).expect("serialize");
    let back: MetricThreshold = serde_json::from_str(&json).expect("deserialize");
    assert!((back.value - 0.92).abs() < f64::EPSILON);
    assert!(matches!(back.operator, ThresholdOperator::GreaterThan));
}

#[test]
fn test_health_check_config_serde_roundtrip() {
    let cfg = HealthCheckConfig::default();
    let json = serde_json::to_string(&cfg).expect("serialize");
    let back: HealthCheckConfig = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(cfg.interval_secs, back.interval_secs);
    assert_eq!(cfg.failure_threshold, back.failure_threshold);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use crate::primal::PrimalCapability;
use crate::primal::PrimalType;
use crate::service::scaling::*;
use crate::service::status::*;

#[test]
fn test_debug_implementations() {
    let service = UniversalService::default();
    let debug_str = format!("{service:?}");
    assert!(debug_str.contains("UniversalService"));
    assert!(debug_str.contains("default-service"));

    let phase = ServicePhase::Running;
    assert!(format!("{phase:?}").contains("Running"));

    let endpoint = EndpointProtocol::Https;
    assert!(format!("{endpoint:?}").contains("Https"));
}

#[test]
fn test_service_spec_with_primal_type_serde() {
    let mut spec = UniversalService::default().spec;
    spec.primal_type = Some(PrimalType::new("compute", "toadstool", "1.0.0"));
    spec.capabilities = vec![PrimalCapability::new("compute", "execution", "1.0")];
    let json = serde_json::to_string(&spec).unwrap();
    let deserialized: ServiceSpec = serde_json::from_str(&json).unwrap();
    assert!(deserialized.primal_type.is_some());
    assert_eq!(deserialized.capabilities.len(), 1);
}

#[test]
fn test_service_condition_serde_json_roundtrip() {
    let condition = ServiceCondition {
        condition_type: "Ready".to_string(),
        status: ConditionStatus::True,
        last_transition_time: Utc::now(),
        reason: Some("ServiceStarted".to_string()),
        message: Some("All replicas ready".to_string()),
    };
    let json = serde_json::to_string(&condition).unwrap();
    let deserialized: ServiceCondition = serde_json::from_str(&json).unwrap();
    assert_eq!(condition.condition_type, deserialized.condition_type);
    assert!(matches!(
        (condition.status, deserialized.status),
        (ConditionStatus::True, ConditionStatus::True)
    ));
}

#[test]
fn test_config_file_serde_json_roundtrip() {
    let config_file = ConfigFile {
        path: "/etc/config/app.json".to_string(),
        format: ConfigFormat::Json,
        required: true,
        watch: false,
    };
    let json = serde_json::to_string(&config_file).unwrap();
    let deserialized: ConfigFile = serde_json::from_str(&json).unwrap();
    assert_eq!(config_file.path, deserialized.path);
    assert!(matches!(
        (config_file.format, deserialized.format),
        (ConfigFormat::Json, ConfigFormat::Json)
    ));
}

#[test]
fn test_lifecycle_hook_serde_json_roundtrip() {
    let hook = LifecycleHook {
        name: "pre-start".to_string(),
        command: vec!["/bin/init.sh".to_string()],
        timeout: Some(10),
        on_failure: LifecycleFailureAction::Abort,
    };
    let json = serde_json::to_string(&hook).unwrap();
    let deserialized: LifecycleHook = serde_json::from_str(&json).unwrap();
    assert_eq!(hook.name, deserialized.name);
    assert_eq!(hook.command, deserialized.command);
}

#[test]
fn test_scaling_metric_type_custom_serde() {
    let metric_type = ScalingMetricType::Custom {
        source: "prometheus".to_string(),
        query: "rate(http_requests_total[5m])".to_string(),
    };
    let json = serde_json::to_string(&metric_type).unwrap();
    let deserialized: ScalingMetricType = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&deserialized).unwrap();
    assert_eq!(json, json2);
}

#[test]
fn test_dependency_status_enum_serde() {
    let statuses = [
        DependencyStatus::Satisfied,
        DependencyStatus::Pending,
        DependencyStatus::Failed,
        DependencyStatus::Timeout,
    ];
    for s in statuses {
        let json = serde_json::to_string(&s).unwrap();
        let _: DependencyStatus = serde_json::from_str(&json).unwrap();
    }
}

#[test]
fn test_service_status_serde_json_roundtrip() {
    let status = UniversalService::default().status;
    let json = serde_json::to_string(&status).unwrap();
    let back: ServiceStatus = serde_json::from_str(&json).unwrap();
    assert!(matches!(
        (status.phase, back.phase),
        (ServicePhase::Pending, ServicePhase::Pending)
    ));
    assert_eq!(status.replicas.desired, back.replicas.desired);
}

#[test]
fn test_service_spec_serde_preserves_runtime_and_scaling() {
    let spec = UniversalService::default().spec;
    let json = serde_json::to_string(&spec).unwrap();
    let back: ServiceSpec = serde_json::from_str(&json).unwrap();
    assert_eq!(back.scaling.min_replicas, spec.scaling.min_replicas);
    assert!(matches!(
        (back.service_type, spec.service_type),
        (
            ServiceType::Application { .. },
            ServiceType::Application { .. }
        )
    ));
}

#[test]
fn test_universal_service_debug_includes_name() {
    let s = UniversalService::default();
    let dbg = format!("{s:?}");
    assert!(dbg.contains("default-service"));
}

#[test]
fn test_replica_status_default_like_values_serde() {
    let r = ReplicaStatus {
        desired: 2,
        current: 2,
        ready: 2,
        available: 2,
        unavailable: 0,
    };
    let json = serde_json::to_string(&r).unwrap();
    let back: ReplicaStatus = serde_json::from_str(&json).unwrap();
    assert_eq!(back.ready, 2);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_health_config_defaults() {
    let config = HealthConfig::default();
    assert_eq!(config.check_interval, Duration::from_secs(30));
    assert_eq!(config.timeout, Duration::from_secs(5));
    assert_eq!(config.failure_threshold, 3);
    assert_eq!(config.health_method, "health");
}

#[test]
fn test_resurrection_config_defaults() {
    let config = ResurrectionConfig::default();
    assert!(config.enabled);
    assert_eq!(config.max_attempts, 5);
    assert_eq!(config.base_delay, Duration::from_secs(2));
    assert_eq!(config.max_delay, Duration::from_secs(60));
}

#[test]
fn test_primal_metrics_default() {
    let metrics = PrimalMetrics::default();
    assert_eq!(metrics.total_uptime_secs, 0);
    assert_eq!(metrics.resurrection_count, 0);
    assert_eq!(metrics.health_failures, 0);
    assert_eq!(metrics.last_health_latency_ms, 0);
    assert_eq!(metrics.requests_served, 0);
}

#[test]
fn test_apoptosis_reason_serialization() {
    let reason = ApoptosisReason::DependencyDeath("beardog".to_string());
    let json = serde_json::to_string(&reason).expect("serialize apoptosis reason");
    let deserialized: ApoptosisReason =
        serde_json::from_str(&json).expect("deserialize apoptosis reason");
    assert_eq!(reason, deserialized);
}

#[test]
fn test_all_apoptosis_reasons_serialize() {
    let reasons = vec![
        ApoptosisReason::UserRequest,
        ApoptosisReason::EcosystemHealth,
        ApoptosisReason::ResourcePressure,
        ApoptosisReason::DependencyDeath("songbird".to_string()),
        ApoptosisReason::ResurrectionExhausted,
        ApoptosisReason::SystemShutdown,
    ];
    for reason in reasons {
        let json = serde_json::to_string(&reason).expect("serialize reason");
        let parsed: ApoptosisReason = serde_json::from_str(&json).expect("parse reason");
        assert_eq!(reason, parsed);
    }
}

#[test]
fn test_lifecycle_state_germinating_serialization() {
    let state = LifecycleState::Germinating;
    let json = serde_json::to_string(&state).expect("serialize");
    let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
    assert_eq!(state, parsed);
}

#[test]
fn test_lifecycle_state_active_serialization() {
    let now = chrono::Utc::now();
    let state = LifecycleState::Active {
        since: now,
        last_health_check: now,
    };
    let json = serde_json::to_string(&state).expect("serialize");
    let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
    assert_eq!(state, parsed);
}

#[test]
fn test_lifecycle_state_degraded_serialization() {
    let state = LifecycleState::Degraded {
        since: chrono::Utc::now(),
        reason: "health check failed".to_string(),
        resurrection_attempts: 2,
    };
    let json = serde_json::to_string(&state).expect("serialize");
    let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
    assert_eq!(state, parsed);
}

#[test]
fn test_lifecycle_state_dead_serialization() {
    let state = LifecycleState::Dead {
        since: chrono::Utc::now(),
        reason: "SystemShutdown".to_string(),
    };
    let json = serde_json::to_string(&state).expect("serialize");
    let parsed: LifecycleState = serde_json::from_str(&json).expect("parse");
    assert_eq!(state, parsed);
}

#[test]
fn test_health_config_serialization() {
    let config = HealthConfig::default();
    let json = serde_json::to_string(&config).expect("serialize health config");
    let parsed: HealthConfig = serde_json::from_str(&json).expect("parse health config");
    assert_eq!(parsed.failure_threshold, config.failure_threshold);
    assert_eq!(parsed.health_method, config.health_method);
}

#[test]
fn test_resurrection_config_serialization() {
    let config = ResurrectionConfig {
        enabled: false,
        max_attempts: 10,
        base_delay: Duration::from_secs(5),
        max_delay: Duration::from_secs(120),
    };
    let json = serde_json::to_string(&config).expect("serialize");
    let parsed: ResurrectionConfig = serde_json::from_str(&json).expect("parse");
    assert!(!parsed.enabled);
    assert_eq!(parsed.max_attempts, 10);
}

#[test]
fn test_primal_metrics_serialization() {
    let metrics = PrimalMetrics {
        total_uptime_secs: 3600,
        resurrection_count: 2,
        health_failures: 5,
        last_health_latency_ms: 12,
        requests_served: 1000,
        last_resurrection_at: Some(chrono::Utc::now()),
    };
    let json = serde_json::to_string(&metrics).expect("serialize");
    let parsed: PrimalMetrics = serde_json::from_str(&json).expect("parse");
    assert_eq!(parsed.total_uptime_secs, 3600);
    assert_eq!(parsed.resurrection_count, 2);
    assert_eq!(parsed.requests_served, 1000);
}

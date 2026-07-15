// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_health_event_trigger_variants() {
    let _ = HealthEventTrigger::ScheduledCheck;
    let _ = HealthEventTrigger::ManualCheck;
    let _ = HealthEventTrigger::MetricThreshold {
        metric: "cpu".to_string(),
        threshold: 0.9,
    };
    let _ = HealthEventTrigger::ExternalEvent {
        source: "monitor".to_string(),
    };
    let _ = HealthEventTrigger::Startup;
    let _ = HealthEventTrigger::Shutdown;
    let _ = HealthEventTrigger::ConfigurationChange;
    let _ = HealthEventTrigger::Custom {
        trigger: "custom".to_string(),
    };
}

#[test]
fn test_health_event_trigger_serialization() {
    let trigger = HealthEventTrigger::MetricThreshold {
        metric: "memory_usage".to_string(),
        threshold: 0.85,
    };
    let json = serde_json::to_string(&trigger).expect("serialize");
    assert!(json.contains("memory_usage"));
    let parsed: HealthEventTrigger = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed, HealthEventTrigger::MetricThreshold { .. }));
}

#[test]
fn test_health_event_creation() {
    let event = HealthEvent {
        id: Uuid::new_v4(),
        timestamp: Utc::now(),
        previous_health: Some(Health::healthy()),
        new_health: Health::degraded(vec![]),
        trigger: HealthEventTrigger::MetricThreshold {
            metric: "cpu".to_string(),
            threshold: 0.9,
        },
        context: HashMap::new(),
    };
    assert!(event.previous_health.is_some());
    assert!(event.new_health.is_operational());
}

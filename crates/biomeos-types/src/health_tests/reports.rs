// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

#[test]
fn test_health_report_creation() {
    let subject = HealthSubject {
        id: "primal-1".to_string(),
        subject_type: HealthSubjectType::Primal,
        name: "Test Primal".to_string(),
        version: "1.0.0".to_string(),
    };

    let report = HealthReport {
        id: Uuid::new_v4(),
        subject,
        health: Health::healthy(),
        components: HashMap::new(),
        metrics: HealthMetrics {
            response_time: None,
            resources: None,
            errors: None,
            availability: None,
            custom: HashMap::new(),
        },
        history: vec![],
        generated_at: Utc::now(),
        next_check_at: None,
    };

    assert!(report.health.is_healthy());
    assert!(report.components.is_empty());
}

#[test]
fn test_health_subject_type_variants() {
    let _ = HealthSubjectType::Primal;
    let _ = HealthSubjectType::Service;
    let _ = HealthSubjectType::System;
    let _ = HealthSubjectType::Component;
    let _ = HealthSubjectType::Custom {
        subject_type: "custom".to_string(),
    };
}

#[test]
fn test_health_subject_type_serialization() {
    let subject_type = HealthSubjectType::Custom {
        subject_type: "custom_primal".to_string(),
    };
    let json = serde_json::to_string(&subject_type).expect("serialize");
    let parsed: HealthSubjectType = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(parsed, HealthSubjectType::Custom { .. }));
}

#[test]
fn test_component_health_creation() {
    let comp = ComponentHealth {
        name: "api".to_string(),
        health: Health::healthy(),
        metrics: HashMap::new(),
        last_check: Utc::now(),
    };
    assert_eq!(comp.name, "api");
    assert!(comp.health.is_healthy());
}

#[test]
fn test_health_report_serialization() {
    let report = HealthReport {
        id: Uuid::new_v4(),
        subject: HealthSubject {
            id: "sys-1".to_string(),
            subject_type: HealthSubjectType::System,
            name: "System".to_string(),
            version: "1.0".to_string(),
        },
        health: Health::healthy(),
        components: HashMap::new(),
        metrics: HealthMetrics {
            response_time: None,
            resources: None,
            errors: None,
            availability: None,
            custom: HashMap::new(),
        },
        history: vec![],
        generated_at: Utc::now(),
        next_check_at: None,
    };
    let json = serde_json::to_string(&report).expect("serialize");
    let parsed: HealthReport = serde_json::from_str(&json).expect("deserialize");
    assert!(parsed.health.is_healthy());
}

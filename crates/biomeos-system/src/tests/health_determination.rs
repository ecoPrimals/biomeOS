// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::{Health, ResourceMetrics, SystemInspector};

#[test]
fn test_determine_health_from_metrics_healthy() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Healthy));
}

#[test]
fn test_determine_health_from_metrics_critical() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.96),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

#[test]
fn test_determine_health_from_metrics_degraded() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.85),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_determine_health_from_metrics_critical_memory() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.96),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

#[test]
fn test_determine_health_from_metrics_critical_disk() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.5),
        disk_usage: Some(0.96),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

#[test]
fn test_determine_health_from_metrics_degraded_disk() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.5),
        memory_usage: Some(0.5),
        disk_usage: Some(0.85),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_determine_health_from_metrics_none_fields_treated_as_zero() {
    let metrics = ResourceMetrics {
        cpu_usage: None,
        memory_usage: None,
        disk_usage: None,
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Healthy));
}

#[test]
fn test_determine_health_from_metrics_degraded_memory_only() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.1),
        memory_usage: Some(0.85),
        disk_usage: Some(0.1),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_determine_health_from_metrics_degraded_cpu_and_memory() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.75),
        memory_usage: Some(0.85),
        disk_usage: Some(0.1),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Degraded { .. }));
}

#[test]
fn test_determine_health_not_critical_when_usage_exactly_at_95_percent() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.95),
        memory_usage: Some(0.5),
        disk_usage: Some(0.5),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(
        matches!(health, Health::Degraded { .. }),
        "strictly above 0.95 is required for Critical"
    );
}

#[test]
fn test_determine_health_critical_requires_strictly_above_95() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.950_000_000_1),
        memory_usage: Some(0.1),
        disk_usage: Some(0.1),
        network_io: None,
    };
    let health = SystemInspector::determine_health_from_metrics(&metrics);
    assert!(matches!(health, Health::Critical { .. }));
}

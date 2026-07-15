// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::collections::HashMap;

#[test]
fn test_response_time_metrics() {
    let metrics = ResponseTimeMetrics {
        average_ms: 50.0,
        p50_ms: 45.0,
        p95_ms: 120.0,
        p99_ms: 200.0,
        max_ms: 350.0,
    };
    assert!((metrics.average_ms - 50.0).abs() < f64::EPSILON);
    assert!((metrics.p95_ms - 120.0).abs() < f64::EPSILON);
}

#[test]
fn test_resource_metrics() {
    let metrics = ResourceMetrics {
        cpu_usage: Some(0.65),
        memory_usage: Some(0.42),
        disk_usage: Some(0.30),
        network_io: Some(NetworkIoMetrics {
            bytes_in_per_sec: 1000.0,
            bytes_out_per_sec: 500.0,
            packets_in_per_sec: 10.0,
            packets_out_per_sec: 5.0,
        }),
    };
    assert!(
        metrics
            .cpu_usage
            .is_some_and(|v| (v - 0.65).abs() < f64::EPSILON)
    );
    assert!(metrics.network_io.is_some());
}

#[test]
fn test_network_io_metrics() {
    let metrics = NetworkIoMetrics {
        bytes_in_per_sec: 1024.0,
        bytes_out_per_sec: 512.0,
        packets_in_per_sec: 100.0,
        packets_out_per_sec: 50.0,
    };
    assert!((metrics.bytes_in_per_sec - 1024.0).abs() < f64::EPSILON);
}

#[test]
fn test_error_metrics() {
    let mut by_cat = HashMap::new();
    by_cat.insert("timeout".to_string(), 0.1);
    by_cat.insert("connection".to_string(), 0.05);

    let metrics = ErrorMetrics {
        error_rate: 0.02,
        errors_by_category: by_cat,
        recent_errors: 5,
    };
    assert!((metrics.error_rate - 0.02).abs() < f64::EPSILON);
    assert_eq!(metrics.recent_errors, 5);
}

#[test]
fn test_availability_metrics() {
    let metrics = AvailabilityMetrics {
        uptime_percentage: 0.999,
        uptime_seconds: 86400,
        downtime_seconds: 86,
        outage_count: 2,
        mttr_seconds: Some(43.0),
    };
    assert!((metrics.uptime_percentage - 0.999).abs() < f64::EPSILON);
    assert_eq!(metrics.outage_count, 2);
}

#[test]
fn test_health_metrics_serialization() {
    let metrics = HealthMetrics {
        response_time: Some(ResponseTimeMetrics {
            average_ms: 25.0,
            p50_ms: 20.0,
            p95_ms: 80.0,
            p99_ms: 150.0,
            max_ms: 200.0,
        }),
        resources: None,
        errors: None,
        availability: None,
        custom: HashMap::new(),
    };
    let json = serde_json::to_string(&metrics).expect("serialize");
    let parsed: HealthMetrics = serde_json::from_str(&json).expect("deserialize");
    assert!(parsed.response_time.is_some());
}

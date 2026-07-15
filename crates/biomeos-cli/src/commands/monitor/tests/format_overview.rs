// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::format::*;

#[test]
fn test_format_system_overview_empty() {
    let system = serde_json::json!({});
    let lines = format_system_overview(&system);
    assert!(lines.is_empty());
}

#[test]
fn test_format_system_overview_with_cpu() {
    let system = serde_json::json!({"cpu_usage_percent": 42});
    let lines = format_system_overview(&system);
    assert_eq!(lines.len(), 1);
    assert!(lines[0].contains("42"));
}

#[test]
fn test_format_system_overview_with_memory() {
    let system = serde_json::json!({
        "memory": {
            "used_gb": 2.5,
            "total_gb": 8.0,
            "usage_percent": 31.25
        }
    });
    let lines = format_system_overview(&system);
    assert!(lines.iter().any(|l| l.contains("Memory")));
    assert!(lines.iter().any(|l| l.contains("2.5") && l.contains("8.0")));
}

#[test]
fn test_format_service_rows_empty() {
    let services = serde_json::json!({});
    let lines = format_service_rows(&services);
    assert!(lines.is_empty());
}

#[test]
fn test_format_service_rows_with_services() {
    let services = serde_json::json!({
        "svc1": {"status": "running", "health": "Healthy"},
        "svc2": {"status": "stopped", "health": "unknown"}
    });
    let lines = format_service_rows(&services);
    assert!(lines.iter().any(|l| l.contains("Service Status")));
    assert!(
        lines
            .iter()
            .any(|l| l.contains("svc1") && l.contains("running"))
    );
}

#[test]
fn test_format_network_activity() {
    let network = serde_json::json!({
        "bytes_in_per_sec": 1024,
        "bytes_out_per_sec": 2048,
        "active_connections": 5
    });
    let lines = format_network_activity(&network);
    assert!(lines.iter().any(|l| l.contains("In")));
    assert!(lines.iter().any(|l| l.contains("Out")));
    assert!(lines.iter().any(|l| l.contains('5')));
}

#[test]
fn test_format_alert_rows_empty() {
    let alerts = serde_json::json!([]);
    let lines = format_alert_rows(&alerts);
    assert!(lines.is_empty());
}

#[test]
fn test_format_alert_rows_with_alerts() {
    let alerts = serde_json::json!([
        {"message": "High CPU", "severity": "warning"},
        {"message": "Disk full", "severity": "critical"}
    ]);
    let lines = format_alert_rows(&alerts);
    assert!(lines.iter().any(|l| l.contains("Active Alerts")));
    assert!(lines.iter().any(|l| l.contains("High CPU")));
}

#[test]
fn test_format_log_entry() {
    let entry = serde_json::json!({
        "timestamp": "2025-01-01T12:00:00Z",
        "level": "info",
        "message": "Hello world"
    });
    let s = format_log_entry(&entry);
    assert!(s.contains("INFO"));
    assert!(s.contains("Hello world"));
    assert!(s.contains("2025-01-01"));
}

#[test]
fn test_format_log_entry_missing_fields() {
    let entry = serde_json::json!({});
    let s = format_log_entry(&entry);
    assert!(s.contains("unknown"));
    assert!(s.contains("INFO"));
}

#[test]
fn test_format_service_rows_with_resources() {
    let services = serde_json::json!({
        "svc1": {
            "status": "running",
            "health": "Healthy",
            "resources": {"cpu_percent": 50, "memory_mb": 256}
        }
    });
    let lines = format_service_rows(&services);
    assert!(lines.iter().any(|l| l.contains("50")));
    assert!(lines.iter().any(|l| l.contains("256")));
}

#[test]
fn test_format_service_rows_empty_object() {
    let services = serde_json::json!({});
    let lines = format_service_rows(&services);
    assert!(lines.is_empty());
}

#[test]
fn test_format_system_overview_with_load() {
    let system = serde_json::json!({
        "load_average": {"1m": 1.5, "5m": 1.2, "15m": 1.0}
    });
    let lines = format_system_overview(&system);
    assert!(lines.iter().any(|l| l.contains("1.5")));
}

#[test]
fn test_format_system_overview_with_disk() {
    let system = serde_json::json!({
        "disk": {"usage_percent": 75}
    });
    let lines = format_system_overview(&system);
    assert!(lines.iter().any(|l| l.contains("75")));
}

#[test]
fn test_format_network_activity_partial() {
    let network = serde_json::json!({"bytes_in_per_sec": 1024});
    let lines = format_network_activity(&network);
    assert!(lines.is_empty());
}

#[test]
fn test_format_alert_rows_severity_info() {
    let alerts = serde_json::json!([
        {"message": "Info msg", "severity": "info"}
    ]);
    let lines = format_alert_rows(&alerts);
    assert!(lines.iter().any(|l| l.contains("Info msg")));
}

#[test]
fn test_format_log_entry_all_levels() {
    for level in ["error", "warn", "info", "debug", "trace"] {
        let entry = serde_json::json!({
            "timestamp": "2025-01-01T12:00:00Z",
            "level": level,
            "message": "test"
        });
        let s = format_log_entry(&entry);
        assert!(!s.is_empty());
    }
}

#[test]
fn test_format_system_overview_memory_without_usage_percent() {
    let system = serde_json::json!({
        "memory": {
            "used_gb": 1.0,
            "total_gb": 4.0
        }
    });
    let lines = format_system_overview(&system);
    assert!(lines.iter().any(|l| l.contains("Memory")));
}

#[test]
fn test_format_system_overview_memory_incomplete() {
    let system = serde_json::json!({
        "memory": { "used_gb": 1.0 }
    });
    let lines = format_system_overview(&system);
    assert!(lines.is_empty());
}

#[test]
fn test_format_service_rows_not_object() {
    let services = serde_json::json!([]);
    let lines = format_service_rows(&services);
    assert!(lines.is_empty());
}

#[test]
fn test_format_service_rows_cpu_without_memory() {
    let services = serde_json::json!({
        "svc": {
            "status": "running",
            "health": "Healthy",
            "resources": { "cpu_percent": 10 }
        }
    });
    let lines = format_service_rows(&services);
    assert!(lines.iter().any(|l| l.contains("svc")));
}

#[test]
fn test_format_network_activity_connections_only() {
    let network = serde_json::json!({ "active_connections": 3 });
    let lines = format_network_activity(&network);
    assert!(lines.iter().any(|l| l.contains("connections")));
}

#[test]
fn test_format_alert_rows_no_message_skipped() {
    let alerts = serde_json::json!([{ "severity": "info" }]);
    let lines = format_alert_rows(&alerts);
    assert!(lines.iter().any(|l| l.contains("Active Alerts")));
    assert!(!lines.iter().any(|l| l.contains('🔵')));
}

#[test]
fn test_format_system_overview_cpu_and_disk_only() {
    let system = serde_json::json!({
        "cpu_usage_percent": 3,
        "disk": { "usage_percent": 44 }
    });
    let lines = format_system_overview(&system);
    assert!(lines.iter().any(|l| l.contains("CPU")));
    assert!(lines.iter().any(|l| l.contains("Disk")));
}

#[test]
fn test_format_network_activity_bytes_out_only() {
    let network = serde_json::json!({ "bytes_out_per_sec": 100 });
    assert!(format_network_activity(&network).is_empty());
}

#[test]
fn test_format_network_activity_bytes_in_only() {
    let network = serde_json::json!({ "bytes_in_per_sec": 200 });
    assert!(format_network_activity(&network).is_empty());
}

#[test]
fn test_format_alert_rows_message_without_severity_uses_default() {
    let alerts = serde_json::json!([{ "message": "hello" }]);
    let lines = format_alert_rows(&alerts);
    assert!(lines.iter().any(|l| l.contains("hello")));
}

#[test]
fn test_format_log_entry_custom_level_icon() {
    let entry = serde_json::json!({
        "timestamp": "t",
        "level": "custom",
        "message": "m"
    });
    let s = format_log_entry(&entry);
    assert!(s.contains("CUSTOM"));
}

#[test]
fn test_format_system_overview_cpu_memory_disk_load_combined() {
    let system = serde_json::json!({
        "cpu_usage_percent": 7,
        "memory": { "used_gb": 1.0, "total_gb": 4.0, "usage_percent": 25.0 },
        "disk": { "usage_percent": 50 },
        "load_average": { "1m": 0.42 }
    });
    let lines = format_system_overview(&system);
    assert_eq!(lines.len(), 4);
    assert!(lines.iter().any(|l| l.contains("CPU")));
    assert!(lines.iter().any(|l| l.contains("Memory")));
    assert!(lines.iter().any(|l| l.contains("Disk")));
    assert!(lines.iter().any(|l| l.contains("Load")));
}

#[test]
fn test_format_service_rows_default_status_health_unknown() {
    let services = serde_json::json!({
        "bare": {}
    });
    let lines = format_service_rows(&services);
    assert!(lines.iter().any(|l| l.contains("unknown")));
}

#[test]
fn test_format_alert_rows_empty_message_entries() {
    let alerts = serde_json::json!([
        { "message": "ok", "severity": "warning" },
        { "severity": "info" }
    ]);
    let lines = format_alert_rows(&alerts);
    assert!(lines.iter().any(|l| l.contains("ok")));
}

#[test]
fn test_format_network_activity_zero_bytes() {
    let network = serde_json::json!({
        "bytes_in_per_sec": 0,
        "bytes_out_per_sec": 0,
        "active_connections": 0
    });
    let lines = format_network_activity(&network);
    assert!(lines.iter().any(|l| l.contains('0')));
}

#[test]
fn test_format_log_entry_empty_message() {
    let entry = serde_json::json!({
        "timestamp": "t",
        "level": "info",
        "message": null
    });
    let s = format_log_entry(&entry);
    assert!(s.ends_with(": "));
}

#[test]
fn test_format_alert_rows_non_array() {
    let alerts = serde_json::json!({"not": "array"});
    assert!(format_alert_rows(&alerts).is_empty());
}

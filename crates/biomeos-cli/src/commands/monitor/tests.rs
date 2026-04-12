// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]

use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

use super::format::*;
use crate::commands::utils::{format_bytes, format_duration};

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
fn test_format_exec_output() {
    let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
        "exit_code": 0,
        "stdout": "hello\nworld",
        "duration_ms": 10
    }))
    .unwrap();
    let lines = format_exec_output(&results);
    assert!(lines.iter().any(|l| l.contains("✅")));
    assert!(lines.iter().any(|l| l.contains("STDOUT")));
    assert!(lines.iter().any(|l| l.contains("10ms")));
}

#[test]
fn test_format_scale_output() {
    let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
        "status": "success",
        "current_replicas": 3,
        "target_replicas": 3
    }))
    .unwrap();
    let lines = format_scale_output(&results, false);
    assert!(lines.iter().any(|l| l.contains("success")));
    assert!(lines.iter().any(|l| l.contains("Current replicas")));
}

#[test]
fn test_should_stop_monitoring_no_duration() {
    assert!(!should_stop_monitoring(Duration::from_secs(100), None));
}

#[test]
fn test_should_stop_monitoring_not_reached() {
    assert!(!should_stop_monitoring(
        Duration::from_secs(50),
        Some(Duration::from_secs(100))
    ));
}

#[test]
fn test_should_stop_monitoring_reached() {
    assert!(should_stop_monitoring(
        Duration::from_secs(100),
        Some(Duration::from_secs(100))
    ));
}

#[test]
fn test_should_stop_monitoring_exceeded() {
    assert!(should_stop_monitoring(
        Duration::from_secs(150),
        Some(Duration::from_secs(100))
    ));
}

#[test]
fn test_service_status_icon_all_variants() {
    assert_eq!(service_status_icon("running"), "✅");
    assert_eq!(service_status_icon("starting"), "🔄");
    assert_eq!(service_status_icon("stopping"), "⏹️");
    assert_eq!(service_status_icon("stopped"), "⏸️");
    assert_eq!(service_status_icon("error"), "❌");
    assert_eq!(service_status_icon("unknown"), "❓");
    assert_eq!(service_status_icon(""), "❓");
}

#[test]
fn test_service_health_icon_all_variants() {
    assert_eq!(service_health_icon("Healthy"), "💚");
    assert_eq!(service_health_icon("Degraded"), "💛");
    assert_eq!(service_health_icon("Critical"), "🧡");
    assert_eq!(service_health_icon("Unhealthy"), "❤️");
    assert_eq!(service_health_icon("unknown"), "🤍");
    assert_eq!(service_health_icon(""), "🤍");
}

#[test]
fn test_alert_severity_icon_all_variants() {
    assert_eq!(alert_severity_icon("critical"), "🔴");
    assert_eq!(alert_severity_icon("warning"), "🟡");
    assert_eq!(alert_severity_icon("info"), "🔵");
    assert_eq!(alert_severity_icon("unknown"), "⚪");
    assert_eq!(alert_severity_icon(""), "⚪");
}

#[test]
fn test_log_level_icon_all_variants() {
    assert_eq!(log_level_icon("error"), "❌");
    assert_eq!(log_level_icon("warn"), "⚠️");
    assert_eq!(log_level_icon("info"), "ℹ️");
    assert_eq!(log_level_icon("debug"), "🐛");
    assert_eq!(log_level_icon("trace"), "🔍");
    assert_eq!(log_level_icon("unknown"), "📝");
    assert_eq!(log_level_icon(""), "📝");
}

#[test]
fn test_scale_status_icon_all_variants() {
    assert_eq!(scale_status_icon("success"), "✅");
    assert_eq!(scale_status_icon("in_progress"), "🔄");
    assert_eq!(scale_status_icon("failed"), "❌");
    assert_eq!(scale_status_icon("unknown"), "🔹");
    assert_eq!(scale_status_icon(""), "🔹");
}

#[test]
fn test_format_bytes_via_utils() {
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
}

#[test]
fn test_format_duration_via_utils() {
    assert_eq!(format_duration(std::time::Duration::from_secs(45)), "45s");
    assert_eq!(
        format_duration(std::time::Duration::from_secs(125)),
        "2m 5s"
    );
}

#[test]
fn test_format_exec_output_exit_code_failure() {
    let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
        "exit_code": 1,
        "stderr": "error message"
    }))
    .unwrap();
    let lines = format_exec_output(&results);
    assert!(lines.iter().any(|l| l.contains("❌")));
    assert!(lines.iter().any(|l| l.contains("STDERR")));
}

#[test]
fn test_format_exec_output_empty_stdout_stderr() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({"exit_code": 0})).unwrap();
    let lines = format_exec_output(&results);
    assert!(lines.iter().any(|l| l.contains("✅")));
}

#[test]
fn test_format_scale_output_with_auto() {
    let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
        "status": "success",
        "current_replicas": 2,
        "target_replicas": 2,
        "auto_scaling": {
            "min_replicas": 1,
            "max_replicas": 5,
            "cpu_threshold_percent": 80
        }
    }))
    .unwrap();
    let lines = format_scale_output(&results, true);
    assert!(lines.iter().any(|l| l.contains("Auto-scaling")));
    assert!(lines.iter().any(|l| l.contains("Min replicas")));
    assert!(lines.iter().any(|l| l.contains("Max replicas")));
    assert!(lines.iter().any(|l| l.contains("CPU threshold")));
}

#[test]
fn test_format_scale_output_with_message() {
    let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
        "status": "in_progress",
        "message": "Scaling in progress"
    }))
    .unwrap();
    let lines = format_scale_output(&results, false);
    assert!(lines.iter().any(|l| l.contains("Scaling in progress")));
}

#[test]
fn test_format_scale_output_status_unknown() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({"status": "unknown"})).unwrap();
    let lines = format_scale_output(&results, false);
    assert!(!lines.is_empty());
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
fn test_format_exec_output_stderr_only() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({"stderr": "oops"})).unwrap();
    let lines = format_exec_output(&results);
    assert!(lines.iter().any(|l| l.contains("STDERR")));
}

#[test]
fn test_format_exec_output_whitespace_stdout_skipped() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({"stdout": "   \n  "})).unwrap();
    let lines = format_exec_output(&results);
    assert!(!lines.iter().any(|l| l.contains("STDOUT")));
}

#[test]
fn test_format_scale_output_auto_false_skips_auto_block() {
    let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
        "status": "success",
        "auto_scaling": { "min_replicas": 1 }
    }))
    .unwrap();
    let lines = format_scale_output(&results, false);
    assert!(
        !lines
            .iter()
            .any(|l| l.contains("Auto-scaling configuration"))
    );
}

#[test]
fn test_format_scale_output_auto_partial_fields() {
    let results: HashMap<String, Value> = serde_json::from_value(serde_json::json!({
        "status": "success",
        "auto_scaling": { "min_replicas": 2 }
    }))
    .unwrap();
    let lines = format_scale_output(&results, true);
    assert!(lines.iter().any(|l| l.contains("Min replicas")));
}

#[tokio::test]
async fn test_handle_dashboard_deprecated_message() {
    let result = super::handlers::handle_dashboard(5, false).await;
    assert!(result.is_ok());
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
fn test_format_exec_output_duration_without_exit() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({ "duration_ms": 99 })).unwrap();
    let lines = format_exec_output(&results);
    assert!(lines.iter().any(|l| l.contains("99ms")));
}

#[test]
fn test_format_scale_output_auto_true_without_auto_scaling_block() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({ "status": "success" })).unwrap();
    let lines = format_scale_output(&results, true);
    assert!(
        !lines
            .iter()
            .any(|l| l.contains("Auto-scaling configuration"))
    );
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
fn test_format_exec_output_stdout_only_no_exit() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({ "stdout": "only out" })).unwrap();
    let lines = format_exec_output(&results);
    assert!(lines.iter().any(|l| l.contains("STDOUT")));
    assert!(!lines.iter().any(|l| l.contains("Exit code")));
}

#[test]
fn test_format_scale_output_only_status() {
    let results: HashMap<String, Value> =
        serde_json::from_value(serde_json::json!({ "status": "failed" })).unwrap();
    let lines = format_scale_output(&results, false);
    assert!(lines.iter().any(|l| l.contains("failed")));
    assert!(lines.iter().any(|l| l.contains('❌')));
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

#[tokio::test]
async fn test_handle_monitor_duration_zero_single_iteration() {
    let result = super::handle_monitor(None, 0, Some(0)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_scale_requires_replicas_or_auto() {
    let result = super::handle_scale("any-service".into(), None, false).await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("Must specify") || (msg.contains("replicas") && msg.contains("auto")),
        "unexpected: {msg}"
    );
}

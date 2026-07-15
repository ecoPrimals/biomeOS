// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_format_health_summary_empty() {
    let results = HashMap::new();
    let lines = format_health_summary(&results, false);
    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0], "");
}

#[test]
fn test_format_health_summary_overall_status() {
    let mut results = HashMap::new();
    results.insert("overall_status".to_string(), serde_json::json!("Healthy"));
    let lines = format_health_summary(&results, false);
    assert!(lines[0].contains("✅"));
    assert!(lines[0].contains("Healthy"));
}

#[test]
fn test_format_health_summary_with_services() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc1": {"status": "Healthy"},
            "svc2": {"status": "Degraded"}
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(lines.iter().any(|l| l.contains("Service Health")));
    assert!(
        lines
            .iter()
            .any(|l| l.contains("svc1") && l.contains("Healthy"))
    );
    assert!(
        lines
            .iter()
            .any(|l| l.contains("svc2") && l.contains("Degraded"))
    );
}

#[test]
fn test_format_health_summary_with_services_detailed() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc1": {
                "status": "Healthy",
                "issues": [{"message": "Minor issue"}],
                "metrics": {"cpu_usage": 50}
            }
        }),
    );
    let lines = format_health_summary(&results, true);
    assert!(lines.iter().any(|l| l.contains("Minor issue")));
    assert!(lines.iter().any(|l| l.contains("50")));
}

#[test]
fn test_format_health_summary_with_system_metrics() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "cpu_usage": 25,
            "memory_usage": {"used_bytes": 1_073_741_824_i64, "total_bytes": 4_294_967_296_i64},
            "disk_usage": {"used_bytes": 5_368_709_120_i64},
            "network": {"bytes_sent": 1000, "bytes_received": 2000}
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(lines.iter().any(|l| l.contains("System Metrics")));
}

#[test]
fn test_format_health_summary_overall_status_unknown() {
    let mut results = HashMap::new();
    results.insert("overall_status".to_string(), serde_json::json!(42));
    let lines = format_health_summary(&results, false);
    assert!(lines[0].contains("Unknown") || lines[0].contains("🔹"));
}

#[test]
fn test_format_health_summary_system_metrics_only() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "cpu_usage": 50,
            "memory_usage": {"used_bytes": 1_073_741_824_u64, "total_bytes": 2_147_483_648_u64},
            "disk_usage": {"used_bytes": 5_368_709_120_u64},
            "network": {"bytes_sent": 1000, "bytes_received": 2000}
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(lines.iter().any(|l| l.contains("System Metrics")));
}

#[test]
fn test_format_health_summary_services_empty_object() {
    let mut results = HashMap::new();
    results.insert("services".to_string(), serde_json::json!({}));
    let lines = format_health_summary(&results, false);
    assert!(lines.iter().any(|l| l.contains("Service Health")));
}

#[test]
fn test_format_health_summary_services_unknown_status() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc": { "status": null }
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(lines.iter().any(|l| l.contains("svc")));
}

#[test]
fn test_format_health_summary_detailed_metrics_nested() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc": {
                "status": "Healthy",
                "metrics": {
                    "cpu_usage": 10,
                    "memory_usage": { "used_bytes": 100, "total_bytes": 0 },
                    "disk_usage": { "used_bytes": 500 },
                    "network": { "bytes_sent": 1, "bytes_received": 2 }
                }
            }
        }),
    );
    let lines = format_health_summary(&results, true);
    assert!(lines.iter().any(|l| l.contains("CPU Usage")));
    assert!(lines.iter().any(|l| l.contains("Disk Usage")));
}

#[test]
fn test_format_health_summary_overall_non_string() {
    let mut results = HashMap::new();
    results.insert("overall_status".to_string(), serde_json::json!([]));
    let lines = format_health_summary(&results, false);
    assert!(!lines.is_empty());
}

#[test]
fn test_format_health_summary_service_issues_non_array() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc": { "status": "Healthy", "issues": "not-array" }
        }),
    );
    let lines = format_health_summary(&results, true);
    assert!(lines.iter().any(|l| l.contains("svc")));
}

#[test]
fn test_format_health_metrics_memory_partial_no_total() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "memory_usage": { "used_bytes": 100 }
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(
        !lines
            .iter()
            .any(|l| l.contains("Memory:") && l.contains("GB"))
    );
}

#[test]
fn test_format_health_metrics_network_partial() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "network": { "bytes_sent": 10 }
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(!lines.iter().any(|l| l.contains('↑') && l.contains('↓')));
}

#[test]
fn test_format_health_summary_services_detailed_issue_without_message() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc": {
                "status": "Healthy",
                "issues": [{ "code": 1 }]
            }
        }),
    );
    let lines = format_health_summary(&results, true);
    assert!(lines.iter().any(|l| l.contains("svc")));
}

#[test]
fn test_format_health_metrics_disk_without_used_bytes() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "disk_usage": { "total_bytes": 100 }
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(!lines.iter().any(|l| l.contains("Disk Usage")));
}

#[test]
fn test_format_health_metrics_memory_used_without_total() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "memory_usage": { "total_bytes": 100 }
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(
        !lines
            .iter()
            .any(|l| l.contains("Memory:") && l.contains("GB"))
    );
}

#[test]
fn test_format_health_summary_detailed_service_metrics_only_cpu() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc": { "status": "Healthy", "metrics": { "cpu_usage": 7 } }
        }),
    );
    let lines = format_health_summary(&results, true);
    assert!(
        lines
            .iter()
            .any(|l| l.contains("CPU Usage") && l.contains('7'))
    );
}

#[test]
fn test_format_health_summary_non_object_services_skipped() {
    let mut results = HashMap::new();
    results.insert("services".to_string(), serde_json::json!("not-an-object"));
    let lines = format_health_summary(&results, false);
    assert!(!lines.iter().any(|l| l.contains("Service Health")));
}

#[test]
fn test_display_health_results_runs() {
    let mut results = HashMap::new();
    results.insert("overall_status".to_string(), serde_json::json!("Healthy"));
    display_health_results(&results, false);
}

#[test]
fn test_format_health_summary_services_not_array_skipped() {
    let mut results = HashMap::new();
    results.insert("services".to_string(), serde_json::json!([]));
    let lines = format_health_summary(&results, true);
    assert!(!lines.iter().any(|l| l.contains("Service Health")));
}

#[test]
fn test_format_health_metrics_system_network_only_received() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "network": { "bytes_received": 100 }
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(!lines.iter().any(|l| l.contains('↑')));
}

#[test]
fn test_format_health_metrics_system_only_disk_used() {
    let mut results = HashMap::new();
    results.insert(
        "system_metrics".to_string(),
        serde_json::json!({
            "disk_usage": { "used_bytes": 1024 }
        }),
    );
    let lines = format_health_summary(&results, false);
    assert!(lines.iter().any(|l| l.contains("Disk Usage")));
}

#[test]
fn test_format_health_summary_detailed_issue_with_empty_message_object() {
    let mut results = HashMap::new();
    results.insert(
        "services".to_string(),
        serde_json::json!({
            "svc": { "status": "Healthy", "issues": [{}] }
        }),
    );
    let lines = format_health_summary(&results, true);
    assert!(lines.iter().any(|l| l.contains("svc")));
}

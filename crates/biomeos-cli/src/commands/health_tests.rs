// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use std::path::PathBuf;

#[test]
fn test_status_to_icon_all_variants() {
    assert_eq!(status_to_icon("Healthy"), "✅");
    assert_eq!(status_to_icon("Degraded"), "⚠️");
    assert_eq!(status_to_icon("Critical"), "🔴");
    assert_eq!(status_to_icon("Unhealthy"), "❌");
    assert_eq!(status_to_icon("Starting"), "🔄");
    assert_eq!(status_to_icon("Stopping"), "⏹️");
    assert_eq!(status_to_icon("Maintenance"), "🔧");
    assert_eq!(status_to_icon("Unknown"), "❓");
    assert_eq!(status_to_icon("custom"), "🔹");
    assert_eq!(status_to_icon(""), "🔹");
}

#[test]
fn test_compute_memory_percent() {
    assert!((compute_memory_percent(0, 0) - 0.0).abs() < f64::EPSILON);
    assert!((compute_memory_percent(512, 1024) - 50.0).abs() < f64::EPSILON);
    assert!((compute_memory_percent(256, 1024) - 25.0).abs() < f64::EPSILON);
    assert!((compute_memory_percent(1024, 1024) - 100.0).abs() < f64::EPSILON);
    let p = compute_memory_percent(1, 3);
    assert!((p - 33.333).abs() < 0.001, "expected ~33.333, got {p}");
}

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
fn test_format_probe_results_empty() {
    let results = HashMap::new();
    let lines = format_probe_results("mysvc", &results);
    assert_eq!(lines[0], "🔍 Deep probe results for 'mysvc':");
}

#[test]
fn test_format_probe_results_with_connectivity() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({
            "reachable": true,
            "response_time_ms": 42
        }),
    );
    let lines = format_probe_results("mysvc", &results);
    assert!(lines.iter().any(|l| l.contains("Connectivity")));
    assert!(lines.iter().any(|l| l.contains("Reachable")));
    assert!(lines.iter().any(|l| l.contains("42ms")));
}

#[test]
fn test_format_scan_results_json() {
    let mut results = HashMap::new();
    results.insert("key".to_string(), serde_json::json!("value"));
    let output = format_scan_results(&results, "json").unwrap();
    assert!(output.contains("\"key\""));
    assert!(output.contains("value"));
}

#[test]
fn test_format_scan_results_summary() {
    let mut results = HashMap::new();
    results.insert("issues_count".to_string(), serde_json::json!(5));
    results.insert("services_scanned".to_string(), serde_json::json!(10));
    let output = format_scan_results(&results, "summary").unwrap();
    assert!(output.contains("System Scan Summary"));
    assert!(output.contains("Issues found"));
    assert!(output.contains('5'));
    assert!(output.contains("Services scanned"));
    assert!(output.contains("10"));
}

#[test]
fn test_format_scan_results_default() {
    let mut results = HashMap::new();
    results.insert("status".to_string(), serde_json::json!("ok"));
    let output = format_scan_results(&results, "table").unwrap();
    assert!(output.contains("System Scan Results"));
    assert!(output.contains("status"));
}

#[tokio::test]
async fn test_handle_health_graph_missing_niche() {
    let result = handle_health(
        None, false, false, 10, true, // use_graph
        None, // niche_path - required for graph health
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("niche"));
}

#[tokio::test]
async fn test_handle_health_graph_deprecated() {
    let result = handle_health(
        None,
        false,
        false,
        10,
        true,
        Some(PathBuf::from("/tmp/test-niche")),
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));
}

#[test]
fn test_format_bytes_edge_cases() {
    use super::super::utils::format_bytes;
    assert_eq!(format_bytes(0), "0 B");
    assert_eq!(format_bytes(1023), "1023 B");
    assert_eq!(format_bytes(1024), "1.0 KB");
    assert_eq!(format_bytes(1536), "1.5 KB");
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
fn test_format_probe_results_with_performance() {
    let mut results = HashMap::new();
    results.insert(
        "performance".to_string(),
        serde_json::json!({
            "throughput_rps": 100,
            "avg_latency_ms": 5,
            "error_rate_percent": 0.1
        }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("Performance")));
    assert!(lines.iter().any(|l| l.contains("100")));
}

#[test]
fn test_format_probe_results_with_diagnostics() {
    let mut results = HashMap::new();
    results.insert(
        "diagnostics".to_string(),
        serde_json::json!({
            "key1": "value1",
            "key2": 42,
            "key3": true,
            "key4": [1, 2, 3],
            "key5": {"nested": "obj"}
        }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("Diagnostics")));
}

#[test]
fn test_format_probe_results_with_endpoints() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({
            "reachable": true,
            "response_time_ms": 10,
            "endpoints": [
                {"url": "http://a", "status": "ok"},
                {"url": "http://b", "status": "fail"}
            ]
        }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("http://a")));
    assert!(lines.iter().any(|l| l.contains("http://b")));
}

#[test]
fn test_format_scan_results_empty() {
    let results = HashMap::new();
    let output = format_scan_results(&results, "default").unwrap();
    assert!(output.contains("No results"));
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

#[tokio::test]
async fn test_handle_health_legacy() {
    let result = handle_health(None, false, false, 10, false, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_health_with_service() {
    let result = handle_health(
        Some("test-service".to_string()),
        false,
        false,
        10,
        false,
        None,
    )
    .await;
    assert!(result.is_ok());
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
fn test_format_probe_results_diagnostics_non_object() {
    let mut results = HashMap::new();
    results.insert(
        "diagnostics".to_string(),
        serde_json::json!("raw diagnostic string"),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("raw diagnostic")));
}

#[test]
fn test_format_scan_results_unknown_format_uses_default() {
    let mut results = HashMap::new();
    results.insert("k".to_string(), serde_json::json!(1));
    let out = format_scan_results(&results, "yaml").unwrap();
    assert!(out.contains("System Scan Results"));
}

#[test]
fn test_format_scan_default_many_keys() {
    let mut results = HashMap::new();
    results.insert("a".to_string(), serde_json::json!(1));
    results.insert("b".to_string(), serde_json::json!(2));
    let out = format_scan_results(&results, "unknown").unwrap();
    assert!(out.contains('a'), "expected key 'a' in: {out}");
    assert!(out.contains('b'), "expected key 'b' in: {out}");
    assert!(out.contains("2 items"));
}

#[test]
fn test_compute_memory_percent_one_byte() {
    let p = compute_memory_percent(1, 1024);
    assert!((p - (100.0 / 1024.0)).abs() < 1e-9);
}

#[test]
fn test_status_to_icon_edge() {
    assert_eq!(status_to_icon("RandomStatus"), "🔹");
}

#[test]
fn test_format_health_summary_overall_non_string() {
    let mut results = HashMap::new();
    results.insert("overall_status".to_string(), serde_json::json!([]));
    let lines = format_health_summary(&results, false);
    assert!(!lines.is_empty());
}

#[test]
fn test_format_probe_results_connectivity_unreachable() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({ "reachable": false }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains('❌')));
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
fn test_format_probe_results_connectivity_endpoints_non_array() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({ "endpoints": "not-array" }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("Connectivity")));
}

#[tokio::test]
async fn test_display_status_results_json_format() {
    let mut results = HashMap::new();
    results.insert("status".to_string(), serde_json::json!("Healthy"));
    results.insert("uptime".to_string(), serde_json::json!(42));
    display_status_results(&results, "json", false)
        .await
        .expect("json branch");
}

#[tokio::test]
async fn test_display_status_results_brief_format() {
    let mut results = HashMap::new();
    results.insert("status".to_string(), serde_json::json!("Degraded"));
    display_status_results(&results, "brief", false)
        .await
        .expect("brief branch");
}

#[tokio::test]
async fn test_display_status_results_brief_missing_status() {
    let results = HashMap::new();
    display_status_results(&results, "brief", false)
        .await
        .expect("brief with empty map");
}

#[tokio::test]
async fn test_display_status_results_default_format_summary() {
    let mut results = HashMap::new();
    results.insert(
        "svc".to_string(),
        serde_json::json!({
            "status": "ok",
            "health": "good"
        }),
    );
    display_status_results(&results, "pretty", false)
        .await
        .expect("default / non-json branch");
}

#[tokio::test]
async fn test_display_status_results_default_empty_results() {
    let results = HashMap::new();
    display_status_results(&results, "text", false)
        .await
        .expect("default empty");
}

#[tokio::test]
async fn test_display_status_results_default_with_metrics_flag() {
    let mut results = HashMap::new();
    results.insert("alpha".to_string(), serde_json::json!({ "status": "up" }));
    display_status_results(&results, "default", true)
        .await
        .expect("default with show_metrics");
}

#[tokio::test]
async fn test_display_scan_results_wrapper() {
    let mut results = HashMap::new();
    results.insert("k".to_string(), serde_json::json!(1));
    display_scan_results(&results, "json")
        .await
        .expect("scan json");
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
fn test_format_probe_results_endpoint_without_url() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({
            "endpoints": [ { "status": "ok" } ]
        }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("Endpoints")));
}

#[test]
fn test_format_diagnostics_via_probe_array_and_null() {
    let mut results = HashMap::new();
    results.insert(
        "diagnostics".to_string(),
        serde_json::json!({
            "arr": [1, 2],
            "n": null
        }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("items")));
    assert!(lines.iter().any(|l| l.contains("null")));
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

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
fn test_format_probe_results_connectivity_endpoints_non_array() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({ "endpoints": "not-array" }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("Connectivity")));
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

#[test]
fn test_format_probe_connectivity_endpoint_unknown_status() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({
            "endpoints": [ {"url": "http://x", "status": "unknown"} ]
        }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(
        lines
            .iter()
            .any(|l| l.contains('❌') && l.contains("http://x"))
    );
}

#[test]
fn test_format_probe_results_all_sections_combined() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({ "reachable": true, "response_time_ms": 3 }),
    );
    results.insert(
        "performance".to_string(),
        serde_json::json!({
            "throughput_rps": 9,
            "avg_latency_ms": 1,
            "error_rate_percent": 0.0
        }),
    );
    results.insert(
        "diagnostics".to_string(),
        serde_json::json!({ "note": "ok" }),
    );
    let lines = format_probe_results("full", &results);
    assert!(lines.iter().any(|l| l.contains("Connectivity")));
    assert!(lines.iter().any(|l| l.contains("Performance")));
    assert!(lines.iter().any(|l| l.contains("Diagnostics")));
    assert!(lines.iter().any(|l| l.contains("note")));
}

#[test]
fn test_display_probe_results_runs() {
    let mut results = HashMap::new();
    results.insert(
        "performance".to_string(),
        serde_json::json!({ "throughput_rps": 1 }),
    );
    display_probe_results("p", &results);
}

#[test]
fn test_format_probe_diagnostics_empty_object() {
    let mut results = HashMap::new();
    results.insert("diagnostics".to_string(), serde_json::json!({}));
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("Diagnostics")));
}

#[test]
fn test_format_probe_connectivity_only_endpoints_array() {
    let mut results = HashMap::new();
    results.insert(
        "connectivity".to_string(),
        serde_json::json!({ "endpoints": [] }),
    );
    let lines = format_probe_results("svc", &results);
    assert!(lines.iter().any(|l| l.contains("Endpoints")));
}

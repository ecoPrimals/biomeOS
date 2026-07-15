// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

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

#[test]
fn test_format_scan_results_empty() {
    let results = HashMap::new();
    let output = format_scan_results(&results, "default").unwrap();
    assert!(output.contains("No results"));
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
fn test_format_scan_summary_only_issues() {
    let mut results = HashMap::new();
    results.insert("issues_count".to_string(), serde_json::json!(3));
    let out = format_scan_results(&results, "summary").unwrap();
    assert!(out.contains("Issues found"));
    assert!(out.contains('3'));
}

#[test]
fn test_format_scan_summary_only_services_scanned() {
    let mut results = HashMap::new();
    results.insert("services_scanned".to_string(), serde_json::json!(12));
    let out = format_scan_results(&results, "summary").unwrap();
    assert!(out.contains("Services scanned"));
    assert!(out.contains("12"));
}

#[test]
fn test_format_scan_results_json_roundtrip_empty() {
    let results = HashMap::new();
    let out = format_scan_results(&results, "json").unwrap();
    assert!(out.contains("{}") || out.contains('\n'));
}

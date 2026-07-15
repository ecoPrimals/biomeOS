// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_display_status_results_json_format() {
    let mut results = HashMap::new();
    results.insert("status".to_string(), serde_json::json!("Healthy"));
    results.insert("uptime".to_string(), serde_json::json!(42));
    display_status_results(&results, "json", false).expect("json branch");
}

#[tokio::test]
async fn test_display_status_results_brief_format() {
    let mut results = HashMap::new();
    results.insert("status".to_string(), serde_json::json!("Degraded"));
    display_status_results(&results, "brief", false).expect("brief branch");
}

#[tokio::test]
async fn test_display_status_results_brief_missing_status() {
    let results = HashMap::new();
    display_status_results(&results, "brief", false).expect("brief with empty map");
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
    display_status_results(&results, "pretty", false).expect("default / non-json branch");
}

#[tokio::test]
async fn test_display_status_results_default_empty_results() {
    let results = HashMap::new();
    display_status_results(&results, "text", false).expect("default empty");
}

#[tokio::test]
async fn test_display_status_results_default_with_metrics_flag() {
    let mut results = HashMap::new();
    results.insert("alpha".to_string(), serde_json::json!({ "status": "up" }));
    display_status_results(&results, "default", true).expect("default with show_metrics");
}

#[tokio::test]
async fn test_display_scan_results_wrapper() {
    let mut results = HashMap::new();
    results.insert("k".to_string(), serde_json::json!(1));
    display_scan_results(&results, "json").expect("scan json");
}

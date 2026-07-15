// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use serde_json::Value;

use super::super::format::*;

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

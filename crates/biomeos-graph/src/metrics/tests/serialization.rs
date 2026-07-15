// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::prefix_end;
use super::super::*;

#[test]
fn test_graph_metrics_serialize() {
    let metrics = GraphMetrics {
        graph_name: "test".to_string(),
        total_executions: 10,
        successful_executions: 8,
        failed_executions: 2,
        avg_duration_ms: 150.5,
        min_duration_ms: 100,
        max_duration_ms: 200,
        success_rate: 0.8,
        last_executed_at: chrono::Utc::now(),
    };
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("10"));
    assert!(json.contains("0.8"));
}

#[test]
fn test_execution_record_serialize() {
    let record = ExecutionRecord {
        id: 12345,
        graph_name: "test_graph".to_string(),
        success: true,
        duration_ms: 150,
        executed_at: chrono::Utc::now(),
        metadata: "{}".to_string(),
    };
    let json = serde_json::to_string(&record).unwrap();
    assert!(json.contains("test_graph"));
    assert!(json.contains("150"));
}

#[test]
fn test_graph_result_default() {
    let result = GraphResult::default();
    assert!(!result.success);
    assert!(result.node_results.is_empty());
    assert!(result.errors.is_empty());
    assert_eq!(result.duration_ms, 0);
}

#[test]
fn test_node_metrics_aggregate_serde_roundtrip() {
    let m = NodeMetricsAggregate {
        node_id: "node1".to_string(),
        total_executions: 10,
        successful_executions: 8,
        avg_duration_ms: 50.5,
        success_rate: 0.8,
    };
    let json = serde_json::to_string(&m).unwrap();
    let restored: NodeMetricsAggregate = serde_json::from_str(&json).unwrap();
    assert_eq!(m.node_id, restored.node_id);
    assert!((m.success_rate - restored.success_rate).abs() < f64::EPSILON);
}

#[test]
fn test_prefix_end_colon_suffix() {
    assert_eq!(prefix_end("exec:graph:"), "exec:graph;");
}

#[test]
fn test_prefix_end_empty_string() {
    assert_eq!(prefix_end(""), "");
}

#[test]
fn test_prefix_end_unicode_max_scalar() {
    // Last scalar U+10FFFF: `last + 1` is not a valid `char`; code uses `unwrap_or(U+10FFFF)`.
    let input = "a\u{10ffff}";
    let p = prefix_end(input);
    assert_eq!(p, input);
}

#[test]
fn test_prefix_end_increments_last_byte() {
    assert_eq!(prefix_end("prefix"), "prefiy");
}

#[test]
fn test_prefix_end_single_ascii_char() {
    assert_eq!(prefix_end("a"), "b");
}

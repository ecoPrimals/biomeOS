// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::prefix_end;
use super::*;
use std::collections::HashMap;
use tempfile::tempdir;

#[tokio::test]
async fn test_metrics_collection_ecobin() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics.redb");

    let collector = MetricsCollector::new(&db_path).unwrap();

    // Record a successful execution
    let result = GraphResult {
        success: true,
        node_results: HashMap::default(),
        errors: vec![],
        duration_ms: 100,
    };

    collector
        .record_execution("test_graph", &result, 100, None)
        .unwrap();

    // Get metrics
    let metrics = collector.get_graph_metrics("test_graph").unwrap();
    assert!(metrics.is_some());

    let metrics = metrics.unwrap();
    assert_eq!(metrics.total_executions, 1);
    assert_eq!(metrics.successful_executions, 1);
    assert!((metrics.success_rate - 1.0).abs() < f64::EPSILON);
}

#[tokio::test(start_paused = true)]
async fn test_multiple_executions() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_multi.redb");

    let collector = MetricsCollector::new(&db_path).unwrap();

    // Record multiple executions (advance ensures unique timestamps)
    for i in 0..5 {
        let result = GraphResult {
            success: i % 2 == 0, // Alternate success/failure
            node_results: HashMap::default(),
            errors: vec![],
            duration_ms: (i + 1) * 100,
        };

        collector
            .record_execution(
                "multi_graph",
                &result,
                (i + 1) * 100,
                Some(1000 + i64::try_from(i).unwrap_or(i64::MAX)),
            )
            .unwrap();
        tokio::time::advance(tokio::time::Duration::from_millis(2)).await;
    }

    let metrics = collector.get_graph_metrics("multi_graph").unwrap();
    assert!(metrics.is_some());

    let m = metrics.unwrap();
    assert_eq!(m.total_executions, 5);
    assert_eq!(m.successful_executions, 3); // 0, 2, 4 are successful
    assert_eq!(m.failed_executions, 2); // 1, 3 are failures
}

#[tokio::test]
async fn test_no_metrics_for_unknown_graph() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_empty.redb");

    let collector = MetricsCollector::new(&db_path).unwrap();

    let metrics = collector.get_graph_metrics("nonexistent").unwrap();
    assert!(metrics.is_none());
}

#[tokio::test]
async fn test_tracked_graphs() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_tracked.redb");

    let collector = MetricsCollector::new(&db_path).unwrap();

    // Record executions for multiple graphs
    for graph in &["graph_a", "graph_b", "graph_c"] {
        let result = GraphResult {
            success: true,
            node_results: HashMap::default(),
            errors: vec![],
            duration_ms: 100,
        };
        collector
            .record_execution(graph, &result, 100, None)
            .unwrap();
    }

    let graphs = collector.get_tracked_graphs().unwrap();
    assert_eq!(graphs.len(), 3);
    assert!(graphs.contains(&"graph_a".to_string()));
    assert!(graphs.contains(&"graph_b".to_string()));
    assert!(graphs.contains(&"graph_c".to_string()));
}

#[tokio::test]
async fn test_clear_all() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_clear.redb");

    let collector = MetricsCollector::new(&db_path).unwrap();

    // Add some data
    let result = GraphResult {
        success: true,
        node_results: HashMap::default(),
        errors: vec![],
        duration_ms: 100,
    };
    collector
        .record_execution("test", &result, 100, None)
        .unwrap();

    // Clear
    collector.clear_all().unwrap();

    // Verify cleared
    let graphs = collector.get_tracked_graphs().unwrap();
    assert!(graphs.is_empty());
}

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

#[tokio::test]
async fn test_record_node_execution_and_get_node_metrics() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_node.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let params = NodeExecutionParams {
        execution_id: 42,
        graph_name: "g1",
        node_id: "n1",
        primal_id: "p1",
        operation: "op",
        success: true,
        duration_ms: 50,
        error: None,
    };
    collector.record_node_execution(params).unwrap();

    let params_fail = NodeExecutionParams {
        execution_id: 42,
        graph_name: "g1",
        node_id: "n1",
        primal_id: "p1",
        operation: "op",
        success: false,
        duration_ms: 10,
        error: Some("boom"),
    };
    collector.record_node_execution(params_fail).unwrap();

    let agg = collector
        .get_node_metrics("g1", "n1")
        .unwrap()
        .expect("node metrics");
    assert_eq!(agg.total_executions, 2);
    assert_eq!(agg.successful_executions, 1);
    assert!((agg.avg_duration_ms - 30.0).abs() < f64::EPSILON);
    assert!((agg.success_rate - 0.5).abs() < f64::EPSILON);

    let none = collector.get_node_metrics("g1", "missing").unwrap();
    assert!(none.is_none());
}

#[tokio::test]
async fn test_get_recent_executions_sorted_and_limit() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_recent.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    for id in [100i64, 300, 200] {
        let result = GraphResult {
            success: true,
            node_results: HashMap::default(),
            errors: vec![],
            duration_ms: 10,
        };
        collector
            .record_execution("rg", &result, 10, Some(id))
            .unwrap();
    }

    let recent = collector.get_recent_executions("rg", 2).unwrap();
    assert_eq!(recent.len(), 2);
    assert_eq!(recent[0].id, 300);
    assert_eq!(recent[1].id, 200);

    let all = collector.get_recent_executions("rg", 10).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].id, 300);
}

#[tokio::test]
async fn test_graph_metrics_failed_executions_and_min_max() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_minmax.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let ok = GraphResult {
        success: true,
        node_results: HashMap::default(),
        errors: vec![],
        duration_ms: 100,
    };
    collector
        .record_execution("mm", &ok, 10, Some(1))
        .unwrap();

    let bad = GraphResult {
        success: false,
        node_results: HashMap::default(),
        errors: vec!["e".to_string()],
        duration_ms: 0,
    };
    collector
        .record_execution("mm", &bad, 1000, Some(2))
        .unwrap();

    let m = collector
        .get_graph_metrics("mm")
        .unwrap()
        .expect("metrics");
    assert_eq!(m.total_executions, 2);
    assert_eq!(m.successful_executions, 1);
    assert_eq!(m.failed_executions, 1);
    assert_eq!(m.min_duration_ms, 10);
    assert_eq!(m.max_duration_ms, 1000);
    assert!((m.success_rate - 0.5).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_get_recent_executions_unknown_graph_empty() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_empty_recent.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();
    let v = collector
        .get_recent_executions("no_such_graph", 5)
        .unwrap();
    assert!(v.is_empty());
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

#[tokio::test]
async fn test_get_recent_executions_limit_zero_truncates_to_empty() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_recent_limit0.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let result = GraphResult {
        success: true,
        node_results: HashMap::default(),
        errors: vec![],
        duration_ms: 1,
    };
    collector
        .record_execution("lim0", &result, 10, Some(1))
        .unwrap();

    let recent = collector.get_recent_executions("lim0", 0).unwrap();
    assert!(recent.is_empty());
}

#[tokio::test(start_paused = true)]
async fn test_graph_metrics_last_executed_at_is_most_recent() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_last_exec.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    for id in [1_i64, 2, 3] {
        let result = GraphResult {
            success: true,
            node_results: HashMap::default(),
            errors: vec![],
            duration_ms: 5,
        };
        collector
            .record_execution("last_at", &result, 5, Some(id))
            .unwrap();
        tokio::time::advance(std::time::Duration::from_secs(1)).await;
    }

    let m = collector
        .get_graph_metrics("last_at")
        .unwrap()
        .expect("metrics");
    assert_eq!(m.total_executions, 3);

    let newest = collector.get_recent_executions("last_at", 1).unwrap();
    let latest_record = newest.first().expect("one");
    assert_eq!(latest_record.id, 3);
    assert_eq!(m.last_executed_at, latest_record.executed_at);
}

#[tokio::test]
async fn test_graph_metrics_single_run_min_equals_max_duration() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_min_eq_max.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let result = GraphResult {
        success: true,
        node_results: HashMap::default(),
        errors: vec![],
        duration_ms: 42,
    };
    collector
        .record_execution("eq", &result, 42, Some(100))
        .unwrap();

    let m = collector
        .get_graph_metrics("eq")
        .unwrap()
        .expect("metrics");
    assert_eq!(m.min_duration_ms, 42);
    assert_eq!(m.max_duration_ms, 42);
    assert!((m.avg_duration_ms - 42.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_graph_metrics_all_failures_zero_success_rate() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_all_fail.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    for id in [1_i64, 2] {
        let result = GraphResult {
            success: false,
            node_results: HashMap::default(),
            errors: vec!["e".to_string()],
            duration_ms: 10,
        };
        collector
            .record_execution("all_fail", &result, 10, Some(id))
            .unwrap();
    }

    let m = collector
        .get_graph_metrics("all_fail")
        .unwrap()
        .expect("metrics");
    assert_eq!(m.successful_executions, 0);
    assert_eq!(m.failed_executions, 2);
    assert!((m.success_rate - 0.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_record_execution_metadata_node_results_and_errors_populated() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metadata_patterns.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let mut node_results = HashMap::new();
    node_results.insert(
        "n1".to_string(),
        serde_json::json!({ "out": [1, 2, 3], "nested": { "k": "v" } }),
    );
    node_results.insert("n2".to_string(), serde_json::json!("plain"));

    let result = GraphResult {
        success: false,
        node_results,
        errors: vec!["root".to_string(), "detail".to_string()],
        duration_ms: 99,
    };
    collector
        .record_execution("meta_pat", &result, 200, Some(7001))
        .unwrap();

    let recent = collector
        .get_recent_executions("meta_pat", 1)
        .unwrap();
    let rec = recent.first().expect("one record");
    assert!(rec.metadata.contains("n1"));
    assert!(rec.metadata.contains("nested"));
    assert!(!rec.success);
    assert_eq!(rec.duration_ms, 200);
}

#[tokio::test]
async fn test_record_node_execution_colon_in_graph_and_node_ids() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("node_colon.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let params = NodeExecutionParams {
        execution_id: 9,
        graph_name: "ns:graph:name",
        node_id: "node:with:colons",
        primal_id: "p",
        operation: "op",
        success: true,
        duration_ms: 7,
        error: None,
    };
    collector.record_node_execution(params).unwrap();

    let agg = collector
        .get_node_metrics("ns:graph:name", "node:with:colons")
        .unwrap()
        .expect("aggregate");
    assert_eq!(agg.total_executions, 1);
    assert_eq!(agg.successful_executions, 1);
}

#[tokio::test]
async fn test_record_node_execution_error_field_and_failure() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("node_err_field.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let params = NodeExecutionParams {
        execution_id: 11,
        graph_name: "ge",
        node_id: "n_err",
        primal_id: "primal-x",
        operation: "invoke",
        success: false,
        duration_ms: 3,
        error: Some("node failed hard"),
    };
    collector.record_node_execution(params).unwrap();

    let agg = collector
        .get_node_metrics("ge", "n_err")
        .unwrap()
        .expect("aggregate");
    assert_eq!(agg.successful_executions, 0);
    assert_eq!(agg.total_executions, 1);
    assert!((agg.success_rate - 0.0).abs() < f64::EPSILON);
}

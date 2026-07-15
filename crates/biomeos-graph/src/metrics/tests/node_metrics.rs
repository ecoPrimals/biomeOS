// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use tempfile::tempdir;

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
    collector.record_node_execution(&params).unwrap();

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
    collector.record_node_execution(&params_fail).unwrap();

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
    collector.record_node_execution(&params).unwrap();

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
    collector.record_node_execution(&params).unwrap();

    let agg = collector
        .get_node_metrics("ge", "n_err")
        .unwrap()
        .expect("aggregate");
    assert_eq!(agg.successful_executions, 0);
    assert_eq!(agg.total_executions, 1);
    assert!((agg.success_rate - 0.0).abs() < f64::EPSILON);
}

#[tokio::test]
async fn test_get_node_metrics_filters_wrong_graph_same_prefix() {
    let dir = tempdir().unwrap();
    let db_path = dir.path().join("metrics_node_filter.redb");
    let collector = MetricsCollector::new(&db_path).unwrap();

    let p_other = NodeExecutionParams {
        execution_id: 1,
        graph_name: "g_other",
        node_id: "n1",
        primal_id: "p",
        operation: "op",
        success: true,
        duration_ms: 5,
        error: None,
    };
    collector.record_node_execution(&p_other).unwrap();

    let p_target = NodeExecutionParams {
        execution_id: 2,
        graph_name: "g_target",
        node_id: "n1",
        primal_id: "p",
        operation: "op",
        success: true,
        duration_ms: 15,
        error: None,
    };
    collector.record_node_execution(&p_target).unwrap();

    let agg = collector
        .get_node_metrics("g_target", "n1")
        .unwrap()
        .expect("aggregate");
    assert_eq!(agg.total_executions, 1);
    assert!((agg.avg_duration_ms - 15.0).abs() < f64::EPSILON);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use crate::handlers::graph::ExecutionStatus;
use serde_json::json;

#[test]
fn test_execution_status_construction() {
    let status = ExecutionStatus {
        execution_id: "graph-123".to_string(),
        state: "running".to_string(),
        current_phase: Some(1),
        total_phases: 3,
        completed_nodes: vec!["node1".to_string()],
        failed_nodes: Vec::new(),
        duration_ms: 100,
        error: None,
    };
    assert_eq!(status.execution_id, "graph-123");
    assert_eq!(status.state, "running");
    assert_eq!(status.current_phase, Some(1));
    assert_eq!(status.total_phases, 3);
    assert_eq!(status.completed_nodes, vec!["node1"]);
    assert!(status.failed_nodes.is_empty());
    assert_eq!(status.duration_ms, 100);
    assert!(status.error.is_none());
}

#[test]
fn test_execution_status_with_error() {
    let status = ExecutionStatus {
        execution_id: "graph-456".to_string(),
        state: "failed".to_string(),
        current_phase: Some(2),
        total_phases: 3,
        completed_nodes: vec!["node1".to_string(), "node2".to_string()],
        failed_nodes: vec!["node3".to_string()],
        duration_ms: 500,
        error: Some("Node execution failed".to_string()),
    };
    assert_eq!(status.state, "failed");
    assert_eq!(status.failed_nodes, vec!["node3"]);
    assert_eq!(status.error.as_deref(), Some("Node execution failed"));
}

#[test]
fn test_execution_status_serialization_roundtrip() {
    let status = ExecutionStatus {
        execution_id: "exec-789".to_string(),
        state: "completed".to_string(),
        current_phase: Some(3),
        total_phases: 3,
        completed_nodes: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        failed_nodes: Vec::new(),
        duration_ms: 1234,
        error: None,
    };
    let json = serde_json::to_value(&status).expect("serialize");
    let restored: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
    assert_eq!(restored.execution_id, status.execution_id);
    assert_eq!(restored.state, status.state);
    assert_eq!(restored.completed_nodes, status.completed_nodes);
    assert_eq!(restored.duration_ms, status.duration_ms);
}

#[test]
fn test_execution_status_deserialize_from_json() {
    let json = json!({
        "execution_id": "test-123",
        "state": "running",
        "current_phase": 0,
        "total_phases": 2,
        "completed_nodes": [],
        "failed_nodes": [],
        "duration_ms": 0,
        "error": null
    });
    let status: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
    assert_eq!(status.execution_id, "test-123");
    assert_eq!(status.state, "running");
    assert_eq!(status.current_phase, Some(0));
    assert_eq!(status.total_phases, 2);
}

#[test]
fn test_execution_status_deserialize_with_optional_error() {
    let json = json!({
        "execution_id": "fail-1",
        "state": "failed",
        "current_phase": null,
        "total_phases": 1,
        "completed_nodes": [],
        "failed_nodes": ["n1"],
        "duration_ms": 50,
        "error": "Something went wrong"
    });
    let status: ExecutionStatus = serde_json::from_value(json).expect("deserialize");
    assert_eq!(status.error, Some("Something went wrong".to_string()));
    assert_eq!(status.current_phase, None);
}

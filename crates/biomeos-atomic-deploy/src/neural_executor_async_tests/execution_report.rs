// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! ExecutionReport and PhaseResult builder tests.

#[test]
fn test_execution_report_new_and_builder() {
    use crate::executor::types::ExecutionReport;

    let mut report = ExecutionReport::new("graph-1");
    assert_eq!(report.graph_id, "graph-1");
    assert!(report.success);
    assert_eq!(report.duration_ms, 0);

    report = report.mark_failed("test error");
    assert!(!report.success);
    assert_eq!(report.error.as_deref(), Some("test error"));

    report = report.with_duration(500).with_phases(2).with_nodes(4);
    assert_eq!(report.duration_ms, 500);
    assert_eq!(report.total_phases(), 2);
    assert_eq!(report.total_nodes(), 4);
}

#[test]
fn test_phase_result_builder() {
    use crate::executor::types::PhaseResult;

    let mut result = PhaseResult::new(3);
    result.add_completed();
    result.add_completed();
    result.add_failed("node3", "timeout");
    result.duration_ms = 100;

    assert_eq!(result.completed, 2);
    assert_eq!(result.failed, 1);
    assert!(!result.is_success());
    assert_eq!(
        result.errors,
        vec![("node3".to_string(), "timeout".to_string())]
    );
}

#[test]
fn test_execution_report_add_phase_result() {
    use crate::executor::types::{ExecutionReport, PhaseResult};

    let mut report = ExecutionReport::new("test");
    let mut phase = PhaseResult::new(2);
    phase.completed = 2;
    phase.duration_ms = 50;
    report.add_phase_result(&phase);

    assert_eq!(report.phase_results.len(), 1);
    assert_eq!(report.total_phases(), 1);
    assert_eq!(report.total_nodes(), 2);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shared types for graph execution
//!
//! This module contains result types and report structures used throughout
//! the graph execution system.

use biomeos_graph::GeneticsTier;
use serde::{Deserialize, Serialize};

/// Result of genetics tier preflight (until BearDog exposes `genetics.tier_available`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeneticsTierValidationReport {
    /// Declared tier (snake_case), stable for JSON consumers.
    pub required_tier: String,
    /// Whether the host family was verified to meet the tier (false until BearDog probing exists).
    #[serde(default)]
    pub infrastructure_verified: bool,
    /// Human-readable note for operators and telemetry.
    pub note: String,
}

impl GeneticsTierValidationReport {
    /// Advisory-only validation while infrastructure probing is unavailable.
    #[must_use]
    pub fn pending_bear_dog_probe(required: GeneticsTier) -> Self {
        let required_tier = required.as_str().to_string();
        let note = format!(
            "Graph declares genetics_tier '{required_tier}'; family infrastructure was not verified against BearDog (pending genetics.tier_available). Deployment continues with this advisory."
        );
        Self {
            required_tier,
            infrastructure_verified: false,
            note,
        }
    }
}

/// Execution report for a completed graph execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// Graph ID
    pub graph_id: String,
    /// Whether execution succeeded
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Results from each phase
    #[serde(default)]
    pub phase_results: Vec<PhaseResultSummary>,
    /// Number of phases executed (computed from `phase_results`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phases_executed: Option<usize>,
    /// Total nodes executed (computed from `phase_results`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodes_executed: Option<usize>,
    /// Node IDs that completed successfully
    #[serde(default)]
    pub completed_nodes: Vec<String>,
    /// Node IDs that failed, with error messages
    #[serde(default)]
    pub failed_nodes: Vec<(String, String)>,
    /// Error message if failed
    pub error: Option<String>,
    /// Genetics tier preflight (when the graph declares `[graph.metadata].genetics_tier`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub genetics_tier_validation: Option<GeneticsTierValidationReport>,
}

/// Serializable summary of a phase result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResultSummary {
    /// Number of nodes completed
    pub completed: usize,
    /// Number of nodes failed
    pub failed: usize,
    /// Total nodes in phase
    pub total: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl From<&PhaseResult> for PhaseResultSummary {
    fn from(result: &PhaseResult) -> Self {
        Self {
            completed: result.completed,
            failed: result.failed,
            total: result.total,
            duration_ms: result.duration_ms,
        }
    }
}

impl ExecutionReport {
    /// Create a new execution report
    pub fn new(graph_id: impl Into<String>) -> Self {
        Self {
            graph_id: graph_id.into(),
            success: true,
            duration_ms: 0,
            phase_results: Vec::new(),
            phases_executed: None,
            nodes_executed: None,
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            error: None,
            genetics_tier_validation: None,
        }
    }

    /// Mark as successful
    #[must_use]
    pub const fn mark_success(mut self) -> Self {
        self.success = true;
        self
    }

    /// Mark as failed with error
    pub fn mark_failed(mut self, error: impl Into<String>) -> Self {
        self.success = false;
        self.error = Some(error.into());
        self
    }

    /// Set duration
    #[must_use]
    pub const fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = duration_ms;
        self
    }

    /// Add a phase result
    pub fn add_phase_result(&mut self, result: &PhaseResult) {
        self.phase_results.push(PhaseResultSummary::from(result));
    }

    /// Set phases executed (explicit override)
    #[must_use]
    pub const fn with_phases(mut self, phases: usize) -> Self {
        self.phases_executed = Some(phases);
        self
    }

    /// Set nodes executed (explicit override)
    #[must_use]
    pub const fn with_nodes(mut self, nodes: usize) -> Self {
        self.nodes_executed = Some(nodes);
        self
    }

    /// Get total phases (from results or explicit)
    #[must_use]
    pub fn total_phases(&self) -> usize {
        self.phases_executed.unwrap_or(self.phase_results.len())
    }

    /// Get total nodes (from results or explicit)
    #[must_use]
    pub fn total_nodes(&self) -> usize {
        self.nodes_executed
            .unwrap_or_else(|| self.phase_results.iter().map(|p| p.total).sum())
    }
}

/// Result from executing a single phase
#[derive(Debug, Clone)]
pub struct PhaseResult {
    /// Number of nodes completed
    pub completed: usize,
    /// Number of nodes failed
    pub failed: usize,
    /// Total nodes in phase
    pub total: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error details (`node_id`, `error_message`)
    pub errors: Vec<(String, String)>,
}

impl PhaseResult {
    /// Create new phase result
    #[must_use]
    pub const fn new(total_nodes: usize) -> Self {
        Self {
            completed: 0,
            failed: 0,
            total: total_nodes,
            duration_ms: 0,
            errors: Vec::new(),
        }
    }

    /// Check if phase succeeded (no failures)
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.failed == 0
    }

    /// Add a completion
    pub const fn add_completed(&mut self) {
        self.completed += 1;
    }

    /// Add a failure
    pub fn add_failed(&mut self, node_id: impl Into<String>, error: impl Into<String>) {
        self.failed += 1;
        self.errors.push((node_id.into(), error.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_report_builder() {
        let report = ExecutionReport::new("test-graph")
            .mark_success()
            .with_duration(1000)
            .with_phases(3)
            .with_nodes(10);

        assert!(report.success);
        assert_eq!(report.duration_ms, 1000);
        assert_eq!(report.total_phases(), 3);
        assert_eq!(report.total_nodes(), 10);
        assert!(report.error.is_none());
    }

    #[test]
    fn test_execution_report_with_phase_results() {
        let mut report = ExecutionReport::new("test-graph");

        let mut phase1 = PhaseResult::new(3);
        phase1.completed = 3;
        phase1.duration_ms = 100;

        let mut phase2 = PhaseResult::new(2);
        phase2.completed = 2;
        phase2.duration_ms = 50;

        report.add_phase_result(&phase1);
        report.add_phase_result(&phase2);

        assert_eq!(report.total_phases(), 2);
        assert_eq!(report.total_nodes(), 5);
    }

    #[test]
    fn test_execution_report_failed() {
        let report = ExecutionReport::new("test-graph").mark_failed("Something went wrong");

        assert!(!report.success);
        assert_eq!(report.error, Some("Something went wrong".to_string()));
    }

    #[test]
    fn test_phase_result() {
        let mut result = PhaseResult::new(5);

        result.add_completed();
        result.add_completed();
        result.add_failed("node3", "timeout");

        assert_eq!(result.completed, 2);
        assert_eq!(result.failed, 1);
        assert!(!result.is_success());
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn test_phase_result_summary_conversion() {
        let mut result = PhaseResult::new(5);
        result.completed = 4;
        result.failed = 1;
        result.duration_ms = 500;

        let summary = PhaseResultSummary::from(&result);

        assert_eq!(summary.completed, 4);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.total, 5);
        assert_eq!(summary.duration_ms, 500);
    }

    // --- New tests for comprehensive coverage ---

    #[test]
    fn test_phase_result_all_success() {
        let mut result = PhaseResult::new(3);
        result.add_completed();
        result.add_completed();
        result.add_completed();
        assert!(result.is_success());
        assert_eq!(result.completed, 3);
        assert_eq!(result.failed, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_phase_result_multiple_failures() {
        let mut result = PhaseResult::new(4);
        result.add_completed();
        result.add_failed("node2", "timeout");
        result.add_failed("node3", "OOM");
        result.add_completed();

        assert!(!result.is_success());
        assert_eq!(result.completed, 2);
        assert_eq!(result.failed, 2);
        assert_eq!(result.errors.len(), 2);
        assert_eq!(
            result.errors[0],
            ("node2".to_string(), "timeout".to_string())
        );
        assert_eq!(result.errors[1], ("node3".to_string(), "OOM".to_string()));
    }

    #[test]
    fn test_execution_report_serialization_roundtrip() {
        let mut report = ExecutionReport::new("my-graph")
            .mark_success()
            .with_duration(2500)
            .with_phases(3)
            .with_nodes(8);

        let mut phase = PhaseResult::new(2);
        phase.completed = 2;
        phase.duration_ms = 100;
        report.add_phase_result(&phase);

        let json = serde_json::to_string(&report).unwrap();
        let parsed: ExecutionReport = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.graph_id, "my-graph");
        assert!(parsed.success);
        assert_eq!(parsed.duration_ms, 2500);
        assert_eq!(parsed.phase_results.len(), 1);
        assert_eq!(parsed.total_phases(), 3); // Explicit override
        assert_eq!(parsed.total_nodes(), 8); // Explicit override
    }

    #[test]
    fn test_execution_report_total_phases_from_results() {
        let mut report = ExecutionReport::new("test");

        let phase1 = PhaseResult::new(3);
        let phase2 = PhaseResult::new(2);
        report.add_phase_result(&phase1);
        report.add_phase_result(&phase2);

        // No explicit phases_executed — should derive from phase_results
        assert_eq!(report.total_phases(), 2);
        assert_eq!(report.total_nodes(), 5); // 3 + 2
    }

    #[test]
    fn test_execution_report_empty() {
        let report = ExecutionReport::new("empty-graph");
        assert!(report.success); // Default is success
        assert_eq!(report.duration_ms, 0);
        assert_eq!(report.total_phases(), 0);
        assert_eq!(report.total_nodes(), 0);
        assert!(report.error.is_none());
    }

    #[test]
    fn test_execution_report_debug() {
        let report = ExecutionReport::new("debug-test").mark_failed("kaboom");
        let debug_str = format!("{report:?}");
        assert!(debug_str.contains("ExecutionReport"));
        assert!(debug_str.contains("debug-test"));
        assert!(debug_str.contains("kaboom"));
    }

    #[test]
    fn test_phase_result_summary_serialization() {
        let summary = PhaseResultSummary {
            completed: 5,
            failed: 1,
            total: 6,
            duration_ms: 750,
        };
        let json = serde_json::to_string(&summary).unwrap();
        let parsed: PhaseResultSummary = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.completed, 5);
        assert_eq!(parsed.failed, 1);
        assert_eq!(parsed.total, 6);
        assert_eq!(parsed.duration_ms, 750);
    }

    #[test]
    fn test_phase_result_clone() {
        let mut result = PhaseResult::new(2);
        result.add_completed();
        result.add_failed("node2", "error");
        result.duration_ms = 100;

        let cloned = result.clone();
        assert_eq!(cloned.completed, result.completed);
        assert_eq!(cloned.failed, result.failed);
        assert_eq!(cloned.errors.len(), result.errors.len());
    }
}

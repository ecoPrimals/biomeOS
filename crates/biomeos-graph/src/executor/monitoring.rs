// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Monitoring and Reporting Module
//!
//! Provides execution metrics, reports, and monitoring capabilities.

use serde::{Deserialize, Serialize};

/// Execution report for the entire graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// Graph ID
    pub graph_id: String,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Results for each phase
    pub phase_results: Vec<PhaseResult>,
}

impl ExecutionReport {
    /// Create new execution report
    pub fn new(graph_id: String) -> Self {
        Self {
            graph_id,
            success: true,
            error: None,
            duration_ms: 0,
            phase_results: Vec::new(),
        }
    }

    /// Get total nodes executed
    pub fn total_nodes(&self) -> usize {
        self.phase_results
            .iter()
            .map(|p| p.nodes_executed)
            .sum()
    }

    /// Get total nodes failed
    pub fn total_failures(&self) -> usize {
        self.phase_results.iter().map(|p| p.nodes_failed).sum()
    }
}

/// Result for a single execution phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    /// Number of nodes in this phase
    pub total_nodes: usize,
    /// Number of nodes successfully executed
    pub nodes_executed: usize,
    /// Number of nodes that failed
    pub nodes_failed: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl PhaseResult {
    /// Create new phase result
    pub fn new(total_nodes: usize) -> Self {
        Self {
            total_nodes,
            nodes_executed: 0,
            nodes_failed: 0,
            duration_ms: 0,
        }
    }

    /// Check if phase was successful
    pub fn is_success(&self) -> bool {
        self.nodes_failed == 0 && self.nodes_executed == self.total_nodes
    }

    /// Get success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_nodes == 0 {
            100.0
        } else {
            (self.nodes_executed as f64 / self.total_nodes as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_report_new() {
        let report = ExecutionReport::new("test-graph".to_string());
        assert_eq!(report.graph_id, "test-graph");
        assert!(report.success);
        assert_eq!(report.duration_ms, 0);
    }

    #[test]
    fn test_phase_result_success_rate() {
        let mut phase = PhaseResult::new(10);
        phase.nodes_executed = 8;
        phase.nodes_failed = 2;
        
        assert_eq!(phase.success_rate(), 80.0);
        assert!(!phase.is_success());
    }

    #[test]
    fn test_phase_result_perfect_success() {
        let mut phase = PhaseResult::new(5);
        phase.nodes_executed = 5;
        phase.nodes_failed = 0;
        
        assert_eq!(phase.success_rate(), 100.0);
        assert!(phase.is_success());
    }

    #[test]
    fn test_report_totals() {
        let mut report = ExecutionReport::new("test".to_string());
        
        let mut phase1 = PhaseResult::new(5);
        phase1.nodes_executed = 5;
        
        let mut phase2 = PhaseResult::new(3);
        phase2.nodes_executed = 2;
        phase2.nodes_failed = 1;
        
        report.phase_results.push(phase1);
        report.phase_results.push(phase2);
        
        assert_eq!(report.total_nodes(), 7);
        assert_eq!(report.total_failures(), 1);
    }
}

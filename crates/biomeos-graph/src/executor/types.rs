// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Execution types and results
//!
//! **EVOLVED:** Domain-based splitting for maintainability.
//!
//! This module contains execution report types and phase result types
//! used throughout the graph execution system.

use serde::{Deserialize, Serialize};

/// Execution report for the entire graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// Graph ID
    pub graph_id: String,
    /// Success status
    pub success: bool,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Results for each phase
    pub phase_results: Vec<PhaseResult>,
    /// Error message if failed
    pub error: Option<String>,
}

impl ExecutionReport {
    /// Create new execution report
    pub fn new(graph_id: String) -> Self {
        Self {
            graph_id,
            success: true,
            duration_ms: 0,
            phase_results: Vec::new(),
            error: None,
        }
    }
}

/// Phase execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseResult {
    /// Number of nodes in this phase
    pub total_nodes: usize,
    /// Number of nodes successfully completed
    pub completed: usize,
    /// Number of nodes that failed
    pub failed: usize,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// List of errors (node_id, error_message)
    pub errors: Vec<(String, String)>,
}

impl PhaseResult {
    /// Create new phase result
    pub fn new(total_nodes: usize) -> Self {
        Self {
            total_nodes,
            completed: 0,
            failed: 0,
            duration_ms: 0,
            errors: Vec::new(),
        }
    }
}

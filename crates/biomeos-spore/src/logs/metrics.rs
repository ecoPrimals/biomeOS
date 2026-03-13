// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Log issues and metrics tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Log issue detected during analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogIssue {
    /// Timestamp of issue
    pub timestamp: DateTime<Utc>,

    /// Severity (error, warning, info)
    pub severity: IssueSeverity,

    /// Primal where issue occurred
    pub primal: String,

    /// Issue description
    pub description: String,

    /// Log line where it occurred
    pub log_line: Option<String>,
}

/// Issue severity levels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IssueSeverity {
    /// System-affecting critical issue
    Critical,
    /// Error requiring attention
    Error,
    /// Potential problem
    Warning,
    /// Informational
    Info,
}

/// Metrics summary for a log session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMetrics {
    /// Total lines logged
    pub total_lines: u64,

    /// Number of error-level lines
    pub errors: u64,
    /// Number of warning-level lines
    pub warnings: u64,
    /// Number of info-level lines
    pub info: u64,

    /// Total size
    pub total_size_bytes: u64,

    /// Session duration
    pub duration_secs: u64,
}

impl LogMetrics {
    /// Create empty metrics.
    pub fn new() -> Self {
        Self {
            total_lines: 0,
            errors: 0,
            warnings: 0,
            info: 0,
            total_size_bytes: 0,
            duration_secs: 0,
        }
    }
}

impl Default for LogMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_equality() {
        assert_eq!(IssueSeverity::Error, IssueSeverity::Error);
        assert_ne!(IssueSeverity::Error, IssueSeverity::Warning);
    }

    #[test]
    fn test_default_metrics() {
        let metrics = LogMetrics::default();
        assert_eq!(metrics.total_lines, 0);
    }
}

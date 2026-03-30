// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
    #[must_use]
    pub const fn new() -> Self {
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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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

    #[test]
    fn test_issue_severity_serde_roundtrip() {
        for sev in [
            IssueSeverity::Critical,
            IssueSeverity::Error,
            IssueSeverity::Warning,
            IssueSeverity::Info,
        ] {
            let json = serde_json::to_string(&sev).unwrap();
            let restored: IssueSeverity = serde_json::from_str(&json).unwrap();
            assert_eq!(sev, restored);
        }
    }

    #[test]
    fn test_log_metrics_serde_roundtrip() {
        let m = LogMetrics {
            total_lines: 100,
            errors: 5,
            warnings: 10,
            info: 85,
            total_size_bytes: 4096,
            duration_secs: 60,
        };
        let json = serde_json::to_string(&m).unwrap();
        let restored: LogMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(m.total_lines, restored.total_lines);
        assert_eq!(m.errors, restored.errors);
    }

    #[test]
    fn test_log_metrics_new() {
        let m = LogMetrics::new();
        assert_eq!(m.total_lines, 0);
        assert_eq!(m.errors, 0);
        assert_eq!(m.warnings, 0);
    }

    #[test]
    fn test_log_issue_serde_roundtrip() {
        let issue = LogIssue {
            timestamp: chrono::Utc::now(),
            severity: IssueSeverity::Warning,
            primal: "songbird".to_string(),
            description: "test issue".to_string(),
            log_line: Some("line 42".to_string()),
        };
        let json = serde_json::to_string(&issue).unwrap();
        let restored: LogIssue = serde_json::from_str(&json).unwrap();
        assert_eq!(issue.primal, restored.primal);
        assert_eq!(issue.severity, restored.severity);
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Pluggable validation output via `ValidationSink`.
//!
//! Absorbed from airSpring, rhizoCrypt, and ludoSpring. Allows validation
//! commands (`biomeos validate`) to output to different targets: stderr for
//! interactive use, buffers for programmatic use, structured JSON for CI.

use std::fmt;

/// Severity level for a validation finding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValidationSeverity {
    /// Informational note — does not affect compliance.
    Info,
    /// Warning — should be fixed but does not block.
    Warning,
    /// Error — blocks compliance.
    Error,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A single validation finding.
#[derive(Debug, Clone)]
pub struct ValidationFinding {
    /// Severity of this finding.
    pub severity: ValidationSeverity,
    /// Short rule identifier (e.g. "ecobin-no-c-deps", "unibin-single-binary").
    pub rule: String,
    /// Human-readable description.
    pub message: String,
    /// Optional file or path context.
    pub location: Option<String>,
}

impl fmt::Display for ValidationFinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref loc) = self.location {
            write!(
                f,
                "[{}] {} ({}): {}",
                self.severity, self.rule, loc, self.message
            )
        } else {
            write!(f, "[{}] {}: {}", self.severity, self.rule, self.message)
        }
    }
}

/// Trait for receiving validation findings.
///
/// Implementors decide how to render/store findings. The default
/// `StderrSink` prints to stderr; `BufferSink` collects for
/// programmatic access.
pub trait ValidationSink {
    /// Emit a validation finding.
    fn emit(&mut self, finding: ValidationFinding);

    /// Convenience: emit an error finding.
    fn error(&mut self, rule: impl Into<String>, message: impl Into<String>) {
        self.emit(ValidationFinding {
            severity: ValidationSeverity::Error,
            rule: rule.into(),
            message: message.into(),
            location: None,
        });
    }

    /// Convenience: emit a warning finding.
    fn warning(&mut self, rule: impl Into<String>, message: impl Into<String>) {
        self.emit(ValidationFinding {
            severity: ValidationSeverity::Warning,
            rule: rule.into(),
            message: message.into(),
            location: None,
        });
    }

    /// Convenience: emit an info finding.
    fn info(&mut self, rule: impl Into<String>, message: impl Into<String>) {
        self.emit(ValidationFinding {
            severity: ValidationSeverity::Info,
            rule: rule.into(),
            message: message.into(),
            location: None,
        });
    }
}

/// Sink that prints findings to stderr (interactive CLI use).
///
/// # Examples
///
/// ```
/// use biomeos_types::validation::{StderrSink, ValidationSink};
/// let mut sink = StderrSink::default();
/// sink.error("ecobin-no-c-deps", "found openssl-sys");
/// sink.warning("lint-check", "unused import");
/// assert!(sink.has_errors());
/// assert_eq!(sink.error_count(), 1);
/// assert_eq!(sink.warning_count(), 1);
/// ```
#[derive(Debug, Default)]
pub struct StderrSink {
    error_count: usize,
    warning_count: usize,
}

impl StderrSink {
    /// Number of error findings emitted so far.
    #[must_use]
    pub const fn error_count(&self) -> usize {
        self.error_count
    }

    /// Number of warning findings emitted so far.
    #[must_use]
    pub const fn warning_count(&self) -> usize {
        self.warning_count
    }

    /// Whether any errors were emitted.
    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

impl ValidationSink for StderrSink {
    fn emit(&mut self, finding: ValidationFinding) {
        match finding.severity {
            ValidationSeverity::Error => self.error_count += 1,
            ValidationSeverity::Warning => self.warning_count += 1,
            ValidationSeverity::Info => {}
        }
        eprintln!("{finding}");
    }
}

/// Sink that collects findings into a `Vec` (programmatic/test use).
///
/// # Examples
///
/// ```
/// use biomeos_types::validation::{BufferSink, ValidationSink};
/// let mut sink = BufferSink::default();
/// sink.error("rule-1", "error message");
/// sink.warning("rule-2", "warning message");
/// sink.info("rule-3", "info message");
/// assert_eq!(sink.findings.len(), 3);
/// assert!(sink.has_errors());
/// assert_eq!(sink.error_count(), 1);
/// ```
#[derive(Debug, Default)]
pub struct BufferSink {
    /// All collected findings.
    pub findings: Vec<ValidationFinding>,
}

impl BufferSink {
    /// Number of error findings.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.findings
            .iter()
            .filter(|f| f.severity == ValidationSeverity::Error)
            .count()
    }

    /// Whether any errors were collected.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == ValidationSeverity::Error)
    }
}

impl ValidationSink for BufferSink {
    fn emit(&mut self, finding: ValidationFinding) {
        self.findings.push(finding);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_ordering() {
        assert!(ValidationSeverity::Info < ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning < ValidationSeverity::Error);
    }

    #[test]
    fn severity_display() {
        assert_eq!(ValidationSeverity::Info.to_string(), "INFO");
        assert_eq!(ValidationSeverity::Warning.to_string(), "WARN");
        assert_eq!(ValidationSeverity::Error.to_string(), "ERROR");
    }

    #[test]
    fn finding_display_with_location() {
        let f = ValidationFinding {
            severity: ValidationSeverity::Error,
            rule: "ecobin-no-c-deps".to_owned(),
            message: "found openssl-sys".to_owned(),
            location: Some("Cargo.toml".to_owned()),
        };
        let s = f.to_string();
        assert!(s.contains("[ERROR]"));
        assert!(s.contains("ecobin-no-c-deps"));
        assert!(s.contains("Cargo.toml"));
    }

    #[test]
    fn finding_display_without_location() {
        let f = ValidationFinding {
            severity: ValidationSeverity::Warning,
            rule: "lint-check".to_owned(),
            message: "unused import".to_owned(),
            location: None,
        };
        let s = f.to_string();
        assert!(s.contains("[WARN]"));
        assert!(!s.contains('('));
    }

    #[test]
    fn buffer_sink_collects() {
        let mut sink = BufferSink::default();
        sink.error("rule-1", "error msg");
        sink.warning("rule-2", "warn msg");
        sink.info("rule-3", "info msg");
        assert_eq!(sink.findings.len(), 3);
        assert_eq!(sink.error_count(), 1);
        assert!(sink.has_errors());
    }

    #[test]
    fn buffer_sink_empty_no_errors() {
        let sink = BufferSink::default();
        assert!(!sink.has_errors());
        assert_eq!(sink.error_count(), 0);
    }

    #[test]
    fn stderr_sink_counts() {
        let mut sink = StderrSink::default();
        sink.emit(ValidationFinding {
            severity: ValidationSeverity::Error,
            rule: "r".to_owned(),
            message: "m".to_owned(),
            location: None,
        });
        sink.emit(ValidationFinding {
            severity: ValidationSeverity::Warning,
            rule: "r".to_owned(),
            message: "m".to_owned(),
            location: None,
        });
        assert_eq!(sink.error_count(), 1);
        assert_eq!(sink.warning_count(), 1);
        assert!(sink.has_errors());
    }
}

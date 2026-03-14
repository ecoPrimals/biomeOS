// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Doctor mode types - Diagnostics, HealthCheck, HealthStatus

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Diagnostics {
    pub(crate) checks: Vec<HealthCheck>,
    pub(crate) overall_status: HealthStatus,
    pub(crate) recommendations: Vec<String>,
}

/// Alias for diagnostic check (used in format/aggregate APIs)
pub(crate) type DiagnosticCheck = HealthCheck;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HealthCheck {
    pub(crate) name: String,
    pub(crate) status: HealthStatus,
    pub(crate) details: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl Diagnostics {
    pub(crate) fn new() -> Self {
        Self {
            checks: Vec::new(),
            overall_status: HealthStatus::Healthy,
            recommendations: Vec::new(),
        }
    }

    pub(crate) fn add_check(&mut self, _name: &str, check: HealthCheck) {
        // Update overall status
        match check.status {
            HealthStatus::Critical => self.overall_status = HealthStatus::Critical,
            HealthStatus::Warning if self.overall_status != HealthStatus::Critical => {
                self.overall_status = HealthStatus::Warning;
            }
            _ => {}
        }

        self.checks.push(check);
    }

    pub(crate) fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_diagnostics_new() {
        let diag = Diagnostics::new();
        assert!(diag.checks.is_empty());
        assert_eq!(diag.overall_status, HealthStatus::Healthy);
        assert!(diag.recommendations.is_empty());
    }

    #[test]
    fn test_diagnostics_add_check_healthy_stays_healthy() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Test",
            HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Healthy,
                details: vec!["ok".to_string()],
            },
        );
        assert_eq!(diag.overall_status, HealthStatus::Healthy);
        assert_eq!(diag.checks.len(), 1);
    }

    #[test]
    fn test_diagnostics_add_check_warning_upgrades_status() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Test",
            HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        assert_eq!(diag.overall_status, HealthStatus::Warning);
    }

    #[test]
    fn test_diagnostics_add_check_critical_upgrades_status() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Test",
            HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Critical,
                details: vec![],
            },
        );
        assert_eq!(diag.overall_status, HealthStatus::Critical);
    }

    #[test]
    fn test_diagnostics_add_check_warning_does_not_downgrade_critical() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Critical",
            HealthCheck {
                name: "Critical".to_string(),
                status: HealthStatus::Critical,
                details: vec![],
            },
        );
        diag.add_check(
            "Warning",
            HealthCheck {
                name: "Warning".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        assert_eq!(diag.overall_status, HealthStatus::Critical);
    }

    #[test]
    fn test_diagnostics_add_recommendation() {
        let mut diag = Diagnostics::new();
        diag.add_recommendation("Fix something".to_string());
        diag.add_recommendation("Fix another".to_string());
        assert_eq!(diag.recommendations.len(), 2);
        assert_eq!(diag.recommendations[0], "Fix something");
        assert_eq!(diag.recommendations[1], "Fix another");
    }

    #[test]
    fn test_health_status_serialization() {
        let statuses = [
            HealthStatus::Healthy,
            HealthStatus::Warning,
            HealthStatus::Critical,
        ];
        for status in statuses {
            let json = serde_json::to_string(&status).expect("serialize");
            let parsed: HealthStatus = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(status, parsed);
        }
    }

    #[test]
    fn test_health_check_serialization() {
        let check = HealthCheck {
            name: "Test Check".to_string(),
            status: HealthStatus::Warning,
            details: vec!["detail1".to_string(), "detail2".to_string()],
        };
        let json = serde_json::to_string(&check).expect("serialize");
        let parsed: HealthCheck = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(check.name, parsed.name);
        assert_eq!(check.status, parsed.status);
        assert_eq!(check.details, parsed.details);
    }

    #[test]
    fn test_diagnostics_serialization() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Binary",
            HealthCheck {
                name: "Binary".to_string(),
                status: HealthStatus::Healthy,
                details: vec!["OK".to_string()],
            },
        );
        diag.add_recommendation("Rec 1".to_string());
        let json = serde_json::to_string(&diag).expect("serialize");
        let parsed: Diagnostics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.checks.len(), 1);
        assert_eq!(parsed.recommendations.len(), 1);
    }

    #[test]
    fn test_diagnostics_json_roundtrip() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Test",
            HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Warning,
                details: vec!["detail".to_string()],
            },
        );
        diag.add_recommendation("rec".to_string());
        let json = serde_json::to_string(&diag).unwrap();
        let parsed: Diagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.checks.len(), 1);
        assert_eq!(parsed.overall_status, HealthStatus::Warning);
        assert_eq!(parsed.recommendations.len(), 1);
    }
}

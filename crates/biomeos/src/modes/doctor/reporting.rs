// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Doctor mode reporting - text/JSON formatting and recommendations

use anyhow::Result;
use colored::Colorize;

#[cfg(test)]
use super::types::HealthCheck;
use super::types::{DiagnosticCheck, Diagnostics, HealthStatus};

/// Format diagnostics as text lines (pure, testable).
pub(crate) fn format_text_report(results: &Diagnostics) -> Vec<String> {
    let mut lines = Vec::new();
    for check in &results.checks {
        let status_icon = match check.status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Warning => "⚠️ ",
            HealthStatus::Critical => "❌",
        };
        lines.push(format!("{} {}", status_icon, check.name.bold()));
        for detail in &check.details {
            lines.push(format!("   {detail}"));
        }
        lines.push(String::new());
    }
    lines.push(
        "═══════════════════════════════════════════════════════════════"
            .bright_black()
            .to_string(),
    );
    let overall_status_text = match results.overall_status {
        HealthStatus::Healthy => "✅ HEALTHY".bright_green().to_string(),
        HealthStatus::Warning => "⚠️  HEALTHY (warnings)".bright_yellow().to_string(),
        HealthStatus::Critical => "❌ CRITICAL".bright_red().to_string(),
    };
    lines.push(format!(
        "{}: {}",
        "Overall Health".bold(),
        overall_status_text
    ));
    if !results.recommendations.is_empty() {
        lines.push(String::new());
        lines.push(format!("{}:", "Recommendations".bold()));
        for rec in &results.recommendations {
            lines.push(format!("  {} {}", "•".bright_cyan(), rec));
        }
    }
    lines
}

/// Format diagnostics as JSON string (pure, testable).
pub(crate) fn format_json_report(results: &Diagnostics) -> Result<String> {
    serde_json::to_string_pretty(results).map_err(Into::into)
}

/// Aggregate recommendations from diagnostic checks (pure, testable).
pub(crate) fn aggregate_recommendations(results: &[DiagnosticCheck]) -> Vec<String> {
    let mut recommendations = Vec::new();
    for check in results {
        match check.name.as_str() {
            "Primal Discovery" if check.status != HealthStatus::Healthy => {
                recommendations.push("Start missing primals for full functionality".to_string());
            }
            "System Resources" if check.status != HealthStatus::Healthy => {
                recommendations
                    .push("System resources under pressure - consider scaling".to_string());
            }
            "Graphs Directory" if check.status != HealthStatus::Healthy => {
                recommendations
                    .push("Create graphs/ directory and add deployment graphs".to_string());
            }
            _ => {}
        }
    }
    recommendations
}

pub(crate) fn add_recommendations(diag: &mut Diagnostics) {
    for rec in aggregate_recommendations(&diag.checks) {
        diag.add_recommendation(rec);
    }
}

/// Print diagnostics to stdout (uses format_text_report)
#[cfg(test)]
pub(crate) fn print_diagnostics(diag: &Diagnostics) {
    for line in format_text_report(diag) {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[test]
    fn test_format_text_report() {
        let diag = Diagnostics {
            checks: vec![HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Healthy,
                details: vec!["ok".to_string()],
            }],
            overall_status: HealthStatus::Healthy,
            recommendations: vec![],
        };
        let lines = format_text_report(&diag);
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|l| l.contains("Test")));
        assert!(lines.iter().any(|l| l.contains("HEALTHY")));
    }

    #[test]
    fn test_format_json_report() {
        let diag = Diagnostics {
            checks: vec![HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Warning,
                details: vec!["detail".to_string()],
            }],
            overall_status: HealthStatus::Warning,
            recommendations: vec!["Fix it".to_string()],
        };
        let json = format_json_report(&diag).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("Warning"));
        assert!(json.contains("Fix it"));
    }

    #[test]
    fn test_aggregate_recommendations() {
        let checks = vec![
            HealthCheck {
                name: "Primal Discovery".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
            HealthCheck {
                name: "Binary".to_string(),
                status: HealthStatus::Healthy,
                details: vec![],
            },
        ];
        let recs = aggregate_recommendations(&checks);
        assert_eq!(recs.len(), 1);
        assert!(recs[0].contains("primals"));
    }

    #[test]
    fn test_add_recommendations_primal_discovery() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Primal Discovery",
            HealthCheck {
                name: "Primal Discovery".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        add_recommendations(&mut diag);
        assert!(
            diag.recommendations
                .contains(&"Start missing primals for full functionality".to_string()),
            "Expected primal discovery recommendation"
        );
    }

    #[test]
    fn test_add_recommendations_system_resources() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "System Resources",
            HealthCheck {
                name: "System Resources".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        add_recommendations(&mut diag);
        assert!(
            diag.recommendations
                .contains(&"System resources under pressure - consider scaling".to_string()),
            "Expected system resources recommendation"
        );
    }

    #[test]
    fn test_add_recommendations_graphs_directory() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Graphs Directory",
            HealthCheck {
                name: "Graphs Directory".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        add_recommendations(&mut diag);
        assert!(
            diag.recommendations
                .contains(&"Create graphs/ directory and add deployment graphs".to_string()),
            "Expected graphs directory recommendation"
        );
    }

    #[test]
    fn test_add_recommendations_healthy_checks_add_nothing() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Primal Discovery",
            HealthCheck {
                name: "Primal Discovery".to_string(),
                status: HealthStatus::Healthy,
                details: vec![],
            },
        );
        add_recommendations(&mut diag);
        assert!(diag.recommendations.is_empty());
    }

    #[test]
    fn test_add_recommendations_unknown_check_adds_nothing() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Unknown",
            HealthCheck {
                name: "Unknown".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        add_recommendations(&mut diag);
        assert!(diag.recommendations.is_empty());
    }

    #[test]
    fn test_add_recommendations_multiple_checks() {
        let mut diag = Diagnostics::new();
        diag.add_check(
            "Primal Discovery",
            HealthCheck {
                name: "Primal Discovery".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        diag.add_check(
            "System Resources",
            HealthCheck {
                name: "System Resources".to_string(),
                status: HealthStatus::Warning,
                details: vec![],
            },
        );
        add_recommendations(&mut diag);
        assert_eq!(diag.recommendations.len(), 2);
    }

    #[test]
    fn test_print_diagnostics_does_not_panic() {
        let diag = Diagnostics {
            checks: vec![HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Healthy,
                details: vec!["detail".to_string()],
            }],
            overall_status: HealthStatus::Healthy,
            recommendations: vec![],
        };
        print_diagnostics(&diag);
    }

    #[test]
    fn test_print_diagnostics_all_statuses() {
        let diag = Diagnostics {
            checks: vec![
                HealthCheck {
                    name: "Healthy".to_string(),
                    status: HealthStatus::Healthy,
                    details: vec!["ok".to_string()],
                },
                HealthCheck {
                    name: "Warning".to_string(),
                    status: HealthStatus::Warning,
                    details: vec!["warning".to_string()],
                },
                HealthCheck {
                    name: "Critical".to_string(),
                    status: HealthStatus::Critical,
                    details: vec!["critical".to_string()],
                },
            ],
            overall_status: HealthStatus::Critical,
            recommendations: vec!["Fix it".to_string()],
        };
        print_diagnostics(&diag);
    }
}

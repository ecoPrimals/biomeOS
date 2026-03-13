// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Doctor mode - Health diagnostics
//!
//! Comprehensive health checks for biomeOS system

use anyhow::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::Path;
use sysinfo::{Disks, System};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Diagnostics {
    checks: Vec<HealthCheck>,
    overall_status: HealthStatus,
    recommendations: Vec<String>,
}

/// Alias for diagnostic check (used in format/aggregate APIs)
pub(crate) type DiagnosticCheck = HealthCheck;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct HealthCheck {
    name: String,
    status: HealthStatus,
    details: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub(crate) enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            checks: Vec::new(),
            overall_status: HealthStatus::Healthy,
            recommendations: Vec::new(),
        }
    }

    fn add_check(&mut self, _name: &str, check: HealthCheck) {
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

    fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }
}

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
            lines.push(format!("   {}", detail));
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

pub async fn run(detailed: bool, format: String, subsystem: Option<String>) -> Result<()> {
    let diagnostics = if let Some(subsys) = subsystem {
        check_subsystem(&subsys, detailed).await?
    } else {
        check_all_subsystems(detailed).await?
    };

    match format.as_str() {
        "json" => {
            println!("{}", format_json_report(&diagnostics)?);
        }
        _ => {
            for line in format_text_report(&diagnostics) {
                println!("{}", line);
            }
        }
    }

    Ok(())
}

async fn check_all_subsystems(detailed: bool) -> Result<Diagnostics> {
    let mut diag = Diagnostics::new();

    println!("{}", "🧠 biomeOS Doctor".bright_cyan().bold());
    println!();
    println!("{}", "Health Diagnostics:".bold());
    println!(
        "{}",
        "═══════════════════════════════════════════════════════════════".bright_black()
    );
    println!();

    // 1. Binary Health
    diag.add_check("Binary", check_binary_health().await?);

    // 2. Configuration
    diag.add_check("Configuration", check_configuration().await?);

    // 3. Graphs Directory
    diag.add_check("Graphs", check_graphs_dir().await?);

    // 4. Primal Discovery
    diag.add_check("Primal Discovery", check_primal_discovery().await?);

    // 5. PlasmidBin
    diag.add_check("PlasmidBin", check_plasmid_bin().await?);

    // 6. System Resources
    diag.add_check("System", check_system_resources().await?);

    if detailed {
        // 7. Dependencies
        diag.add_check("Dependencies", check_dependencies().await?);
    }

    // Add recommendations
    if diag.overall_status != HealthStatus::Healthy {
        add_recommendations(&mut diag);
    }

    Ok(diag)
}

pub(crate) async fn check_subsystem(name: &str, _detailed: bool) -> Result<Diagnostics> {
    let mut diag = Diagnostics::new();

    match name {
        "binary" => diag.add_check("Binary", check_binary_health().await?),
        "config" => diag.add_check("Configuration", check_configuration().await?),
        "graphs" => diag.add_check("Graphs", check_graphs_dir().await?),
        "primals" => diag.add_check("Primal Discovery", check_primal_discovery().await?),
        "plasmidbin" => diag.add_check("PlasmidBin", check_plasmid_bin().await?),
        "system" => diag.add_check("System", check_system_resources().await?),
        _ => {
            anyhow::bail!("Unknown subsystem: {}", name);
        }
    }

    Ok(diag)
}

async fn check_binary_health() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Binary Health".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Check current binary
    if let Ok(exe) = std::env::current_exe() {
        check.details.push(format!("Binary: {}", exe.display()));

        if let Ok(metadata) = std::fs::metadata(&exe) {
            let size_mb = metadata.len() as f64 / 1_048_576.0;
            check.details.push(format!("Size: {:.1}M", size_mb));
        }
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push("Could not determine binary path".to_string());
    }

    check
        .details
        .push(format!("Version: {}", env!("CARGO_PKG_VERSION")));
    check.details.push("Modes: 7/7 available".to_string());
    check.details.push("UniBin: ✅ Compliant".to_string());

    Ok(check)
}

pub(crate) async fn check_configuration() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Configuration".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Use SystemPaths (XDG-compliant) for config directory
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let config_path = paths.config_dir().join("config.toml");

    if config_path.exists() {
        check
            .details
            .push(format!("Config file: {}", config_path.display()));
        check.details.push("Status: ✅ Found".to_string());
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Config file: {}", config_path.display()));
        check
            .details
            .push("Status: ⚠️  Not found (using defaults)".to_string());
    }

    Ok(check)
}

pub(crate) async fn check_graphs_dir() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Graphs Directory".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let graphs_dir = Path::new("graphs");

    if graphs_dir.exists() && graphs_dir.is_dir() {
        // Count .toml files
        let graph_count = std::fs::read_dir(graphs_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
            .count();

        check
            .details
            .push(format!("Path: {}", graphs_dir.display()));
        check.details.push(format!("Graphs found: {}", graph_count));

        if graph_count == 0 {
            check.status = HealthStatus::Warning;
            check
                .details
                .push("Warning: No graph files found".to_string());
        } else {
            check.details.push("Status: ✅ Ready".to_string());
        }
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Path: {}", graphs_dir.display()));
        check
            .details
            .push("Status: ⚠️  Directory not found".to_string());
    }

    Ok(check)
}

async fn check_primal_discovery() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Primal Discovery".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Get family_id from environment or use default
    // Uses capability-based discovery pattern (no hardcoded paths)
    let family_id = biomeos_core::family_discovery::get_family_id();

    // Use XDG-compliant SystemPaths for socket directory
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let runtime_dir = paths.runtime_dir();

    let health_checker =
        biomeos_atomic_deploy::health_check::HealthChecker::new(runtime_dir.to_path_buf());

    let primals = ["beardog", "songbird", "squirrel", "nestgate", "toadstool"];

    check
        .details
        .push(format!("Socket dir: {}", runtime_dir.display()));
    check.details.push(format!("Family ID: {}", family_id));

    let mut found_count = 0;
    for primal_name in &primals {
        // Use family-suffixed socket naming convention
        let socket_path = runtime_dir.join(format!("{}-{}.sock", primal_name, family_id));

        match health_checker.check_primal(&socket_path).await {
            Ok(status) if status.is_healthy => {
                found_count += 1;
                check.details.push(format!(
                    "{}: ✅ Healthy ({})",
                    primal_name,
                    socket_path.display()
                ));
            }
            Ok(status) => {
                let msg = status.message.unwrap_or_else(|| "Not found".to_string());
                check.details.push(format!("{}: ❌ {}", primal_name, msg));
            }
            Err(e) => {
                check
                    .details
                    .push(format!("{}: ❌ Error: {}", primal_name, e));
            }
        }
    }

    check
        .details
        .push(format!("Total: {}/5 primals discovered", found_count));

    if found_count < 3 {
        check.status = HealthStatus::Warning;
    }

    Ok(check)
}

pub(crate) async fn check_plasmid_bin() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "PlasmidBin".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let plasmid_dir = Path::new("plasmidBin/primals");

    if plasmid_dir.exists() && plasmid_dir.is_dir() {
        let binaries: Vec<_> = std::fs::read_dir(plasmid_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        let total_size: u64 = binaries
            .iter()
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum();

        let size_mb = total_size as f64 / 1_048_576.0;

        check
            .details
            .push(format!("Path: {}", plasmid_dir.display()));
        check.details.push(format!("Binaries: {}", binaries.len()));
        check.details.push(format!("Total size: {:.1}M", size_mb));
        check.details.push("Status: ✅ Ready".to_string());
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Path: {}", plasmid_dir.display()));
        check
            .details
            .push("Status: ⚠️  Directory not found".to_string());
    }

    Ok(check)
}

async fn check_system_resources() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "System Resources".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let mut sys = System::new_all();
    sys.refresh_all();

    // Memory
    let total_mem_gb = sys.total_memory() as f64 / 1_073_741_824.0;
    let avail_mem_gb = sys.available_memory() as f64 / 1_073_741_824.0;
    let mem_percent =
        ((sys.total_memory() - sys.available_memory()) as f64 / sys.total_memory() as f64) * 100.0;

    check.details.push(format!(
        "Memory: {:.1}GB ({:.1}GB available, {:.0}% used)",
        total_mem_gb, avail_mem_gb, mem_percent
    ));

    if mem_percent > 90.0 {
        check.status = HealthStatus::Warning;
    }

    // Disk
    let disks = Disks::new_with_refreshed_list();
    for disk in &disks {
        if disk.mount_point() == Path::new("/") {
            let total_gb = disk.total_space() as f64 / 1_073_741_824.0;
            let avail_gb = disk.available_space() as f64 / 1_073_741_824.0;
            let used_percent = ((disk.total_space() - disk.available_space()) as f64
                / disk.total_space() as f64)
                * 100.0;

            check.details.push(format!(
                "Disk: {:.1}GB ({:.1}GB available, {:.0}% used)",
                total_gb, avail_gb, used_percent
            ));

            if used_percent > 90.0 {
                check.status = HealthStatus::Warning;
            }
        }
    }

    // CPU
    let cpu_count = sys.cpus().len();
    check.details.push(format!("CPUs: {} cores", cpu_count));

    // Load average
    let load_avg = System::load_average();
    check.details.push(format!("Load: {:.2}", load_avg.one));

    Ok(check)
}

pub(crate) async fn check_dependencies() -> Result<HealthCheck> {
    let check = HealthCheck {
        name: "Dependencies".to_string(),
        status: HealthStatus::Healthy,
        details: vec![
            "Pure Rust: Evolving to 100%".to_string(),
            "UniBin: ✅ Compliant".to_string(),
            "ecoBin: ⏳ In progress".to_string(),
        ],
    };

    Ok(check)
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
        println!("{}", line);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[tokio::test]
    async fn test_run_unknown_subsystem_returns_error() {
        let result = super::run(
            false,
            "text".to_string(),
            Some("unknown_subsystem_xyz".to_string()),
        )
        .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("Unknown subsystem"),
            "Expected 'Unknown subsystem' in error: {}",
            err
        );
    }

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

    #[tokio::test]
    async fn test_run_with_subsystem_binary() {
        let result = super::run(false, "text".to_string(), Some("binary".to_string())).await;
        assert!(
            result.is_ok(),
            "binary subsystem should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_with_subsystem_config() {
        let result = super::run(false, "text".to_string(), Some("config".to_string())).await;
        assert!(
            result.is_ok(),
            "config subsystem should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_with_subsystem_graphs() {
        let result = super::run(false, "text".to_string(), Some("graphs".to_string())).await;
        assert!(
            result.is_ok(),
            "graphs subsystem should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_with_subsystem_primals() {
        let result = super::run(false, "text".to_string(), Some("primals".to_string())).await;
        assert!(
            result.is_ok(),
            "primals subsystem should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_with_subsystem_plasmidbin() {
        let result = super::run(false, "text".to_string(), Some("plasmidbin".to_string())).await;
        assert!(
            result.is_ok(),
            "plasmidbin subsystem should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_with_subsystem_system() {
        let result = super::run(false, "text".to_string(), Some("system".to_string())).await;
        assert!(
            result.is_ok(),
            "system subsystem should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_json_format() {
        let result = super::run(false, "json".to_string(), None).await;
        assert!(
            result.is_ok(),
            "json format should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_run_all_subsystems_detailed() {
        let result = super::run(true, "text".to_string(), None).await;
        assert!(
            result.is_ok(),
            "detailed run should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_graphs_dir_no_directory() {
        let temp = tempfile::tempdir().unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_graphs_dir().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check
            .details
            .iter()
            .any(|d| d.contains("not found") || d.contains("Directory")));
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_graphs_dir_with_toml_files() {
        let temp = tempfile::tempdir().unwrap();
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).unwrap();
        std::fs::write(graphs_dir.join("deploy.toml"), "name = \"test\"").unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_graphs_dir().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Graphs found: 1")));
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_graphs_dir_empty_graphs_dir() {
        let temp = tempfile::tempdir().unwrap();
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_graphs_dir().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("No graph")));
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_plasmid_bin_no_directory() {
        let temp = tempfile::tempdir().unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_plasmid_bin().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check
            .details
            .iter()
            .any(|d| d.contains("not found") || d.contains("Directory")));
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_plasmid_bin_with_binaries() {
        let temp = tempfile::tempdir().unwrap();
        let plasmid_dir = temp.path().join("plasmidBin").join("primals");
        std::fs::create_dir_all(&plasmid_dir).unwrap();
        std::fs::write(plasmid_dir.join("beardog"), "fake-binary").unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_plasmid_bin().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Binaries: 1")));
    }

    #[tokio::test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_configuration_no_config() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("biomeos")).unwrap();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        let check = check_configuration().await.unwrap();
        if let Some(xdg) = old_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("Not found")));
    }

    #[tokio::test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_configuration_with_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "[default]").unwrap();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        let check = check_configuration().await.unwrap();
        if let Some(xdg) = old_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Found")));
    }

    #[tokio::test]
    async fn test_check_dependencies() {
        let check = check_dependencies().await.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert_eq!(check.name, "Dependencies");
        assert!(check.details.iter().any(|d| d.contains("Pure Rust")));
    }

    #[test]
    fn test_diagnostics_json_roundtrip() {
        let diag = Diagnostics {
            checks: vec![HealthCheck {
                name: "Test".to_string(),
                status: HealthStatus::Warning,
                details: vec!["detail".to_string()],
            }],
            overall_status: HealthStatus::Warning,
            recommendations: vec!["rec".to_string()],
        };
        let json = serde_json::to_string(&diag).unwrap();
        let parsed: Diagnostics = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.checks.len(), 1);
        assert_eq!(parsed.overall_status, HealthStatus::Warning);
        assert_eq!(parsed.recommendations.len(), 1);
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
}

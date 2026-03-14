// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Doctor mode - Health diagnostics
//!
//! Comprehensive health checks for biomeOS system

mod checks_config;
mod checks_primal;
mod checks_system;
mod reporting;
mod types;

use anyhow::Result;
use colored::Colorize;

// Re-export public API for crate-internal consumers
pub(crate) use checks_config::{check_configuration, check_graphs_dir};
pub(crate) use checks_primal::check_primal_discovery;
pub(crate) use checks_system::{check_dependencies, check_system_resources};
pub(crate) use reporting::{add_recommendations, format_json_report, format_text_report};
#[allow(unused_imports)]
pub(crate) use types::{DiagnosticCheck, Diagnostics, HealthCheck, HealthStatus};

use checks_config::check_binary_health;
use checks_primal::check_plasmid_bin;

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
                println!("{line}");
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
            anyhow::bail!("Unknown subsystem: {name}");
        }
    }

    Ok(diag)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

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
            "Expected 'Unknown subsystem' in error: {err}"
        );
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
}

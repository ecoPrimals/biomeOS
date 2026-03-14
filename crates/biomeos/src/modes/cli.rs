// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! CLI mode - System management commands
//!
//! EVOLVED (Jan 27, 2026): Provides a bridge to the full biomeos-cli functionality.
//! The CLI commands are implemented in biomeos-cli and can be invoked via:
//! - Direct invocation: `biomeos cli <command>`
//! - Standalone binary: `biomeos <command>` (via PATH/symlink)
//!
//! This architecture allows the unified binary to be the single entry point
//! while maintaining the full CLI feature set.

use anyhow::Result;
use colored::Colorize;

/// CLI command type
#[derive(Debug)]
pub struct CliCommand;

/// Run CLI mode
///
/// EVOLVED (Jan 27, 2026): Now provides helpful guidance and quick commands
pub async fn run(_command: CliCommand) -> Result<()> {
    println!("{}", "🧠 biomeOS CLI Mode".bright_cyan().bold());
    println!();

    // Show system status summary
    show_system_summary().await?;

    println!();
    println!("{}", "Quick Commands:".bright_white().bold());
    println!();
    println!(
        "  {} - View available commands",
        "biomeos --help".bright_cyan()
    );
    println!(
        "  {} - Discover running primals",
        "biomeos discover".bright_cyan()
    );
    println!("  {} - Check system health", "biomeos doctor".bright_cyan());
    println!(
        "  {} - Deploy a graph",
        "biomeos deploy <graph.toml>".bright_cyan()
    );
    println!(
        "  {} - Start Neural API server",
        "biomeos neural-api".bright_cyan()
    );
    println!(
        "  {} - Verify genetic lineage",
        "biomeos verify-lineage <path>".bright_cyan()
    );
    println!();

    // Show tip based on environment
    show_context_tip().await;

    Ok(())
}

/// Show system status summary
async fn show_system_summary() -> Result<()> {
    use std::path::Path;

    println!("{}", "System Status:".bright_white().bold());

    // Check for running primals by looking for sockets
    let socket_dirs = [
        std::env::var("XDG_RUNTIME_DIR")
            .map(|d| format!("{}/biomeos", d))
            .ok(),
        Some("/tmp".to_string()),
    ];

    let mut primal_count = 0;
    for dir in socket_dirs.iter().flatten() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".sock")
                    && biomeos_types::primal_names::CORE_PRIMALS
                        .iter()
                        .any(|p| name.contains(p))
                {
                    primal_count += 1;
                }
            }
        }
    }

    if primal_count > 0 {
        println!(
            "  {} Active primals: {}",
            "✅".green(),
            primal_count.to_string().bright_green()
        );
    } else {
        println!("  {} No running primals detected", "⚠️".yellow());
    }

    // Check for family seed
    if Path::new(".family.seed").exists() {
        println!("  {} Family seed: present", "✅".green());
    } else {
        println!(
            "  {} No family seed (run 'biomeos spore imprint' first)",
            "⚠️".yellow()
        );
    }

    // Check for graphs
    if Path::new("graphs").exists() {
        let graph_count = std::fs::read_dir("graphs")
            .map(|d| d.filter(|e| e.is_ok()).count())
            .unwrap_or(0);
        if graph_count > 0 {
            println!(
                "  {} Deployment graphs: {}",
                "✅".green(),
                graph_count.to_string().bright_green()
            );
        }
    }

    Ok(())
}

/// Show contextual tip based on environment
async fn show_context_tip() {
    use std::path::Path;

    let tip = choose_context_tip(
        !Path::new(".family.seed").exists(),
        !Path::new("graphs").exists(),
    );
    println!("{}", tip.to_colored_string());
}

/// Context tip variant - which tip to show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContextTip {
    FamilySeed,
    Graphs,
    NeuralApi,
}

impl ContextTip {
    fn message(self) -> &'static str {
        match self {
            Self::FamilySeed => "💡 Tip: Run 'biomeos spore imprint' to create your family seed",
            Self::Graphs => "💡 Tip: Create a 'graphs/' directory with deployment TOML files",
            Self::NeuralApi => "💡 Tip: Run 'biomeos neural-api' to start the orchestration server",
        }
    }

    fn to_colored_string(self) -> colored::ColoredString {
        match self {
            Self::FamilySeed | Self::Graphs => self.message().bright_yellow(),
            Self::NeuralApi => self.message().bright_green(),
        }
    }
}

/// Choose which tip to show based on environment state.
/// Extracted for testability.
pub(crate) fn choose_context_tip(family_seed_missing: bool, graphs_missing: bool) -> ContextTip {
    if family_seed_missing {
        ContextTip::FamilySeed
    } else if graphs_missing {
        ContextTip::Graphs
    } else {
        ContextTip::NeuralApi
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_command_debug() {
        let cmd = CliCommand;
        let s = format!("{:?}", cmd);
        assert_eq!(s, "CliCommand");
    }

    #[test]
    fn test_choose_context_tip_family_seed_missing() {
        let tip = choose_context_tip(true, true);
        assert_eq!(tip, ContextTip::FamilySeed);
        assert!(tip.message().contains("spore imprint"));
    }

    #[test]
    fn test_choose_context_tip_graphs_missing_but_family_present() {
        let tip = choose_context_tip(false, true);
        assert_eq!(tip, ContextTip::Graphs);
        assert!(tip.message().contains("graphs/"));
    }

    #[test]
    fn test_choose_context_tip_all_present() {
        let tip = choose_context_tip(false, false);
        assert_eq!(tip, ContextTip::NeuralApi);
        assert!(tip.message().contains("neural-api"));
    }

    #[test]
    fn test_choose_context_tip_family_takes_priority_over_graphs() {
        // When both missing, family seed tip takes priority
        let tip = choose_context_tip(true, true);
        assert_eq!(tip, ContextTip::FamilySeed);
    }

    #[test]
    fn test_context_tip_messages() {
        assert!(ContextTip::FamilySeed.message().starts_with("💡"));
        assert!(ContextTip::Graphs.message().starts_with("💡"));
        assert!(ContextTip::NeuralApi.message().starts_with("💡"));
    }

    #[tokio::test]
    async fn test_run_does_not_error() {
        let result = run(CliCommand).await;
        result.expect("run should succeed");
    }
}

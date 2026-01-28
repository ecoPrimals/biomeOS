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
                    && (name.contains("beardog")
                        || name.contains("songbird")
                        || name.contains("nestgate")
                        || name.contains("toadstool"))
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

    // Tip based on what's missing
    if !Path::new(".family.seed").exists() {
        println!(
            "{}",
            "💡 Tip: Run 'biomeos spore imprint' to create your family seed".bright_yellow()
        );
    } else if !Path::new("graphs").exists() {
        println!(
            "{}",
            "💡 Tip: Create a 'graphs/' directory with deployment TOML files".bright_yellow()
        );
    } else {
        println!(
            "{}",
            "💡 Tip: Run 'biomeos neural-api' to start the orchestration server".bright_green()
        );
    }
}

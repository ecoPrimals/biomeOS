// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Version mode - Version information

use anyhow::Result;
use colored::Colorize;

pub async fn run(detailed: bool) -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");

    if detailed {
        println!(
            "{}",
            "╔══════════════════════════════════════════════════════════════════════════╗"
                .bright_cyan()
        );
        println!(
            "{}",
            "║                                                                          ║"
                .bright_cyan()
        );
        println!(
            "{}",
            "║                  🧠 biomeOS Version Information 🧠                       ║"
                .bright_cyan()
        );
        println!(
            "{}",
            "║                                                                          ║"
                .bright_cyan()
        );
        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════════════════════╝"
                .bright_cyan()
        );
        println!();
        println!("{} {}", "Version:".bold(), version.bright_green());
        println!(
            "{} {}",
            "Architecture:".bold(),
            "UniBin v1.0.0".bright_yellow()
        );
        println!(
            "{} {}",
            "Modes:".bold(),
            "7 (cli, neural-api, deploy, api, verify-lineage, doctor, version)".bright_blue()
        );
        println!(
            "{} {}",
            "Pure Rust:".bold(),
            "Evolving to 100%".bright_magenta()
        );
        println!(
            "{} {}",
            "Build Date:".bold(),
            option_env!("BUILD_DATE").unwrap_or("unknown")
        );
        println!(
            "{} {}",
            "Rust Version:".bold(),
            option_env!("RUSTC_VERSION").unwrap_or("unknown")
        );
        println!();
        println!("{}", "Capabilities:".bold().underline());
        println!(
            "  {} System management and orchestration",
            "•".bright_cyan()
        );
        println!(
            "  {} Graph-based deployment (Neural API)",
            "•".bright_cyan()
        );
        println!("  {} Genetic lineage verification", "•".bright_cyan());
        println!("  {} Health diagnostics", "•".bright_cyan());
        println!("  {} Primal discovery and coordination", "•".bright_cyan());
        println!();
        println!(
            "{}",
            "🦀 100% Rust  •  🌱 Self-Evolving  •  ⚛️  TRUE PRIMAL".bright_green()
        );
    } else {
        println!("biomeOS v{version}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_simple_does_not_error() {
        let result = run(false).await;
        result.expect("run(false) should succeed");
    }

    #[tokio::test]
    async fn test_run_detailed_does_not_error() {
        let result = run(true).await;
        result.expect("run(true) should succeed");
    }

    #[test]
    fn test_version_format_string() {
        let version = env!("CARGO_PKG_VERSION");
        let simple = format!("biomeOS v{version}");
        assert!(
            simple.starts_with("biomeOS v"),
            "Version string should start with 'biomeOS v'"
        );
        assert!(!version.is_empty(), "CARGO_PKG_VERSION should be non-empty");
    }
}

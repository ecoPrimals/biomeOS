// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Integration Test Runner
//! 
//! Pure Rust replacement for shell-based integration testing.
//! Provides comprehensive ecosystem testing with detailed reporting.

use anyhow::Result;
use clap::{Parser, Subcommand};
use biomeos_tools::{
    integration::{IntegrationConfig, run_integration_tests, get_test_coverage},
    print_section, print_success,
};

#[derive(Parser)]
#[command(name = "integration-test-runner")]
#[command(about = "biomeOS Integration Test Runner - Rust until the very edge")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Workspace root directory
    #[arg(short, long, default_value = ".")]
    workspace: String,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Timeout in seconds
    #[arg(short, long, default_value = "300")]
    timeout: u64,

    /// Run tests in parallel
    #[arg(short, long, default_value = "true")]
    parallel: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all integration tests
    All,
    /// Run only core tests
    Core,
    /// Run only UI tests
    Ui,
    /// Get test coverage report
    Coverage,
    /// Quick health check
    Quick,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(if cli.verbose { "debug" } else { "info" })
        .init();

    let config = IntegrationConfig {
        workspace_root: cli.workspace,
        timeout_seconds: cli.timeout,
        parallel_tests: cli.parallel,
        verbose: cli.verbose,
    };

    match cli.command {
        Some(Commands::All) | None => {
            run_all_tests(&config).await?;
        }
        Some(Commands::Core) => {
            run_core_tests(&config).await?;
        }
        Some(Commands::Ui) => {
            run_ui_tests(&config).await?;
        }
        Some(Commands::Coverage) => {
            run_coverage_analysis(&config).await?;
        }
        Some(Commands::Quick) => {
            run_quick_check(&config).await?;
        }
    }

    Ok(())
}

/// Run all integration tests
async fn run_all_tests(config: &IntegrationConfig) -> Result<()> {
    print_section("🚀 COMPLETE INTEGRATION TEST SUITE");
    run_integration_tests(config).await?;
    
    // Get coverage after tests
    let coverage = get_test_coverage(config).await?;
    
    if coverage >= 50.0 {
        print_success(&format!("🎯 SUCCESS: {:.1}% coverage achieved (target: 50%)", coverage));
    } else {
        biomeos_tools::print_warning(&format!("⚠️ Coverage {:.1}% below 50% target", coverage));
    }
    
    Ok(())
}

/// Run only core tests
async fn run_core_tests(config: &IntegrationConfig) -> Result<()> {
    print_section("🧬 CORE INTEGRATION TESTS");
    
    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");
    
    biomeos_tools::execute_command(
        "cargo", 
        &["test", "-p", "biomeos-core", "--", "--test-threads=1"],
        Some(&workspace_path)
    ).await?;
    
    print_success("Core integration tests completed successfully");
    Ok(())
}

/// Run only UI tests
async fn run_ui_tests(config: &IntegrationConfig) -> Result<()> {
    print_section("🎨 UI INTEGRATION TESTS");
    
    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");
    
    // Test UI compilation
    biomeos_tools::execute_command(
        "cargo",
        &["test", "-p", "biomeos-ui"],
        Some(&workspace_path)
    ).await?;
    
    // Test UI builds
    biomeos_tools::execute_command(
        "cargo",
        &["build", "-p", "biomeos-ui"],
        Some(&workspace_path)
    ).await?;
    
    print_success("UI integration tests completed successfully");
    Ok(())
}

/// Run coverage analysis
async fn run_coverage_analysis(config: &IntegrationConfig) -> Result<()> {
    print_section("📊 TEST COVERAGE ANALYSIS");
    
    let coverage = get_test_coverage(config).await?;
    
    println!("\n📈 COVERAGE REPORT:");
    println!("  Current Coverage: {:.1}%", coverage);
    println!("  Target Coverage: 50.0%");
    println!("  Status: {}", if coverage >= 50.0 { "✅ PASSED" } else { "❌ NEEDS IMPROVEMENT" });
    
    if coverage >= 50.0 {
        print_success("Coverage target achieved!");
    } else {
        biomeos_tools::print_warning("Add more tests to reach 50% coverage target");
    }
    
    Ok(())
}

/// Run quick health check
async fn run_quick_check(config: &IntegrationConfig) -> Result<()> {
    print_section("⚡ QUICK HEALTH CHECK");
    
    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");
    
    // Quick compilation check
    biomeos_tools::print_info("Checking compilation...");
    biomeos_tools::execute_command(
        "cargo",
        &["check", "--workspace"],
        Some(&workspace_path)
    ).await?;
    
    // Quick test run
    biomeos_tools::print_info("Running quick tests...");
    biomeos_tools::execute_command(
        "cargo",
        &["test", "--workspace", "--lib", "--", "--test-threads=1"],
        Some(&workspace_path)
    ).await?;
    
    print_success("Quick health check completed - system is operational");
    Ok(())
} 
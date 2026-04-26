// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Testing Module
//!
//! Enhanced testing utilities and coverage analysis for biomeOS.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::{discover_workspace_root, execute_command, print_info, print_section, print_success};

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Workspace root — discovered at runtime, never hardcoded.
    pub workspace_root: PathBuf,
    /// Minimum acceptable coverage percentage.
    pub coverage_threshold: f64,
    /// Per-test timeout in seconds.
    pub test_timeout: u64,
    /// Whether to run tests in parallel.
    pub parallel: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            workspace_root: discover_workspace_root().unwrap_or_else(|_| PathBuf::from(".")),
            coverage_threshold: 90.0,
            test_timeout: 300,
            parallel: true,
        }
    }
}

/// Run comprehensive test suite
pub async fn run_all_tests(config: &TestConfig) -> Result<()> {
    print_section("biomeOS COMPREHENSIVE TEST SUITE");

    let workspace_path = &config.workspace_root;

    run_unit_tests(workspace_path).await?;
    run_integration_tests(workspace_path).await?;
    run_ui_tests(workspace_path).await?;
    run_benchmarks(workspace_path).await?;

    print_success("All test suites completed successfully");
    Ok(())
}

/// Run unit tests for all packages
async fn run_unit_tests(workspace_path: &Path) -> Result<()> {
    print_section("Unit Tests");

    execute_command(
        "cargo",
        &["test", "--workspace", "--lib"],
        Some(workspace_path),
    )
    .await?;

    print_success("Unit tests passed");
    Ok(())
}

/// Run integration tests
async fn run_integration_tests(workspace_path: &Path) -> Result<()> {
    print_section("Integration Tests");

    execute_command(
        "cargo",
        &["test", "--workspace", "--test", "*"],
        Some(workspace_path),
    )
    .await?;

    print_success("Integration tests passed");
    Ok(())
}

/// Run UI-specific tests
async fn run_ui_tests(workspace_path: &Path) -> Result<()> {
    print_section("UI Tests");

    // Test UI compilation
    execute_command("cargo", &["test", "-p", "biomeos-ui"], Some(workspace_path)).await?;

    // Test UI binary builds
    execute_command(
        "cargo",
        &["build", "-p", "biomeos-ui"],
        Some(workspace_path),
    )
    .await?;

    print_success("UI tests passed");
    Ok(())
}

/// Run benchmark tests
async fn run_benchmarks(workspace_path: &Path) -> Result<()> {
    print_section("Benchmark Tests");

    execute_command("cargo", &["bench", "--workspace"], Some(workspace_path)).await?;

    print_success("Benchmarks completed");
    Ok(())
}

/// Generate test coverage report using `cargo llvm-cov`.
pub async fn generate_coverage_report(config: &TestConfig) -> Result<f64> {
    print_section("Test Coverage Analysis (llvm-cov)");

    let workspace_path = &config.workspace_root;

    if !crate::binary_exists("cargo-llvm-cov") {
        print_info("Installing cargo-llvm-cov...");
        execute_command("cargo", &["install", "cargo-llvm-cov"], None).await?;
    }

    let output = execute_command(
        "cargo",
        &["llvm-cov", "--workspace", "--summary-only"],
        Some(workspace_path),
    )
    .await?;

    let coverage = extract_llvm_cov_region_percent(&output);

    print_info(&format!("Region coverage: {coverage:.1}%"));

    if coverage >= config.coverage_threshold {
        print_success(&format!(
            "Coverage {coverage:.1}% meets threshold of {:.1}%",
            config.coverage_threshold
        ));
    } else {
        crate::print_warning(&format!(
            "Coverage {coverage:.1}% below threshold of {:.1}%",
            config.coverage_threshold
        ));
    }

    Ok(coverage)
}

/// Parse the TOTAL line from `cargo llvm-cov --summary-only`.
fn extract_llvm_cov_region_percent(output: &str) -> f64 {
    for line in output.lines().rev() {
        let trimmed = line.trim();
        if trimmed.starts_with("TOTAL") {
            for token in trimmed.split_whitespace() {
                if let Some(pct_str) = token.strip_suffix('%') {
                    if let Ok(pct) = pct_str.parse::<f64>() {
                        return pct;
                    }
                }
            }
        }
    }
    0.0
}

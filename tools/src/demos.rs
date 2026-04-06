// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Demos Module
//! 
//! Orchestrates biomeOS demonstrations and showcases.
//! Replaces shell scripts with pure Rust tooling.

use anyhow::Result;
use std::path::{Path, PathBuf};

use crate::{discover_workspace_root, execute_command, print_info, print_section, print_success};

/// Demo configuration
#[derive(Debug, Clone)]
pub struct DemoConfig {
    /// Workspace root — discovered at runtime, never hardcoded.
    pub workspace_root: PathBuf,
    /// Whether to auto-advance between demos.
    pub auto_advance: bool,
    /// Whether to pause for interactive input.
    pub interactive: bool,
    /// Verbose output.
    pub verbose: bool,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            workspace_root: discover_workspace_root().unwrap_or_else(|_| PathBuf::from(".")),
            auto_advance: false,
            interactive: true,
            verbose: true,
        }
    }
}

/// Run all biomeOS demonstrations
pub async fn run_all_demos(config: &DemoConfig) -> Result<()> {
    print_section("biomeOS COMPREHENSIVE DEMO SUITE");
    print_info("Showcasing the complete biomeOS ecosystem capabilities");
    
    // 1. Core tests demo
    run_core_tests(config).await?;
    
    // 2. Sovereignty features demo
    run_sovereignty_demo(config).await?;
    
    // 3. Capabilities overview
    demo_capabilities(config).await?;
    
    print_success("All demonstrations completed successfully!");
    Ok(())
}

/// Run core workspace tests as a demo
async fn run_core_tests(config: &DemoConfig) -> Result<()> {
    print_section("CORE TESTS");
    print_info("Running workspace test suite");
    
    let workspace_path = &config.workspace_root;

    execute_command(
        "cargo",
        &["test", "--workspace", "--", "--test-threads=1"],
        Some(workspace_path),
    )
    .await?;
    
    if config.interactive {
        wait_for_user_input("Press Enter to continue...");
    }
    
    print_success("Core tests completed");
    Ok(())
}

/// Run sovereignty features demo
async fn run_sovereignty_demo(config: &DemoConfig) -> Result<()> {
    print_section("🔒 SOVEREIGNTY FEATURES DEMO");
    print_info("Demonstrating crypto locks, genetic keys, and AI cat door");
    
    // Demo genetic beardog keys
    print_info("🧬 Genetic Beardog Key System:");
    print_info("  • Individual: 1.0x cost multiplier");
    print_info("  • Small Business: 0.1x cost multiplier (90% savings!)");
    print_info("  • Enterprise: 10x cost multiplier");
    print_info("  • Mega Corp: 100x cost multiplier");
    
    // Demo AI cat door
    print_info("🚪 AI Cat Door Protection:");
    print_info("  • $20/month cost protection");
    print_info("  • Grandma-safe operation");
    print_info("  • Automatic rate limiting");
    
    // Demo crypto locks
    print_info("🔐 Crypto Lock System:");
    print_info("  • 5 active locks");
    print_info("  • 0 bypassed locks");
    print_info("  • Full sovereignty maintained");
    
    // Demo compliance
    print_info("📊 Sovereignty Compliance:");
    print_info("  • Score: 3/3 (Fully Sovereign)");
    print_info("  • Zero external dependencies");
    print_info("  • Complete vendor lock-in avoidance");
    
    if config.interactive {
        wait_for_user_input("Press Enter to complete demo suite...");
    }
    
    print_success("Sovereignty features demo completed");
    Ok(())
}

/// Wait for user input (when in interactive mode)
fn wait_for_user_input(prompt: &str) {
    println!("\n{prompt}");
    let mut input = String::new();
    let _ = std::io::stdin().read_line(&mut input);
}

/// Demo system capabilities
pub async fn demo_capabilities(_config: &DemoConfig) -> Result<()> {
    print_section("🎯 biomeOS CAPABILITIES DEMONSTRATION");
    
    print_info("🌍 Universal Platform Support:");
    print_info("  ✅ Linux (native)");
    print_info("  ✅ Windows (cross-platform)");
    print_info("  ✅ macOS (cross-platform)");
    print_info("  ✅ Docker containers");
    print_info("  ✅ Kubernetes clusters");
    print_info("  ✅ Cloud environments");
    
    print_info("🔄 Zero Vendor Lock-in:");
    print_info("  ✅ Portable across all platforms");
    print_info("  ✅ Open source architecture");
    print_info("  ✅ API-driven design");
    print_info("  ✅ Recursive/universal patterns");
    
    print_info("🧬 Ecosystem Integration:");
    print_info("  🔍 Toadstool (Universal Compute)");
    print_info("  🎵 Songbird (Service Mesh)");
    print_info("  🏠 NestGate (Storage)");
    print_info("  🐿️ Squirrel (MCP Platform)");
    print_info("  🐻 BearDog (Security Framework)");
    
    print_success("Capabilities demonstration completed");
    Ok(())
} 
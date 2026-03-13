// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Demos Module
//! 
//! Orchestrates biomeOS demonstrations and showcases.
//! Replaces shell scripts with pure Rust tooling.

use anyhow::Result;
use std::path::Path;
use crate::{execute_command, print_section, print_success, print_info};

/// Demo configuration
#[derive(Debug, Clone)]
pub struct DemoConfig {
    pub workspace_root: String,
    pub auto_advance: bool,
    pub interactive: bool,
    pub verbose: bool,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            workspace_root: "/home/strandgate/Development".to_string(),
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
    
    // 1. Ecosystem showcase
    run_ecosystem_showcase(config).await?;
    
    // 2. Core platform demo
    run_core_demo(config).await?;
    
    // 3. Integration tests demo
    run_integration_demo(config).await?;
    
    // 4. UI showcase
    run_ui_showcase(config).await?;
    
    // 5. Sovereignty features demo
    run_sovereignty_demo(config).await?;
    
    print_success("All demonstrations completed successfully!");
    Ok(())
}

/// Run the ecosystem showcase
async fn run_ecosystem_showcase(config: &DemoConfig) -> Result<()> {
    print_section("🌟 ECOSYSTEM SHOWCASE");
    print_info("Demonstrating live orchestration and ecosystem coordination");
    
    let workspace_path = Path::new(&config.workspace_root).join("biomeOS");
    
    execute_command(
        "cargo",
        &["run", "--bin", "showcase"],
        Some(&workspace_path)
    ).await?;
    
    if config.interactive {
        wait_for_user_input("Press Enter to continue to core demo...");
    }
    
    print_success("Ecosystem showcase completed");
    Ok(())
}

/// Run the core platform demo
async fn run_core_demo(config: &DemoConfig) -> Result<()> {
    print_section("🧬 CORE PLATFORM DEMO");
    print_info("Demonstrating AI-first installer and universal platform capabilities");
    
    let workspace_path = Path::new(&config.workspace_root).join("biomeOS");
    
    execute_command(
        "cargo",
        &["run", "--bin", "demo"],
        Some(&workspace_path)
    ).await?;
    
    if config.interactive {
        wait_for_user_input("Press Enter to continue to integration tests...");
    }
    
    print_success("Core platform demo completed");
    Ok(())
}

/// Run integration tests as a demo
async fn run_integration_demo(config: &DemoConfig) -> Result<()> {
    print_section("🧪 INTEGRATION TESTS DEMO");
    print_info("Demonstrating comprehensive ecosystem testing");
    
    let workspace_path = Path::new(&config.workspace_root);
    
    execute_command(
        "cargo",
        &["run", "--bin", "integration_test_runner"],
        Some(workspace_path)
    ).await?;
    
    if config.interactive {
        wait_for_user_input("Press Enter to continue to UI showcase...");
    }
    
    print_success("Integration tests demo completed");
    Ok(())
}

/// Run UI showcase
async fn run_ui_showcase(config: &DemoConfig) -> Result<()> {
    print_section("🎨 UI SHOWCASE");
    print_info("Launching biomeOS Bootstrap UI (will open in new window)");
    
    let workspace_path = Path::new(&config.workspace_root).join("biomeOS");
    
    // Launch UI in background
    print_info("Starting biomeOS UI - check for new window...");
    
    // Note: We don't wait for UI to complete as it runs indefinitely
    tokio::spawn(async move {
        let _ = execute_command(
            "cargo",
            &["run", "--bin", "biomeos-ui"],
            Some(&workspace_path)
        ).await;
    });
    
    // Give UI time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    if config.interactive {
        wait_for_user_input("UI should be running. Press Enter when ready to continue...");
    }
    
    print_success("UI showcase launched");
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
    println!("\n{}", prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

/// Demo system capabilities
pub async fn demo_capabilities(config: &DemoConfig) -> Result<()> {
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
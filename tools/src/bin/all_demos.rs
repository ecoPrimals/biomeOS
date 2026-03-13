// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! All Demos Runner
//! 
//! Pure Rust replacement for shell-based demo scripts.
//! Orchestrates comprehensive biomeOS ecosystem demonstrations.

use anyhow::Result;
use clap::{Parser, Subcommand};
use biomeos_tools::{
    demos::{DemoConfig, run_all_demos, demo_capabilities},
    print_section, print_success,
};

#[derive(Parser)]
#[command(name = "all-demos")]
#[command(about = "biomeOS Demo Suite - Showcasing ecosystem capabilities")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Workspace root directory
    #[arg(short, long, default_value = "/home/strandgate/Development")]
    workspace: String,

    /// Run in non-interactive mode
    #[arg(short, long)]
    non_interactive: bool,

    /// Auto-advance between demos
    #[arg(short, long)]
    auto_advance: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all demonstrations
    All,
    /// Run ecosystem showcase only
    Ecosystem,
    /// Run core platform demo only
    Core,
    /// Run UI showcase only
    Ui,
    /// Run sovereignty features demo
    Sovereignty,
    /// Demo system capabilities
    Capabilities,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(if cli.verbose { "debug" } else { "info" })
        .init();

    let config = DemoConfig {
        workspace_root: cli.workspace,
        auto_advance: cli.auto_advance,
        interactive: !cli.non_interactive,
        verbose: cli.verbose,
    };

    match cli.command {
        Some(Commands::All) | None => {
            run_complete_demo_suite(&config).await?;
        }
        Some(Commands::Ecosystem) => {
            run_ecosystem_only(&config).await?;
        }
        Some(Commands::Core) => {
            run_core_only(&config).await?;
        }
        Some(Commands::Ui) => {
            run_ui_only(&config).await?;
        }
        Some(Commands::Sovereignty) => {
            run_sovereignty_only(&config).await?;
        }
        Some(Commands::Capabilities) => {
            demo_capabilities(&config).await?;
        }
    }

    Ok(())
}

/// Run complete demo suite
async fn run_complete_demo_suite(config: &DemoConfig) -> Result<()> {
    print_section("🎬 biomeOS COMPLETE DEMO SUITE");
    biomeos_tools::print_info("Showcasing the full biomeOS ecosystem");
    
    // Show intro
    show_demo_intro();
    
    if config.interactive {
        wait_for_user("Press Enter to begin comprehensive demo suite...");
    }
    
    // Run all demos
    run_all_demos(config).await?;
    
    // Show outro
    show_demo_outro();
    
    print_success("🎉 Complete demo suite finished successfully!");
    Ok(())
}

/// Run ecosystem showcase only
async fn run_ecosystem_only(config: &DemoConfig) -> Result<()> {
    print_section("🌟 ECOSYSTEM SHOWCASE ONLY");
    
    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");
    
    biomeos_tools::execute_command(
        "cargo",
        &["run", "--bin", "showcase"],
        Some(&workspace_path)
    ).await?;
    
    print_success("Ecosystem showcase completed");
    Ok(())
}

/// Run core platform demo only
async fn run_core_only(config: &DemoConfig) -> Result<()> {
    print_section("🧬 CORE PLATFORM DEMO ONLY");
    
    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");
    
    biomeos_tools::execute_command(
        "cargo",
        &["run", "--bin", "demo"],
        Some(&workspace_path)
    ).await?;
    
    print_success("Core platform demo completed");
    Ok(())
}

/// Run UI showcase only
async fn run_ui_only(config: &DemoConfig) -> Result<()> {
    print_section("🎨 UI SHOWCASE ONLY");
    
    let workspace_path = std::path::Path::new(&config.workspace_root).join("biomeOS");
    
    biomeos_tools::print_info("Launching biomeOS Bootstrap UI...");
    
    // Launch UI (non-blocking)
    tokio::spawn(async move {
        let _ = biomeos_tools::execute_command(
            "cargo",
            &["run", "--bin", "biomeos-ui"],
            Some(&workspace_path)
        ).await;
    });
    
    // Give UI time to start
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    biomeos_tools::print_info("UI should be running in a new window");
    
    if config.interactive {
        wait_for_user("Press Enter when done exploring the UI...");
    }
    
    print_success("UI showcase completed");
    Ok(())
}

/// Run sovereignty features demo only
async fn run_sovereignty_only(config: &DemoConfig) -> Result<()> {
    print_section("🔒 SOVEREIGNTY FEATURES DEMO");
    
    // Detailed sovereignty demonstration
    biomeos_tools::print_info("🧬 Genetic Beardog Key System:");
    println!("  📊 Cost Scaling Matrix:");
    println!("    Individual:     1.0x  (baseline)");
    println!("    Small Business: 0.1x  (90% savings!)");
    println!("    Enterprise:    10.0x  (premium)");
    println!("    Mega Corp:    100.0x  (maximum)");
    
    if config.interactive {
        wait_for_user("Press Enter to continue to AI Cat Door...");
    }
    
    biomeos_tools::print_info("🚪 AI Cat Door Protection:");
    println!("  💰 Cost Protection: $20/month maximum");
    println!("  👵 Grandma-Safe: Intuitive operation");
    println!("  🔄 Rate Limiting: Automatic protection");
    println!("  🛡️ Provider Shield: Multi-provider failover");
    
    if config.interactive {
        wait_for_user("Press Enter to continue to Crypto Locks...");
    }
    
    biomeos_tools::print_info("🔐 Crypto Lock System:");
    println!("  🔒 Active Locks: 5");
    println!("  🚫 Bypassed: 0");
    println!("  ✅ Integrity: 100%");
    println!("  🛡️ Sovereignty: Full");
    
    if config.interactive {
        wait_for_user("Press Enter to complete sovereignty demo...");
    }
    
    print_success("Sovereignty features demo completed");
    Ok(())
}

/// Show demo introduction
fn show_demo_intro() {
    println!("\n🎯 WELCOME TO THE biomeOS ECOSYSTEM");
    println!("=====================================");
    println!("This demonstration showcases:");
    println!("  🌍 Universal platform capabilities");
    println!("  🧬 Genetic beardog key system");
    println!("  🚪 AI cat door protection");
    println!("  🔒 Crypto lock sovereignty");
    println!("  🎨 Bootstrap UI interface");
    println!("  🔄 Live ecosystem coordination");
    println!("  📊 3/3 compliance score");
    println!("  💰 Inverse cost scaling advantage");
}

/// Show demo conclusion
fn show_demo_outro() {
    println!("\n🎉 DEMO SUITE COMPLETE");
    println!("======================");
    println!("You've seen the complete biomeOS ecosystem:");
    println!("  ✅ Universal platform - runs everywhere");
    println!("  ✅ Zero vendor lock-in - complete freedom");
    println!("  ✅ Sovereignty-first - your data, your rules");
    println!("  ✅ AI-powered - grandma-safe operation");
    println!("  ✅ Ecosystem-ready - Toadstool, Songbird, etc.");
    println!("  ✅ Cost-optimized - small business advantage");
    println!("  ✅ Developer-friendly - optional complexity");
    println!("\n🚀 Ready to bootstrap your biome!");
}

/// Wait for user input
fn wait_for_user(prompt: &str) {
    println!("\n{}", prompt);
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
} 
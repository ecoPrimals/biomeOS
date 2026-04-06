// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Neural API + Atomic Integration Demo
//!
//! Demonstrates how Neural API graphs adapt to deployment mode and coordinate
//! with live atomic deployments (Tower, Node).
//!
//! # Features
//!
//! - DeploymentMode detection and adaptation
//! - Graph execution with mode-aware timeouts
//! - Cross-atomic communication testing
//! - AI-driven optimization
//!
//! # Usage
//!
//! ```bash
//! # Run with default (Sibling Spore) mode
//! cargo run --example neural_atomic_integration
//!
//! # Run with Cold Spore mode simulation
//! BIOMEOS_DEPLOYMENT_MODE=cold cargo run --example neural_atomic_integration
//! ```

use anyhow::{Context, Result};
use biomeos_core::deployment_mode::DeploymentMode;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🧠 Neural API + Atomic Integration Demo\n");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Step 1: Detect deployment mode
    let mode = DeploymentMode::detect().context("Failed to detect deployment mode")?;

    println!("📍 Step 1: Deployment Mode Detection");
    println!("   Mode: {}", mode.description());
    println!("   Socket Prefix: {}\n", mode.socket_prefix().display());

    // Step 2: Calculate adaptive timeouts
    let (base_timeout, multiplier) = calculate_adaptive_timeout(&mode);
    let total_timeout = base_timeout as f64 * multiplier;

    println!("⏱️  Step 2: Adaptive Timeout Calculation");
    println!("   Base Timeout: {base_timeout}ms");
    println!("   Mode Multiplier: {multiplier:.1}x");
    println!("   Adaptive Timeout: {total_timeout:.0}ms\n");

    // Step 3: Check for available atomics
    println!("🔍 Step 3: Atomic Availability Check");
    let available_atomics = check_atomic_availability(&mode).await?;

    for atomic in &available_atomics {
        println!("   ✅ {atomic} Atomic: Available");
    }

    if available_atomics.is_empty() {
        println!("   ⚠️  No atomics currently running");
        println!("   💡 Tip: Start atomics with `biomeos nucleus --mode tower --node-id tower1`\n");
    } else {
        println!();
    }

    // Step 4: Graph selection based on availability
    println!("📊 Step 4: Graph Selection");
    let selected_graph = select_graph_for_mode(&mode, &available_atomics);
    println!("   Selected: {selected_graph}");
    println!(
        "   Reason: {}\n",
        graph_selection_reason(&mode, &available_atomics)
    );

    // Step 5: Demonstrate adaptive execution strategy
    println!("🎯 Step 5: Adaptive Execution Strategy");
    demonstrate_execution_strategy(&mode, &available_atomics);

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    println!("💡 Key Insights:");
    println!();
    println!("1. Deployment Mode Awareness:");
    println!("   - Graphs adapt timeouts based on deployment environment");
    println!("   - Cold Spore (USB): 1.5x timeout (slower media)");
    println!("   - Live Spore (SSD): 1.0x timeout (full performance)");
    println!("   - Sibling Spore (shared): 1.2x timeout (resource contention)");
    println!();
    println!("2. Atomic Coordination:");
    println!("   - Graphs automatically discover available atomics");
    println!("   - Execution adapts to what's actually running");
    println!("   - Graceful degradation if primals unavailable");
    println!();
    println!("3. Neural API Integration:");
    println!("   - AI optimization via Squirrel (optional)");
    println!("   - Resource estimation via ToadStool");
    println!("   - Learning from execution metrics");
    println!();
    println!("Different orders of the same architecture. 🍄🐸🌱\n");

    Ok(())
}

fn calculate_adaptive_timeout(mode: &DeploymentMode) -> (u32, f64) {
    let base_timeout = 10000; // 10 seconds base

    let multiplier = match mode {
        DeploymentMode::ColdSpore { .. } => 1.5, // USB/SD is slower
        DeploymentMode::LiveSpore { .. } => 1.0, // Full performance
        DeploymentMode::SiblingSpore { .. } => 1.2, // Shared resources
    };

    (base_timeout, multiplier)
}

async fn check_atomic_availability(mode: &DeploymentMode) -> Result<Vec<String>> {
    let socket_prefix = mode.socket_prefix();
    let mut available = Vec::new();

    let family_id = biomeos_core::family_discovery::get_family_id();

    // Check for Tower atomic (BearDog + Songbird)
    if socket_exists(&socket_prefix, &format!("beardog-{family_id}.sock"))
        && socket_exists(&socket_prefix, &format!("songbird-{family_id}.sock"))
    {
        available.push("Tower".to_string());
    }

    // Check for Node atomic (Tower + ToadStool)
    if socket_exists(&socket_prefix, &format!("beardog-{family_id}.sock"))
        && socket_exists(&socket_prefix, &format!("songbird-{family_id}.sock"))
        && socket_exists(&socket_prefix, &format!("toadstool-{family_id}.sock"))
    {
        available.push("Node".to_string());
    }

    // Check for Nest atomic (Tower + NestGate)
    if socket_exists(&socket_prefix, &format!("beardog-{family_id}.sock"))
        && socket_exists(&socket_prefix, &format!("songbird-{family_id}.sock"))
        && socket_exists(&socket_prefix, &format!("nestgate-{family_id}.sock"))
    {
        available.push("Nest".to_string());
    }

    Ok(available)
}

fn socket_exists(prefix: &std::path::Path, socket_name: &str) -> bool {
    prefix.join(socket_name).exists()
}

fn select_graph_for_mode(mode: &DeploymentMode, available: &[String]) -> String {
    if available.contains(&"Tower".to_string()) && available.contains(&"Node".to_string()) {
        "graphs/tower_node_interaction.toml".to_string()
    } else if available.contains(&"Tower".to_string()) {
        "graphs/adaptive_tower_deploy.toml".to_string()
    } else {
        match mode {
            DeploymentMode::ColdSpore { .. } => {
                "graphs/tower_deploy.toml (cold optimized)".to_string()
            }
            DeploymentMode::LiveSpore { .. } => {
                "graphs/tower_deploy.toml (live optimized)".to_string()
            }
            DeploymentMode::SiblingSpore { .. } => {
                "graphs/tower_deploy.toml (sibling optimized)".to_string()
            }
        }
    }
}

fn graph_selection_reason(mode: &DeploymentMode, available: &[String]) -> String {
    if available.contains(&"Tower".to_string()) && available.contains(&"Node".to_string()) {
        "Both Tower and Node available - test cross-atomic interaction".to_string()
    } else if available.contains(&"Tower".to_string()) {
        "Tower available - adaptive deployment with AI optimization".to_string()
    } else {
        format!(
            "No atomics running - use {} mode-specific deployment",
            mode.description()
        )
    }
}

fn demonstrate_execution_strategy(mode: &DeploymentMode, available: &[String]) {
    println!("   Execution Plan:");

    if available.contains(&"Tower".to_string()) && available.contains(&"Node".to_string()) {
        println!("   1. Verify Tower atomic health");
        println!("   2. Verify Node atomic health");
        println!("   3. Test BearDog cross-atomic encryption");
        println!("   4. Test Songbird cross-atomic discovery");
        println!("   5. Test ToadStool resource coordination");
        println!("   6. AI analysis via Squirrel (optional)");
        println!("   7. Generate interaction report");
        println!();
        println!("   Execution Style: Cross-atomic communication test");
        println!("   Parallelization: Phases 1-2 parallel, 3-5 parallel");
        println!("   Learning: Enabled (metrics → Squirrel)");
    } else if available.contains(&"Tower".to_string()) {
        println!("   1. Detect deployment mode");
        println!("   2. Estimate resources via ToadStool");
        println!("   3. Launch BearDog (encryption)");
        println!("   4. Launch Songbird (discovery)");
        println!("   5. Verify Tower health");
        println!("   6. Get AI optimization suggestions");
        println!("   7. Apply optimizations (user approval)");
        println!();
        println!("   Execution Style: Adaptive deployment");
        println!("   Parallelization: Phases 3-4 parallel");
        println!("   Learning: Enabled (AI-driven)");
    } else {
        println!("   1. Detect deployment mode");
        println!("   2. Create mode-specific socket paths");
        println!("   3. Launch Tower primals");
        println!("   4. Verify deployment");
        println!();
        println!("   Execution Style: Fresh deployment");
        println!("   Mode-Specific: {}", mode.description());
    }

    println!();
    println!("   Resource Optimization:");
    match mode {
        DeploymentMode::ColdSpore { persistence, .. } => {
            println!("   - Optimize for portable execution");
            println!("   - Minimize disk I/O");
            if *persistence {
                println!("   - Use persistent storage on USB");
            } else {
                println!("   - Ephemeral execution (no state saved)");
            }
        }
        DeploymentMode::LiveSpore { .. } => {
            println!("   - Optimize for full performance");
            println!("   - Utilize all available cores");
            println!("   - Persistent state on fast SSD");
        }
        DeploymentMode::SiblingSpore { isolation, .. } => {
            println!("   - Optimize for coexistence with host OS");
            println!("   - Respect host resource limits");
            println!("   - Isolation: {isolation:?}");
        }
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! # Full Lab Demo
//!
//! Comprehensive demo showing BiomeOS orchestrating multiple lab experiments.
//! Tests all major benchScale features.

use anyhow::Result;
use biomeos_core::lab::LabManager;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  BiomeOS Full Lab Demo                                        ║");
    println!("║  Comprehensive benchScale Integration Test                    ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    let lab_manager = LabManager::new();

    println!("📋 Test Suite:");
    println!("   • Experiment 1: Simple LAN (2 nodes, BTSP test)");
    println!("   • Experiment 2: Multi-Tower (3 nodes, P2P coordination)");
    println!("   • Experiment 3: NAT Traversal (4 nodes, relay test)");
    println!();

    let mut results = Vec::new();

    // Experiment 1: Simple LAN
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧪 Experiment 1: Simple LAN");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    match run_experiment(&lab_manager, "simple-lan", "exp1-simple", "btsp-tunnels").await {
        Ok(success) => {
            results.push(("Simple LAN", success));
            println!(
                "✅ Experiment 1 complete: {}",
                if success { "PASSED" } else { "FAILED" }
            );
        }
        Err(e) => {
            results.push(("Simple LAN", false));
            println!("❌ Experiment 1 error: {e}");
        }
    }
    println!();

    // Experiment 2: Multi-Tower
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧪 Experiment 2: Multi-Tower P2P");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    match run_experiment(
        &lab_manager,
        "p2p-3-tower",
        "exp2-multi",
        "p2p-coordination",
    )
    .await
    {
        Ok(success) => {
            results.push(("Multi-Tower", success));
            println!(
                "✅ Experiment 2 complete: {}",
                if success { "PASSED" } else { "FAILED" }
            );
        }
        Err(e) => {
            results.push(("Multi-Tower", false));
            println!("❌ Experiment 2 error: {e}");
        }
    }
    println!();

    // Experiment 3: NAT Traversal
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧪 Experiment 3: NAT Traversal");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    match run_experiment(&lab_manager, "nat-traversal", "exp3-nat", "nat-traversal").await {
        Ok(success) => {
            results.push(("NAT Traversal", success));
            println!(
                "✅ Experiment 3 complete: {}",
                if success { "PASSED" } else { "FAILED" }
            );
        }
        Err(e) => {
            results.push(("NAT Traversal", false));
            println!("❌ Experiment 3 error: {e}");
        }
    }
    println!();

    // Summary
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║  Test Suite Complete!                                          ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    println!("Results:");
    let passed = results.iter().filter(|(_, success)| *success).count();
    let total = results.len();

    for (name, success) in &results {
        println!("  {} {}", if *success { "✅" } else { "❌" }, name);
    }

    println!();
    println!("Summary: {passed}/{total} experiments passed");
    println!();

    if passed == total {
        println!("🎉 ALL EXPERIMENTS PASSED!");
        println!();
        println!("✨ benchScale is fully validated as a primal tool!");
        println!("   BiomeOS successfully orchestrated all lab experiments.");
        println!();
        println!("🚀 Ready to push to GitHub and separate into its own repo!");
    } else {
        println!("⚠️  Some experiments had issues.");
        println!("   Lab infrastructure works, but tests may need real primal binaries.");
        println!();
        println!("📝 Next steps:");
        println!("   1. Add real primal binaries to ../phase1bins/");
        println!("   2. Re-run experiments");
        println!("   3. Push to GitHub when all pass");
    }

    println!();
    Ok(())
}

async fn run_experiment(
    manager: &LabManager,
    topology: &str,
    name: &str,
    test: &str,
) -> Result<bool> {
    println!("Creating lab: {name} ({topology})");
    let lab = manager.create_lab(topology, name).await?;
    println!("✅ Lab created");

    // Deploy (may fail without binaries, that's OK)
    let _ = lab.deploy("templates/p2p-secure-mesh.biome.yaml").await;
    println!("✅ Deploy attempted");

    println!("Running test: {test}");
    let result = lab.run_test(test).await?;
    let success = result.passed();
    println!(
        "{} Test {}",
        if success { "✅" } else { "❌" },
        if success { "passed" } else { "failed" }
    );

    println!("Cleaning up...");
    lab.destroy().await?;
    println!("✅ Lab destroyed");

    Ok(success)
}

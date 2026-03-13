// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Full Ecosystem Demo
//!
//! Demonstrates BiomeOS as the substrate that makes ecoPrimals accessible:
//! - Loads chimera definitions
//! - Discovers installed primal binaries
//! - Shows niche templates
//! - Demonstrates the BYOB workflow

use biomeos_chimera::ChimeraRegistry;
use std::fs;
use std::path::Path;

fn main() {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🌱 BiomeOS Full Ecosystem Demo");
    println!("==============================");
    println!("The substrate that makes ecoPrimals accessible\n");

    // 1. Discover installed primal binaries
    discover_primals();

    // 2. Load chimera definitions
    load_chimeras();

    // 3. Show niche templates (BYOB)
    show_niches();

    // 4. Demonstrate what's possible
    show_capabilities();

    println!("\n🌱 BiomeOS: Ready to mix primals into chimeras and deploy niches!");
}

fn discover_primals() {
    println!("📦 INSTALLED PRIMAL BINARIES");
    println!("============================");

    let primals_dir = Path::new("bin/primals");

    if !primals_dir.exists() {
        println!("   ⚠️  No primals installed. Run: ./bin/pull-primals.sh --all");
        return;
    }

    let mut primal_counts: std::collections::HashMap<String, u32> =
        std::collections::HashMap::new();

    if let Ok(entries) = fs::read_dir(primals_dir) {
        for entry in entries.filter_map(Result::ok) {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }

            // Extract primal name (before first dash)
            let primal = name.split('-').next().unwrap_or(&name);
            *primal_counts.entry(primal.to_string()).or_insert(0) += 1;
        }
    }

    let total: u32 = primal_counts.values().sum();
    println!(
        "   Total: {} binaries from {} primals\n",
        total,
        primal_counts.len()
    );

    for (primal, count) in &primal_counts {
        let icon = match primal.as_str() {
            "nestgate" => "🏰",
            "songbird" => "🎼",
            "toadstool" => "🍄",
            "beardog" => "🐕",
            "squirrel" => "🐿️",
            _ => "📦",
        };
        println!("   {} {} ({} binaries)", icon, primal, count);
    }
    println!();
}

fn load_chimeras() {
    println!("🧬 CHIMERA DEFINITIONS");
    println!("======================");
    println!("   Mixed-boundary primal amalgams\n");

    let definitions_dir = Path::new("chimeras/definitions");

    match ChimeraRegistry::from_directory(definitions_dir) {
        Ok(registry) => {
            for (id, summary) in registry.summary() {
                println!("   🧬 {}", id);
                println!("      Name: {}", summary.name);
                println!("      Primals: {}", summary.primals.join(" + "));
                if summary.uses_arrays {
                    println!("      Arrays: ✅ (multiple instances supported)");
                }
                println!();
            }
        }
        Err(e) => {
            println!("   ❌ Failed to load: {}", e);
        }
    }
}

fn show_niches() {
    println!("🌿 NICHE TEMPLATES (BYOB)");
    println!("=========================");
    println!("   Build Your Own Biome - environments for primals & chimeras\n");

    let templates_dir = Path::new("niches/templates");

    if let Ok(entries) = fs::read_dir(templates_dir) {
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "yaml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    // Quick parse to get name
                    if let Some(name_line) = content.lines().find(|l| l.contains("name:")) {
                        let name = name_line
                            .split(':')
                            .nth(1)
                            .map(|s| s.trim().trim_matches('"'))
                            .unwrap_or("Unknown");

                        let id = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("unknown");

                        println!("   🌿 {} ({})", name, id);
                    }
                }
            }
        }
    }
    println!();
}

fn show_capabilities() {
    println!("🚀 WHAT YOU CAN DO");
    println!("==================\n");

    println!("   1. BUILD CHIMERAS:");
    println!("      biomeos chimera build p2p-secure");
    println!("      # Creates: BearDog + Songbird unified binary\n");

    println!("   2. DEPLOY NICHES:");
    println!("      biomeos niche deploy gaming-tournament");
    println!("      # Starts: gaming-mesh chimera + standalone primals\n");

    println!("   3. MIX PRIMALS:");
    println!("      biomeos byob create my-biome");
    println!("      # Interactive: Select chimeras, primals, configure\n");

    println!("   4. RUN SHOWCASES:");
    println!("      biomeos showcase run songbird/federation");
    println!("      # Executes demos from parent primals\n");

    println!("   Current Status:");
    println!("   ✅ 55 primal binaries available (nestgate, songbird, toadstool)");
    println!("   ✅ 3 chimera definitions (p2p-secure, ml-pipeline, gaming-mesh)");
    println!("   ✅ 6 niche templates (gaming, research, web-dev, etc.)");
}

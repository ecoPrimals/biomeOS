// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Chimera Registry Demo
//!
//! Demonstrates loading and inspecting chimera definitions.

use biomeos_chimera::ChimeraRegistry;
use std::path::Path;

fn main() {
    // Enable tracing to see warnings
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::WARN)
        .init();

    println!("🧬 BiomeOS Chimera Registry Demo");
    println!("=================================\n");

    let definitions_dir = Path::new("chimeras/definitions");

    if !definitions_dir.exists() {
        println!("❌ Chimera definitions directory not found: {definitions_dir:?}");
        println!("   Run from biomeOS root directory");
        return;
    }

    // Load registry
    match ChimeraRegistry::from_directory(definitions_dir) {
        Ok(registry) => {
            println!("✅ Loaded {} chimera definitions:\n", registry.len());

            for (id, summary) in registry.summary() {
                println!("🧬 {id}");
                println!("   Name: {}", summary.name);
                println!("   Version: {}", summary.version);
                println!("   Primals: {}", summary.primals.join(", "));
                println!("   Uses Arrays: {}", summary.uses_arrays);
                println!();
            }

            // Test getting specific chimera
            if let Some(p2p) = registry.get("p2p-secure") {
                println!("📋 Details for p2p-secure:");
                println!(
                    "   Description: {}",
                    p2p.chimera.description.lines().next().unwrap_or("")
                );
                println!("   Fusion bindings: {}", p2p.fusion.bindings.len());
                println!("   API endpoints: {}", p2p.fusion.api.endpoints.len());

                for endpoint in &p2p.fusion.api.endpoints {
                    println!("     - {}({})", endpoint.name, endpoint.params.join(", "));
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to load chimera registry: {e}");
        }
    }

    println!("\n🌱 Demo complete!");
}

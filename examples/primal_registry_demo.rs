// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal Registry Demo
//!
//! Demonstrates BiomeOS as a bootable platform that can:
//! 1. Discover primal binaries from local USB (../phase1bins/)
//! 2. List available primals and versions
//! 3. Deploy primals to targets
//! 4. Pull updates from GitHub releases

use biomeos_core::primal_registry::PrimalRegistry;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("🖥️  BiomeOS Primal Registry Demo");
    info!("================================");
    info!("");
    info!("Vision: BiomeOS as a bootable platform (like PopOS/Windows bootloader)");
    info!("  • Boot from USB (Live OS mode)");
    info!("  • Deploy available binaries from USB");
    info!("  • Pull updates from GitHub/registries");
    info!("  • Deploy custom primals");
    info!("");

    // Create registry pointing to phase1bins
    let phase1bins = std::path::PathBuf::from("../phase1bins");
    let mut registry = PrimalRegistry::new(&phase1bins);

    info!("📂 Scanning local directory: {:?}", phase1bins);
    registry.scan_local().await?;

    // List available primals
    let primals = registry.list_primals();
    if primals.is_empty() {
        info!("");
        info!("⚠️  No primal binaries found in ../phase1bins/");
        info!("");
        info!("To populate:");
        info!("  mkdir -p ../phase1bins");
        info!("  # Copy stable Phase 1 primal binaries to ../phase1bins/");
        info!("  # Example: cp /path/to/beardog ../phase1bins/");
        info!("");
        info!("For now, simulating with mock data...");
        info!("");

        // Mock demonstration
        demonstrate_mock_registry();
    } else {
        info!("");
        info!("✅ Found {} primal types:", primals.len());
        for primal in &primals {
            let versions = registry.get_primal_versions(primal);
            info!("  • {} ({} versions)", primal, versions.len());
            for binary in versions {
                info!("    - v{} @ {:?}", binary.version, binary.path);
            }
        }

        info!("");
        info!("🚀 Deployment Example:");
        for primal in &primals {
            if let Some(latest) = registry.get_latest(primal) {
                info!("  • Latest {}: v{}", primal, latest.version);
                info!("    Capabilities: {:?}", latest.metadata.capabilities);
                info!("    Default ports: {:?}", latest.metadata.default_ports);
            }
        }
    }

    info!("");
    info!("🌐 Future: GitHub Integration");
    info!("  registry.fetch_from_github(\"ecoPrimals\", &[\"beardog\", \"songbird\"]).await?;");
    info!("");

    info!("✅ Demo complete!");

    Ok(())
}

fn demonstrate_mock_registry() {
    info!("🎭 Mock Registry Contents:");
    info!("");
    info!("  📦 beardog");
    info!("    - v1.0.0 (stable) @ ../phase1bins/beardog");
    info!("    - Capabilities: crypto, security, btsp");
    info!("    - Default port: 9000");
    info!("");
    info!("  📦 songbird");
    info!("    - v1.0.0 (stable) @ ../phase1bins/songbird");
    info!("    - Capabilities: discovery, federation, mesh");
    info!("    - Default port: 8000");
    info!("");
    info!("  📦 toadstool");
    info!("    - v1.0.0 (stable) @ ../phase1bins/toadstool");
    info!("    - Capabilities: compute, orchestration");
    info!("    - Default port: 7000");
    info!("");
    info!("  📦 nestgate");
    info!("    - v1.0.0 (stable) @ ../phase1bins/nestgate");
    info!("    - Capabilities: storage, data");
    info!("    - Default port: 6000");
    info!("");
    info!("  📦 squirrel");
    info!("    - v1.0.0 (stable) @ ../phase1bins/squirrel");
    info!("    - Capabilities: ai, ml");
    info!("    - Default port: 5000");
    info!("");
    info!("🚀 Deployment Flow:");
    info!("  1. User boots BiomeOS USB");
    info!("  2. BiomeOS scans ../phase1bins/ for available primals");
    info!("  3. User selects deployment configuration (biome.yaml)");
    info!("  4. BiomeOS orchestrates:");
    info!("     • Uses benchScale for testing");
    info!("     • Deploys primals to target nodes");
    info!("     • Configures P2P coordination");
    info!("     • Sets up networking");
    info!("  5. System running!");
    info!("");
}

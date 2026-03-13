// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Platypus - A Fused Chimera Demo
//!
//! Demonstrates the novel capabilities that emerge from
//! deep genetic mixing of BearDog + Songbird.

use platypus::{Platypus, PlatypusConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║  🦆🦫 PLATYPUS - A Fused Chimera                              ║");
    println!("║                                                               ║");
    println!("║  Not BearDog + Songbird orchestrated together.                ║");
    println!("║  A genuinely NEW organism with mixed genetics.                ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");
    println!();
    
    // Create a Platypus with default config
    let config = PlatypusConfig::default();
    let platypus = Platypus::new(config);
    
    // Show identity
    let identity = platypus.identity().await;
    println!("🆔 Identity: {}", identity.id);
    println!("   Generation: {}", identity.generation);
    println!("   Lineage depth: {}", identity.lineage_hashes.len());
    println!();
    
    // Demonstrate genetic evolution
    println!("🧬 GENETIC EVOLUTION");
    println!("   Unlike static keys, Platypus keys can evolve");
    println!("   while maintaining verifiable lineage.");
    println!();
    
    println!("   Evolving to generation 1...");
    platypus.evolve().await?;
    
    println!("   Evolving to generation 2...");
    platypus.evolve().await?;
    
    let evolved_identity = platypus.identity().await;
    println!("   ✅ Now at generation {}", evolved_identity.generation);
    println!("   Lineage chain: {:?}", evolved_identity.lineage_hashes);
    println!();
    
    // Demonstrate genetic signing
    println!("✍️  GENETIC SIGNATURES");
    println!("   Signatures include lineage information.");
    println!();
    
    let message = b"Hello from the aquatic mesh!";
    let signature = platypus.sign(message).await;
    
    println!("   Message: \"{}\"", String::from_utf8_lossy(message));
    println!("   Signature: {}...{}", 
        hex_encode(&signature.signature[..4]),
        hex_encode(&signature.signature[signature.signature.len()-4..]));
    println!("   Signed at generation: {}", signature.generation);
    println!();
    
    // Explain novel capabilities
    println!("🌟 NOVEL CAPABILITIES (Fusion Creates Emergence)");
    println!();
    println!("   These capabilities don't exist in either parent:");
    println!();
    println!("   1. GENETIC MESH DISCOVERY");
    println!("      • Songbird discovers peers via broadcast");
    println!("      • BearDog verifies cryptographic identity");
    println!("      • Platypus discovers ONLY peers sharing genetic lineage");
    println!("      → Neither parent can do this alone!");
    println!();
    println!("   2. EVOLVING ENCRYPTED STATE");
    println!("      • Keys rotate while maintaining lineage");
    println!("      • Old peers remain verified through ancestor chain");
    println!("      • New peers can verify our evolutionary history");
    println!();
    println!("   3. AQUATIC TOPOLOGY");
    println!("      • Fluid, adaptive mesh topology");
    println!("      • Connections shift based on trust + proximity");
    println!("      • Designed for unstable network conditions");
    println!();
    
    // The biological metaphor
    println!("🦆🦫 THE PLATYPUS PATTERN");
    println!();
    println!("   Nature creates weird niches. The platypus isn't");
    println!("   a duck AND a beaver cooperating - it's a genuinely");
    println!("   new species with mixed genetic material.");
    println!();
    println!("   Similarly, this Platypus isn't BearDog + Songbird");
    println!("   running as separate processes. It's a NEW PRIMAL");
    println!("   that deeply integrates:");
    println!();
    println!("   • Genetic cryptography (from BearDog)");
    println!("   • Mesh networking (from Songbird)");
    println!();
    println!("   The fusion creates emergent behavior impossible");
    println!("   with orchestration alone.");
    println!();
    
    // Usage example
    println!("💡 CREATING YOUR OWN FUSED CHIMERA");
    println!();
    println!("   1. Identify parent genetics:");
    println!("      ls ../beardog/crates/");
    println!("      ls ../songbird/crates/");
    println!();
    println!("   2. Create new crate with mixed dependencies:");
    println!("      [dependencies]");
    println!("      beardog-crypto = {{ path = \"...\" }}");
    println!("      songbird-mesh = {{ path = \"...\" }}");
    println!();
    println!("   3. Fuse the genetics into novel capabilities:");
    println!("      struct MyChimera {{");
    println!("          crypto: beardog_crypto::Keys,");
    println!("          mesh: songbird_mesh::Node,");
    println!("          // Novel integration...");
    println!("      }}");
    println!();
    println!("   Nature doesn't know the niche ahead of time.");
    println!("   Evolution experiments. So can you.");
    println!();
    
    println!("🌱 Platypus ready. The weird niches are where innovation happens!");
    
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}


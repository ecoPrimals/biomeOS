//! Verify genetic lineage between spores using BearDog
//!
//! This tool reads .family.seed files from spores and verifies their
//! cryptographic relationships using BearDog's HKDF-SHA256 lineage system.

use anyhow::{Context, Result};
use biomeos_federation::beardog_client::BearDogClient;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    println!("\n🧬 BearDog Genetic Lineage Verifier");
    println!("═══════════════════════════════════\n");
    
    // Discover BearDog
    println!("🔍 Discovering BearDog...");
    let client = BearDogClient::from_discovery().await
        .context("Failed to discover BearDog. Is beardog-server running?")?;
    println!("✅ BearDog found!\n");
    
    // Load spore seeds
    let spore_paths = vec![
        ("/media/eastgate/biomeOS1/biomeOS", "node-alpha"),
        ("/media/eastgate/BEA6-BBCE/biomeOS", "node-gamma"),
        ("/media/eastgate/BEA6-BBCE1/biomeOS", "node-delta"),
    ];
    
    let mut spores = Vec::new();
    
    println!("📂 Loading spore seeds...");
    for (path, node_id) in spore_paths {
        let seed_path = PathBuf::from(path).join(".family.seed");
        
        if !seed_path.exists() {
            println!("⚠️  Skipping {}: seed not found at {}", node_id, seed_path.display());
            continue;
        }
        
        match std::fs::read(&seed_path) {
            Ok(seed_bytes) => {
                // Hash the seed for identification
                let mut hasher = Sha256::new();
                hasher.update(&seed_bytes);
                let seed_hash = format!("{:x}", hasher.finalize());
                
                println!("  ✅ {}: {}...", node_id, &seed_hash[..16]);
                
                spores.push((node_id.to_string(), seed_bytes, seed_hash));
            }
            Err(e) => {
                println!("  ⚠️  Failed to read {}: {}", node_id, e);
            }
        }
    }
    
    if spores.is_empty() {
        println!("\n❌ No spore seeds found! Are USBs mounted?");
        return Ok(());
    }
    
    if spores.len() < 2 {
        println!("\n⚠️  Only {} spore found - need at least 2 to compare", spores.len());
        return Ok(());
    }
    
    println!("\n🔬 Verifying Genetic Relationships");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Compare each pair
    let mut all_siblings = true;
    let mut any_siblings = false;
    
    for i in 0..spores.len() {
        for j in (i + 1)..spores.len() {
            let (node_a, _seed_a, hash_a) = &spores[i];
            let (node_b, seed_b_bytes, hash_b) = &spores[j];
            
            println!("Testing: {} ↔ {}", node_a, node_b);
            
            // Convert seed_b to base64 for BearDog API
            let seed_b_base64 = base64::encode(seed_b_bytes);
            
            match client.verify_same_family("nat0", hash_b, node_b).await {
                Ok(response) => {
                    if response.is_family_member {
                        println!("  ✅ RELATED: {}", response.relationship);
                        println!("     Parent: {}", response.parent_seed_hash);
                        any_siblings = true;
                    } else {
                        println!("  ❌ UNRELATED: Different genetic families");
                        all_siblings = false;
                    }
                }
                Err(e) => {
                    println!("  ⚠️  Verification failed: {}", e);
                    println!("     This might indicate:");
                    println!("     - BearDog API not fully implemented");
                    println!("     - Seeds from different deployment batches");
                    println!("     - BearDog using different family context");
                    all_siblings = false;
                }
            }
            
            println!();
        }
    }
    
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    // Summary
    println!("📊 Summary");
    println!("══════════");
    println!("  Spores tested: {}", spores.len());
    
    if all_siblings {
        println!("  ✅ All spores are SIBLINGS (same parent)");
        println!("     → Perfect for genetic lineage testing!");
        println!("     → Can test sub-federation key derivation");
        println!("     → Ideal for hierarchical trust networks");
    } else if any_siblings {
        println!("  ⚠️  MIXED relationships detected");
        println!("     → Some spores are related, some are not");
        println!("     → Can test both sibling and cross-family scenarios");
    } else {
        println!("  ❌ Spores are UNRELATED (different parents)");
        println!("     → Good for testing cross-family federation");
        println!("     → Cannot test genetic lineage features");
        println!("     → Consider resetting with unified lineage");
    }
    
    println!("\n🎯 Next Steps:");
    if all_siblings {
        println!("  1. Test sub-federation key derivation");
        println!("  2. Verify hierarchical trust works");
        println!("  3. Deploy Tower niche on all 3");
        println!("  4. Test encrypted federation");
    } else {
        println!("  1. Option A: Test cross-family federation");
        println!("  2. Option B: Reset spores with unified lineage");
        println!("  3. See docs/SPORE_GENETIC_LINEAGE_ANALYSIS_JAN9.md");
    }
    
    println!("\n✨ Done!\n");
    
    Ok(())
}


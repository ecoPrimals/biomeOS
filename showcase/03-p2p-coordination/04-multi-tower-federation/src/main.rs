//! Multi-Tower P2P Federation Demo
//!
//! This demo shows BiomeOS coordinating P2P across multiple Songbird towers
//! in a federated mesh.
//!
//! "Local discovery with global reach"
//!
//! Key Features:
//! - Multi-tower federation
//! - Cross-tower P2P coordination
//! - Distributed mesh formation
//! - Tower-to-tower routing

use anyhow::Result;
use std::collections::HashMap;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    info!("🌱 BiomeOS P2P Coordination Demo: Multi-Tower Federation");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");
    info!("🌐 \"Local discovery with global reach\"");
    info!("");

    // Scenario: Multiple Songbird towers form a federation
    info!("📋 Scenario:");
    info!("   Tower A (San Francisco): 3 nodes");
    info!("   Tower B (New York): 2 nodes");
    info!("   Tower C (London): 2 nodes");
    info!("   Goal: Node in SF connects to node in London");
    info!("");

    // Step 1: Initialize towers
    info!("🏗️  Step 1: Initializing Songbird towers...");
    info!("");

    let mut towers = HashMap::new();

    let tower_a = Tower {
        name: "tower-sf".to_string(),
        location: "San Francisco, US".to_string(),
        nodes: vec!["alice".to_string(), "bob".to_string(), "carol".to_string()],
        federated_with: vec!["tower-ny".to_string(), "tower-lon".to_string()],
    };

    let tower_b = Tower {
        name: "tower-ny".to_string(),
        location: "New York, US".to_string(),
        nodes: vec!["dave".to_string(), "eve".to_string()],
        federated_with: vec!["tower-sf".to_string(), "tower-lon".to_string()],
    };

    let tower_c = Tower {
        name: "tower-lon".to_string(),
        location: "London, UK".to_string(),
        nodes: vec!["frank".to_string(), "grace".to_string()],
        federated_with: vec!["tower-sf".to_string(), "tower-ny".to_string()],
    };

    towers.insert("tower-sf".to_string(), tower_a);
    towers.insert("tower-ny".to_string(), tower_b);
    towers.insert("tower-lon".to_string(), tower_c);

    info!("✅ Tower A (San Francisco): 3 nodes online");
    info!("✅ Tower B (New York): 2 nodes online");
    info!("✅ Tower C (London): 2 nodes online");
    info!("");
    info!("🔗 Federation established between all towers");
    info!("");

    // Step 2: Discover primals across towers
    info!("🔍 Step 2: Discovering primals by capability (federated)...");
    info!("   Looking for: discovery capability across all towers");
    info!("   BiomeOS discovers: 3 Songbird towers in federation");
    info!("");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    info!("✅ Discovered federated mesh:");
    info!("   • Tower A (SF): 3 nodes");
    info!("   • Tower B (NY): 2 nodes");
    info!("   • Tower C (London): 2 nodes");
    info!("   Total: 7 nodes across 3 towers");
    info!("");

    // Step 3: Cross-tower service discovery
    info!("📡 Step 3: Cross-tower service discovery...");
    info!("   Alice (SF) searching for: storage capability");
    info!("   BiomeOS queries all federated towers...");
    info!("");

    tokio::time::sleep(std::time::Duration::from_millis(150)).await;

    info!("✅ Found services:");
    info!("   • Bob (SF, Tower A): storage, 10ms latency");
    info!("   • Eve (NY, Tower B): storage, 45ms latency");
    info!("   • Grace (London, Tower C): storage, 85ms latency");
    info!("");
    info!("🎯 BiomeOS selects: Bob (same tower, lowest latency)");
    info!("");

    // Step 4: Cross-tower P2P connection
    info!("🌍 Step 4: Cross-tower P2P connection...");
    info!("   Alice (SF) wants to connect to Frank (London)");
    info!("   Route: Tower A → Tower C");
    info!("");

    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    info!("🔗 Establishing cross-tower connection:");
    info!("   1. Alice queries Tower A for Frank");
    info!("   2. Tower A forwards to Tower C (federation)");
    info!("   3. Tower C returns Frank's endpoint");
    info!("   4. BiomeOS establishes BTSP tunnel");
    info!("   5. Alice ↔ Frank (cross-tower P2P!)");
    info!("");

    info!("✅ Cross-tower P2P connection established!");
    info!("   Alice (SF) ↔ Frank (London)");
    info!("   Latency: 85ms");
    info!("   Encrypted: Yes (BTSP)");
    info!("   Route: Direct (tower-facilitated discovery)");
    info!("");

    // Step 5: Federation benefits
    info!("📊 Step 5: Federation Benefits Demonstration");
    info!("");
    info!("🌐 Without Federation (Single Tower):");
    info!("   ❌ Can only discover local nodes");
    info!("   ❌ No cross-geography connections");
    info!("   ❌ Limited to tower's local network");
    info!("   ❌ Single point of failure");
    info!("");
    info!("✅ With Federation (Multi-Tower):");
    info!("   ✅ Discover nodes globally");
    info!("   ✅ Cross-geography P2P");
    info!("   ✅ Redundancy (multiple towers)");
    info!("   ✅ Load balancing across towers");
    info!("   ✅ Geographic optimization");
    info!("");

    // Step 6: Tower failure resilience
    info!("🔄 Step 6: Tower Failure Resilience");
    info!("");
    info!("Simulating Tower B (NY) failure...");
    info!("");

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    info!("❌ Tower B (NY) is offline");
    info!("");
    info!("BiomeOS re-routing:");
    info!("   • Dave & Eve (NY nodes) connect to Tower A (SF)");
    info!("   • Federation automatically adjusts");
    info!("   • No service interruption for users");
    info!("");
    info!("✅ Federation resilience maintained!");
    info!("   Active towers: 2/3");
    info!("   Active nodes: 7/7 (all nodes still connected)");
    info!("");

    // Step 7: Geographic optimization
    info!("🗺️  Step 7: Geographic Optimization");
    info!("");
    info!("BiomeOS optimizes connections by geography:");
    info!("");
    info!("Local Connections (same tower):");
    info!("   • Alice → Bob: 10ms (both in SF)");
    info!("   • Dave → Eve: 8ms (both in NY)");
    info!("   Optimization: Use same tower");
    info!("");
    info!("Regional Connections (nearby towers):");
    info!("   • Alice → Dave: 45ms (SF → NY)");
    info!("   Optimization: Direct cross-tower route");
    info!("");
    info!("Global Connections (distant towers):");
    info!("   • Alice → Frank: 85ms (SF → London)");
    info!("   Optimization: May route through intermediate tower");
    info!("");

    // Step 8: Distributed mesh formation
    info!("🕸️  Step 8: Distributed Mesh Formation");
    info!("");
    info!("BiomeOS coordinates mesh across all towers:");
    info!("");
    info!("Tower A (SF) - Local Mesh:");
    info!("   Alice ← → Bob");
    info!("   Bob ← → Carol");
    info!("   Alice ← → Carol");
    info!("");
    info!("Tower B (NY) - Local Mesh:");
    info!("   Dave ← → Eve");
    info!("");
    info!("Tower C (London) - Local Mesh:");
    info!("   Frank ← → Grace");
    info!("");
    info!("Cross-Tower Mesh:");
    info!("   Alice (SF) ← → Frank (London)");
    info!("   Bob (SF) ← → Eve (NY)");
    info!("   Carol (SF) ← → Grace (London)");
    info!("");
    info!("✅ Fully connected distributed mesh!");
    info!("   7 nodes × 7 nodes = 49 potential connections");
    info!("   Optimized by geography and latency");
    info!("");

    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🎉 Demo complete!");
    info!("");
    info!("Key Takeaways:");
    info!("  ✅ Multi-tower federation: Global discovery, local optimization");
    info!("  ✅ Cross-tower P2P: Seamless connections across geography");
    info!("  ✅ Resilience: Tower failures don't break the mesh");
    info!("  ✅ Geographic optimization: Prefer local, support global");
    info!("  ✅ Distributed mesh: Fully connected yet optimized");
    info!("");
    info!("Next Steps:");
    info!("  - Run demo 05: Full Ecosystem Integration");
    info!("  - Deploy with BYOB: templates/multi-tower-federation.biome.yaml");
    info!("  - Test with real Songbird federation");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// Mock tower representation
#[derive(Debug, Clone)]
struct Tower {
    name: String,
    location: String,
    nodes: Vec<String>,
    federated_with: Vec<String>,
}

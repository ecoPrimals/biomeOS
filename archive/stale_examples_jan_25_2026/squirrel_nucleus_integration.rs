//! Squirrel AI Integration with NUCLEUS
//!
//! Demonstrates how Squirrel coordinates AI capabilities across the NUCLEUS ecosystem.
//!
//! # Architecture
//!
//! ```text
//! NUCLEUS (3 Atomics)
//! ├─ Tower (BearDog + Songbird)    - Security & Discovery
//! ├─ Node (Tower + Toadstool)      - Compute & GPU  
//! ├─ Nest (Tower + NestGate)       - Storage & Persistence
//! └─ Squirrel                      - AI Coordination Layer
//!     ├─ Analyzes system state via Songbird discovery
//!     ├─ Recommends compute strategies via Toadstool
//!     ├─ Stores AI insights via NestGate
//!     └─ Secured by BearDog genetic lineage
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run --example squirrel_nucleus_integration
//! ```

use anyhow::Result;
use biomeos_core::clients::nestgate::NestGateClient;
use biomeos_core::clients::songbird::SongbirdClient;
use biomeos_core::clients::squirrel::SquirrelClient;
use biomeos_core::clients::toadstool::ToadStoolClient;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🐿️  Squirrel AI + NUCLEUS Integration Demo");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let family_id = "nat0";

    // Step 1: Discover all NUCLEUS components
    println!("🔍 Discovering NUCLEUS components...\n");

    let songbird = SongbirdClient::discover(family_id)
        .await
        .expect("Songbird (discovery) required for NUCLEUS");
    println!("✅ Songbird: Discovery & registry");

    let toadstool = ToadStoolClient::discover(family_id)
        .await
        .expect("Toadstool (compute) required for Node atomic");
    println!("✅ Toadstool: Compute & GPU");

    let nestgate = NestGateClient::discover(family_id)
        .await
        .expect("NestGate (storage) required for Nest atomic");
    println!("✅ NestGate: Storage & persistence");

    let squirrel = SquirrelClient::discover(family_id)
        .await
        .expect("Squirrel (AI) required for coordination");
    println!("✅ Squirrel: AI coordination\n");

    // Step 2: Get system state from Toadstool
    println!("📊 Gathering system metrics from Toadstool...\n");

    let resource_metrics = toadstool.get_resource_usage("nucleus-node").await?;
    println!("CPU Usage: {}%", resource_metrics.cpu_percent);
    println!("Memory Usage: {} MB", resource_metrics.memory_mb);
    println!("Network I/O: {:?}\n", resource_metrics.network_io);

    // Convert to JSON for Squirrel
    let system_state = json!({
        "cpu": resource_metrics.cpu_percent,
        "memory_mb": resource_metrics.memory_mb,
        "network_io": resource_metrics.network_io,
        "timestamp": resource_metrics.timestamp
    });

    // Step 3: Ask Squirrel to analyze and optimize
    println!("🤖 Asking Squirrel for AI-driven optimization...\n");

    let analysis = squirrel.analyze_system_optimization(&system_state).await?;
    println!("Squirrel Analysis:");
    println!("  Score: {}", analysis.score);
    println!(
        "  Opportunities: {} identified\n",
        analysis.opportunities.len()
    );

    for (i, opp) in analysis.opportunities.iter().enumerate() {
        println!("  {}. {}", i + 1, opp);
    }

    // Step 4: Store AI insights in NestGate
    println!("\n💾 Storing AI insights in NestGate...\n");

    let insight_key = format!("ai/insights/{}", chrono::Utc::now().timestamp());
    let insight_data = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "analysis": analysis,
        "system_state": system_state,
    });

    nestgate.store(&insight_key, &insight_data).await?;
    println!("✅ Stored AI insights: {}\n", insight_key);

    // Step 5: Summary
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ SQUIRREL + NUCLEUS INTEGRATION COMPLETE");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("📋 Summary:");
    println!("  • Squirrel analyzed system via Toadstool");
    println!("  • AI recommendations generated");
    println!("  • Insights persisted to NestGate");
    println!("  • All via Unix socket JSON-RPC");
    println!("  • Zero TCP ports used for IPC\n");

    println!("🎯 Next Steps:");
    println!("  • Deploy AI-recommended optimizations");
    println!("  • Integrate with petalTongue for visualization");
    println!("  • Add BearDog security layer");
    println!("  • Enable multi-node coordination\n");

    Ok(())
}

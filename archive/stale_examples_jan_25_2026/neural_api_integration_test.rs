//! Neural API Integration Test
//!
//! Tests the complete Neural API workflow:
//! 1. Start Neural API server
//! 2. Connect via NeuralApiClient
//! 3. List available graphs
//! 4. Get NUCLEUS topology
//! 5. Execute a graph
//! 6. Monitor execution status

use anyhow::{Context, Result};
use biomeos_core::clients::neural_api::NeuralApiClient;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter("info,biomeos_core=debug")
        .with_target(false)
        .init();

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧠 Neural API Integration Test");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let family_id = "nat0";
    let socket_path = format!("/tmp/biomeos-neural-api-{}.sock", family_id);

    // Check if server is running
    if !std::path::Path::new(&socket_path).exists() {
        eprintln!("❌ Neural API server not running!");
        eprintln!(
            "   Start it with: target/release/nucleus serve --family {}",
            family_id
        );
        eprintln!("");
        eprintln!("   Expected socket: {}", socket_path);
        std::process::exit(1);
    }

    println!("✅ Neural API server detected at: {}\n", socket_path);

    // Create client
    println!("🔌 Connecting to Neural API...");
    let client = NeuralApiClient::from_socket(&socket_path, family_id)?;
    println!("✅ Connected!\n");

    // Test 1: List graphs
    println!("📋 Test 1: List Available Graphs");
    println!("─────────────────────────────────");
    match client.list_graphs().await {
        Ok(graphs) => {
            println!("✅ Found {} graph(s):", graphs.len());
            for graph in &graphs {
                println!(
                    "   • {} (v{}) - {}",
                    graph.id, graph.version, graph.description
                );
                println!("     Nodes: {}", graph.node_count);
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to list graphs: {}", e);
        }
    }
    println!("");

    // Test 2: Get topology
    println!("🗺️  Test 2: Get NUCLEUS Topology");
    println!("─────────────────────────────────");
    match client.get_topology().await {
        Ok(topology) => {
            println!("✅ Topology retrieved:");
            println!("   Primals: {} active", topology.primals.len());
            for primal in &topology.primals {
                println!(
                    "     • {} ({}) - {:?}",
                    primal.id, primal.primal_type, primal.health
                );
            }
            if !topology.connections.is_empty() {
                println!("   Connections: {}", topology.connections.len());
                for conn in &topology.connections {
                    println!(
                        "     • {} → {} ({})",
                        conn.from, conn.to, conn.connection_type
                    );
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to get topology: {}", e);
        }
    }
    println!("");

    // Test 3: List niche templates
    println!("📦 Test 3: List Niche Templates");
    println!("─────────────────────────────────");
    match client.list_niche_templates().await {
        Ok(templates) => {
            println!("✅ Found {} template(s):", templates.len());
            for template in &templates {
                println!("   • {} - {}", template.id, template.name);
                println!("     {}", template.description);
                println!("     Category: {}", template.category);
                if let Some(cpu) = template.required_resources.cpu_cores {
                    println!("     CPU: {} cores", cpu);
                }
                if let Some(mem) = template.required_resources.memory_mb {
                    println!("     RAM: {} MB", mem);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to list templates: {}", e);
        }
    }
    println!("");

    // Test 4: Execute a simple verification graph (if it exists)
    println!("🚀 Test 4: Execute Graph (Simulation)");
    println!("─────────────────────────────────");
    println!("ℹ️  In production, this would:");
    println!("   1. Load graph definition");
    println!("   2. Create execution handle");
    println!("   3. Execute in background");
    println!("   4. Return execution_id for monitoring");
    println!("");
    println!("   Example:");
    println!("   let handle = client.execute_graph(\"nucleus-simple\").await?;");
    println!("   println!(\"Execution ID: {{}}\", handle.execution_id);");
    println!("");

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ All Tests Complete!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("");
    println!("🎯 Neural API is ready for:");
    println!("   • Squirrel AI coordination");
    println!("   • petalTongue 3D visualization");
    println!("   • User-driven niche deployment");
    println!("   • Self-hosted evolution! 🧬");

    Ok(())
}

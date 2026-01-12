//! Example: Neural API Graph Execution
//!
//! Demonstrates deterministic deployment using TOML graphs

use biomeos_graph::{NeuralGraph, NeuralGraphExecutor};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧠 Neural API Graph Execution Example");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Phase 1: Load graph from TOML
    println!("📍 Phase 1: Load Graph\n");

    let graph_path = std::path::Path::new("graphs/genetic_lineage_full_nucleus.toml");
    
    if !graph_path.exists() {
        println!("   ⚠️  Graph file not found: {}", graph_path.display());
        println!("   💡 This example demonstrates the API structure.");
        println!("   The actual graph execution would work when the file exists.\n");
        return Ok(());
    }

    println!("   Loading: {}", graph_path.display());
    let graph = NeuralGraph::from_toml_file(graph_path)?;
    println!("   ✅ Graph loaded: {}", graph.id);
    println!("   📊 Nodes: {}", graph.nodes.len());
    println!("   ⚙️  Config: {:?}\n", graph.config);

    // Phase 2: Configure environment
    println!("📍 Phase 2: Configure Environment\n");

    let mut env = HashMap::new();
    env.insert("USB_SEED_PATH".to_string(), "/tmp/biomeos-test/.family.seed".to_string());
    env.insert("FAMILY_ID".to_string(), "nat0".to_string());
    env.insert("DEPLOYMENT_BATCH".to_string(), chrono::Utc::now().format("%Y%m%d").to_string());
    env.insert("BINARY_DIR".to_string(), "/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin".to_string());
    env.insert("RUNTIME_DIR".to_string(), 
        std::env::var("XDG_RUNTIME_DIR")
            .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }))
    );

    println!("   Environment variables:");
    for (key, value) in &env {
        println!("      • {} = {}", key, value);
    }
    println!();

    // Phase 3: Create executor
    println!("📍 Phase 3: Initialize Executor\n");

    let mut executor = NeuralGraphExecutor::new(graph, env);
    println!("   ✅ Executor ready\n");

    // Phase 4: Execute graph
    println!("📍 Phase 4: Execute Graph\n");

    let report = executor.execute().await?;

    // Phase 5: Report results
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Execution Report");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("   Graph: {}", report.graph_id);
    println!("   Success: {}", if report.success { "✅" } else { "❌" });
    println!("   Duration: {} ms", report.duration_ms);
    println!("   Phases: {}", report.phase_results.len());

    if let Some(error) = &report.error {
        println!("\n   ❌ Error: {}", error);
    }

    println!("\n   Phase Details:");
    for (i, phase) in report.phase_results.iter().enumerate() {
        println!("      Phase {}: {}/{} nodes completed ({} ms)",
            i + 1,
            phase.completed,
            phase.total_nodes,
            phase.duration_ms
        );
        
        if !phase.errors.is_empty() {
            for (node_id, error) in &phase.errors {
                println!("         ❌ {}: {}", node_id, error);
            }
        }
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎊 Example Complete");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("🧠 Neural API Graph Features:");
    println!("   ✅ TOML-based declarative graphs");
    println!("   ✅ Deterministic execution order");
    println!("   ✅ Parallel phase execution");
    println!("   ✅ Automatic dependency resolution");
    println!("   ✅ Checkpoint/rollback support");
    println!("   ✅ Live monitoring & metrics");
    println!("   ✅ Type-safe node execution");
    println!();

    println!("🦀 Rust Integration:");
    println!("   • Zero bash scripts");
    println!("   • Type-safe configuration");
    println!("   • Async/await execution");
    println!("   • Error handling (Result<T, E>)");
    println!("   • Genetic lineage support");
    println!();

    println!("Different orders of the same architecture. 🍄🐸\n");

    Ok(())
}


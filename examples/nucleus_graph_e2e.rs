//! E2E Example: NUCLEUS Integration with Graph Execution
//!
//! This example demonstrates the complete NUCLEUS + Graph integration:
//! 1. Initialize NUCLEUS (discovers Songbird & BearDog)
//! 2. Create a simple graph
//! 3. Execute with NucleusPrimalExecutor
//! 4. Show verified, secure primal coordination
//!
//! **To run**: Ensure Songbird and BearDog are running!
//! ```bash
//! cargo run --example nucleus_graph_e2e
//! ```

use biomeos_graph::{
    CoordinationPattern, GraphExecutor, GraphId, GraphNode, NucleusPrimalExecutor, Operation,
    PrimalGraph, PrimalSelector,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("🔒 NUCLEUS + Graph E2E Example");
    println!("=".repeat(60));
    println!();

    // Step 1: Initialize NUCLEUS executor
    println!("📡 Step 1: Initializing NUCLEUS executor...");
    println!("   (This will discover Songbird & BearDog via 5-layer protocol)");

    let nucleus_executor = match NucleusPrimalExecutor::new().await {
        Ok(executor) => {
            println!("   ✅ NUCLEUS executor initialized!");
            executor
        }
        Err(e) => {
            eprintln!("   ❌ Failed to initialize NUCLEUS: {}", e);
            eprintln!();
            eprintln!("   Make sure Songbird and BearDog are running:");
            eprintln!("   - Songbird: Provides discovery capability");
            eprintln!("   - BearDog: Provides identity & trust verification");
            eprintln!();
            return Err(e.into());
        }
    };

    println!();

    // Step 2: Create a simple test graph
    println!("📊 Step 2: Creating test graph...");

    let graph = PrimalGraph {
        id: GraphId::new("nucleus_test"),
        name: "NUCLEUS Integration Test".to_string(),
        description: "Tests NUCLEUS-based secure primal discovery and execution".to_string(),
        version: "1.0.0".to_string(),
        coordination: CoordinationPattern::Sequential,
        nodes: vec![
            GraphNode {
                id: "discover".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "discovery".to_string(),
                },
                operation: Operation {
                    name: "get_capabilities".to_string(),
                    params: serde_json::json!({}),
                },
                input: None,
                output: Some("discovery_result".to_string()),
                constraints: None,
                parallel_group: None,
            },
            GraphNode {
                id: "verify".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "identity".to_string(),
                },
                operation: Operation {
                    name: "get_status".to_string(),
                    params: serde_json::json!({}),
                },
                input: None,
                output: Some("verify_result".to_string()),
                constraints: None,
                parallel_group: None,
            },
        ],
        edges: vec![],
    };

    println!("   ✅ Graph created: {} nodes", graph.nodes.len());
    println!();

    // Step 3: Execute graph with NUCLEUS
    println!("🚀 Step 3: Executing graph via NUCLEUS...");
    println!("   (NUCLEUS will verify identity, capabilities, and trust)");
    println!();

    let executor = GraphExecutor::new(nucleus_executor);

    match executor.execute(graph).await {
        Ok(result) => {
            println!("✅ Graph execution complete!");
            println!();
            println!("Results:");
            println!("  Success: {}", result.success);
            println!("  Nodes executed: {}", result.metrics.len());
            println!();

            for metric in &result.metrics {
                println!("  Node: {}", metric.node_id);
                println!("    Primal: {}", metric.primal_id);
                println!("    Operation: {}", metric.operation);
                println!("    Duration: {}ms", metric.duration_ms);
                println!("    Success: {}", metric.success);
                if let Some(error) = &metric.error {
                    println!("    Error: {}", error);
                }
                println!();
            }

            println!("🎊 NUCLEUS Integration Test Successful!");
        }
        Err(e) => {
            eprintln!("❌ Graph execution failed: {}", e);
            eprintln!();
            eprintln!("This could mean:");
            eprintln!("  - Primals are not responding");
            eprintln!("  - Identity verification failed");
            eprintln!("  - Capabilities don't match");
            return Err(e.into());
        }
    }

    Ok(())
}

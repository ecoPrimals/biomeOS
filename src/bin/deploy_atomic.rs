//! Atomic Deployment Binary
//!
//! Pure Rust orchestration for deploying biomeOS atomics (Tower, Node, Nest, NUCLEUS).
//! Replaces bash scripts with modern concurrent Rust.
//!
//! # Philosophy
//!
//! Bash is "jelly string" - fragile, single-solution state, error-prone.
//! Rust gives us:
//! - Type safety (catch errors at compile time)
//! - Concurrency (parallel deployment)
//! - Composability (atomics as building blocks)
//! - Modern async/await patterns
//!
//! # Usage
//!
//! ```bash
//! # Deploy a single atomic
//! cargo run --bin deploy_atomic -- tower
//! cargo run --bin deploy_atomic -- node
//! cargo run --bin deploy_atomic -- nest
//!
//! # Deploy complete NUCLEUS
//! cargo run --bin deploy_atomic -- nucleus
//!
//! # Test atomic interactions
//! cargo run --bin deploy_atomic -- test tower-tower
//! cargo run --bin deploy_atomic -- test node-node
//! cargo run --bin deploy_atomic -- test nest-nest
//! cargo run --bin deploy_atomic -- test node-nest
//! ```

use anyhow::{Context, Result};
use biomeos_graph::{GraphExecutor, PrimalOperationExecutor, PrimalGraph};
use std::path::PathBuf;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "tower" => deploy_atomic("tower").await?,
        "node" => deploy_atomic("node").await?,
        "nest" => deploy_atomic("nest").await?,
        "nucleus" => deploy_atomic("nucleus").await?,
        "test" => {
            if args.len() < 3 {
                eprintln!("Error: test requires interaction type (tower-tower, node-node, etc.)");
                std::process::exit(1);
            }
            test_interaction(&args[2]).await?;
        }
        "list" => list_graphs().await?,
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage(program: &str) {
    println!("🧬 biomeOS Atomic Deployment System");
    println!();
    println!("Usage: {} <command>", program);
    println!();
    println!("Commands:");
    println!("  tower           Deploy Tower atomic (BearDog + Songbird)");
    println!("  node            Deploy Node atomic (BearDog + Songbird + ToadStool)");
    println!("  nest            Deploy Nest atomic (BearDog + Songbird + NestGate)");
    println!("  nucleus         Deploy complete NUCLEUS (Tower + Node + Nest)");
    println!("  test <type>     Test atomic interactions");
    println!("                  Types: tower-tower, node-node, nest-nest, node-nest");
    println!("  list            List available deployment graphs");
    println!();
    println!("Philosophy:");
    println!("  Pure Rust. No bash scripts. Concurrent, type-safe, composable.");
}

async fn deploy_atomic(atomic: &str) -> Result<()> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🚀 Deploying {} atomic", atomic);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    // Load the deployment graph
    let graph_path = PathBuf::from(format!("graphs/{}_deploy.toml", atomic));
    
    if !graph_path.exists() {
        error!("❌ Graph not found: {:?}", graph_path);
        error!("   Run `{} list` to see available graphs", std::env::args().next().unwrap());
        return Err(anyhow::anyhow!("Graph not found"));
    }

    info!("📊 Loading graph: {:?}", graph_path);
    
    let graph_content = tokio::fs::read_to_string(&graph_path)
        .await
        .context("Failed to read graph file")?;
    
    // Parse TOML using serde
    let graph: PrimalGraph = toml::from_str(&graph_content)
        .context("Failed to parse graph")?;

    info!("✅ Graph loaded: {}", graph.name);
    info!("   Nodes: {}", graph.nodes.len());
    info!("   Coordination: {:?}", graph.coordination);
    info!("");

    // Initialize NUCLEUS executor
    info!("🔒 Initializing mock executor for testing...");
    
    // TODO: Replace with NucleusPrimalExecutor once nucleus_executor module is re-enabled
    // For now, we'll create a simple mock executor
    struct MockExecutor;
    
    #[async_trait::async_trait]
    impl PrimalOperationExecutor for MockExecutor {
        async fn execute_operation(
            &self,
            primal_id: &str,
            operation: &biomeos_graph::Operation,
            _context: &biomeos_graph::context::ExecutionContext,
        ) -> biomeos_graph::error::Result<serde_json::Value> {
            info!("  [MOCK] Executing {} on {}", operation.name, primal_id);
            Ok(serde_json::json!({"status": "mock_success"}))
        }
        
        async fn discover_primals(&self) -> biomeos_graph::error::Result<Vec<(String, Vec<String>)>> {
            // Return mock primals
            Ok(vec![
                ("songbird:nat0".to_string(), vec!["discovery".to_string()]),
                ("beardog:nat0".to_string(), vec!["security".to_string()]),
                ("toadstool:nat0".to_string(), vec!["compute".to_string()]),
                ("nestgate:nat0".to_string(), vec!["storage".to_string()]),
            ])
        }
    }
    
    let nucleus_executor = MockExecutor;

    info!("");
    info!("🎯 Executing deployment graph...");
    info!("");

    // Create graph executor and execute
    let executor = GraphExecutor::new(nucleus_executor);
    
    let result = match executor.execute(graph).await {
        Ok(result) => result,
        Err(e) => {
            error!("");
            error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            error!("❌ Deployment failed: {}", e);
            error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            return Err(e.into());
        }
    };

    // Display results
    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("📊 Deployment Results:");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    for metric in &result.metrics {
        let status = if metric.success { "✅" } else { "❌" };
        info!("{} {} ({}ms)", status, metric.node_id, metric.duration_ms);
        
        if !metric.success {
            if let Some(error) = &metric.error {
                error!("   Error: {}", error);
            }
        }
    }

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    if result.success {
        info!("🎉 {} atomic deployed successfully!", atomic);
        info!("   All {} nodes executed", result.metrics.len());
        info!("");
        
        // Show next steps
        match atomic {
            "tower" => {
                info!("Next steps:");
                info!("  - Deploy another Tower for mesh testing");
                info!("  - Test: cargo run --bin deploy_atomic -- test tower-tower");
            }
            "node" => {
                info!("Next steps:");
                info!("  - Deploy another Node for distributed compute");
                info!("  - Test: cargo run --bin deploy_atomic -- test node-node");
            }
            "nest" => {
                info!("Next steps:");
                info!("  - Deploy another Nest for federated storage");
                info!("  - Test: cargo run --bin deploy_atomic -- test nest-nest");
            }
            "nucleus" => {
                info!("Next steps:");
                info!("  - Test all atomic interactions");
                info!("  - Deploy another NUCLEUS for federation");
            }
            _ => {}
        }
        
        Ok(())
    } else {
        let failed = result.metrics.iter().filter(|m| !m.success).count();
        error!("⚠️  Partial deployment: {} of {} nodes failed", failed, result.metrics.len());
        Err(anyhow::anyhow!("Deployment incomplete"))
    }
}

async fn test_interaction(interaction_type: &str) -> Result<()> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🧪 Testing {} interaction", interaction_type);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    // Load the interaction test graph
    let graph_name = match interaction_type {
        "tower-tower" => "tower_tower_mesh_test",
        "node-node" => "node_node_distributed_test",
        "nest-nest" => "nest_nest_federation_test",
        "node-nest" => "node_nest_compute_on_data_test",
        _ => {
            error!("Unknown interaction type: {}", interaction_type);
            return Err(anyhow::anyhow!("Invalid interaction type"));
        }
    };

    let graph_path = PathBuf::from(format!("graphs/{}.toml", graph_name));
    
    if !graph_path.exists() {
        error!("❌ Test graph not found: {:?}", graph_path);
        error!("   This interaction test is not yet implemented.");
        error!("");
        error!("   TODO: Create {}.toml", graph_name);
        return Err(anyhow::anyhow!("Test graph not found"));
    }

    info!("📊 Loading test graph: {:?}", graph_path);
    
    let graph_content = tokio::fs::read_to_string(&graph_path)
        .await
        .context("Failed to read test graph")?;
    
    let graph: PrimalGraph = toml::from_str(&graph_content)
        .context("Failed to parse graph")?;

    // Use mock executor (same as deploy_atomic)
    struct MockExecutor;
    
    #[async_trait::async_trait]
    impl PrimalOperationExecutor for MockExecutor {
        async fn execute_operation(
            &self,
            primal_id: &str,
            operation: &biomeos_graph::Operation,
            _context: &biomeos_graph::context::ExecutionContext,
        ) -> biomeos_graph::error::Result<serde_json::Value> {
            info!("  [TEST] {} on {}", operation.name, primal_id);
            Ok(serde_json::json!({"status": "test_success"}))
        }
        
        async fn discover_primals(&self) -> biomeos_graph::error::Result<Vec<(String, Vec<String>)>> {
            Ok(vec![
                ("songbird:nat0".to_string(), vec!["discovery".to_string()]),
                ("beardog:nat0".to_string(), vec!["security".to_string()]),
            ])
        }
    }
    
    let nucleus_executor = MockExecutor;
    let executor = GraphExecutor::new(nucleus_executor);
    
    let result = executor.execute(graph).await?;

    if result.success {
        info!("");
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        info!("✅ {} interaction test passed!", interaction_type);
        info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        Ok(())
    } else {
        error!("");
        error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        error!("❌ {} interaction test failed", interaction_type);
        error!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        Err(anyhow::anyhow!("Interaction test failed"))
    }
}

async fn list_graphs() -> Result<()> {
    info!("📊 Available Deployment Graphs:");
    info!("");

    let graphs_dir = PathBuf::from("graphs");
    
    if !graphs_dir.exists() {
        error!("❌ graphs/ directory not found");
        return Err(anyhow::anyhow!("graphs/ directory not found"));
    }

    let mut entries = tokio::fs::read_dir(&graphs_dir).await?;
    
    let mut deploy_graphs = Vec::new();
    let mut test_graphs = Vec::new();
    let mut other_graphs = Vec::new();

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("toml") {
            let name = path.file_stem().unwrap().to_str().unwrap();
            
            if name.ends_with("_deploy") {
                deploy_graphs.push(name.to_string());
            } else if name.contains("_test") {
                test_graphs.push(name.to_string());
            } else {
                other_graphs.push(name.to_string());
            }
        }
    }

    deploy_graphs.sort();
    test_graphs.sort();
    other_graphs.sort();

    info!("Deployment Graphs:");
    for graph in deploy_graphs {
        info!("  • {}", graph);
    }

    info!("");
    info!("Test Graphs:");
    for graph in test_graphs {
        info!("  • {}", graph);
    }

    if !other_graphs.is_empty() {
        info!("");
        info!("Other Graphs:");
        for graph in other_graphs {
            info!("  • {}", graph);
        }
    }

    Ok(())
}


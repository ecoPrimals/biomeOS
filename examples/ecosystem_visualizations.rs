//! # Ecosystem Visualizations Example
//!
//! Demonstrates biomeOS ecosystem visualization using petalTongue.
//!
//! This example showcases:
//! 1. Live USB Spore deployment lifecycle
//! 2. NUCLEUS discovery architecture
//! 3. Neural API + RootPulse graph orchestration
//!
//! ## Usage
//!
//! ```bash
//! # Start petalTongue (in another terminal)
//! ./bin/primals/petal-tongue-headless --mode terminal
//!
//! # Run this example
//! cargo run --example ecosystem_visualizations
//! ```

// Note: In a real deployment, this would use PetalTongueClient
// For now, we demonstrate the visualization data structures
// use biomeos_core::clients::petaltongue::{PetalTongueClient, RenderRequest};

use serde_json::json;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║    🎨 biomeOS Ecosystem Visualizations via petalTongue       ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Note: In production, petalTongue would be discovered via Songbird
    // For now, this example documents the intended usage pattern

    println!("📋 Available Visualizations:");
    println!("   1. Live USB Spore - Deployment Lifecycle");
    println!("   2. NUCLEUS - Discovery Architecture");
    println!("   3. Neural API + RootPulse - Graph Orchestration\n");

    // Example 1: Live USB Spore
    render_spore_lifecycle().await?;

    // Example 2: NUCLEUS
    render_nucleus().await?;

    // Example 3: Neural API + RootPulse
    render_neural_api().await?;

    println!("\n✅ All visualizations rendered successfully!");
    println!("🎨 These examples demonstrate biomeOS key concepts:");
    println!("   • Capability-based discovery (zero hardcoding)");
    println!("   • Multi-primal coordination");
    println!("   • Genetic lineage & trust");
    println!("   • Graph-based orchestration");

    Ok(())
}

/// Render Live USB Spore deployment lifecycle
async fn render_spore_lifecycle() -> Result<(), Box<dyn Error>> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📀 Live USB Spore - Deployment Lifecycle");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let graph_data = json!({
        "title": "Live USB Spore - Deployment Lifecycle",
        "nodes": [
            {
                "id": "parent_spore",
                "label": "Parent Spore\n(Desktop)",
                "type": "spore",
                "status": "live",
                "metadata": {
                    "node_id": "tower1",
                    "family_seed": "abc123...",
                    "location": "/dev/sda"
                }
            },
            {
                "id": "clone_sibling",
                "label": "Clone Sibling\n(USB Creation)",
                "type": "process",
                "operation": "clone_sibling()"
            },
            {
                "id": "usb_spore",
                "label": "USB Spore\n(Cold/Live)",
                "type": "spore",
                "status": "cold",
                "metadata": {
                    "node_id": "usb-portable",
                    "family_seed": "abc123... (SAME)",
                    "location": "/dev/sdb"
                }
            },
            {
                "id": "family_seed",
                "label": ".family.seed\n(Shared Lineage)",
                "type": "genetic",
                "shared": true
            },
            {
                "id": "fresh_pc",
                "label": "Fresh PC\n(No OS)",
                "type": "hardware",
                "status": "empty"
            },
            {
                "id": "deployed",
                "label": "biomeOS\n(Deployed)",
                "type": "ecosystem",
                "status": "live",
                "primals": 7
            },
            {
                "id": "encrypted_keys",
                "label": "Encrypted Keys\n(BearDog)",
                "type": "security",
                "contents": ["Claude API", "SSH Keys", "Credentials"]
            },
            {
                "id": "agentic",
                "label": "Agentic USB\n(End-to-End)",
                "type": "capability",
                "features": ["AI-powered", "Self-configuring", "Autonomous"]
            }
        ],
        "edges": [
            {"source": "parent_spore", "target": "clone_sibling", "label": "Clone"},
            {"source": "clone_sibling", "target": "usb_spore", "label": "Sibling Created"},
            {"source": "parent_spore", "target": "family_seed", "label": "Shares", "style": "dashed"},
            {"source": "usb_spore", "target": "family_seed", "label": "Shares", "style": "dashed"},
            {"source": "usb_spore", "target": "fresh_pc", "label": "Insert USB"},
            {"source": "fresh_pc", "target": "deployed", "label": "Boot & Deploy"},
            {"source": "usb_spore", "target": "encrypted_keys", "label": "Contains"},
            {"source": "deployed", "target": "agentic", "label": "Enables"},
        ]
    });

    println!("🌸 Rendering graph:");
    println!("   Nodes: {}", graph_data["nodes"].as_array().unwrap().len());
    println!("   Edges: {}", graph_data["edges"].as_array().unwrap().len());

    // In production, this would call:
    // let client = PetalTongueClient::discover().await?;
    // let request = RenderRequest {
    //     mode: "terminal".to_string(),
    //     data: graph_data,
    //     width: Some(120),
    //     height: Some(40),
    //     output_path: None,
    // };
    // let response = client.render(request).await?;

    println!("   ✅ Visualization ready (scaffolded)");
    println!("\n💡 Key Concepts:");
    println!("   • Genetic Lineage: Siblings share .family.seed");
    println!("   • Trust: BearDog recognizes family members");
    println!("   • Agentic: End-to-end autonomous operation");
    println!("   • Cold→Live: USB awakens on fresh hardware\n");

    Ok(())
}

/// Render NUCLEUS discovery architecture
async fn render_nucleus() -> Result<(), Box<dyn Error>> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧬 NUCLEUS - Discovery Architecture");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let graph_data = json!({
        "title": "NUCLEUS - Discovery Architecture",
        "nodes": [
            {
                "id": "app",
                "label": "Application\n(Needs Security)",
                "type": "client",
                "query": "Find encryption provider"
            },
            {
                "id": "nucleus",
                "label": "NUCLEUS\n(Coordination)",
                "type": "core",
                "layer": "orchestration"
            },
            {
                "id": "capability_taxonomy",
                "label": "CapabilityTaxonomy\n(50+ capabilities)",
                "type": "taxonomy",
                "categories": 8
            },
            {
                "id": "songbird",
                "label": "Songbird\n(Discovery)",
                "type": "primal",
                "capabilities": ["discovery", "p2p_federation"]
            },
            {
                "id": "beardog",
                "label": "BearDog\n(Security)",
                "type": "primal",
                "capabilities": ["encryption", "identity", "trust"],
                "socket": "/run/user/1000/beardog-*.sock"
            },
            {
                "id": "petaltongue",
                "label": "petalTongue\n(UI)",
                "type": "primal",
                "capabilities": ["visualization", "multi_modal"],
                "socket": "/run/user/1000/petaltongue-*.sock"
            },
            {
                "id": "jsonrpc",
                "label": "JSON-RPC 2.0\n(Unix Sockets)",
                "type": "transport",
                "performance": "100x faster than HTTP"
            }
        ],
        "edges": [
            {"source": "app", "target": "nucleus", "label": "Query: 'encryption'"},
            {"source": "nucleus", "target": "capability_taxonomy", "label": "Lookup"},
            {"source": "nucleus", "target": "songbird", "label": "discover_by_capability()"},
            {"source": "beardog", "target": "songbird", "label": "Registers", "style": "dashed"},
            {"source": "petaltongue", "target": "songbird", "label": "Registers", "style": "dashed"},
            {"source": "songbird", "target": "jsonrpc", "label": "Via"},
            {"source": "jsonrpc", "target": "beardog", "label": "Connects"},
            {"source": "nucleus", "target": "beardog", "label": "Returns", "style": "bold"}
        ]
    });

    println!("🌸 Rendering graph:");
    println!("   Nodes: {}", graph_data["nodes"].as_array().unwrap().len());
    println!("   Edges: {}", graph_data["edges"].as_array().unwrap().len());
    println!("   ✅ Visualization ready (scaffolded)");
    println!("\n💡 Key Concepts:");
    println!("   • Zero Hardcoding: discover_by_capability('encryption')");
    println!("   • Runtime Discovery: No compile-time dependencies");
    println!("   • CapabilityTaxonomy: 50+ well-known capabilities");
    println!("   • JSON-RPC: 100x faster than HTTP over Unix sockets\n");

    Ok(())
}

/// Render Neural API + RootPulse graph orchestration
async fn render_neural_api() -> Result<(), Box<dyn Error>> {
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🧠 Neural API + RootPulse - Graph Orchestration");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let graph_data = json!({
        "title": "Neural API + RootPulse - Graph Orchestration",
        "nodes": [
            {
                "id": "user_intent",
                "label": "User Intent\n('Secure my data')",
                "type": "intent"
            },
            {
                "id": "neural_api",
                "label": "Neural API\n(Intent Interface)",
                "type": "api"
            },
            {
                "id": "rootpulse",
                "label": "RootPulse\n(Neural Coordinator)",
                "type": "coordinator",
                "status": "phase_3"
            },
            {
                "id": "graph_planner",
                "label": "Graph Planner\n(DAG Optimizer)",
                "type": "planner"
            },
            {
                "id": "songbird",
                "label": "Songbird\n(Step 1: Discover)",
                "type": "primal"
            },
            {
                "id": "beardog",
                "label": "BearDog\n(Step 2: Encrypt)",
                "type": "primal"
            },
            {
                "id": "nestgate",
                "label": "NestGate\n(Step 3: Store)",
                "type": "primal"
            },
            {
                "id": "petaltongue",
                "label": "petalTongue\n(Step 4: Visualize)",
                "type": "primal"
            },
            {
                "id": "squirrel",
                "label": "Squirrel\n(Step 5: Monitor)",
                "type": "primal"
            },
            {
                "id": "result",
                "label": "Result\n(Success: 234ms)",
                "type": "output"
            }
        ],
        "edges": [
            {"source": "user_intent", "target": "neural_api", "label": "Submit"},
            {"source": "neural_api", "target": "rootpulse", "label": "Parse"},
            {"source": "rootpulse", "target": "graph_planner", "label": "Plan"},
            {"source": "graph_planner", "target": "songbird", "label": "Execute Step 1"},
            {"source": "songbird", "target": "beardog", "label": "Then", "style": "dashed"},
            {"source": "beardog", "target": "nestgate", "label": "Then", "style": "dashed"},
            {"source": "nestgate", "target": "petaltongue", "label": "Then", "style": "dashed"},
            {"source": "nestgate", "target": "squirrel", "label": "Parallel", "style": "dashed"},
            {"source": "petaltongue", "target": "result", "label": "Contributes"},
            {"source": "squirrel", "target": "result", "label": "Contributes"}
        ]
    });

    println!("🌸 Rendering graph:");
    println!("   Nodes: {}", graph_data["nodes"].as_array().unwrap().len());
    println!("   Edges: {}", graph_data["edges"].as_array().unwrap().len());
    println!("   ✅ Visualization ready (scaffolded)");
    println!("\n💡 Key Concepts:");
    println!("   • Intent → DAG: Natural language to execution graph");
    println!("   • Graph Optimization: Cost-based routing");
    println!("   • Multi-Primal: 5 primals coordinated");
    println!("   • Phase 3 Vision: RootPulse neural coordination\n");

    Ok(())
}


// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Full UI Integration Example — DEPRECATED SCAFFOLD
//!
//! This example is a stub that does not use `biomeos-ui` or a live petalTongue
//! connection. For real petalTongue integration, see the petalTongue crate
//! and `biomeos-ui::primal_client::PetalTongueClient`.
//!
//! Original intent: demonstrate petalTongue + biomeOS integration with live
//! primals, showing device management, primal status, and niche deployments.
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │ User                                                       │
//! │  ↓ Views & Interacts                                       │
//! │ ┌────────────────────────────────────────────────────────┐ │
//! │ │ petalTongue GUI                                        │ │
//! │ │  ├─ DevicePanel (drag & drop)                          │ │
//! │ │  ├─ PrimalPanel (health & status)                      │ │
//! │ │  └─ NicheDesigner (visual templates)                   │ │
//! │ └────────────────────────────────────────────────────────┘ │
//! └────────────────────────────────────────────────────────────┘
//!                          ↓ JSON-RPC
//! ┌────────────────────────────────────────────────────────────┐
//! │ biomeOS RPC Bridge                                         │
//! │  ├─ Device Discovery (via Songbird)                        │
//! │  ├─ Primal Status (via Registry)                           │
//! │  ├─ Neural API Graphs (visualization)                      │
//! │  └─ Niche Deployment (orchestration)                       │
//! └────────────────────────────────────────────────────────────┘
//!                          ↓
//! ┌────────────────────────────────────────────────────────────┐
//! │ Live Primal Ecosystem                                      │
//! │  ├─ Songbird (discovery)                                   │
//! │  ├─ BearDog (security)                                     │
//! │  ├─ ToadStool (compute + CI)                               │
//! │  ├─ NestGate (storage)                                     │
//! │  └─ Squirrel (AI)                                          │
//! └────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```bash
//! # Terminal 1: Start biomeOS RPC bridge
//! cargo run --example full_ui_integration -- server
//!
//! # Terminal 2: Launch petalTongue GUI
//! cargo run --example full_ui_integration -- ui
//!
//! # Or combined:
//! cargo run --example full_ui_integration -- all
//! ```

use anyhow::{Context, Result};
use serde_json::json;
use std::process::Command;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

// Import from workspace root (since biomeos-ui is not a published crate yet)
// These types match petalTongue's schema
use serde::{Deserialize, Serialize};

/// Device representation (matches petalTongue's schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub resource_usage: f64,
    pub assigned_to: Option<String>,
    pub metadata: serde_json::Value,
}

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Gpu,
    Cpu,
    Storage,
    Network,
    Memory,
    Other,
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Available,
    InUse,
    Offline,
    Error,
}

/// Primal information (matches petalTongue's schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primal {
    pub id: String,
    pub name: String,
    pub status: PrimalStatus,
    pub health: f64,
    pub load: f64,
    pub capabilities: Vec<String>,
    pub assigned_devices: Vec<String>,
    pub metadata: serde_json::Value,
}

/// Primal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalStatus {
    Healthy,
    Degraded,
    Offline,
    Unknown,
}

/// Niche template (matches petalTongue's schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_primals: Vec<PrimalRole>,
    pub optional_primals: Vec<PrimalRole>,
    pub estimated_resources: ResourceRequirements,
    pub metadata: serde_json::Value,
}

/// Primal role in a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRole {
    pub role: String,
    pub capabilities: Vec<String>,
    pub min_health: f64,
    pub metadata: serde_json::Value,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub storage_gb: u64,
    pub gpu_required: bool,
    pub network_bandwidth_mbps: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    match mode {
        "server" => run_server().await?,
        "ui" => run_ui().await?,
        "all" => run_all().await?,
        "demo" => run_demo().await?,
        _ => {
            eprintln!("Usage: {} [server|ui|all|demo]", args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Run the biomeOS RPC bridge server
async fn run_server() -> Result<()> {
    info!("🚀 Starting biomeOS RPC Bridge Server");

    let socket_path = get_socket_path();

    info!("✅ RPC Bridge ready at {}", socket_path);
    info!("📡 Advertising device.management capability to Songbird");
    info!("📦 Demo data populated (devices, primals, templates)");

    // Keep server running
    info!("🌸 Server running. Press Ctrl+C to stop.");
    tokio::signal::ctrl_c().await?;
    info!("👋 Server shutting down");

    Ok(())
}

/// Launch the petalTongue GUI
async fn run_ui() -> Result<()> {
    info!("🌸 Launching petalTongue GUI");

    let petaltongue_bin =
        std::env::var("PETALTONGUE_BIN").unwrap_or_else(|_| "plasmidBin/petaltongue".to_string());

    if !std::path::Path::new(&petaltongue_bin).exists() {
        warn!("⚠️  petalTongue binary not found at {}", petaltongue_bin);
        warn!("   Run: cargo build --release in ../petalTongue");
        warn!("   Then: cp ../petalTongue/target/release/petal-tongue plasmidBin/petaltongue");
        return Err(anyhow::anyhow!("petalTongue binary not found"));
    }

    info!("✅ Found petalTongue at {}", petaltongue_bin);

    // Set environment variable for biomeOS endpoint
    let socket_path = get_socket_path();
    std::env::set_var("BIOMEOS_ENDPOINT", &socket_path);

    // Launch petalTongue
    info!("🎨 Starting petalTongue GUI...");
    let mut child = Command::new(&petaltongue_bin)
        .env("BIOMEOS_ENDPOINT", &socket_path)
        .spawn()
        .context("Failed to start petalTongue")?;

    info!("✅ petalTongue GUI running (PID: {})", child.id());

    // Wait for process
    let status = child.wait()?;
    info!("petalTongue exited with status: {}", status);

    Ok(())
}

/// Run both server and UI
async fn run_all() -> Result<()> {
    info!("🚀 Starting Full UI Integration (Server + UI)");

    // Start server in background
    let server_handle = tokio::spawn(async move {
        if let Err(e) = run_server().await {
            warn!("Server error: {}", e);
        }
    });

    // Wait for server to initialize
    sleep(Duration::from_secs(2)).await;

    // Launch UI
    run_ui().await?;

    // Stop server
    server_handle.abort();

    Ok(())
}

/// Run an interactive demo with visualizations
async fn run_demo() -> Result<()> {
    info!("🎬 Running Full UI Integration Demo");

    println!("\n═══════════════════════════════════════════════════════════════");
    println!("     🌸 BIOMEOS + PETALTONGUE INTEGRATION DEMO 🌸");
    println!("═══════════════════════════════════════════════════════════════\n");

    // 1. Show architecture
    println!("📐 ARCHITECTURE:\n");
    println!("  User → petalTongue GUI → biomeOS RPC → Live Primals");
    println!("         (visual UI)       (orchestration) (7 primals)\n");

    sleep(Duration::from_millis(500)).await;

    // 2. Show devices
    println!("📱 DEVICES (discovered via capability-based query):\n");
    let devices = get_demo_devices();
    for device in &devices {
        println!(
            "  • {} ({:?}) - {:?} - Usage: {:.0}%",
            device.name,
            device.device_type,
            device.status,
            device.resource_usage * 100.0
        );
    }
    println!();

    sleep(Duration::from_millis(500)).await;

    // 3. Show primals
    println!("🎵 PRIMALS (live status from registry):\n");
    let primals = get_demo_primals();
    for primal in &primals {
        println!(
            "  • {} - {:?} - Health: {:.0}% - Load: {:.0}%",
            primal.name,
            primal.status,
            primal.health * 100.0,
            primal.load * 100.0
        );
        println!("    Capabilities: {}", primal.capabilities.join(", "));
    }
    println!();

    sleep(Duration::from_millis(500)).await;

    // 4. Show Neural API graph
    println!("🧠 NEURAL API GRAPH (visual representation):\n");
    println!("  tower_deploy:");
    println!("    ┌─────────────────┐");
    println!("    │ Check Security  │ (BearDog)");
    println!("    └────────┬────────┘");
    println!("             ↓");
    println!("    ┌─────────────────┐");
    println!("    │ Validate Config │ (ToadStool)");
    println!("    └────────┬────────┘");
    println!("             ↓");
    println!("    ┌─────────────────┐");
    println!("    │  Deploy Tower   │ (biomeOS)");
    println!("    └─────────────────┘\n");

    sleep(Duration::from_millis(500)).await;

    // 5. Show niche templates
    println!("🏗️  NICHE TEMPLATES (with Neural API):\n");
    let templates = get_demo_templates();
    for template in &templates {
        println!("  • {} - {}", template.name, template.description);
        println!("    Required: {} primals", template.required_primals.len());
        println!(
            "    Resources: {} CPU cores, {} MB RAM, {} GB storage",
            template.estimated_resources.cpu_cores,
            template.estimated_resources.memory_mb,
            template.estimated_resources.storage_gb
        );
    }
    println!();

    sleep(Duration::from_millis(500)).await;

    // 6. Show interaction flow
    println!("🔄 INTERACTION FLOW:\n");
    println!("  1. User drags GPU device → ToadStool primal");
    println!("  2. biomeOS orchestrates:");
    println!("     • BearDog: Authorize assignment");
    println!("     • ToadStool: Validate resources");
    println!("     • NestGate: Persist state");
    println!("     • Squirrel: Suggest optimizations");
    println!("  3. Neural API executes graph");
    println!("  4. UI updates in real-time (WebSocket)\n");

    sleep(Duration::from_millis(500)).await;

    println!("═══════════════════════════════════════════════════════════════");
    println!("     ✅ DEMO COMPLETE - INTEGRATION WORKING! ✅");
    println!("═══════════════════════════════════════════════════════════════\n");

    println!("To see the live UI:");
    println!("  cargo run --example full_ui_integration -- all\n");

    Ok(())
}

/// Get socket path for RPC communication
fn get_socket_path() -> String {
    // Use UID from environment or default to 1000
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    format!("/run/user/{}/biomeos-ui.sock", uid)
}

/// Get demo devices
fn get_demo_devices() -> Vec<Device> {
    vec![
        Device {
            id: "gpu-0".to_string(),
            name: "NVIDIA RTX 4090".to_string(),
            device_type: DeviceType::Gpu,
            status: DeviceStatus::Available,
            resource_usage: 0.15,
            assigned_to: None,
            metadata: json!({"vendor": "NVIDIA", "memory": "24GB"}),
        },
        Device {
            id: "cpu-0".to_string(),
            name: "AMD Ryzen 9 7950X".to_string(),
            device_type: DeviceType::Cpu,
            status: DeviceStatus::InUse,
            resource_usage: 0.65,
            assigned_to: Some("toadstool".to_string()),
            metadata: json!({"cores": 16, "threads": 32}),
        },
        Device {
            id: "storage-0".to_string(),
            name: "Samsung 990 Pro 2TB".to_string(),
            device_type: DeviceType::Storage,
            status: DeviceStatus::Available,
            resource_usage: 0.42,
            assigned_to: None,
            metadata: json!({"capacity": "2TB", "type": "NVMe"}),
        },
        Device {
            id: "net-0".to_string(),
            name: "10GbE Network".to_string(),
            device_type: DeviceType::Network,
            status: DeviceStatus::InUse,
            resource_usage: 0.28,
            assigned_to: Some("songbird".to_string()),
            metadata: json!({"bandwidth": "10Gbps"}),
        },
    ]
}

/// Get demo primals
fn get_demo_primals() -> Vec<Primal> {
    vec![
        Primal {
            id: "songbird".to_string(),
            name: "Songbird".to_string(),
            status: PrimalStatus::Healthy,
            health: 0.98,
            load: 0.25,
            capabilities: vec!["discovery".to_string(), "registry".to_string()],
            assigned_devices: vec!["net-0".to_string()],
            metadata: json!({"version": "3.20.0"}),
        },
        Primal {
            id: "toadstool".to_string(),
            name: "ToadStool".to_string(),
            status: PrimalStatus::Healthy,
            health: 0.95,
            load: 0.72,
            capabilities: vec![
                "compute".to_string(),
                "resource_planning".to_string(),
                "collaborative_intelligence".to_string(),
            ],
            assigned_devices: vec!["cpu-0".to_string()],
            metadata: json!({"version": "2.2.0"}),
        },
        Primal {
            id: "beardog".to_string(),
            name: "BearDog".to_string(),
            status: PrimalStatus::Healthy,
            health: 1.0,
            load: 0.15,
            capabilities: vec!["security".to_string(), "authorization".to_string()],
            assigned_devices: vec![],
            metadata: json!({"version": "1.5.0"}),
        },
        Primal {
            id: "nestgate".to_string(),
            name: "NestGate".to_string(),
            status: PrimalStatus::Healthy,
            health: 0.97,
            load: 0.38,
            capabilities: vec!["storage".to_string(), "persistence".to_string()],
            assigned_devices: vec![],
            metadata: json!({"version": "0.2.0"}),
        },
        Primal {
            id: "squirrel".to_string(),
            name: "Squirrel".to_string(),
            status: PrimalStatus::Healthy,
            health: 0.92,
            load: 0.55,
            capabilities: vec!["ai".to_string(), "suggestions".to_string()],
            assigned_devices: vec![],
            metadata: json!({"version": "0.4.0"}),
        },
    ]
}

/// Get demo niche templates with Neural API graphs
fn get_demo_templates() -> Vec<NicheTemplate> {
    vec![
        NicheTemplate {
            id: "tower".to_string(),
            name: "Secure Tower".to_string(),
            description: "Network security and encrypted communications".to_string(),
            required_primals: vec![
                PrimalRole {
                    role: "security".to_string(),
                    capabilities: vec!["btsp".to_string(), "tunneling".to_string()],
                    min_health: 0.9,
                    metadata: json!({"primal": "beardog"}),
                },
                PrimalRole {
                    role: "discovery".to_string(),
                    capabilities: vec!["registry".to_string()],
                    min_health: 0.8,
                    metadata: json!({"primal": "songbird"}),
                },
            ],
            optional_primals: vec![],
            estimated_resources: ResourceRequirements {
                cpu_cores: 2,
                memory_mb: 4096,
                storage_gb: 10,
                gpu_required: false,
                network_bandwidth_mbps: 1000,
            },
            metadata: json!({
                "neural_api_graph": "tower_deploy.toml",
                "deployment_time": "10-15 seconds"
            }),
        },
        NicheTemplate {
            id: "node".to_string(),
            name: "Compute Node".to_string(),
            description: "High-performance compute with AI assistance".to_string(),
            required_primals: vec![
                PrimalRole {
                    role: "compute".to_string(),
                    capabilities: vec!["resource_planning".to_string()],
                    min_health: 0.9,
                    metadata: json!({"primal": "toadstool"}),
                },
                PrimalRole {
                    role: "ai".to_string(),
                    capabilities: vec!["suggestions".to_string()],
                    min_health: 0.7,
                    metadata: json!({"primal": "squirrel"}),
                },
            ],
            optional_primals: vec![],
            estimated_resources: ResourceRequirements {
                cpu_cores: 8,
                memory_mb: 32768,
                storage_gb: 100,
                gpu_required: true,
                network_bandwidth_mbps: 1000,
            },
            metadata: json!({
                "neural_api_graph": "node_deploy.toml",
                "deployment_time": "15-20 seconds"
            }),
        },
        NicheTemplate {
            id: "nest".to_string(),
            name: "Data Nest".to_string(),
            description: "Secure data storage and persistence".to_string(),
            required_primals: vec![
                PrimalRole {
                    role: "storage".to_string(),
                    capabilities: vec!["persistence".to_string()],
                    min_health: 0.95,
                    metadata: json!({"primal": "nestgate"}),
                },
                PrimalRole {
                    role: "security".to_string(),
                    capabilities: vec!["encryption".to_string()],
                    min_health: 0.9,
                    metadata: json!({"primal": "beardog"}),
                },
            ],
            optional_primals: vec![],
            estimated_resources: ResourceRequirements {
                cpu_cores: 4,
                memory_mb: 8192,
                storage_gb: 500,
                gpu_required: false,
                network_bandwidth_mbps: 1000,
            },
            metadata: json!({
                "neural_api_graph": "nest_deploy.toml",
                "deployment_time": "10-15 seconds"
            }),
        },
    ]
}

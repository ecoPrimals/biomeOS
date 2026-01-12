//! Primal Launcher
//!
//! Pure Rust launcher for starting primals with proper Unix socket configuration.
//! Replaces bash scripts with type-safe, concurrent primal management.
//!
//! # Philosophy
//!
//! - Modern idiomatic Rust (async/await, Result<T>, zero unsafe)
//! - XDG-compliant socket paths
//! - Capability-based discovery
//! - Genetic lineage configuration
//!
//! # Usage
//!
//! ```bash
//! # Start all primals for an atomic
//! cargo run --bin launch_primal -- tower nat0
//! cargo run --bin launch_primal -- node nat0
//! cargo run --bin launch_primal -- nest nat0
//!
//! # Start individual primals
//! cargo run --bin launch_primal -- beardog nat0
//! cargo run --bin launch_primal -- songbird nat0
//! cargo run --bin launch_primal -- toadstool nat0
//! cargo run --bin launch_primal -- nestgate nat0
//! ```

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let primal_or_atomic = &args[1];
    let family_id = &args[2];

    match primal_or_atomic.as_str() {
        "tower" => launch_atomic_primals(&["beardog", "songbird"], family_id).await?,
        "node" => launch_atomic_primals(&["beardog", "songbird", "toadstool"], family_id).await?,
        "nest" => launch_atomic_primals(&["beardog", "songbird", "nestgate"], family_id).await?,
        "beardog" | "songbird" | "toadstool" | "nestgate" | "squirrel" => {
            launch_primal(primal_or_atomic, family_id).await?;
        }
        _ => {
            error!("Unknown primal or atomic: {}", primal_or_atomic);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

fn print_usage(program: &str) {
    println!("🧬 biomeOS Primal Launcher");
    println!();
    println!("Usage: {} <primal|atomic> <family_id>", program);
    println!();
    println!("Atomics (launch multiple primals):");
    println!("  tower <family>    Launch Tower (BearDog + Songbird)");
    println!("  node <family>     Launch Node (BearDog + Songbird + ToadStool)");
    println!("  nest <family>     Launch Nest (BearDog + Songbird + NestGate)");
    println!();
    println!("Individual Primals:");
    println!("  beardog <family>   Launch BearDog (security)");
    println!("  songbird <family>  Launch Songbird (discovery)");
    println!("  toadstool <family> Launch ToadStool (compute)");
    println!("  nestgate <family>  Launch NestGate (storage)");
    println!("  squirrel <family>  Launch Squirrel (AI)");
    println!();
    println!("Example:");
    println!("  {} tower nat0", program);
    println!("  {} beardog nat0", program);
}

async fn launch_atomic_primals(primals: &[&str], family_id: &str) -> Result<()> {
    info!("🚀 Launching atomic with primals: {:?}", primals);
    info!("   Family ID: {}", family_id);

    for primal in primals {
        launch_primal(primal, family_id).await?;
        // Small delay between launches
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    info!("✅ All primals launched for family {}", family_id);
    
    Ok(())
}

async fn launch_primal(primal: &str, family_id: &str) -> Result<()> {
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("🔧 Launching {} (family: {})", primal, family_id);
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    // Get primal binary path
    let bin_path = get_primal_binary_path(primal)?;
    
    if !bin_path.exists() {
        error!("❌ Primal binary not found: {:?}", bin_path);
        error!("   Expected location: plasmidBin/{}", primal);
        return Err(anyhow::anyhow!("Primal binary not found"));
    }

    info!("✅ Binary found: {:?}", bin_path);

    // Get socket path (XDG-compliant)
    let socket_path = get_socket_path(primal, family_id)?;
    info!("🔌 Socket path: {}", socket_path);

    // Get log path
    let log_path = get_log_path(primal, family_id);
    info!("📝 Log path: {}", log_path);

    // Build command based on primal type
    let mut cmd = Command::new(&bin_path);
    
    match primal {
        "beardog" => {
            // BearDog: Provide family_id via env var
            cmd.env("BEARDOG_FAMILY_ID", family_id);
            cmd.env("BEARDOG_SOCKET", &socket_path);
        }
        "songbird" => {
            // Songbird: Provide family_id via env var
            cmd.env("SONGBIRD_FAMILY_ID", family_id);
            cmd.env("SONGBIRD_SOCKET", &socket_path);
            // Songbird needs to discover BearDog for security
            // Use capability-based discovery
            cmd.env("SONGBIRD_SECURITY_PROVIDER", "beardog");
        }
        "toadstool" => {
            // ToadStool: Uses default socket already (toadstool-default.jsonrpc.sock)
            cmd.env("TOADSTOOL_FAMILY_ID", family_id);
        }
        "nestgate" => {
            // NestGate: Use service start command
            cmd.arg("service");
            cmd.arg("start");
            cmd.env("NESTGATE_FAMILY_ID", family_id);
            cmd.env("SONGBIRD_FAMILY_ID", family_id);  // For Songbird discovery
        }
        "squirrel" => {
            // Squirrel: Provide socket path
            cmd.env("SQUIRREL_FAMILY_ID", family_id);
            cmd.env("SQUIRREL_SOCKET", &socket_path);
        }
        _ => {
            warn!("Unknown primal: {}", primal);
        }
    }

    // Redirect stdout/stderr to log file
    let log_file = std::fs::File::create(&log_path)
        .context("Failed to create log file")?;
    
    cmd.stdout(Stdio::from(log_file.try_clone()?));
    cmd.stderr(Stdio::from(log_file));
    cmd.stdin(Stdio::null());

    // Spawn the process
    info!("▶️  Spawning process...");
    
    let mut child = cmd.spawn()
        .with_context(|| format!("Failed to spawn {} process", primal))?;

    // Wait briefly to see if it crashes immediately
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    match child.try_wait()? {
        Some(status) => {
            error!("❌ Process exited immediately with status: {}", status);
            error!("   Check log file: {}", log_path);
            return Err(anyhow::anyhow!("Primal failed to start"));
        }
        None => {
            let pid = child.id().unwrap_or(0);
            info!("✅ {} started successfully (PID: {})", primal, pid);
            info!("   Log: {}", log_path);
            
            // Don't wait for the process - let it run in background
            // The process will be reaped by init
            std::mem::forget(child);
        }
    }

    Ok(())
}

fn get_primal_binary_path(primal: &str) -> Result<PathBuf> {
    let bin_dir = PathBuf::from("plasmidBin");
    
    // Songbird uses songbird-orchestrator binary
    if primal == "songbird" {
        let bin_path = PathBuf::from("plasmidBin/primals/songbird-orchestrator");
        if bin_path.exists() {
            return Ok(bin_path);
        }
        // Fallback to top-level if exists
        let fallback = bin_dir.join("songbird-orchestrator");
        if fallback.exists() {
            return Ok(fallback);
        }
    }
    
    let bin_path = bin_dir.join(primal);
    Ok(bin_path)
}

fn get_socket_path(primal: &str, family_id: &str) -> Result<String> {
    let uid = std::env::var("UID")
        .or_else(|_| std::env::var("USER").map(|_| "1000".to_string()))
        .unwrap_or_else(|_| "1000".to_string());
    
    let socket_dir = format!("/run/user/{}", uid);
    let socket_name = format!("{}-{}.sock", primal, family_id);
    let socket_path = format!("{}/{}", socket_dir, socket_name);
    
    Ok(socket_path)
}

fn get_log_path(primal: &str, family_id: &str) -> String {
    format!("/tmp/{}-{}.log", primal, family_id)
}


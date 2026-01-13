//! NUCLEUS Deployment Binary
//!
//! Pure Rust orchestration system for deploying complete biomeOS NUCLEUS.
//! Replaces bash scripts with proper Neural API + graph orchestration.
//!
//! NUCLEUS = Node + Tower + Nest on a single gate (liveSpore)

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🧬 NUCLEUS Deployment System");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("deploy");

    match mode {
        "deploy" => deploy_nucleus().await?,
        "verify" => verify_nucleus().await?,
        "status" => show_status().await?,
        "ui" => launch_ui().await?,
        "all" => deploy_and_launch().await?,
        _ => {
            eprintln!("Usage: {} [deploy|verify|status|ui|all]", args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Deploy complete NUCLEUS using Neural API graph
async fn deploy_nucleus() -> Result<()> {
    info!("🚀 Deploying NUCLEUS (Node + Tower + Nest)");
    info!("");

    // Verify graph exists
    let graph_path = PathBuf::from("graphs/nucleus_deploy.toml");
    info!("📊 Graph template: {:?}", graph_path);

    if !tokio::fs::metadata(&graph_path).await.is_ok() {
        warn!("⚠️  Graph template not found");
        warn!("   Expected: graphs/nucleus_deploy.toml");
        return Err(anyhow::anyhow!("NUCLEUS deployment graph not found"));
    }

    info!("✅ Graph template found");
    info!("");

    // TODO: Load and execute via Neural API once graph executor is integrated
    // For now, provide clear next steps
    info!("📋 NUCLEUS Deployment Steps:");
    info!("   1. ✅ Graph defined (graphs/nucleus_deploy.toml)");
    info!("   2. ⏳ Neural API integration (pending)");
    info!("   3. ⏳ Primal coordination via Songbird");
    info!("   4. ⏳ Real-time visualization in petalTongue");
    info!("");
    info!("💡 Next: Deploy via UI (petalTongue NicheDesigner)");

    Ok(())
}

/// Verify NUCLEUS health
async fn verify_nucleus() -> Result<()> {
    info!("🔍 Verifying NUCLEUS health...");
    info!("");

    // Check for required primal sockets
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let socket_dir = format!("/run/user/{}", uid);

    let required_primals = vec!["beardog", "toadstool", "nestgate"];
    let mut healthy = true;

    for primal in required_primals {
        match check_primal_health(&socket_dir, primal).await {
            Ok(()) => info!("  ✅ {} is healthy", primal),
            Err(e) => {
                warn!("  ❌ {} is unhealthy: {}", primal, e);
                healthy = false;
            }
        }
    }

    info!("");

    if healthy {
        info!("✅ NUCLEUS is healthy");
        Ok(())
    } else {
        warn!("⚠️  NUCLEUS has issues");
        Err(anyhow::anyhow!("Some components are unhealthy"))
    }
}

/// Check individual primal health
async fn check_primal_health(socket_dir: &str, primal: &str) -> Result<()> {
    // Look for primal socket
    let mut entries = tokio::fs::read_dir(socket_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if let Some(name) = entry.file_name().to_str() {
            if name.starts_with(&format!("{}-", primal)) && name.ends_with(".sock") {
                // Found socket, try to connect
                let socket_path = format!("{}/{}", socket_dir, name);
                match tokio::net::UnixStream::connect(&socket_path).await {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(anyhow::anyhow!("Cannot connect: {}", e)),
                }
            }
        }
    }

    Err(anyhow::anyhow!("Socket not found"))
}

/// Show NUCLEUS status
async fn show_status() -> Result<()> {
    info!("📊 NUCLEUS Status");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    // Show running primals
    info!("Running Primals:");
    let output = tokio::process::Command::new("pgrep")
        .args(&["-a", "-f", "beardog|toadstool|nestgate|squirrel"])
        .output()
        .await?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            info!("  {}", line);
        }
    } else {
        warn!("  No primals detected");
    }

    info!("");

    // Show sockets
    info!("Available Sockets:");
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let socket_dir = format!("/run/user/{}", uid);

    let mut entries = tokio::fs::read_dir(&socket_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        if let Some(name) = entry.file_name().to_str() {
            if name.contains("beardog")
                || name.contains("toadstool")
                || name.contains("nestgate")
                || name.contains("biomeos")
            {
                info!("  {}", name);
            }
        }
    }

    info!("");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    Ok(())
}

/// Launch petalTongue UI
async fn launch_ui() -> Result<()> {
    info!("🌸 Launching petalTongue UI...");

    let petaltongue_bin = "plasmidBin/petaltongue";

    if !tokio::fs::metadata(petaltongue_bin).await.is_ok() {
        return Err(anyhow::anyhow!(
            "petalTongue binary not found at {}",
            petaltongue_bin
        ));
    }

    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let biomeos_socket = format!("unix:///run/user/{}/biomeos-device-management.sock", uid);

    info!("🔌 Connecting to: {}", biomeos_socket);

    let mut child = tokio::process::Command::new(petaltongue_bin)
        .env("BIOMEOS_URL", &biomeos_socket)
        .env("RUST_LOG", "info")
        .spawn()
        .context("Failed to launch petalTongue")?;

    info!("✅ petalTongue launched (PID: {})", child.id().unwrap_or(0));

    // Wait for process
    child.wait().await?;

    Ok(())
}

/// Deploy NUCLEUS and launch UI
async fn deploy_and_launch() -> Result<()> {
    info!("🚀 Complete NUCLEUS Deployment + UI Launch");
    info!("");

    // Deploy NUCLEUS
    deploy_nucleus().await?;

    info!("");
    info!("⏳ Waiting 2 seconds for system to stabilize...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Launch UI
    launch_ui().await?;

    Ok(())
}

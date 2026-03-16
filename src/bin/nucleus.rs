// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! NUCLEUS Deployment Binary
//!
//! Pure Rust orchestration system for deploying complete biomeOS NUCLEUS.
//! Uses Neural API graph executor for ecosystem orchestration.
//!
//! NUCLEUS = Tower (BearDog + Songbird) + Node (Toadstool) + Nest (NestGate)
//!         + AI (Squirrel) + Visualization (petalTongue)

use anyhow::{Context, Result};
use biomeos_atomic_deploy::neural_api_server::NeuralApiServer;
use biomeos_atomic_deploy::neural_executor::GraphExecutor;
use biomeos_atomic_deploy::neural_graph::Graph;
use biomeos_core::family_discovery;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, warn};

/// Parse mode from args. Extracted for testability.
fn parse_mode(args: &[String]) -> &str {
    args.get(1).map(|s| s.as_str()).unwrap_or("deploy")
}

/// Parse --family FAMILY_ID from args. Returns None if not present.
fn parse_family_id_arg(args: &[String]) -> Option<String> {
    args.iter()
        .position(|arg| arg == "--family")
        .and_then(|i| args.get(i + 1))
        .cloned()
}

/// Parse --graph PATH from args. Returns default if not present.
fn parse_graph_path_arg(args: &[String]) -> String {
    args.iter()
        .position(|arg| arg == "--graph")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| "graphs/nucleus_ecosystem.toml".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    info!("🧬 NUCLEUS Ecosystem Deployment");
    info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    info!("");

    let args: Vec<String> = std::env::args().collect();
    let mode = parse_mode(&args);
    let family_id = parse_family_id_arg(&args).unwrap_or_else(family_discovery::get_family_id);
    let graph_path = parse_graph_path_arg(&args);

    match mode {
        "deploy" => deploy_nucleus(&family_id, &graph_path).await?,
        "serve" => serve_neural_api(&family_id).await?,
        "verify" => verify_nucleus().await?,
        "status" => show_status().await?,
        "ui" => launch_ui().await?,
        "all" => deploy_and_launch(&family_id, &graph_path).await?,
        _ => {
            eprintln!(
                "Usage: {} [deploy|serve|verify|status|ui|all] [--family FAMILY_ID] [--graph PATH]",
                args[0]
            );
            eprintln!();
            eprintln!("Commands:");
            eprintln!("  deploy    Deploy NUCLEUS ecosystem from graph");
            eprintln!("  serve     Start Neural API JSON-RPC server");
            eprintln!("  verify    Verify NUCLEUS health");
            eprintln!("  status    Show NUCLEUS status");
            eprintln!("  ui        Launch visualization UI");
            eprintln!("  all       Deploy and launch everything");
            eprintln!();
            eprintln!("Options:");
            eprintln!(
                "  --family FAMILY_ID    Genetic family ID (auto-discovered from .family.seed)"
            );
            eprintln!(
                "  --graph PATH          Graph definition (default: graphs/nucleus_ecosystem.toml)"
            );
            eprintln!();
            eprintln!("Examples:");
            eprintln!("  {} deploy", args[0]);
            eprintln!("  {} serve --family cf7e8729dc4ff05f", args[0]);
            eprintln!("  {} deploy --graph graphs/nucleus_ecosystem.toml", args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Deploy complete NUCLEUS using Neural API graph
async fn deploy_nucleus(family_id: &str, graph_path: &str) -> Result<()> {
    info!("🚀 Deploying NUCLEUS Ecosystem");
    info!("   Family: {}", family_id);
    info!("   Graph: {}", graph_path);
    info!("");

    // Verify graph exists
    let graph_file = PathBuf::from(graph_path);
    if tokio::fs::metadata(&graph_file).await.is_err() {
        return Err(anyhow::anyhow!("Graph not found: {graph_path}"));
    }

    // Load graph
    info!("📊 Loading graph definition...");
    let graph_content = tokio::fs::read_to_string(&graph_file).await?;
    let graph = Graph::from_toml_str(&graph_content).context("Failed to parse graph TOML")?;

    info!(
        "✅ Graph loaded: {} ({} nodes)",
        graph.id,
        graph.nodes.len()
    );
    info!("");

    // Prepare environment
    let uid = std::env::var("UID").unwrap_or_else(|_| {
        // Fallback: read /proc/self for current UID
        "1000".to_string()
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), family_id.to_string());
    env.insert("UID".to_string(), uid.clone());
    env.insert("RUNTIME_DIR".to_string(), format!("/run/user/{uid}"));
    env.insert("SOCKET_DIR".to_string(), format!("/run/user/{uid}"));
    env.insert("LOG_DIR".to_string(), "/tmp".to_string());

    // JWT secret for NestGate
    // NOTE: In production, NestGate should use BearDog for authentication
    // This env var is for development/testing only
    env.insert(
        "JWT_SECRET".to_string(),
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "DEVELOPMENT_ONLY_SECRET".to_string()),
    );

    info!("🌍 Environment:");
    info!("   FAMILY_ID: {}", family_id);
    info!("   UID: {}", uid);
    info!("   SOCKET_DIR: /run/user/{}", uid);
    info!("");

    // Execute graph
    info!("🧠 Executing Neural API graph...");
    info!("");

    let mut executor = GraphExecutor::new(graph, env);
    match executor.execute().await {
        Ok(report) => {
            info!("");
            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            if report.success {
                info!("✅ NUCLEUS ECOSYSTEM DEPLOYED!");
            } else {
                warn!("⚠️  NUCLEUS DEPLOYMENT COMPLETED WITH ISSUES");
            }

            info!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            info!("");
            info!("Graph: {}", report.graph_id);
            info!("Success: {}", report.success);
            info!("Duration: {} ms", report.duration_ms);
            info!("Phases: {}", report.phase_results.len());
            info!("");

            if !report.success {
                if let Some(error) = &report.error {
                    warn!("Error: {}", error);
                }
                return Err(anyhow::anyhow!("Deployment failed"));
            }

            info!("🎯 Next Steps:");
            info!("  1. Verify: nucleus verify");
            info!("  2. Status: nucleus status");
            info!("  3. Visualize: nucleus ui");
            info!("");

            Ok(())
        }
        Err(e) => {
            warn!("❌ Deployment failed: {}", e);
            Err(e)
        }
    }
}

/// Verify NUCLEUS health
async fn verify_nucleus() -> Result<()> {
    info!("🔍 Verifying NUCLEUS health...");
    info!("");

    // Check for required primal sockets
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let socket_dir = format!("/run/user/{uid}");

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
            if name.starts_with(&format!("{primal}-")) && name.ends_with(".sock") {
                // Found socket, try to connect
                let socket_path = format!("{socket_dir}/{name}");
                match tokio::net::UnixStream::connect(&socket_path).await {
                    Ok(_) => return Ok(()),
                    Err(e) => return Err(anyhow::anyhow!("Cannot connect: {e}")),
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
        .args(["-a", "-f", "beardog|toadstool|nestgate|squirrel"])
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
    let socket_dir = format!("/run/user/{uid}");

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

    if tokio::fs::metadata(petaltongue_bin).await.is_err() {
        return Err(anyhow::anyhow!(
            "petalTongue binary not found at {petaltongue_bin}"
        ));
    }

    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let biomeos_socket = format!("unix:///run/user/{uid}/biomeos-device-management.sock");

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
async fn deploy_and_launch(family_id: &str, graph_path: &str) -> Result<()> {
    info!("🚀 Complete NUCLEUS Deployment + UI Launch");
    info!("");

    // Deploy NUCLEUS
    deploy_nucleus(family_id, graph_path).await?;

    info!("");
    info!("⏳ Waiting 5 seconds for ecosystem to stabilize...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    // Launch UI
    launch_ui().await?;

    Ok(())
}

/// Serve Neural API JSON-RPC server
async fn serve_neural_api(family_id: &str) -> Result<()> {
    info!("🧠 Starting Neural API Server");
    info!("   Family: {}", family_id);
    info!("");

    // Determine socket path
    let socket_path = format!("/tmp/biomeos-neural-api-{family_id}.sock");

    // Graphs directory
    let graphs_dir = "graphs";

    info!("📊 Configuration:");
    info!("   Socket: {}", socket_path);
    info!("   Graphs: {}", graphs_dir);
    info!("");

    // Create Neural API server
    let server = NeuralApiServer::new(graphs_dir, family_id, &socket_path);

    info!("✅ Neural API server ready");
    info!("");
    info!("📡 Squirrel and petalTongue can now connect to:");
    info!("   {}", socket_path);
    info!("");
    info!("Press Ctrl+C to stop");
    info!("");

    // Serve (blocks until interrupted)
    server.serve().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mode() {
        assert_eq!(parse_mode(&[]), "deploy");
        assert_eq!(parse_mode(&["nucleus".into(), "deploy".into()]), "deploy");
        assert_eq!(parse_mode(&["nucleus".into(), "serve".into()]), "serve");
        assert_eq!(parse_mode(&["nucleus".into(), "verify".into()]), "verify");
        assert_eq!(parse_mode(&["nucleus".into(), "status".into()]), "status");
        assert_eq!(parse_mode(&["nucleus".into(), "ui".into()]), "ui");
        assert_eq!(parse_mode(&["nucleus".into(), "all".into()]), "all");
    }

    #[test]
    fn test_parse_family_id_arg() {
        assert!(parse_family_id_arg(&["nucleus".into(), "deploy".into()]).is_none());
        assert_eq!(
            parse_family_id_arg(&[
                "nucleus".into(),
                "deploy".into(),
                "--family".into(),
                "cf7e8729".into()
            ]),
            Some("cf7e8729".to_string())
        );
        assert_eq!(
            parse_family_id_arg(&[
                "nucleus".into(),
                "--family".into(),
                "nat0".into(),
                "serve".into()
            ]),
            Some("nat0".to_string())
        );
    }

    #[test]
    fn test_parse_graph_path_arg() {
        assert_eq!(
            parse_graph_path_arg(&["nucleus".into(), "deploy".into()]),
            "graphs/nucleus_ecosystem.toml"
        );
        assert_eq!(
            parse_graph_path_arg(&[
                "nucleus".into(),
                "deploy".into(),
                "--graph".into(),
                "graphs/custom.toml".into()
            ]),
            "graphs/custom.toml"
        );
    }
}

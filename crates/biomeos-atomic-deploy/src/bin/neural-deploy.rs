// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![forbid(unsafe_code)]

//! Neural Deploy Client Binary
//!
//! Client for deploying ecosystems via Neural API graphs.
//!
//! Usage:
//!   `neural-deploy <graph-id> [--family-id <ID>]`
//!
//! Examples:
//!   neural-deploy 01_nucleus_enclave
//!   neural-deploy 00_full_ecosystem --family-id nat0

use anyhow::{Context, Result};
use serde_json::json;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::filter::EnvFilter::new("info")),
        )
        .init();

    // Parse command line args
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: neural-deploy <graph-id> [--family-id <ID>]");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  neural-deploy 01_nucleus_enclave");
        eprintln!("  neural-deploy 00_full_ecosystem --family-id nat0");
        std::process::exit(1);
    }

    let graph_id = &args[1];
    let family_id = args
        .iter()
        .position(|arg| arg == "--family-id")
        .and_then(|i| args.get(i + 1))
        .map_or("nat0", std::string::String::as_str);

    let socket_path = format!("/tmp/neural-api-{family_id}.sock");

    info!("╔══════════════════════════════════════════════════════════════════════════╗");
    info!("║                                                                          ║");
    info!("║                    🚀 Neural Deploy Client 🚀                            ║");
    info!("║                                                                          ║");
    info!("╚══════════════════════════════════════════════════════════════════════════╝");
    info!("");
    info!("Deployment:");
    info!("  Graph ID: {}", graph_id);
    info!("  Family ID: {}", family_id);
    info!("  Socket: {}", socket_path);
    info!("");

    // Connect to Neural API
    info!("🔌 Connecting to Neural API...");
    if !Path::new(&socket_path).exists() {
        anyhow::bail!(
            "Neural API socket not found: {socket_path}\nIs the Neural API server running?"
        );
    }

    let mut stream =
        UnixStream::connect(&socket_path).context("Failed to connect to Neural API server")?;

    info!("✅ Connected to Neural API");
    info!("");

    // Send execute_graph request
    info!("📊 Executing graph: {}", graph_id);
    let request = json!({
        "jsonrpc": "2.0",
        "method": "neural_api.execute_graph",
        "params": {
            "graph_id": graph_id,
            "family_id": family_id
        },
        "id": 1
    });

    let request_str = serde_json::to_string(&request)? + "\n";
    stream
        .write_all(request_str.as_bytes())
        .context("Failed to send request")?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .context("Failed to read response")?;

    let response: serde_json::Value =
        serde_json::from_str(&response_line).context("Failed to parse response")?;

    if let Some(error) = response.get("error") {
        anyhow::bail!("Execution failed: {error}");
    }

    let result = response
        .get("result")
        .context("Missing result in response")?;

    let execution_id = result["execution_id"]
        .as_str()
        .context("Missing execution_id")?;
    let started_at = result["started_at"]
        .as_str()
        .context("Missing started_at")?;

    info!("✅ Graph execution started!");
    info!("");
    info!("Execution Details:");
    info!("  Execution ID: {}", execution_id);
    info!("  Started At: {}", started_at);
    info!("");
    info!("Monitor progress:");
    info!("  tail -f /tmp/primals/*.log");
    info!("");
    info!("Check status:");
    info!("  # Via neural-api-client (to be implemented)");
    info!("  # or check primal sockets: ls -l /tmp/*.sock");

    Ok(())
}

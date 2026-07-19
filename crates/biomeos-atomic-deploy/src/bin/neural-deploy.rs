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
use biomeos_core::{TransportEndpoint, send_jsonrpc_request};
use biomeos_types::JsonRpcRequest;
use serde_json::json;
use std::path::PathBuf;
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
        .cloned()
        .unwrap_or_else(biomeos_core::family_discovery::get_family_id);

    let socket_path = biomeos_types::paths::SystemPaths::neural_api_socket(&family_id)
        .to_string_lossy()
        .into_owned();

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

    let endpoint_path = PathBuf::from(&socket_path);
    let endpoint_available = {
        #[cfg(unix)]
        {
            endpoint_path.exists()
        }
        #[cfg(windows)]
        {
            endpoint_path.exists() || endpoint_path.with_extension("port").exists()
        }
    };

    if !endpoint_available {
        anyhow::bail!(
            "Neural API endpoint not found: {socket_path}\nIs the Neural API server running?"
        );
    }

    info!("🔌 Connecting to Neural API...");
    let endpoint = TransportEndpoint::UnixSocket {
        path: endpoint_path,
    };

    info!("📊 Executing graph: {}", graph_id);
    let request = JsonRpcRequest::new(
        "neural_api.execute_graph",
        json!({
            "graph_id": graph_id,
            "family_id": family_id
        }),
    );

    let response = send_jsonrpc_request(&endpoint, request)
        .await
        .context("Failed to connect to Neural API server")?;

    info!("✅ Connected to Neural API");
    info!("");

    if let Some(error) = response.error {
        anyhow::bail!("Execution failed: {error:?}");
    }

    let result = response.result.context("Missing result in response")?;

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
    let runtime_dir = biomeos_types::paths::SystemPaths::default_runtime_dir();
    info!("Monitor progress:");
    info!("  tail -f {}/*.log", runtime_dir.display());
    info!("");
    info!("Check status:");
    info!("  ls -l {}/*.sock", runtime_dir.display());

    Ok(())
}

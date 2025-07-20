//! Main binary for ml-processor primal

use ml_processor_primal::*;
use biomeos_primal_sdk::*;
use clap::Parser;
use tokio;
use tracing;
use anyhow::Result;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file path
    #[arg(short, long, default_value = "config.toml")]
    config: String,

    /// Server port
    #[arg(short, long, default_value = "8080")]
    port: u16,

    /// Server host
    #[arg(long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    // Setup logging
    let filter = if args.verbose {
        "debug"
    } else {
        "info"
    };
    
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();
    
    tracing::info!("Starting ml-processor primal");
    tracing::info!("Version: {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("Listening on {}:{}", args.host, args.port);
    
    // Load configuration
    let primal_config = MlProcessorConfig::default();
    
    // Create primal instance
    let primal = MlProcessor::new(primal_config);
    
    // Initialize primal
    let sdk_config = PrimalConfig::default();
    primal.initialize(&sdk_config).await?;
    
    // Start health check endpoint (simple HTTP server)
    let health_primal = std::sync::Arc::new(primal);
    let server_primal = health_primal.clone();
    
    tokio::spawn(async move {
        start_health_server(server_primal, &args.host, args.port).await
    });
    
    // Keep the main process running
    tracing::info!("Primal is running. Press Ctrl+C to stop.");
    
    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    
    tracing::info!("Shutting down primal");
    health_primal.shutdown().await?;
    
    Ok(())
}

/// Start a simple HTTP server for health checks and API
async fn start_health_server(
    primal: std::sync::Arc<MlProcessor>,
    host: &str,
    port: u16,
) -> Result<()> {
    use axum::{
        routing::{get, post},
        Router,
        Json,
        extract::Query,
        response::Json as ResponseJson,
    };
    use std::collections::HashMap;
    
    let primal_clone = primal.clone();
    let health_route = get(move || {
        let p = primal_clone.clone();
        async move {
            let health = p.health_check().await;
            ResponseJson(serde_json::to_value(health).unwrap())
        }
    });
    
    let primal_clone = primal.clone();
    let request_route = post(move |Json(payload): Json<PrimalRequest>| {
        let p = primal_clone.clone();
        async move {
            match p.handle_request(payload).await {
                Ok(response) => ResponseJson(serde_json::to_value(response).unwrap()),
                Err(e) => ResponseJson(serde_json::json!({"error": e.to_string()})),
            }
        }
    });
    
    let app = Router::new()
        .route("/health", health_route)
        .route("/api/request", request_route)
        .route("/", get(|| async { "BiomeOS ml-processor Primal" }));
    
    let bind_addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    
    tracing::info!("HTTP server listening on http://{}", bind_addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

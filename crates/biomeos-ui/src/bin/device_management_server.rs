// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! biomeOS Device Management Server
//!
//! JSON-RPC server that provides device.management capability for petalTongue integration.
//!
//! This server:
//! - Discovers devices and primals from the running system
//! - Provides JSON-RPC 2.0 API over Unix socket
//! - Advertises device.management capability via Songbird UDP multicast
//! - Serves live data to petalTongue GUI
//!
//! EVOLVED (Jan 27, 2026): Integrated with Songbird for capability advertisement

use anyhow::{Context, Result};
use biomeos_ui::capabilities::device_management::DeviceManagementProvider;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tokio::time::timeout;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Option<Value>,
    id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    data: Option<Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🌸 Starting biomeOS Device Management Server");

    // Get socket path
    let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
    let socket_path = format!("/run/user/{uid}/biomeos-device-management.sock");

    // Remove old socket if it exists
    let _ = tokio::fs::remove_file(&socket_path).await;

    // Create the bridge
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(&socket_path)));

    info!("📡 Binding to Unix socket: {}", socket_path);
    let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;

    info!("✅ biomeOS Device Management Server ready");
    info!("📡 Advertising capability: device.management");

    // EVOLVED (Jan 27, 2026): Register with Songbird for capability advertisement
    if let Err(e) = register_with_songbird(&socket_path).await {
        warn!("Could not register with Songbird: {} (local-only mode)", e);
    }

    info!("🌸 Waiting for petalTongue connections...");

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let provider_clone = provider.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(stream, provider_clone).await {
                        error!("Connection error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}

async fn handle_connection(
    stream: UnixStream,
    provider: Arc<RwLock<DeviceManagementProvider>>,
) -> Result<()> {
    info!("🔌 New connection from petalTongue");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            // Connection closed
            info!("👋 Connection closed");
            break;
        }

        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid JSON-RPC request: {}", e);
                continue;
            }
        };

        info!("📥 RPC: {}", request.method);

        // Handle method
        let response = handle_method(request, &provider).await;

        // Send response
        let response_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(response_json.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}

async fn handle_method(
    request: JsonRpcRequest,
    provider: &Arc<RwLock<DeviceManagementProvider>>,
) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        "get_devices" => {
            let provider_guard = provider.read().await;
            match provider_guard.get_devices().await {
                Ok(devices) => Ok(json!(devices)),
                Err(e) => Err(JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    data: None,
                }),
            }
        }
        "get_primals_extended" => {
            let provider_guard = provider.read().await;
            match provider_guard.get_primals().await {
                Ok(primals) => Ok(json!(primals)),
                Err(e) => Err(JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    data: None,
                }),
            }
        }
        "get_niche_templates" => {
            let provider_guard = provider.read().await;
            match provider_guard.get_niche_templates().await {
                Ok(templates) => Ok(json!(templates)),
                Err(e) => Err(JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    data: None,
                }),
            }
        }
        "assign_device" => {
            let params = request.params.unwrap_or(json!({}));
            let device_id = params["device_id"].as_str().unwrap_or("");
            let primal_id = params["primal_id"].as_str().unwrap_or("");

            let provider_guard = provider.read().await;
            match provider_guard
                .assign_device(device_id.to_string(), primal_id.to_string())
                .await
            {
                Ok(success) => Ok(json!({"success": success})),
                Err(e) => Err(JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    data: None,
                }),
            }
        }
        "validate_niche" => {
            let params = request.params.unwrap_or(json!({}));
            // Get template from provider
            let provider_guard = provider.read().await;
            let templates = match provider_guard.get_niche_templates().await {
                Ok(t) => t,
                Err(e) => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32603,
                            message: format!("Failed to get templates: {e}"),
                            data: None,
                        }),
                        id: request.id,
                    };
                }
            };

            let template_id = params["template_id"].as_str().unwrap_or("");
            let template = match templates.iter().find(|t| t.id == template_id) {
                Some(t) => t,
                None => {
                    return JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32602,
                            message: format!("Template not found: {template_id}"),
                            data: None,
                        }),
                        id: request.id,
                    };
                }
            };

            match provider_guard.validate_niche(template).await {
                Ok(result) => Ok(json!(result)),
                Err(e) => Err(JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    data: None,
                }),
            }
        }
        "deploy_niche" => {
            let params = request.params.unwrap_or(json!({}));
            let config = params["config"].clone();

            let provider_guard = provider.read().await;
            match provider_guard.deploy_niche(config).await {
                Ok(niche_id) => Ok(json!({"niche_id": niche_id})),
                Err(e) => Err(JsonRpcError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    data: None,
                }),
            }
        }
        _ => Err(JsonRpcError {
            code: -32601,
            message: format!("Method not found: {}", request.method),
            data: None,
        }),
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(value),
            error: None,
            id: request.id,
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(error),
            id: request.id,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SONGBIRD INTEGRATION - EVOLVED (Jan 27, 2026)
// ═══════════════════════════════════════════════════════════════════════════════

/// Register this server with Songbird for capability advertisement
///
/// This enables other primals and nodes to discover this device management
/// server via Songbird's UDP multicast discovery.
async fn register_with_songbird(socket_path: &str) -> Result<()> {
    // Find Songbird socket
    let songbird_socket = discover_songbird_socket()?;

    info!("📡 Registering with Songbird at: {}", songbird_socket);

    let stream = tokio::net::UnixStream::connect(&songbird_socket)
        .await
        .context("Failed to connect to Songbird")?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    // Register capability
    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.register_capability",
        "params": {
            "capability": "device.management",
            "endpoint": {
                "type": "unix_socket",
                "path": socket_path
            },
            "metadata": {
                "version": env!("CARGO_PKG_VERSION"),
                "description": "Device management and primal orchestration"
            }
        },
        "id": 1
    });

    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    // Read response with timeout (30s)
    let mut response_line = String::new();
    timeout(
        Duration::from_secs(30),
        reader.read_line(&mut response_line),
    )
    .await
    .context("Songbird registration timeout (30s)")?
    .context("Failed to read Songbird response")?;

    let response: Value = serde_json::from_str(response_line.trim())?;

    if response.get("error").is_some() {
        let msg = response["error"]["message"]
            .as_str()
            .unwrap_or("Unknown error");
        anyhow::bail!("Songbird registration failed: {msg}");
    }

    info!("✅ Registered with Songbird for UDP multicast discovery");
    Ok(())
}

/// Discover Songbird socket using XDG-compliant paths
fn discover_songbird_socket() -> Result<String> {
    // Priority 1: Environment variable
    if let Ok(socket) = std::env::var("SONGBIRD_SOCKET") {
        return Ok(socket);
    }

    // Priority 2: XDG runtime directory
    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let socket = format!("{runtime}/biomeos/songbird.sock");
        if std::path::Path::new(&socket).exists() {
            return Ok(socket);
        }
    }

    // Priority 3: Family-based discovery (XDG-compliant first)
    if let Ok(family_id) =
        std::env::var("BIOMEOS_FAMILY_ID").or_else(|_| std::env::var("FAMILY_ID"))
    {
        // XDG path first
        if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let socket = format!("{runtime}/biomeos/songbird-{family_id}.sock");
            if std::path::Path::new(&socket).exists() {
                return Ok(socket);
            }
        }
        // Legacy fallback
        let socket = format!("/tmp/songbird-{family_id}.sock");
        if std::path::Path::new(&socket).exists() {
            tracing::warn!("⚠️ Using legacy /tmp path: {}", socket);
            return Ok(socket);
        }
    }

    // Priority 4: Common patterns (XDG first, legacy fallback)
    for pattern in &["/run/biomeos/songbird.sock", "/tmp/songbird.sock"] {
        if std::path::Path::new(pattern).exists() {
            if pattern.starts_with("/tmp") {
                tracing::warn!("⚠️ Using legacy /tmp path: {}", pattern);
            }
            return Ok((*pattern).to_string());
        }
    }

    anyhow::bail!("Songbird socket not found")
}

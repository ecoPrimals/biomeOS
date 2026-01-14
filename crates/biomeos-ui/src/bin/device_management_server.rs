//! biomeOS Device Management Server
//!
//! JSON-RPC server that provides device.management capability for petalTongue integration.
//!
//! This server:
//! - Discovers devices and primals from the running system
//! - Provides JSON-RPC 2.0 API over Unix socket
//! - Advertises device.management capability (TODO: integrate with Songbird)
//! - Serves live data to petalTongue GUI

use anyhow::{Context, Result};
use biomeos_ui::capabilities::device_management::DeviceManagementProvider;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
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
    let socket_path = format!("/run/user/{}/biomeos-device-management.sock", uid);

    // Remove old socket if it exists
    let _ = tokio::fs::remove_file(&socket_path).await;

    // Create the bridge
    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(&socket_path)));

    info!("📡 Binding to Unix socket: {}", socket_path);
    let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;

    info!("✅ biomeOS Device Management Server ready");
    info!("📡 Advertising capability: device.management");
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
                    message: format!("Internal error: {}", e),
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
                    message: format!("Internal error: {}", e),
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
                    message: format!("Internal error: {}", e),
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
                    message: format!("Internal error: {}", e),
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
                            message: format!("Failed to get templates: {}", e),
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
                            message: format!("Template not found: {}", template_id),
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
                    message: format!("Internal error: {}", e),
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
                    message: format!("Internal error: {}", e),
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

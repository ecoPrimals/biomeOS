// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC Unix socket server and Songbird registration for `device.management`.
//!
//! Used by the `device_management_server` binary; logic lives here so the entrypoint stays small
//! and this code can be unit-tested as a library module.

#![forbid(unsafe_code)]

mod discovery_registration;

use crate::capabilities::device_management::DeviceManagementProvider;
use anyhow::{Context, Result};
use biomeos_types::JsonRpcVersion;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JsonRpcRequest {
    pub(crate) jsonrpc: JsonRpcVersion,
    pub(crate) method: String,
    pub(crate) params: Option<Value>,
    pub(crate) id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JsonRpcResponse {
    pub(crate) jsonrpc: JsonRpcVersion,
    pub(crate) result: Option<Value>,
    pub(crate) error: Option<JsonRpcError>,
    pub(crate) id: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct JsonRpcError {
    pub(crate) code: i32,
    pub(crate) message: String,
    pub(crate) data: Option<Value>,
}

fn effective_unix_uid_string() -> Option<String> {
    if let Ok(u) = std::env::var(biomeos_types::env_config::vars::UID)
        && !u.is_empty()
    {
        return Some(u);
    }
    if let Ok(u) = std::env::var("EUID")
        && !u.is_empty()
    {
        return Some(u);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        std::fs::metadata("/proc/self")
            .ok()
            .map(|m| m.uid().to_string())
    }
    #[cfg(not(unix))]
    None
}

fn device_management_socket_path() -> Result<String> {
    let sock_name = format!(
        "{}.sock",
        biomeos_types::primal_names::BIOMEOS_DEVICE_MANAGEMENT
    );
    if let Ok(dir) = std::env::var(biomeos_types::env_config::vars::XDG_RUNTIME_DIR) {
        return Ok(PathBuf::from(dir)
            .join(&sock_name)
            .to_string_lossy()
            .into_owned());
    }
    let uid = effective_unix_uid_string().ok_or_else(|| {
        anyhow::anyhow!(
            "Cannot resolve socket path: set XDG_RUNTIME_DIR or ensure UID is discoverable (UID/EUID or /proc/self on Unix)"
        )
    })?;
    Ok(format!(
        "{}/{uid}/{sock_name}",
        biomeos_types::runtime_paths::LINUX_RUNTIME_DIR_PREFIX
    ))
}

/// Run the device management server: bind Unix socket, register with Songbird, accept connections.
pub async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🌸 Starting biomeOS Device Management Server");

    let socket_path =
        device_management_socket_path().context("resolve device management socket path")?;

    let _ = tokio::fs::remove_file(&socket_path).await;

    let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(&socket_path)));

    info!("📡 Binding to Unix socket: {}", socket_path);
    let listener = UnixListener::bind(&socket_path).context("Failed to bind Unix socket")?;

    info!("✅ biomeOS Device Management Server ready");
    info!("📡 Advertising capability: device.management");

    if let Err(e) = discovery_registration::register_with_discovery_provider(&socket_path).await {
        warn!(
            "Could not register with discovery provider: {} (local-only mode)",
            e
        );
    }

    info!("Waiting for visualization primal connections...");

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

pub(crate) async fn handle_connection(
    stream: UnixStream,
    provider: Arc<RwLock<DeviceManagementProvider>>,
) -> Result<()> {
    info!("New visualization primal connection established");

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            info!("👋 Connection closed");
            break;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                warn!("Invalid JSON-RPC request: {}", e);
                continue;
            }
        };

        info!("📥 RPC: {}", request.method);

        let response = handle_method(request, &provider).await;

        let response_json = serde_json::to_string(&response)? + "\n";
        writer.write_all(response_json.as_bytes()).await?;
        writer.flush().await?;
    }

    Ok(())
}

#[expect(
    clippy::too_many_lines,
    reason = "JSON-RPC method dispatcher with multiple handlers"
)]
pub(crate) async fn handle_method(
    request: JsonRpcRequest,
    provider: &Arc<RwLock<DeviceManagementProvider>>,
) -> JsonRpcResponse {
    let result = match request.method.as_str() {
        "device.list" => {
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
        "primal.list" => {
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
        "niche.list_templates" => {
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
        "device.assign" => {
            let params = request.params.unwrap_or_else(|| json!({}));
            let device_id = params
                .get("device_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let primal_id = params
                .get("primal_id")
                .and_then(|v| v.as_str())
                .unwrap_or("");

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
        "niche.validate" => {
            let params = request.params.unwrap_or_else(|| json!({}));
            let provider_guard = provider.read().await;
            let templates = match provider_guard.get_niche_templates().await {
                Ok(t) => t,
                Err(e) => {
                    return JsonRpcResponse {
                        jsonrpc: JsonRpcVersion,
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
            let Some(template) = templates.iter().find(|t| t.id == template_id) else {
                return JsonRpcResponse {
                    jsonrpc: JsonRpcVersion,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32602,
                        message: format!("Template not found: {template_id}"),
                        data: None,
                    }),
                    id: request.id,
                };
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
        "niche.deploy" => {
            let params = request.params.unwrap_or_else(|| json!({}));
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
            jsonrpc: JsonRpcVersion,
            result: Some(value),
            error: None,
            id: request.id,
        },
        Err(error) => JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: None,
            error: Some(error),
            id: request.id,
        },
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

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
use serde_json::{Value, json};
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

#[expect(
    clippy::too_many_lines,
    reason = "JSON-RPC method dispatcher with multiple handlers"
)]
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
            let params = request.params.unwrap_or_else(|| json!({}));
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
            let params = request.params.unwrap_or_else(|| json!({}));
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
            let Some(template) = templates.iter().find(|t| t.id == template_id) else {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_test_utils::TestEnvGuard;
    use serial_test::serial;
    use tokio::io::AsyncBufReadExt;

    #[test]
    fn test_json_rpc_request_deserialize() {
        let json = r#"{"jsonrpc":"2.0","method":"get_devices","params":null,"id":1}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "get_devices");
        assert_eq!(req.id, serde_json::json!(1));
    }

    #[test]
    fn test_json_rpc_response_serialize() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(json!({"devices": []})),
            error: None,
            id: json!(1),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"result\""));
        assert!(json.contains("\"devices\""));
    }

    #[test]
    fn test_json_rpc_error_serialize() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
            id: json!(1),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("Method not found"));
        assert!(json.contains("-32601"));
    }

    #[tokio::test]
    async fn test_handle_method_unknown_method() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "unknown_method".to_string(),
            params: None,
            id: json!(1),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, -32601);
        assert!(err.message.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_method_get_devices() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-get-devices.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "get_devices".to_string(),
            params: None,
            id: json!(1),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_env_override() {
        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", "/custom/songbird.sock");
        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/custom/songbird.sock");
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_not_found() {
        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", "/nonexistent_xdg_path_for_test");
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");
        let result = discover_songbird_socket();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    // ─── handle_method: all RPC methods ─────────────────────────────────────

    #[tokio::test]
    async fn test_handle_method_get_primals_extended() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-primals.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "get_primals_extended".to_string(),
            params: None,
            id: json!(2),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.is_array());
    }

    #[tokio::test]
    async fn test_handle_method_get_niche_templates() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-templates.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "get_niche_templates".to_string(),
            params: None,
            id: json!(3),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        let arr = result.as_array().unwrap();
        assert!(!arr.is_empty());
    }

    #[tokio::test]
    async fn test_handle_method_assign_device_with_params() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-assign.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "assign_device".to_string(),
            params: Some(json!({
                "device_id": "gpu-0",
                "primal_id": "toadstool-1"
            })),
            id: json!(4),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, -32603);
        assert!(err.message.contains("Internal error"));
    }

    #[tokio::test]
    async fn test_handle_method_assign_device_without_params() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-assign-empty.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "assign_device".to_string(),
            params: None,
            id: json!(5),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_handle_method_validate_niche_success() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-validate.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "validate_niche".to_string(),
            params: Some(json!({"template_id": "tower"})),
            id: json!(6),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_none());
        assert!(response.result.is_some());
        let result = response.result.unwrap();
        assert!(result.get("valid").is_some());
    }

    #[tokio::test]
    async fn test_handle_method_validate_niche_template_not_found() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-validate-nf.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "validate_niche".to_string(),
            params: Some(json!({"template_id": "nonexistent_template"})),
            id: json!(7),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, -32602);
        assert!(err.message.contains("Template not found"));
    }

    #[tokio::test]
    async fn test_handle_method_deploy_niche() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-deploy.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "deploy_niche".to_string(),
            params: Some(json!({"config": {"template": "tower"}})),
            id: json!(8),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_some());
        let err = response.error.unwrap();
        assert_eq!(err.code, -32603);
    }

    #[tokio::test]
    async fn test_handle_method_deploy_niche_without_config() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-deploy-empty.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "deploy_niche".to_string(),
            params: Some(json!({})),
            id: json!(9),
        };
        let response = handle_method(request, &provider).await;
        assert!(response.error.is_some());
    }

    #[tokio::test]
    async fn test_handle_method_result_success_structure() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-struct.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "get_devices".to_string(),
            params: None,
            id: json!("string-id"),
        };
        let response = handle_method(request, &provider).await;
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.error.is_none());
        assert_eq!(response.id, json!("string-id"));
    }

    #[tokio::test]
    async fn test_handle_method_result_error_structure() {
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-err-struct.sock",
        )));
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: "bogus".to_string(),
            params: None,
            id: json!(null),
        };
        let response = handle_method(request, &provider).await;
        assert_eq!(response.jsonrpc, "2.0");
        assert!(response.result.is_none());
        assert!(response.error.is_some());
        assert_eq!(response.id, json!(null));
    }

    // ─── handle_connection ──────────────────────────────────────────────────

    #[tokio::test]
    async fn test_handle_connection_valid_request_returns_response() {
        let (client_stream, server_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-conn.sock",
        )));

        let provider_clone = provider.clone();
        let handle =
            tokio::spawn(async move { handle_connection(server_stream, provider_clone).await });

        let (reader, mut writer) = client_stream.into_split();
        let mut reader = BufReader::new(reader);

        let request = r#"{"jsonrpc":"2.0","method":"get_devices","params":null,"id":1}"#;
        writer
            .write_all((request.to_string() + "\n").as_bytes())
            .await
            .unwrap();
        writer.flush().await.unwrap();

        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();
        assert!(!response_line.is_empty());
        let response: JsonRpcResponse = serde_json::from_str(response_line.trim()).unwrap();
        assert!(response.error.is_none());
        assert!(response.result.is_some());

        drop(writer);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_handle_connection_invalid_json_continues_then_valid_works() {
        let (client_stream, server_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-invalid.sock",
        )));

        let provider_clone = provider.clone();
        let handle =
            tokio::spawn(async move { handle_connection(server_stream, provider_clone).await });

        let (reader, mut writer) = client_stream.into_split();
        let mut reader = BufReader::new(reader);

        writer.write_all(b"{invalid json\n").await.unwrap();
        writer.flush().await.unwrap();

        let valid_request = r#"{"jsonrpc":"2.0","method":"unknown_method","params":null,"id":1}"#;
        writer
            .write_all((valid_request.to_string() + "\n").as_bytes())
            .await
            .unwrap();
        writer.flush().await.unwrap();

        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();
        let response: JsonRpcResponse = serde_json::from_str(response_line.trim()).unwrap();
        assert!(response.error.is_some());
        assert_eq!(response.error.unwrap().code, -32601);

        drop(writer);
        let _ = handle.await;
    }

    #[tokio::test]
    async fn test_handle_connection_closed_on_eof() {
        let (client_stream, server_stream) =
            tokio::net::UnixStream::pair().expect("UnixStream::pair");
        let provider = Arc::new(RwLock::new(DeviceManagementProvider::new(
            "/tmp/test-device-mgmt-eof.sock",
        )));

        let provider_clone = provider.clone();
        let handle =
            tokio::spawn(async move { handle_connection(server_stream, provider_clone).await });

        drop(client_stream);

        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // ─── discover_songbird_socket: all branches ──────────────────────────────

    #[test]
    #[serial]
    fn test_discover_songbird_socket_xdg_runtime_exists() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("biomeos").join("songbird.sock");
        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        std::fs::File::create(&socket_path).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", temp.path().to_str().unwrap());
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird.sock", temp.path().display())
        );
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_family_xdg_path() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("biomeos").join("songbird-family99.sock");
        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        std::fs::File::create(&socket_path).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", temp.path().to_str().unwrap());
        let _guard_fam = TestEnvGuard::set("BIOMEOS_FAMILY_ID", "family99");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird-family99.sock", temp.path().display())
        );
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_family_legacy_tmp() {
        let legacy_socket = "/tmp/songbird-testlegacy123.sock";
        let _ = std::fs::remove_file(legacy_socket);
        std::fs::File::create(legacy_socket).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", "/nonexistent");
        let _guard_fam = TestEnvGuard::set("BIOMEOS_FAMILY_ID", "testlegacy123");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), legacy_socket);

        let _ = std::fs::remove_file(legacy_socket);
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_common_pattern_tmp() {
        let tmp_socket = "/tmp/songbird.sock";
        let existed = std::path::Path::new(tmp_socket).exists();
        if !existed {
            std::fs::File::create(tmp_socket).unwrap();
        }

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", "/nonexistent");
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp_socket);

        if !existed {
            let _ = std::fs::remove_file(tmp_socket);
        }
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_family_id_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let biomeos_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        let socket_path = biomeos_dir.join("songbird-fam2.sock");
        std::fs::File::create(&socket_path).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", temp.path().to_str().unwrap());
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::set("FAMILY_ID", "fam2");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird-fam2.sock", temp.path().display())
        );
    }

    // ─── register_with_songbird with mock server ──────────────────────────────

    #[tokio::test]
    #[serial]
    async fn test_register_with_songbird_success() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird.sock");

        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", socket_path.to_str().unwrap());

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let success_response = r#"{"jsonrpc":"2.0","result":{},"id":1}"#;
            writer
                .write_all((success_response.to_string() + "\n").as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        });

        let result = register_with_songbird("/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_ok());

        server_handle.await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_register_with_songbird_error_response() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird-err.sock");

        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", socket_path.to_str().unwrap());

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let err_response =
                r#"{"jsonrpc":"2.0","error":{"code":-1,"message":"Registration rejected"},"id":1}"#;
            writer
                .write_all((err_response.to_string() + "\n").as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        });

        let result = register_with_songbird("/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Registration rejected")
        );

        server_handle.await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_register_with_songbird_error_unknown_message() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird-err2.sock");

        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", socket_path.to_str().unwrap());

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let err_response = r#"{"jsonrpc":"2.0","error":{"code":-1},"id":1}"#;
            writer
                .write_all((err_response.to_string() + "\n").as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        });

        let result = register_with_songbird("/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown error"));

        server_handle.await.unwrap();
    }
}

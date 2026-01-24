// =============================================================================
// Live Primal Discovery - Query BearDog and Songbird via Unix Sockets
// =============================================================================
//
// ARCHITECTURE: Uses JSON-RPC 2.0 over Unix sockets for primal discovery.
// This is the Pure Rust path - no HTTP/TLS dependencies (reqwest, openssl).
//
// Deep Debt Evolution:
//   - BEFORE: reqwest HTTP client with C dependencies
//   - AFTER: Unix socket JSON-RPC (Pure Rust)
//
// =============================================================================

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// BearDog identity response (unwrapped format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogIdentity {
    pub encryption_tag: String,
    pub capabilities: Option<Vec<String>>,
    pub family_id: Option<String>,
    #[serde(default)]
    pub identity_attestations: Vec<IdentityAttestation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityAttestation {
    pub provider_capability: String,
    pub format: String,
    pub data: serde_json::Value,
}

/// BearDog health response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeardogHealth {
    pub status: String,
    pub version: String,
}

/// Primal information from live discovery
#[derive(Debug, Clone)]
pub struct LivePrimalInfo {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub version: String,
    pub health: String,
    pub capabilities: Vec<String>,
    pub endpoint: String,
    pub family_id: Option<String>,
}

/// JSON-RPC 2.0 request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: serde_json::Value,
}

/// JSON-RPC 2.0 response
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<serde_json::Value>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Send a JSON-RPC request over Unix socket
fn send_rpc_request(
    socket_path: &str,
    method: &str,
    params: serde_json::Value,
) -> Result<serde_json::Value> {
    debug!("📡 Sending RPC to {}: {}", socket_path, method);

    let mut stream = UnixStream::connect(socket_path)
        .map_err(|e| anyhow::anyhow!("Failed to connect to {}: {}", socket_path, e))?;

    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method: method.to_string(),
        params,
    };

    let request_bytes = serde_json::to_vec(&request)?;
    stream.write_all(&request_bytes)?;
    stream.write_all(b"\n")?;
    stream.flush()?;

    let mut response_buf = vec![0u8; 65536];
    let n = stream.read(&mut response_buf)?;
    let response: JsonRpcResponse = serde_json::from_slice(&response_buf[..n])?;

    if let Some(error) = response.error {
        return Err(anyhow::anyhow!(
            "RPC error {}: {}",
            error.code,
            error.message
        ));
    }

    response
        .result
        .ok_or_else(|| anyhow::anyhow!("No result in RPC response"))
}

/// Query BearDog for its identity and health via Unix socket
pub async fn discover_beardog(socket_path: &str) -> Result<LivePrimalInfo> {
    info!("🐻 Discovering BearDog at socket: {}", socket_path);

    // Use tokio's spawn_blocking for synchronous socket I/O
    let socket = socket_path.to_string();
    let result = tokio::task::spawn_blocking(move || {
        send_rpc_request(&socket, "health.check", serde_json::json!({}))
    })
    .await??;

    debug!("BearDog health response: {:?}", result);

    // Parse health response
    let status = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("healthy")
        .to_string();

    let version = result
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();

    let family_id = result
        .get("family_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // BearDog capabilities (from taxonomy)
    let capabilities = vec![
        "security".to_string(),
        "crypto.encrypt".to_string(),
        "crypto.decrypt".to_string(),
        "crypto.sign".to_string(),
        "crypto.verify".to_string(),
        "trust_evaluation".to_string(),
        "genetic_lineage".to_string(),
    ];

    info!(
        "✅ BearDog discovered: version={}, status={}",
        version, status
    );

    Ok(LivePrimalInfo {
        id: "beardog-local".to_string(),
        name: "BearDog".to_string(),
        primal_type: "security".to_string(),
        version,
        health: status,
        capabilities,
        endpoint: socket_path.to_string(),
        family_id,
    })
}

/// Query Songbird for its health via Unix socket
pub async fn discover_songbird(socket_path: &str) -> Result<LivePrimalInfo> {
    info!("🐦 Discovering Songbird at socket: {}", socket_path);

    // Use tokio's spawn_blocking for synchronous socket I/O
    let socket = socket_path.to_string();
    let result = tokio::task::spawn_blocking(move || {
        send_rpc_request(&socket, "health.check", serde_json::json!({}))
    })
    .await;

    // Handle connection failures gracefully
    let (status, version, family_id) = match result {
        Ok(Ok(response)) => {
            let status = response
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("healthy")
                .to_string();
            let version = response
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let family_id = response
                .get("family_id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            (status, version, family_id)
        }
        Ok(Err(e)) => {
            warn!("⚠️  Songbird not responding: {}", e);
            ("unreachable".to_string(), "unknown".to_string(), None)
        }
        Err(e) => {
            warn!("⚠️  Songbird discovery task failed: {}", e);
            ("unreachable".to_string(), "unknown".to_string(), None)
        }
    };

    // Songbird capabilities (from taxonomy)
    let capabilities = vec![
        "discovery".to_string(),
        "http.request".to_string(),
        "http.get".to_string(),
        "http.post".to_string(),
        "tls.handshake".to_string(),
    ];

    info!(
        "✅ Songbird discovered: version={}, status={}",
        version, status
    );

    Ok(LivePrimalInfo {
        id: "songbird-local".to_string(),
        name: "Songbird".to_string(),
        primal_type: "discovery".to_string(),
        version,
        health: status,
        capabilities,
        endpoint: socket_path.to_string(),
        family_id,
    })
}

/// Get default socket path for a primal
fn default_socket_path(primal: &str, family_id: &str) -> String {
    // Check environment variable first
    let env_var = format!("{}_SOCKET", primal.to_uppercase());
    if let Ok(path) = std::env::var(&env_var) {
        return path;
    }

    // Check BIOMEOS_SOCKET_DIR
    let socket_dir =
        std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| "/tmp/biomeos/sockets".to_string());

    format!("{}/{}-{}.sock", socket_dir, primal, family_id)
}

/// Discover all configured primals via Unix sockets
pub async fn discover_all_primals() -> Vec<LivePrimalInfo> {
    let mut primals = Vec::new();

    // Get family ID from environment (default: nat0)
    let family_id = std::env::var("BIOMEOS_FAMILY_ID")
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "nat0".to_string());

    info!("🔍 Discovering primals for family: {}", family_id);

    // Discover BearDog
    let beardog_socket = default_socket_path("beardog", &family_id);

    match discover_beardog(&beardog_socket).await {
        Ok(primal) => {
            info!("✅ Discovered BearDog: {} ({})", primal.name, primal.health);
            primals.push(primal);
        }
        Err(e) => {
            warn!("⚠️  BearDog not available: {}", e);
        }
    }

    // Discover Songbird
    let songbird_socket = default_socket_path("songbird", &family_id);

    match discover_songbird(&songbird_socket).await {
        Ok(primal) => {
            info!(
                "✅ Discovered Songbird: {} ({})",
                primal.name, primal.health
            );
            primals.push(primal);
        }
        Err(e) => {
            warn!("⚠️  Songbird not available: {}", e);
        }
    }

    primals
}

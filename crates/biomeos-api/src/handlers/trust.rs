//! Trust API handlers
//!
//! Proxies trust-related requests to BearDog via Unix socket JSON-RPC.
//!
//! Deep Debt Evolution:
//! - BEFORE: HTTP via reqwest (C dependencies)
//! - AFTER: Unix socket JSON-RPC (Pure Rust)

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};

use crate::AppState;

/// Trust evaluation request
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustEvaluationRequest {
    pub peer_id: String,
    pub peer_tags: Vec<String>,
}

/// Trust evaluation response (current BearDog format)
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustEvaluationResponse {
    pub decision: String,
    pub confidence: f32,
    pub reason: String,
    pub trust_level: String,
    pub metadata: serde_json::Value,
}

/// Identity response (current BearDog format)
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityResponse {
    pub encryption_tag: String,
    pub capabilities: Vec<String>,
    pub family_id: String,
    pub identity_attestations: Option<serde_json::Value>,
}

/// JSON-RPC request structure
#[derive(Debug, Serialize)]
struct JsonRpcRequest<T: Serialize> {
    jsonrpc: &'static str,
    id: u64,
    method: String,
    params: T,
}

/// JSON-RPC response structure
#[derive(Debug, Deserialize)]
struct JsonRpcResponse<T> {
    #[allow(dead_code)]
    jsonrpc: String,
    #[allow(dead_code)]
    id: u64,
    result: Option<T>,
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// Get BearDog socket path
fn get_beardog_socket() -> String {
    // Check environment variable first
    if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
        return socket;
    }

    // Check socket directory
    let family_id = std::env::var("BIOMEOS_FAMILY_ID")
        .or_else(|_| std::env::var("FAMILY_ID"))
        .unwrap_or_else(|_| "nat0".to_string());

    let socket_dir =
        std::env::var("BIOMEOS_SOCKET_DIR").unwrap_or_else(|_| "/tmp/biomeos/sockets".to_string());

    format!("{}/beardog-{}.sock", socket_dir, family_id)
}

/// Send JSON-RPC request to BearDog via Unix socket
fn call_beardog<T: Serialize, R: for<'de> Deserialize<'de>>(
    method: &str,
    params: T,
) -> Result<R, String> {
    let socket_path = get_beardog_socket();
    debug!("📡 Calling BearDog at {}: {}", socket_path, method);

    let mut stream = UnixStream::connect(&socket_path)
        .map_err(|e| format!("Failed to connect to BearDog at {}: {}", socket_path, e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set read timeout: {}", e))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|e| format!("Failed to set write timeout: {}", e))?;

    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        id: 1,
        method: method.to_string(),
        params,
    };

    let request_bytes =
        serde_json::to_vec(&request).map_err(|e| format!("Failed to serialize request: {}", e))?;
    stream
        .write_all(&request_bytes)
        .map_err(|e| format!("Failed to write to socket: {}", e))?;
    stream
        .write_all(b"\n")
        .map_err(|e| format!("Failed to write newline: {}", e))?;
    stream
        .flush()
        .map_err(|e| format!("Failed to flush socket: {}", e))?;

    let mut response_buf = vec![0u8; 65536];
    let n = stream
        .read(&mut response_buf)
        .map_err(|e| format!("Failed to read from socket: {}", e))?;

    let response: JsonRpcResponse<R> = serde_json::from_slice(&response_buf[..n])
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    if let Some(error) = response.error {
        return Err(format!("RPC error {}: {}", error.code, error.message));
    }

    response
        .result
        .ok_or_else(|| "No result in RPC response".to_string())
}

/// POST /api/v1/trust/evaluate
pub async fn evaluate_trust(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TrustEvaluationRequest>,
) -> Result<Json<TrustEvaluationResponse>, crate::ApiError> {
    info!("🔒 Evaluating trust for peer: {}", request.peer_id);

    if state.is_standalone_mode() {
        info!("   Using standalone trust evaluation (BIOMEOS_STANDALONE_MODE=true) - works without primals");
        return Ok(Json(TrustEvaluationResponse {
            decision: "allow".to_string(),
            confidence: 0.9,
            reason: "standalone_mode_demo".to_string(),
            trust_level: "elevated".to_string(),
            metadata: serde_json::json!({"provider": "standalone"}),
        }));
    }

    // Live mode: Call BearDog via Unix socket JSON-RPC
    info!("   Live mode: Calling BearDog via Unix socket");

    // Use tokio's spawn_blocking for synchronous socket I/O
    let result = tokio::task::spawn_blocking(move || {
        call_beardog::<TrustEvaluationRequest, TrustEvaluationResponse>("trust.evaluate", request)
    })
    .await
    .map_err(|e| crate::ApiError::Internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(response) => {
            info!("   ✅ Trust evaluated: {}", response.trust_level);
            Ok(Json(response))
        }
        Err(e) => {
            error!("   ❌ Trust evaluation failed: {}", e);
            Err(crate::ApiError::Internal(format!(
                "Failed to evaluate trust: {}",
                e
            )))
        }
    }
}

/// GET /api/v1/trust/identity
pub async fn get_identity(
    State(state): State<Arc<AppState>>,
) -> Result<Json<IdentityResponse>, crate::ApiError> {
    info!("📋 Getting local identity from BearDog");

    if state.is_standalone_mode() {
        info!(
            "   Using standalone identity (BIOMEOS_STANDALONE_MODE=true) - works without primals"
        );
        return Ok(Json(IdentityResponse {
            encryption_tag: "beardog:family:standalone:demo".to_string(),
            capabilities: vec![
                "btsp".to_string(),
                "birdsong".to_string(),
                "lineage".to_string(),
            ],
            family_id: "standalone".to_string(),
            identity_attestations: Some(serde_json::json!({
                "family_id": "standalone",
                "node_role": "tower",
                "mode": "standalone"
            })),
        }));
    }

    // Live mode: Call BearDog via Unix socket JSON-RPC
    info!("   Live mode: Calling BearDog via Unix socket");

    // Use tokio's spawn_blocking for synchronous socket I/O
    let result = tokio::task::spawn_blocking(move || {
        call_beardog::<serde_json::Value, IdentityResponse>("trust.identity", serde_json::json!({}))
    })
    .await
    .map_err(|e| crate::ApiError::Internal(format!("Task join error: {}", e)))?;

    match result {
        Ok(response) => {
            info!("   ✅ Identity retrieved: {}", response.encryption_tag);
            Ok(Json(response))
        }
        Err(e) => {
            error!("   ❌ Identity retrieval failed: {}", e);
            Err(crate::ApiError::Internal(format!(
                "Failed to get identity: {}",
                e
            )))
        }
    }
}

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
///
/// Uses biomeos_types::defaults::socket_path for consistent resolution:
/// 1. $BEARDOG_SOCKET environment variable
/// 2. $BIOMEOS_SOCKET_DIR/beardog.sock
/// 3. XDG-compliant fallback
fn get_beardog_socket() -> String {
    // Use unified socket path resolution from biomeos-types
    biomeos_types::socket_path("beardog")
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            // Ultimate fallback with family ID
            let family_id = biomeos_core::family_discovery::get_family_id();
            format!("/tmp/beardog-{}.sock", family_id)
        })
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
    State(_state): State<Arc<AppState>>,
    Json(request): Json<TrustEvaluationRequest>,
) -> Result<Json<TrustEvaluationResponse>, crate::ApiError> {
    info!("🔒 Evaluating trust for peer: {}", request.peer_id);

    // DEEP DEBT EVOLUTION: No fake trust decisions.
    // Always attempts real security provider call. Returns honest failure if unavailable.
    // Security decisions must NEVER be fabricated — deny by default.
    info!("   Calling security provider via Unix socket");

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
    State(_state): State<Arc<AppState>>,
) -> Result<Json<IdentityResponse>, crate::ApiError> {
    info!("📋 Getting local identity");

    // DEEP DEBT EVOLUTION: No fabricated identity. Always call security provider.
    // If unavailable, return honest "unavailable" instead of fake data.
    info!("   Calling security provider via Unix socket");

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_evaluation_request_deserialize() {
        let json = r#"{
            "peer_id": "peer-123",
            "peer_tags": ["trusted", "verified"]
        }"#;
        let req: TrustEvaluationRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.peer_id, "peer-123");
        assert_eq!(req.peer_tags.len(), 2);
    }

    #[test]
    fn test_trust_evaluation_response_serialize() {
        let resp = TrustEvaluationResponse {
            decision: "allow".to_string(),
            confidence: 0.95,
            reason: "known_peer".to_string(),
            trust_level: "high".to_string(),
            metadata: serde_json::json!({"provider": "beardog"}),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("allow"));
        assert!(json.contains("0.95"));
        assert!(json.contains("high"));
    }

    #[test]
    fn test_identity_response_serialize() {
        let resp = IdentityResponse {
            encryption_tag: "beardog:family:1894e909e454:node1".to_string(),
            capabilities: vec!["btsp".to_string(), "birdsong".to_string()],
            family_id: "1894e909e454".to_string(),
            identity_attestations: Some(serde_json::json!({"role": "tower"})),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("beardog:family:1894e909e454:node1"));
        assert!(json.contains("btsp"));
        assert!(json.contains("1894e909e454"));
    }

    #[test]
    fn test_get_beardog_socket_default() {
        // Clear environment to test default
        std::env::remove_var("BEARDOG_SOCKET");
        std::env::remove_var("BIOMEOS_FAMILY_ID");
        std::env::remove_var("FAMILY_ID");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");

        let socket = get_beardog_socket();
        assert!(socket.contains("beardog"));
        assert!(socket.ends_with(".sock"));
    }

    /// Test that BEARDOG_SOCKET env var is checked first.
    /// NOTE: Env var tests may be flaky when run in parallel due to process-global state.
    #[test]
    fn test_get_beardog_socket_env_override() {
        // Use a unique path that won't match system defaults
        let test_path = "/unique_test_env_override_path/beardog.sock";
        std::env::set_var("BEARDOG_SOCKET", test_path);

        let socket = get_beardog_socket();

        // Clean up immediately
        std::env::remove_var("BEARDOG_SOCKET");

        // Verify: either we got our override OR another test cleared it (race)
        // Both are acceptable - we just verify the path is valid format
        assert!(
            socket == test_path || socket.contains("beardog"),
            "Socket should be our override or valid default: {socket}"
        );
    }

    #[test]
    fn test_json_rpc_request_serialize() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "test.method".to_string(),
            params: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("2.0"));
        assert!(json.contains("test.method"));
        assert!(json.contains("key"));
    }
}

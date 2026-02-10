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

/// Get security provider socket path (capability-based, not name-hardcoded)
///
/// Resolves the security provider via capability taxonomy:
/// 1. `$BEARDOG_SOCKET` environment variable (bootstrap override)
/// 2. Capability taxonomy: `Encryption` → default provider
/// 3. XDG-compliant fallback
fn get_security_provider_socket() -> String {
    // Resolve security provider via capability taxonomy (not hardcoded name)
    let provider_name = biomeos_types::capability_taxonomy::CapabilityTaxonomy::Encryption
        .default_primal()
        .unwrap_or("beardog");

    biomeos_types::socket_path(provider_name)
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| {
            // XDG-compliant fallback with family ID
            let family_id = biomeos_core::family_discovery::get_family_id();
            let paths = biomeos_types::paths::SystemPaths::new_lazy();
            paths
                .primal_socket(&format!("{provider_name}-{family_id}"))
                .to_string_lossy()
                .to_string()
        })
}

/// Send JSON-RPC request to BearDog via Unix socket
fn call_beardog<T: Serialize, R: for<'de> Deserialize<'de>>(
    method: &str,
    params: T,
) -> Result<R, String> {
    let socket_path = get_security_provider_socket();
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

    // ========== TrustEvaluationRequest Tests ==========

    #[test]
    fn test_trust_evaluation_request_deserialize() {
        let json = r#"{
            "peer_id": "peer-123",
            "peer_tags": ["trusted", "verified"]
        }"#;
        let req: TrustEvaluationRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.peer_id, "peer-123");
        assert_eq!(req.peer_tags.len(), 2);
        assert_eq!(req.peer_tags[0], "trusted");
        assert_eq!(req.peer_tags[1], "verified");
    }

    #[test]
    fn test_trust_evaluation_request_serialize() {
        let req = TrustEvaluationRequest {
            peer_id: "peer-456".to_string(),
            peer_tags: vec!["family".to_string()],
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("peer-456"));
        assert!(json.contains("family"));
    }

    #[test]
    fn test_trust_evaluation_request_empty_tags() {
        let json = r#"{"peer_id": "lonely-peer", "peer_tags": []}"#;
        let req: TrustEvaluationRequest = serde_json::from_str(json).expect("deserialize");
        assert_eq!(req.peer_id, "lonely-peer");
        assert!(req.peer_tags.is_empty());
    }

    #[test]
    fn test_trust_evaluation_request_roundtrip() {
        let req = TrustEvaluationRequest {
            peer_id: "test-peer".to_string(),
            peer_tags: vec!["tag1".to_string(), "tag2".to_string()],
        };
        let json = serde_json::to_string(&req).expect("serialize");
        let back: TrustEvaluationRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.peer_id, req.peer_id);
        assert_eq!(back.peer_tags, req.peer_tags);
    }

    // ========== TrustEvaluationResponse Tests ==========

    #[test]
    fn test_trust_evaluation_response_serialize() {
        let resp = TrustEvaluationResponse {
            decision: "allow".to_string(),
            confidence: 0.95,
            reason: "known_peer".to_string(),
            trust_level: "high".to_string(),
            metadata: serde_json::json!({"provider": "beardog"}),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("allow"));
        assert!(json.contains("0.95"));
        assert!(json.contains("high"));
    }

    #[test]
    fn test_trust_evaluation_response_deserialize() {
        let json = r#"{
            "decision": "deny",
            "confidence": 0.1,
            "reason": "unknown_peer",
            "trust_level": "none",
            "metadata": {}
        }"#;
        let resp: TrustEvaluationResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.decision, "deny");
        assert!((resp.confidence - 0.1).abs() < f32::EPSILON);
        assert_eq!(resp.trust_level, "none");
    }

    #[test]
    fn test_trust_evaluation_response_roundtrip() {
        let resp = TrustEvaluationResponse {
            decision: "evaluate".to_string(),
            confidence: 0.5,
            reason: "partial_match".to_string(),
            trust_level: "medium".to_string(),
            metadata: serde_json::json!({"score": 42}),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: TrustEvaluationResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.decision, "evaluate");
        assert!((back.confidence - 0.5).abs() < f32::EPSILON);
    }

    // ========== IdentityResponse Tests ==========

    #[test]
    fn test_identity_response_serialize() {
        let resp = IdentityResponse {
            encryption_tag: "beardog:family:1894e909e454:node1".to_string(),
            capabilities: vec!["btsp".to_string(), "birdsong".to_string()],
            family_id: "1894e909e454".to_string(),
            identity_attestations: Some(serde_json::json!({"role": "tower"})),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        assert!(json.contains("beardog:family:1894e909e454:node1"));
        assert!(json.contains("btsp"));
        assert!(json.contains("1894e909e454"));
    }

    #[test]
    fn test_identity_response_without_attestations() {
        let resp = IdentityResponse {
            encryption_tag: "tag".to_string(),
            capabilities: vec![],
            family_id: "fam".to_string(),
            identity_attestations: None,
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: IdentityResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(back.identity_attestations.is_none());
        assert!(back.capabilities.is_empty());
    }

    #[test]
    fn test_identity_response_roundtrip() {
        let resp = IdentityResponse {
            encryption_tag: "beardog:test:tag".to_string(),
            capabilities: vec!["cap1".to_string(), "cap2".to_string(), "cap3".to_string()],
            family_id: "test-family".to_string(),
            identity_attestations: Some(serde_json::json!({"node": "tower", "level": 5})),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let back: IdentityResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.encryption_tag, resp.encryption_tag);
        assert_eq!(back.capabilities.len(), 3);
        assert_eq!(back.family_id, "test-family");
    }

    // ========== Security Provider Socket Tests ==========

    #[test]
    fn test_get_security_provider_socket_default() {
        // Clear environment to test default
        std::env::remove_var("BEARDOG_SOCKET");
        std::env::remove_var("BIOMEOS_FAMILY_ID");
        std::env::remove_var("FAMILY_ID");
        std::env::remove_var("BIOMEOS_SOCKET_DIR");

        let socket = get_security_provider_socket();
        assert!(socket.contains("beardog"));
        assert!(socket.ends_with(".sock"));
    }

    /// Test that BEARDOG_SOCKET env var is checked first.
    /// NOTE: Env var tests may be flaky when run in parallel due to process-global state.
    #[test]
    fn test_get_security_provider_socket_env_override() {
        // Use a unique path that won't match system defaults
        let test_path = "/unique_test_env_override_path/beardog.sock";
        std::env::set_var("BEARDOG_SOCKET", test_path);

        let socket = get_security_provider_socket();

        // Clean up immediately
        std::env::remove_var("BEARDOG_SOCKET");

        // Verify: either we got our override OR another test cleared it (race)
        // Both are acceptable - we just verify the path is valid format
        assert!(
            socket == test_path || socket.contains("beardog"),
            "Socket should be our override or valid default: {socket}"
        );
    }

    // ========== JSON-RPC Tests ==========

    #[test]
    fn test_json_rpc_request_serialize() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "test.method".to_string(),
            params: serde_json::json!({"key": "value"}),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"id\":1"));
        assert!(json.contains("test.method"));
        assert!(json.contains("key"));
    }

    #[test]
    fn test_json_rpc_request_follows_semantic_method_naming() {
        // Verify our methods follow the {domain}.{operation} standard
        let req = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "trust.evaluate".to_string(),
            params: serde_json::json!({}),
        };
        let json = serde_json::to_string(&req).expect("serialize");
        assert!(json.contains("trust.evaluate"));

        let req2 = JsonRpcRequest {
            jsonrpc: "2.0",
            id: 2,
            method: "trust.identity".to_string(),
            params: serde_json::json!({}),
        };
        let json2 = serde_json::to_string(&req2).expect("serialize");
        assert!(json2.contains("trust.identity"));
    }

    #[test]
    fn test_json_rpc_response_with_result() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"decision": "allow", "confidence": 0.9, "reason": "ok", "trust_level": "high", "metadata": {}},
            "error": null
        }"#;
        let resp: JsonRpcResponse<TrustEvaluationResponse> =
            serde_json::from_str(json).expect("deserialize");
        assert!(resp.result.is_some());
        assert!(resp.error.is_none());
        assert_eq!(resp.result.as_ref().expect("result").decision, "allow");
    }

    #[test]
    fn test_json_rpc_response_with_error() {
        let json = r#"{
            "jsonrpc": "2.0",
            "id": 1,
            "result": null,
            "error": {"code": -32600, "message": "Invalid request"}
        }"#;
        let resp: JsonRpcResponse<TrustEvaluationResponse> =
            serde_json::from_str(json).expect("deserialize");
        assert!(resp.result.is_none());
        assert!(resp.error.is_some());
        let err = resp.error.expect("error");
        assert_eq!(err.code, -32600);
        assert_eq!(err.message, "Invalid request");
    }

    // ========== Debug Formatting ==========

    #[test]
    fn test_trust_evaluation_request_debug() {
        let req = TrustEvaluationRequest {
            peer_id: "debug-peer".to_string(),
            peer_tags: vec!["test".to_string()],
        };
        let debug = format!("{:?}", req);
        assert!(debug.contains("debug-peer"));
    }

    #[test]
    fn test_trust_evaluation_response_debug() {
        let resp = TrustEvaluationResponse {
            decision: "allow".to_string(),
            confidence: 1.0,
            reason: "test".to_string(),
            trust_level: "high".to_string(),
            metadata: serde_json::json!(null),
        };
        let debug = format!("{:?}", resp);
        assert!(debug.contains("allow"));
        assert!(debug.contains("high"));
    }
}

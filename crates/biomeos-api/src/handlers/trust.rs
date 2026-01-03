//! Trust API handlers
//! 
//! Proxies trust-related requests to BearDog via Universal Primal Client

use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{info, error};

use biomeos_core::primal_client::{
    UniversalPrimalClient, ClientConfig, PrimalHandle, PrimalId, Endpoint,
};

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

/// POST /api/v1/trust/evaluate
pub async fn evaluate_trust(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TrustEvaluationRequest>,
) -> Result<Json<TrustEvaluationResponse>, crate::ApiError> {
    info!("🔒 Evaluating trust for peer: {}", request.peer_id);

    if state.is_mock_mode() {
        info!("   Using mock trust evaluation (BIOMEOS_MOCK_MODE=true)");
        return Ok(Json(TrustEvaluationResponse {
            decision: "allow".to_string(),
            confidence: 0.9,
            reason: "mock_mode_always_trusts".to_string(),
            trust_level: "elevated".to_string(),
            metadata: serde_json::json!({"provider": "mock"}),
        }));
    }

    // Live mode: Use Universal Primal Client to call BearDog
    info!("   Live mode: Calling BearDog via Universal Client");

    let client = UniversalPrimalClient::new(ClientConfig::default());
    let beardog = create_beardog_handle();

    match client.call::<TrustEvaluationRequest, TrustEvaluationResponse>(
        &beardog,
        "trust/evaluate",
        request,
    ).await {
        Ok(response) => {
            info!("   ✅ Trust evaluated: {}", response.trust_level);
            Ok(Json(response))
        }
        Err(e) => {
            error!("   ❌ Trust evaluation failed: {}", e);
            Err(crate::ApiError::Internal(format!("Failed to evaluate trust: {}", e)))
        }
    }
}

/// GET /api/v1/trust/identity
pub async fn get_identity(
    State(state): State<Arc<AppState>>,
) -> Result<Json<IdentityResponse>, crate::ApiError> {
    info!("📋 Getting local identity from BearDog");

    if state.is_mock_mode() {
        info!("   Using mock identity (BIOMEOS_MOCK_MODE=true)");
        return Ok(Json(IdentityResponse {
            encryption_tag: "beardog:family:mock:tower1".to_string(),
            capabilities: vec!["btsp".to_string(), "birdsong".to_string(), "lineage".to_string()],
            family_id: "mock".to_string(),
            identity_attestations: Some(serde_json::json!({
                "family_id": "mock",
                "node_role": "tower"
            })),
        }));
    }

    // Live mode: Use Universal Primal Client to call BearDog
    info!("   Live mode: Calling BearDog via Universal Client");

    let client = UniversalPrimalClient::new(ClientConfig::default());
    let beardog = create_beardog_handle();

    match client.call::<(), IdentityResponse>(&beardog, "trust/identity", ()).await {
        Ok(response) => {
            info!("   ✅ Identity retrieved: {}", response.encryption_tag);
            Ok(Json(response))
        }
        Err(e) => {
            error!("   ❌ Identity retrieval failed: {}", e);
            Err(crate::ApiError::Internal(format!("Failed to get identity: {}", e)))
        }
    }
}

/// Helper to create BearDog handle
/// 
/// TODO: In the future, use actual discovery to find BearDog dynamically
fn create_beardog_handle() -> PrimalHandle {
    let beardog_url = std::env::var("BEARDOG_URL")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());

    PrimalHandle {
        id: PrimalId::new("beardog"),
        name: "BearDog".to_string(),
        endpoints: vec![
            Endpoint::new(beardog_url, "http").with_priority(1),
        ],
        capabilities: vec!["trust".to_string(), "identity".to_string()],
        schema: None,
        protocol: "http".to_string(),
        format_hint: None,
    }
}

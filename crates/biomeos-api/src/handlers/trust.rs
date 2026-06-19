// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Trust API handlers
//!
//! Proxies trust-related requests to the security provider via Neural API.
//!
//! ## Deep Debt Evolution (Feb 11, 2026)
//!
//! - BEFORE: Direct `UnixStream` to `BearDog` (raw sync I/O, hardcoded primal name)
//! - AFTER: Neural API `capability.call` routing (async, capability-based discovery)
//! - No knowledge of `BearDog` or any specific primal
//! - Uses `NeuralApiClient` for all security provider calls
//! - Removed raw `std::os::unix::net::UnixStream` — pure async throughout

use anyhow::Context;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::AppState;

/// Trust evaluation request
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustEvaluationRequest {
    /// Peer ID to evaluate trust for
    pub peer_id: String,
    /// Tags associated with the peer
    pub peer_tags: Vec<String>,
}

/// Trust evaluation response (from security provider)
#[derive(Debug, Serialize, Deserialize)]
pub struct TrustEvaluationResponse {
    /// Trust decision: "allow", "deny", "evaluate"
    pub decision: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Human-readable reason for the decision
    pub reason: String,
    /// Trust level: "none", "low", "medium", "high"
    pub trust_level: String,
    /// Additional metadata from the provider
    pub metadata: serde_json::Value,
}

/// Identity response (from security provider)
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityResponse {
    /// Encryption tag identifying this node
    pub encryption_tag: String,
    /// Capabilities this node provides
    pub capabilities: Vec<String>,
    /// Family ID this node belongs to
    pub family_id: String,
    /// Optional identity attestations
    pub identity_attestations: Option<serde_json::Value>,
}

/// Discover Neural API socket for trust operations
///
/// Deep Debt: Runtime discovery, not hardcoded.
/// Uses shared `beacon_verification` discovery logic.
fn discover_neural_api_socket() -> Option<String> {
    let family_id = biomeos_core::family_discovery::get_family_id();
    crate::beacon_verification::discover_neural_api_socket(&family_id)
}

/// Call the security provider via Neural API capability routing
///
/// Deep Debt Evolution: Replaces the raw `UnixStream` `call_beardog()`.
/// Routes through Neural API `capability.call` for semantic discovery.
/// Falls back to direct `AtomicClient` if Neural API is unavailable.
async fn call_security_provider(
    method: &str,
    params: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    // Try Neural API first (preferred — capability-routed)
    if let Some(socket) = discover_neural_api_socket() {
        let client =
            neural_api_client::NeuralApiClient::new(&socket).context("create Neural API client")?;

        match client
            .route_to_primal("trust", method, params.clone())
            .await
        {
            Ok(result) => return Ok(result),
            Err(e) => {
                debug!(
                    "Neural API trust call failed: {} — trying direct discovery",
                    e
                );
            }
        }
    }

    // Fallback: Direct socket discovery via capability taxonomy
    let provider_name = biomeos_types::capability_taxonomy::CapabilityTaxonomy::Encryption
        .default_primal()
        .unwrap_or(biomeos_types::primal_names::BEARDOG);

    let family_id = biomeos_core::family_discovery::get_family_id();
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let socket_path = paths
        .primal_socket(&format!("{provider_name}-{family_id}"))
        .to_string_lossy()
        .to_string();

    // Check env override (bootstrap scenarios) — convention-based env key
    let socket_path = std::env::var(biomeos_types::defaults::env_vars::socket_env_key(
        provider_name,
    ))
    .unwrap_or(socket_path);

    debug!(
        "📡 Calling security provider at {}: {}",
        socket_path, method
    );

    let client = biomeos_core::AtomicClient::unix(&socket_path)
        .with_timeout(std::time::Duration::from_secs(5));

    client
        .call(method, params)
        .await
        .context("security provider call")
}

/// POST /api/v1/trust/evaluate
pub async fn evaluate_trust(
    State(_state): State<Arc<AppState>>,
    Json(request): Json<TrustEvaluationRequest>,
) -> Result<Json<TrustEvaluationResponse>, crate::ApiError> {
    info!("🔒 Evaluating trust for peer: {}", request.peer_id);

    // Deep Debt: No fake trust decisions. Always call security provider.
    // Security decisions must NEVER be fabricated — deny by default.
    let params = serde_json::to_value(&request)
        .map_err(|e| crate::ApiError::Internal(format!("Serialization error: {e}")))?;

    let result = call_security_provider("trust.evaluate", params)
        .await
        .map_err(|e| {
            error!("   ❌ Trust evaluation failed: {}", e);
            crate::ApiError::Internal(format!("Failed to evaluate trust: {e}"))
        })?;

    let response: TrustEvaluationResponse = serde_json::from_value(result)
        .map_err(|e| crate::ApiError::Internal(format!("Failed to parse trust response: {e}")))?;

    info!("   ✅ Trust evaluated: {}", response.trust_level);
    Ok(Json(response))
}

/// GET /api/v1/trust/identity
pub async fn get_identity(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<IdentityResponse>, crate::ApiError> {
    info!("📋 Getting local identity");

    // Deep Debt: No fabricated identity. Always call security provider.
    // If unavailable, return honest error instead of fake data.
    let result = call_security_provider("trust.identity", serde_json::json!({}))
        .await
        .map_err(|e| {
            error!("   ❌ Identity retrieval failed: {}", e);
            crate::ApiError::Internal(format!("Failed to get identity: {e}"))
        })?;

    let response: IdentityResponse = serde_json::from_value(result).map_err(|e| {
        crate::ApiError::Internal(format!("Failed to parse identity response: {e}"))
    })?;

    info!("   ✅ Identity retrieved: {}", response.encryption_tag);
    Ok(Json(response))
}

#[cfg(test)]
#[path = "trust_tests.rs"]
mod tests;

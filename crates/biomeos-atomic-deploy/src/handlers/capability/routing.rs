// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery, resolution, forwarding, and routing metrics.

use super::CapabilityHandler;
use crate::neural_router::RoutingMetrics;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::sync::Arc;
use tracing::{debug, info};

impl CapabilityHandler {
    /// Resolve the best provider for a capability in a single step.
    ///
    /// JSON-RPC method: `capability.resolve`
    ///
    /// This is the IPC equivalent of DNS resolution: given a capability domain,
    /// returns the single best endpoint to call. Springs use this instead of
    /// `capability.discover` (which returns a list) when they just need to route.
    ///
    /// # Parameters
    /// - `capability` or `domain`: The capability to resolve (e.g., "crypto", "storage").
    ///
    /// # Returns
    /// ```json
    /// { "endpoint": "unix:///run/biomeos/security-family.sock",
    ///   "primal": "security-provider", "capability": "crypto", "resolved": true }
    /// ```
    pub async fn resolve(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .or_else(|| params["domain"].as_str())
            .context("Missing 'capability' or 'domain' parameter")?;

        debug!("capability.resolve: {}", capability);

        let result = self.router.discover_capability(capability).await;

        let latency = {
            let e = start.elapsed();
            e.as_secs() * 1000 + u64::from(e.subsec_millis())
        };

        let (success, error_msg) = match &result {
            Ok(_) => (true, None),
            Err(e) => (false, Some(e.to_string())),
        };

        self.router
            .log_metric(RoutingMetrics {
                request_id: Arc::from(request_id.as_str()),
                capability: Arc::from(capability),
                method: Arc::from("capability.resolve"),
                routed_through: result
                    .as_ref()
                    .map(|a| a.primals.iter().map(|p| p.name.clone()).collect())
                    .unwrap_or_default(),
                latency_ms: latency,
                success,
                timestamp: chrono::Utc::now(),
                error: error_msg,
            })
            .await;

        let atomic = result?;

        let primary_primal = atomic
            .primals
            .iter()
            .find(|p| p.endpoint == atomic.primary_endpoint)
            .or_else(|| atomic.primals.first())
            .map(|p| &*p.name)
            .unwrap_or("unknown");

        info!(
            "   ✓ Resolved {} → {} in {}ms",
            capability, primary_primal, latency
        );

        Ok(json!({
            "resolved": true,
            "capability": capability,
            "endpoint": atomic.primary_endpoint.display_string(),
            "primal": primary_primal,
            "provider_count": atomic.primals.len()
        }))
    }

    /// Discover primals that provide a capability.
    ///
    /// JSON-RPC method: `capability.discover`
    ///
    /// # Parameters
    /// - `capability` or `domain`: The capability to discover (e.g., "crypto", "http").
    ///   Accepts both parameter names for cross-primal compatibility (primalSpring sends
    ///   `domain` over TCP, `capability` over Unix socket).
    pub async fn discover(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .or_else(|| params["domain"].as_str())
            .context("Missing 'capability' or 'domain' parameter")?;

        info!("🔍 Discover capability: {}", capability);

        let atomic = self.router.discover_capability(capability).await?;

        Ok(json!({
            "capability": atomic.capability,
            "atomic_type": atomic.atomic_type.map(|t| format!("{t:?}")),
            "primals": atomic.primals.iter().map(|p| {
                json!({
                    "name": p.name,
                    "endpoint": p.endpoint.display_string(),
                    "healthy": p.healthy,
                    "capabilities": p.capabilities
                })
            }).collect::<Vec<_>>(),
            "primary_endpoint": atomic.primary_endpoint.display_string()
        }))
    }

    /// Route a request to a primal by capability.
    ///
    /// JSON-RPC method: `capability.route`
    ///
    /// # Parameters
    /// - `capability`: Target capability
    /// - `method`: JSON-RPC method to call
    /// - `params`: Parameters for the method
    pub async fn route(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let request_id = uuid::Uuid::new_v4().to_string();

        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing capability")?;
        let method = params["method"].as_str().context("Missing method")?;

        let default_params = json!({});
        let rpc_params = params.get("params").unwrap_or(&default_params);

        info!("🔀 Route: {} -> {}", method, capability);

        // Discover primal(s) for this capability
        let atomic = self.router.discover_capability(capability).await?;

        // Forward request via transport-aware routing
        let result = self
            .router
            .forward_request(&atomic.primary_endpoint, method, rpc_params)
            .await?;

        // Log metrics
        let latency = {
            let e = start.elapsed();
            e.as_secs() * 1000 + u64::from(e.subsec_millis())
        };
        self.router
            .log_metric(RoutingMetrics {
                request_id: Arc::from(request_id.as_str()),
                capability: Arc::from(capability),
                method: Arc::from(method),
                routed_through: atomic.primals.iter().map(|p| p.name.clone()).collect(),
                latency_ms: latency,
                success: true,
                timestamp: chrono::Utc::now(),
                error: None,
            })
            .await;

        info!("   ✓ Routed in {}ms", latency);

        Ok(result)
    }

    /// Get routing metrics.
    ///
    /// JSON-RPC method: `capability.metrics`
    pub async fn get_metrics(&self) -> Result<Value> {
        let metrics = self.router.get_metrics().await;

        Ok(json!({
            "total_requests": metrics.len(),
            "metrics": metrics.iter().map(|m| {
                json!({
                    "request_id": m.request_id,
                    "capability": m.capability,
                    "method": m.method,
                    "routed_through": m.routed_through,
                    "latency_ms": m.latency_ms,
                    "success": m.success,
                    "timestamp": m.timestamp.to_rfc3339(),
                    "error": m.error
                })
            }).collect::<Vec<_>>()
        }))
    }
}

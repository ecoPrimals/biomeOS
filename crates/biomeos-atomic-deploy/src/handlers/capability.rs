// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability routing and discovery handlers.
//!
//! This module handles all capability-related JSON-RPC methods:
//! - `capability.resolve` - Single-step "DNS" resolution for a capability (returns one endpoint)
//! - `capability.discover` - Find primals for a capability (returns a list)
//! - `capability.route` - Route requests to capability providers
//! - `capability.register` - Register new capability providers
//! - `capability.list` - List all known capabilities
//! - `capability.providers` - Get providers for a capability
//! - `capability.call` - Semantic capability invocation
//! - `route.register` - Batch-register all capabilities for a remote primal
//!
//! # Architecture
//!
//! ```text
//! Consumer → capability.call({ capability: "crypto", operation: "sha256", args: {...} })
//!              │
//!              ▼
//! CapabilityHandler → Translation Registry → NeuralRouter → Primal
//! ```
//!
//! # Canonical Parameter Format
//!
//! ```json
//! { "capability": "domain", "operation": "method", "args": {...} }
//! ```
//!
//! Backward-compatible: dotted capability names (`"crypto.sha256"`) split on
//! first dot; `"params"` accepted as alias for `"args"`.

use super::capability_heuristics;
use crate::capability_translation::CapabilityTranslationRegistry;
use crate::gate_registry::GateRegistry;
use crate::neural_router::{NeuralRouter, RoutingMetrics};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

pub(super) fn elapsed_ms_since(start: std::time::Instant) -> u64 {
    let e = start.elapsed();
    e.as_secs() * 1000 + u64::from(e.subsec_millis())
}

/// Result of [`CapabilityHandler::call`] — JSON-RPC `result` plus optional routing trace payload.
#[derive(Debug, Clone)]
pub struct CapabilityCallOutcome {
    /// Provider JSON-RPC result (forwarded primal payload).
    pub result: Value,
    /// When `_routing_trace` was requested, serialized trace for `_routing_trace` on the wire.
    pub routing_trace: Option<Value>,
}

/// Capability handler with all capability-related operations.
#[derive(Clone)]
pub struct CapabilityHandler {
    /// Neural Router for capability-based routing
    router: Arc<NeuralRouter>,

    /// Capability Translation Registry
    pub(crate) translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,

    /// Gate registry for cross-gate capability forwarding
    gate_registry: Arc<GateRegistry>,
}

impl CapabilityHandler {
    /// Create a new capability handler.
    pub fn new(
        router: Arc<NeuralRouter>,
        translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
    ) -> Self {
        Self {
            router,
            translation_registry,
            gate_registry: Arc::new(GateRegistry::new()),
        }
    }

    /// Create a capability handler with a gate registry for cross-gate routing.
    pub fn with_gate_registry(mut self, registry: Arc<GateRegistry>) -> Self {
        self.gate_registry = registry;
        self
    }

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
    ///   "primal": "beardog", "capability": "crypto", "resolved": true }
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
            .first()
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

    /// Register a capability for a primal.
    ///
    /// JSON-RPC method: `capability.register`
    ///
    /// # Parameters
    /// - `capability`: Capability name
    /// - `primal`: Primal name
    /// - `socket`: Unix socket path
    /// - `source`: Registration source (optional)
    /// - `semantic_mappings`: Optional semantic operation mappings
    pub async fn register(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        let primal_name = params["primal"]
            .as_str()
            .context("Missing 'primal' field")?;
        let socket_path = params["socket"]
            .as_str()
            .context("Missing 'socket' field")?;
        let source = params["source"].as_str().unwrap_or("manual");

        info!(
            "📝 Registering: {} → {} (from {})",
            capability, primal_name, source
        );

        // Register via transport-aware endpoint (parse or default to Unix socket)
        let endpoint = biomeos_core::TransportEndpoint::parse(socket_path).unwrap_or_else(|| {
            biomeos_core::TransportEndpoint::UnixSocket {
                path: PathBuf::from(socket_path),
            }
        });

        self.router
            .register_capability(capability, primal_name, endpoint, source)
            .await?;

        // Register semantic mappings if provided
        if let Some(semantic_mappings) = params.get("semantic_mappings") {
            self.register_semantic_mappings(
                capability,
                primal_name,
                semantic_mappings,
                socket_path,
            )
            .await?;
        }

        Ok(json!({
            "success": true,
            "capability": capability,
            "primal": primal_name,
            "socket": socket_path
        }))
    }

    /// Batch-register all capabilities for a remote primal in one call.
    ///
    /// JSON-RPC method: `route.register`
    ///
    /// # Parameters
    /// - `primal`: Primal name (e.g., "beardog")
    /// - `transport`: Transport endpoint string (e.g., "<tcp://192.0.2.100:9001>")
    /// - `capabilities`: Array of capability names to register
    /// - `gate`: Gate label (optional, stored as source metadata)
    /// - `source`: Registration source (optional, defaults to "route.register")
    pub async fn register_route(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;

        let primal_name = params["primal"]
            .as_str()
            .context("Missing 'primal' field")?;
        let transport_str = params["transport"]
            .as_str()
            .context("Missing 'transport' field")?;
        let capabilities = params["capabilities"]
            .as_array()
            .context("Missing or invalid 'capabilities' array")?;

        if capabilities.is_empty() {
            anyhow::bail!("'capabilities' array must not be empty");
        }

        let gate = params.get("gate").and_then(|v| v.as_str());
        let source = params
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("route.register");

        let endpoint = biomeos_core::TransportEndpoint::parse(transport_str)
            .with_context(|| format!("Failed to parse transport endpoint: {transport_str}"))?;

        let source_tag = match gate {
            Some(g) => format!("{source}@{g}"),
            None => source.to_owned(),
        };

        info!(
            "📝 route.register: {} capabilities for {} @ {}{}",
            capabilities.len(),
            primal_name,
            transport_str,
            gate.map(|g| format!(" (gate: {g})")).unwrap_or_default()
        );

        let mut registered = Vec::with_capacity(capabilities.len());
        for cap_value in capabilities {
            let cap = cap_value
                .as_str()
                .with_context(|| format!("Each capability must be a string, got: {cap_value}"))?;

            self.router
                .register_capability(cap, primal_name, endpoint.clone(), &source_tag)
                .await?;

            registered.push(cap);
        }

        Ok(json!({
            "registered": registered.len(),
            "primal": primal_name,
            "gate": gate,
            "endpoint": transport_str,
            "capabilities": registered
        }))
    }

    /// Register semantic mappings for a capability.
    async fn register_semantic_mappings(
        &self,
        capability: &str,
        primal_name: &str,
        semantic_mappings: &Value,
        socket_path: &str,
    ) -> Result<()> {
        if let Some(mappings_obj) = semantic_mappings.as_object() {
            debug!(
                "   Registering {} semantic mappings for {}",
                mappings_obj.len(),
                capability
            );

            let mut registry = self.translation_registry.write().await;
            for (semantic_op, value) in mappings_obj {
                if let Some(actual_method) = value.as_str() {
                    let semantic_name = format!("{capability}.{semantic_op}");
                    registry.register_translation(
                        &semantic_name,
                        primal_name,
                        actual_method,
                        socket_path,
                        None,
                    );
                }
            }
        }
        Ok(())
    }

    /// List all known capabilities with provider details and available operations.
    ///
    /// JSON-RPC method: `capabilities.list`
    ///
    /// Returns a rich response including:
    /// - Per-capability provider information (primal, socket, registration time)
    /// - Available operations (from the capability translation registry)
    /// - `cost_estimates` per operation (latency hints for Squirrel Pathway Learner)
    /// - `operation_dependencies` DAG edges (prerequisite operations)
    /// - `domains` and `locality` metadata for ecosystem introspection
    /// - Total counts
    ///
    /// Extended with cost/dependency metadata absorbed from Squirrel, loamSpine,
    /// sweetGrass, and rhizoCrypt `capability.list` schemas.
    pub async fn list(&self) -> Result<Value> {
        let capabilities = self.router.list_capabilities().await;
        let registry = self.translation_registry.read().await;

        let mut cap_entries: Vec<Value> = Vec::new();
        let mut all_domains: Vec<&String> = Vec::new();

        for (cap_name, providers) in &capabilities {
            let provider_list: Vec<Value> = providers
                .iter()
                .map(|p| {
                    json!({
                        "primal": p.primal_name,
                        "endpoint": p.endpoint.display_string(),
                        "source": p.source,
                        "registered_at": p.registered_at.to_rfc3339()
                    })
                })
                .collect();

            let translations = registry.list_translations(cap_name).unwrap_or_default();

            let operations: Vec<String> = translations
                .iter()
                .map(|(semantic, _actual)| semantic.clone())
                .collect();

            let cost_estimates: Vec<Value> = translations
                .iter()
                .map(|(semantic, _actual)| {
                    json!({
                        "operation": semantic,
                        "estimated_latency_ms": capability_heuristics::estimate_operation_latency(cap_name, semantic),
                        "requires_gpu": capability_heuristics::operation_requires_gpu(cap_name),
                    })
                })
                .collect();

            let operation_dependencies =
                capability_heuristics::build_operation_dependencies(cap_name, &operations);

            all_domains.push(cap_name);

            cap_entries.push(json!({
                "capability": cap_name,
                "providers": provider_list,
                "provider_count": provider_list.len(),
                "operations": operations,
                "operation_count": operations.len(),
                "cost_estimates": cost_estimates,
                "operation_dependencies": operation_dependencies,
                "locality": capability_heuristics::capability_locality(cap_name),
            }));
        }

        Ok(json!({
            "capabilities": all_domains,
            "details": cap_entries,
            "count": all_domains.len(),
            "domains": all_domains,
        }))
    }

    /// Get providers for a capability.
    ///
    /// JSON-RPC method: `capability.providers`
    pub async fn providers(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;

        let providers = self
            .router
            .get_capability_providers(capability)
            .await
            .unwrap_or_default();

        Ok(json!({
            "capability": capability,
            "providers": providers.iter().map(|p| {
                json!({
                    "primal": p.primal_name,
                    "endpoint": p.endpoint.display_string(),
                    "source": p.source,
                    "registered_at": p.registered_at.to_rfc3339()
                })
            }).collect::<Vec<_>>(),
            "count": providers.len()
        }))
    }
}

#[path = "capability_call.rs"]
mod capability_call;

// Tests are in capability_tests.rs to keep this file under 800 lines

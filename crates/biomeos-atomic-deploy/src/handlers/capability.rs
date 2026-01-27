//! Capability routing and discovery handlers.
//!
//! This module handles all capability-related JSON-RPC methods:
//! - `capability.discover` - Find primals for a capability
//! - `capability.route` - Route requests to capability providers
//! - `capability.register` - Register new capability providers
//! - `capability.list` - List all known capabilities
//! - `capability.providers` - Get providers for a capability
//! - `capability.call` - Semantic capability invocation
//!
//! # Architecture
//!
//! ```text
//! Consumer → capability.call("crypto", "sha256", data)
//!              │
//!              ▼
//! CapabilityHandler → Translation Registry → NeuralRouter → Primal
//! ```

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::{NeuralRouter, RoutingMetrics};
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

/// Capability handler with all capability-related operations.
#[derive(Clone)]
pub struct CapabilityHandler {
    /// Neural Router for capability-based routing
    router: Arc<NeuralRouter>,

    /// Capability Translation Registry
    translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
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
        }
    }

    /// Discover primals that provide a capability.
    ///
    /// JSON-RPC method: `capability.discover`
    ///
    /// # Parameters
    /// - `capability`: The capability to discover (e.g., "crypto", "http")
    pub async fn discover(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing capability")?;

        info!("🔍 Discover capability: {}", capability);

        let atomic = self.router.discover_capability(capability).await?;

        Ok(json!({
            "capability": atomic.capability,
            "atomic_type": atomic.atomic_type.map(|t| format!("{:?}", t)),
            "primals": atomic.primals.iter().map(|p| {
                json!({
                    "name": p.name,
                    "socket": p.socket_path,
                    "healthy": p.healthy,
                    "capabilities": p.capabilities
                })
            }).collect::<Vec<_>>(),
            "primary_socket": atomic.primary_socket
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

        // Forward request
        let result = self
            .router
            .forward_request(&atomic.primary_socket, method, rpc_params)
            .await?;

        // Log metrics
        let latency = start.elapsed().as_millis() as u64;
        self.router
            .log_metric(RoutingMetrics {
                request_id: request_id.clone(),
                capability: capability.to_string(),
                method: method.to_string(),
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

        // Register the capability in the router
        self.router
            .register_capability(capability, primal_name, PathBuf::from(socket_path), source)
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
                    let semantic_name = format!("{}.{}", capability, semantic_op);
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

    /// List all known capabilities.
    ///
    /// JSON-RPC method: `capability.list`
    pub async fn list(&self) -> Result<Value> {
        let capabilities = self.router.list_capabilities().await;
        let cap_list: Vec<String> = capabilities.keys().cloned().collect();

        Ok(json!({
            "capabilities": cap_list,
            "count": cap_list.len()
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
                    "socket": p.socket_path.display().to_string(),
                    "source": p.source,
                    "registered_at": p.registered_at.to_rfc3339()
                })
            }).collect::<Vec<_>>(),
            "count": providers.len()
        }))
    }

    /// Semantic capability call with automatic translation.
    ///
    /// JSON-RPC method: `capability.call`
    ///
    /// This is the main entry point for TRUE PRIMAL communication.
    /// Consumers use semantic names; we translate and route.
    ///
    /// # Parameters
    /// - `capability`: Target capability (e.g., "crypto")
    /// - `operation`: Semantic operation (e.g., "sha256")
    /// - `args`: Arguments for the operation
    pub async fn call(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let params = params.as_ref().context("Missing parameters")?;

        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;
        let operation = params["operation"]
            .as_str()
            .context("Missing 'operation' field")?;
        let args = params.get("args").cloned().unwrap_or(json!({}));

        trace!("capability.call: {}.{}", capability, operation);

        // Construct semantic name
        let semantic_name = format!("{}.{}", capability, operation);

        // Look up translation
        let registry = self.translation_registry.read().await;
        let translation = registry.get_translation(&semantic_name);

        match translation {
            Some(trans) => {
                debug!(
                    "   Translated: {} → {}:{}",
                    semantic_name, trans.provider, trans.actual_method
                );

                let method = trans.actual_method.clone();
                let provider = trans.provider.clone();

                // Drop the read lock before making the call
                drop(registry);

                // Discover primal socket
                let atomic = self.router.discover_capability(capability).await?;

                // Forward request
                let result = self
                    .router
                    .forward_request(&atomic.primary_socket, &method, &args)
                    .await?;

                let latency = start.elapsed().as_millis();
                trace!(
                    "   ✓ {} completed in {}ms via {}",
                    semantic_name,
                    latency,
                    provider
                );

                Ok(result)
            }
            None => {
                // No translation - try direct routing
                drop(registry);
                warn!(
                    "No translation for {}, attempting direct route",
                    semantic_name
                );

                let atomic = self.router.discover_capability(capability).await?;

                self.router
                    .forward_request(&atomic.primary_socket, &semantic_name, &args)
                    .await
            }
        }
    }

    /// Discover available translations for a capability.
    ///
    /// JSON-RPC method: `capability.discover_translations`
    pub async fn discover_translations(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;

        let registry = self.translation_registry.read().await;
        let translations = registry.provider_capabilities(capability);

        Ok(json!({
            "capability": capability,
            "translations": translations,
            "count": translations.len()
        }))
    }

    /// List all registered translations.
    ///
    /// JSON-RPC method: `capability.list_translations`
    pub async fn list_translations(&self) -> Result<Value> {
        let registry = self.translation_registry.read().await;
        let all_translations = registry.list_all();

        Ok(json!({
            "translations": all_translations.iter().map(|t| {
                json!({
                    "semantic": t.semantic,
                    "provider": t.provider,
                    "method": t.actual_method
                })
            }).collect::<Vec<_>>(),
            "count": all_translations.len()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capability_handler_creation() {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));

        let handler = CapabilityHandler::new(router, registry);

        // Should be able to list (empty) capabilities
        let result = handler.list().await.unwrap();
        assert!(result["capabilities"].as_array().unwrap().is_empty());
    }
}

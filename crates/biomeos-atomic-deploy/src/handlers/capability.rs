// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
    /// # Parameters (canonical format)
    /// - `capability`: Target capability domain (e.g., "crypto")
    /// - `operation`: Semantic operation (e.g., "sha256")
    /// - `args`: Arguments for the operation
    ///
    /// # Backward-compatible formats
    /// - Dotted capability: `{ "capability": "crypto.sha256", "args": {...} }`
    ///   splits on first dot into domain + operation.
    /// - `params` alias: `{ "capability": "crypto", "operation": "sha256", "params": {...} }`
    ///   treated as `args`.
    pub async fn call(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let params = params.as_ref().context("Missing parameters")?;

        let raw_capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;

        // Support dotted capability names: "crypto.sha256" → domain="crypto", op="sha256"
        let (capability, operation) = if let Some(explicit_op) = params["operation"].as_str() {
            (raw_capability, explicit_op.to_string())
        } else if let Some(dot_pos) = raw_capability.find('.') {
            (&raw_capability[..dot_pos], raw_capability[dot_pos + 1..].to_string())
        } else {
            anyhow::bail!("Missing 'operation' field and capability '{}' has no dotted operation", raw_capability);
        };

        // Accept both "args" and "params" (backward compat for older callers)
        let args = params.get("args")
            .or_else(|| params.get("params"))
            .cloned()
            .unwrap_or(json!({}));

        trace!("capability.call: {}.{}", capability, &operation);

        // Construct semantic name
        let semantic_name = format!("{}.{}", capability, &operation);

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

    fn make_handler() -> CapabilityHandler {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
        CapabilityHandler::new(router, registry)
    }

    async fn handler_with_registration() -> CapabilityHandler {
        let handler = make_handler();
        // Register a primal with a capability
        let params = Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog-test.sock",
            "source": "test"
        }));
        handler.register(&params).await.unwrap();
        handler
    }

    // ── Constructor ────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_capability_handler_creation() {
        let handler = make_handler();
        let result = handler.list().await.unwrap();
        assert!(result["capabilities"].as_array().unwrap().is_empty());
        assert_eq!(result["count"], 0);
    }

    // ── capability.list ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_list_empty() {
        let handler = make_handler();
        let result = handler.list().await.unwrap();
        assert_eq!(result["count"], 0);
        assert!(result["capabilities"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_after_register() {
        let handler = handler_with_registration().await;
        let result = handler.list().await.unwrap();
        assert_eq!(result["count"], 1);
        let caps = result["capabilities"].as_array().unwrap();
        assert!(caps.iter().any(|c| c.as_str() == Some("crypto")));
    }

    // ── capability.register ────────────────────────────────────────────

    #[tokio::test]
    async fn test_register_basic() {
        let handler = make_handler();
        let params = Some(json!({
            "capability": "http",
            "primal": "songbird",
            "socket": "/tmp/songbird.sock",
            "source": "unit_test"
        }));
        let result = handler.register(&params).await.unwrap();
        assert_eq!(result["success"], true);
        assert_eq!(result["capability"], "http");
        assert_eq!(result["primal"], "songbird");
    }

    #[tokio::test]
    async fn test_register_missing_params() {
        let handler = make_handler();
        let result = handler.register(&None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_missing_capability() {
        let handler = make_handler();
        let params = Some(json!({
            "primal": "beardog",
            "socket": "/tmp/test.sock"
        }));
        let result = handler.register(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_missing_primal() {
        let handler = make_handler();
        let params = Some(json!({
            "capability": "crypto",
            "socket": "/tmp/test.sock"
        }));
        let result = handler.register(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_missing_socket() {
        let handler = make_handler();
        let params = Some(json!({
            "capability": "crypto",
            "primal": "beardog"
        }));
        let result = handler.register(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_register_with_semantic_mappings() {
        let handler = make_handler();
        let params = Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock",
            "semantic_mappings": {
                "sha256": "crypto.blake3_hash",
                "sign": "crypto.sign"
            }
        }));
        let result = handler.register(&params).await.unwrap();
        assert_eq!(result["success"], true);

        // Verify translations were registered
        let translations_result = handler.list_translations().await.unwrap();
        assert!(translations_result["count"].as_u64().unwrap() >= 2);
    }

    #[tokio::test]
    async fn test_register_default_source() {
        let handler = make_handler();
        // No "source" field — should default to "manual"
        let params = Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock"
        }));
        let result = handler.register(&params).await.unwrap();
        assert_eq!(result["success"], true);
    }

    // ── capability.providers ───────────────────────────────────────────

    #[tokio::test]
    async fn test_providers_empty() {
        let handler = make_handler();
        let params = Some(json!({ "capability": "nonexistent" }));
        let result = handler.providers(&params).await.unwrap();
        assert_eq!(result["count"], 0);
    }

    #[tokio::test]
    async fn test_providers_after_register() {
        let handler = handler_with_registration().await;
        let params = Some(json!({ "capability": "crypto" }));
        let result = handler.providers(&params).await.unwrap();
        assert_eq!(result["count"], 1);
        let providers = result["providers"].as_array().unwrap();
        assert_eq!(providers[0]["primal"], "beardog");
    }

    #[tokio::test]
    async fn test_providers_missing_params() {
        let handler = make_handler();
        let result = handler.providers(&None).await;
        assert!(result.is_err());
    }

    // ── capability.metrics ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_metrics_empty() {
        let handler = make_handler();
        let result = handler.get_metrics().await.unwrap();
        assert_eq!(result["total_requests"], 0);
        assert!(result["metrics"].as_array().unwrap().is_empty());
    }

    // ── capability.discover_translations ───────────────────────────────

    #[tokio::test]
    async fn test_discover_translations_empty() {
        let handler = make_handler();
        let params = Some(json!({ "capability": "crypto" }));
        let result = handler.discover_translations(&params).await.unwrap();
        assert_eq!(result["count"], 0);
        assert_eq!(result["capability"], "crypto");
    }

    #[tokio::test]
    async fn test_discover_translations_after_register() {
        let handler = make_handler();
        // Register with semantic mappings
        let params = Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock",
            "semantic_mappings": {
                "sha256": "crypto.blake3_hash"
            }
        }));
        handler.register(&params).await.unwrap();

        let params = Some(json!({ "capability": "crypto" }));
        let result = handler.discover_translations(&params).await.unwrap();
        // Should find the translation via provider_capabilities
        assert_eq!(result["capability"], "crypto");
    }

    #[tokio::test]
    async fn test_discover_translations_missing_params() {
        let handler = make_handler();
        let result = handler.discover_translations(&None).await;
        assert!(result.is_err());
    }

    // ── capability.list_translations ───────────────────────────────────

    #[tokio::test]
    async fn test_list_translations_empty() {
        let handler = make_handler();
        let result = handler.list_translations().await.unwrap();
        assert_eq!(result["count"], 0);
    }

    #[tokio::test]
    async fn test_list_translations_after_register() {
        let handler = make_handler();
        let params = Some(json!({
            "capability": "crypto",
            "primal": "beardog",
            "socket": "/tmp/beardog.sock",
            "semantic_mappings": {
                "sha256": "crypto.blake3_hash",
                "sign": "crypto.sign"
            }
        }));
        handler.register(&params).await.unwrap();

        let result = handler.list_translations().await.unwrap();
        let count = result["count"].as_u64().unwrap();
        assert!(
            count >= 2,
            "Expected at least 2 translations, got {}",
            count
        );

        let translations = result["translations"].as_array().unwrap();
        let semantics: Vec<&str> = translations
            .iter()
            .filter_map(|t| t["semantic"].as_str())
            .collect();
        assert!(semantics.contains(&"crypto.sha256"));
        assert!(semantics.contains(&"crypto.sign"));
    }

    // ── capability.discover ────────────────────────────────────────────

    #[tokio::test]
    async fn test_discover_registered_capability() {
        let handler = handler_with_registration().await;
        let params = Some(json!({ "capability": "crypto" }));
        let result = handler.discover(&params).await.unwrap();
        assert_eq!(result["capability"], "crypto");
        let primals = result["primals"].as_array().unwrap();
        assert!(!primals.is_empty());
        assert_eq!(primals[0]["name"], "beardog");
    }

    #[tokio::test]
    async fn test_discover_missing_params() {
        let handler = make_handler();
        let result = handler.discover(&None).await;
        assert!(result.is_err());
    }

    // ── capability.call ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_call_missing_params() {
        let handler = make_handler();
        let result = handler.call(&None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_missing_capability() {
        let handler = make_handler();
        let params = Some(json!({ "operation": "sha256" }));
        let result = handler.call(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_missing_operation() {
        let handler = make_handler();
        let params = Some(json!({ "capability": "crypto" }));
        let result = handler.call(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_dotted_capability() {
        let handler = handler_with_registration().await;
        // "crypto.sha256" should split into capability="crypto", operation="sha256"
        let params = Some(json!({
            "capability": "crypto.sha256",
            "args": { "data": "test" }
        }));
        // Will fail at socket level but should NOT fail at param parsing
        let result = handler.call(&params).await;
        // Reaches routing (no socket) rather than "missing operation" error
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(!err.contains("Missing 'operation'"));
    }

    #[tokio::test]
    async fn test_call_params_alias_for_args() {
        let handler = handler_with_registration().await;
        // "params" should be accepted as alias for "args"
        let params = Some(json!({
            "capability": "crypto",
            "operation": "sha256",
            "params": { "data": "test" }
        }));
        let result = handler.call(&params).await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(!err.contains("Missing"));
    }

    // ── capability.route ───────────────────────────────────────────────

    #[tokio::test]
    async fn test_route_missing_params() {
        let handler = make_handler();
        let result = handler.route(&None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_route_missing_method() {
        let handler = make_handler();
        let params = Some(json!({ "capability": "crypto" }));
        let result = handler.route(&params).await;
        assert!(result.is_err());
    }

    // ── Multiple registrations ─────────────────────────────────────────

    #[tokio::test]
    async fn test_multiple_capabilities() {
        let handler = make_handler();

        handler
            .register(&Some(json!({
                "capability": "crypto",
                "primal": "beardog",
                "socket": "/tmp/beardog.sock"
            })))
            .await
            .unwrap();

        handler
            .register(&Some(json!({
                "capability": "http",
                "primal": "songbird",
                "socket": "/tmp/songbird.sock"
            })))
            .await
            .unwrap();

        let result = handler.list().await.unwrap();
        assert_eq!(result["count"], 2);
    }

    #[tokio::test]
    async fn test_multiple_providers_same_capability() {
        let handler = make_handler();

        handler
            .register(&Some(json!({
                "capability": "compute",
                "primal": "toadstool-tower",
                "socket": "/tmp/toadstool-tower.sock"
            })))
            .await
            .unwrap();

        handler
            .register(&Some(json!({
                "capability": "compute",
                "primal": "toadstool-gate2",
                "socket": "/tmp/toadstool-gate2.sock"
            })))
            .await
            .unwrap();

        let params = Some(json!({ "capability": "compute" }));
        let result = handler.providers(&params).await.unwrap();
        assert_eq!(result["count"], 2);
    }
}

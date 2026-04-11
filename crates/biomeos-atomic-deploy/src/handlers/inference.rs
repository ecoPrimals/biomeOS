// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Inference scheduling, provider registration, and cross-gate model orchestration.
//!
//! Routes AI inference requests to the best available GPU gate by querying
//! each gate's compute capabilities and selecting the optimal target based
//! on VRAM, availability, and model requirements.
//!
//! # JSON-RPC Methods (canonical `inference.*` namespace)
//!
//! - `inference.schedule` — schedule an inference job on the best gate
//! - `inference.gates` — list gates with GPU/AI capabilities
//! - `inference.register_provider` — register an inference backend (e.g. neuralSpring → Squirrel)
//! - `inference.providers` — list registered inference providers
//! - `inference.complete` — route a completion request to the best provider
//! - `inference.embed` — route an embedding request to the best provider
//! - `inference.models` — list models available across all providers
//!
//! # Architecture
//!
//! ```text
//! Consumer → inference.schedule({ model: "llama-3-70b", prompt: "..." })
//!              │
//!              ▼
//! InferenceHandler → query registered gates for compute.capabilities
//!              │
//!              ▼
//!          select best gate (VRAM, availability, affinity)
//!              │
//!              ▼
//!          capability.call → ai.query on selected gate's Squirrel
//! ```

use crate::gate_registry::GateRegistry;
use crate::neural_router::NeuralRouter;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Registered gate with its known GPU/compute capabilities.
#[derive(Debug, Clone)]
struct GateCapabilities {
    name: String,
    _endpoint: String,
    vram_gb: Option<f64>,
    gpu_model: Option<String>,
    available: bool,
}

/// Registered inference provider (e.g. neuralSpring, Ollama wrapper).
#[derive(Debug, Clone)]
pub struct InferenceProvider {
    /// Provider name (e.g. "neuralSpring", "ollama-local")
    pub name: Arc<str>,
    /// Transport endpoint string
    pub endpoint: biomeos_core::TransportEndpoint,
    /// Capabilities this provider offers (e.g. `complete`, `embed`, `models`)
    pub capabilities: Vec<String>,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Health status from last probe
    pub healthy: bool,
}

/// Inference scheduling handler.
#[derive(Clone)]
pub struct InferenceHandler {
    router: Arc<NeuralRouter>,
    gate_registry: Arc<GateRegistry>,
    providers: Arc<RwLock<Vec<InferenceProvider>>>,
}

impl InferenceHandler {
    /// Create a new inference handler with access to routing and gate registry.
    #[must_use]
    pub fn new(router: Arc<NeuralRouter>, gate_registry: Arc<GateRegistry>) -> Self {
        Self {
            router,
            gate_registry,
            providers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// List gates with their GPU/AI capabilities.
    ///
    /// JSON-RPC method: `inference.gates`
    ///
    /// Queries each registered gate's `compute.capabilities` to build a
    /// live view of available inference targets.
    pub async fn gates(&self, _params: &Option<Value>) -> Result<Value> {
        info!("🧠 Listing inference-capable gates");

        let gate_names = self.gate_registry.gate_names();
        let mut gates = Vec::new();

        // Local gate always available
        gates.push(json!({
            "gate": "local",
            "endpoint": "local",
            "status": "available",
            "source": "self",
        }));

        for name in &gate_names {
            let endpoint = match self.gate_registry.resolve(name) {
                Some(ep) => ep.display_string(),
                None => continue,
            };

            let status = match self.probe_gate_capabilities(name).await {
                Ok(caps) => json!({
                    "gate": name,
                    "endpoint": endpoint,
                    "status": "available",
                    "gpu": caps.gpu_model.unwrap_or_default(),
                    "vram_gb": caps.vram_gb.unwrap_or(0.0),
                }),
                Err(_) => json!({
                    "gate": name,
                    "endpoint": endpoint,
                    "status": "unreachable",
                }),
            };

            gates.push(status);
        }

        Ok(json!({
            "gates": gates,
            "count": gates.len(),
        }))
    }

    /// Schedule an inference job on the best available gate.
    ///
    /// JSON-RPC method: `inference.schedule`
    ///
    /// # Parameters
    /// - `model`: Model name or size hint (e.g., "llama-3-70b", "small", "large")
    /// - `prompt`: The inference prompt
    /// - `gate` (optional): Force a specific gate (bypasses scheduling)
    /// - `params` (optional): Additional model parameters (temperature, `max_tokens`, etc.)
    pub async fn schedule(&self, params: &Option<Value>) -> Result<Value> {
        let start = std::time::Instant::now();
        let params = params.as_ref().context("Missing parameters")?;

        let model = params
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let prompt = params
            .get("prompt")
            .and_then(|v| v.as_str())
            .context("Missing 'prompt' field")?;

        let forced_gate = params.get("gate").and_then(|v| v.as_str());
        let extra_params = params.get("params").cloned().unwrap_or(json!({}));

        info!(
            "🧠 Scheduling inference: model={}, prompt_len={}",
            model,
            prompt.len()
        );

        let target_gate = if let Some(gate) = forced_gate {
            debug!("   Gate forced: {}", gate);
            gate.to_string()
        } else {
            self.select_best_gate(model).await?
        };

        debug!("   Selected gate: {}", target_gate);

        let ai_params = json!({
            "capability": "ai",
            "operation": "query",
            "args": {
                "model": model,
                "prompt": prompt,
                "params": extra_params,
            }
        });

        let result = if target_gate == "local" {
            self.call_local_ai(&ai_params).await?
        } else {
            self.call_remote_ai(&target_gate, &ai_params).await?
        };

        let latency_ms = start.elapsed().as_millis();
        info!(
            "   ✓ Inference completed in {}ms on gate '{}'",
            latency_ms, target_gate
        );

        Ok(json!({
            "result": result,
            "gate": target_gate,
            "model": model,
            "latency_ms": latency_ms,
        }))
    }

    /// Register an inference provider.
    ///
    /// JSON-RPC method: `inference.register_provider`
    ///
    /// Springs (e.g. neuralSpring) call this to announce themselves as inference
    /// backends. Squirrel and biomeOS can then route `inference.complete`,
    /// `inference.embed`, etc. to the registered provider.
    ///
    /// # Parameters
    /// - `name`: Provider name (e.g. "neuralSpring")
    /// - `endpoint`: Transport endpoint (e.g. "/run/biomeos/neuralspring.sock")
    /// - `capabilities` (optional): Array of supported operations (defaults to
    ///   `["complete", "embed", "models"]`)
    pub async fn register_provider(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let name = params["name"]
            .as_str()
            .context("Missing 'name' field")?;
        let endpoint_str = params["endpoint"]
            .as_str()
            .context("Missing 'endpoint' field")?;

        let endpoint = biomeos_core::TransportEndpoint::parse(endpoint_str)
            .unwrap_or_else(|| biomeos_core::TransportEndpoint::UnixSocket {
                path: std::path::PathBuf::from(endpoint_str),
            });

        let capabilities: Vec<String> = params
            .get("capabilities")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_else(|| {
                vec![
                    "complete".to_owned(),
                    "embed".to_owned(),
                    "models".to_owned(),
                ]
            });

        info!(
            "🧠 Registering inference provider: {} @ {} (capabilities: {:?})",
            name, endpoint_str, capabilities
        );

        let provider = InferenceProvider {
            name: Arc::from(name),
            endpoint: endpoint.clone(),
            capabilities: capabilities.clone(),
            registered_at: chrono::Utc::now(),
            healthy: true,
        };

        let mut providers = self.providers.write().await;
        providers.retain(|p| *p.name != *name);
        providers.push(provider);

        self.router
            .register_capability("inference", name, endpoint, "inference.register_provider")
            .await?;

        Ok(json!({
            "registered": true,
            "name": name,
            "endpoint": endpoint_str,
            "capabilities": capabilities,
        }))
    }

    /// List registered inference providers.
    ///
    /// JSON-RPC method: `inference.providers`
    pub async fn list_providers(&self, _params: &Option<Value>) -> Result<Value> {
        let providers = self.providers.read().await;
        let entries: Vec<Value> = providers
            .iter()
            .map(|p| {
                json!({
                    "name": p.name,
                    "endpoint": p.endpoint.display_string(),
                    "capabilities": p.capabilities,
                    "registered_at": p.registered_at.to_rfc3339(),
                    "healthy": p.healthy,
                })
            })
            .collect();

        Ok(json!({
            "providers": entries,
            "count": entries.len(),
        }))
    }

    /// Route a completion request to the best inference provider.
    ///
    /// JSON-RPC method: `inference.complete`
    ///
    /// Discovers registered inference providers and forwards the request.
    /// Falls back to Squirrel via `capability.call` if no dedicated provider.
    pub async fn complete(&self, params: &Option<Value>) -> Result<Value> {
        self.forward_to_provider("complete", "inference.complete", params)
            .await
    }

    /// Route an embedding request to the best inference provider.
    ///
    /// JSON-RPC method: `inference.embed`
    pub async fn embed(&self, params: &Option<Value>) -> Result<Value> {
        self.forward_to_provider("embed", "inference.embed", params)
            .await
    }

    /// List models available across all inference providers.
    ///
    /// JSON-RPC method: `inference.models`
    pub async fn models(&self, params: &Option<Value>) -> Result<Value> {
        self.forward_to_provider("models", "inference.models", params)
            .await
    }

    /// Forward a request to the best provider that supports the given operation.
    async fn forward_to_provider(
        &self,
        operation: &str,
        rpc_method: &str,
        params: &Option<Value>,
    ) -> Result<Value> {
        let providers = self.providers.read().await;
        let target = providers
            .iter()
            .find(|p| p.healthy && p.capabilities.iter().any(|c| c == operation));

        if let Some(provider) = target {
            let endpoint = provider.endpoint.clone();
            let provider_name = provider.name.clone();
            drop(providers);

            debug!("🧠 {} → provider '{}'", rpc_method, provider_name);
            let args = params.clone().unwrap_or(json!({}));
            self.router
                .forward_request(&endpoint, rpc_method, &args)
                .await
        } else {
            drop(providers);
            debug!(
                "🧠 {} → no dedicated provider, routing via capability layer",
                rpc_method
            );
            let atomic = self.router.discover_capability("inference").await?;
            let args = params.clone().unwrap_or(json!({}));
            self.router
                .forward_request(&atomic.primary_endpoint, rpc_method, &args)
                .await
        }
    }

    /// Select the best gate for a given model based on VRAM requirements.
    ///
    /// Strategy:
    /// 1. Probe all registered gates for compute capabilities
    /// 2. Filter to gates that are reachable and have `ai` capability
    /// 3. For large models (70b+), prefer gates with >40GB VRAM
    /// 4. For medium models (7b-30b), prefer gates with >8GB VRAM
    /// 5. Fall back to local if no remote gates qualify
    async fn select_best_gate(&self, model: &str) -> Result<String> {
        let required_vram = Self::estimate_vram_requirement(model);
        let gate_names = self.gate_registry.gate_names();

        let mut candidates: Vec<GateCapabilities> = Vec::new();

        for name in &gate_names {
            match self.probe_gate_capabilities(name).await {
                Ok(caps) if caps.available => candidates.push(caps),
                Ok(_) => debug!("   Gate '{}' not available", name),
                Err(e) => debug!("   Gate '{}' probe failed: {}", name, e),
            }
        }

        // Sort by VRAM descending — prefer the gate with the most headroom
        candidates.sort_by(|a, b| {
            b.vram_gb
                .unwrap_or(0.0)
                .partial_cmp(&a.vram_gb.unwrap_or(0.0))
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for candidate in &candidates {
            let vram = candidate.vram_gb.unwrap_or(0.0);
            if vram >= required_vram {
                info!(
                    "   🎯 Selected gate '{}' ({}, {:.0}GB VRAM) for model requiring {:.0}GB",
                    candidate.name,
                    candidate.gpu_model.as_deref().unwrap_or("unknown"),
                    vram,
                    required_vram
                );
                return Ok(candidate.name.clone());
            }
        }

        if !candidates.is_empty() {
            let best = &candidates[0];
            warn!(
                "   No gate meets VRAM requirement ({:.0}GB), using best available: '{}' ({:.0}GB)",
                required_vram,
                best.name,
                best.vram_gb.unwrap_or(0.0)
            );
            return Ok(best.name.clone());
        }

        debug!("   No remote gates available, using local");
        Ok("local".to_string())
    }

    /// Estimate VRAM required for a model based on name heuristics.
    fn estimate_vram_requirement(model: &str) -> f64 {
        let lower = model.to_lowercase();
        if lower.contains("70b") || lower.contains("65b") {
            40.0
        } else if lower.contains("30b") || lower.contains("34b") || lower.contains("33b") {
            20.0
        } else if lower.contains("13b") || lower.contains("14b") {
            10.0
        } else if lower.contains("7b") || lower.contains("8b") {
            6.0
        } else if lower.contains("3b") || lower.contains("1b") {
            3.0
        } else if lower.contains("large") {
            24.0
        } else if lower.contains("small") || lower.contains("mini") {
            4.0
        } else {
            // API models (gpt-4, claude, etc.) don't need local VRAM
            0.0
        }
    }

    async fn probe_gate_capabilities(&self, gate: &str) -> Result<GateCapabilities> {
        let endpoint = self
            .gate_registry
            .resolve(gate)
            .context("Gate not in registry")?;

        let client = biomeos_core::AtomicClient::from_endpoint(endpoint.clone());
        let result = client
            .call("compute.capabilities", json!({}))
            .await
            .with_context(|| format!("Failed to probe gate '{gate}'"))?;

        Ok(GateCapabilities {
            name: gate.to_string(),
            _endpoint: endpoint.display_string(),
            vram_gb: result.get("vram_gb").and_then(|v| v.as_f64()),
            gpu_model: result
                .get("gpu_model")
                .and_then(|v| v.as_str())
                .map(String::from),
            available: result
                .get("available")
                .and_then(|v| v.as_bool())
                .unwrap_or(true),
        })
    }

    async fn call_local_ai(&self, params: &Value) -> Result<Value> {
        let capability = params["args"]["capability"].as_str().unwrap_or("ai");
        let atomic = self.router.discover_capability(capability).await?;
        self.router
            .forward_request(&atomic.primary_endpoint, "query_ai", &params["args"])
            .await
    }

    async fn call_remote_ai(&self, gate: &str, params: &Value) -> Result<Value> {
        let endpoint = self
            .gate_registry
            .resolve(gate)
            .context("Gate not in registry")?;
        let client = biomeos_core::AtomicClient::from_endpoint(endpoint.clone());
        client
            .call("capability.call", params.clone())
            .await
            .with_context(|| format!("Inference on gate '{gate}' failed"))
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test")]
mod tests {
    use super::*;
    use crate::gate_registry::GateRegistry;
    use crate::neural_router::NeuralRouter;

    fn make_handler() -> InferenceHandler {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let mut registry = GateRegistry::new();
        registry.register(
            "gate2",
            biomeos_core::TransportEndpoint::parse("tcp://192.0.2.132:9001").unwrap(),
        );
        InferenceHandler::new(router, Arc::new(registry))
    }

    #[test]
    fn test_estimate_vram_70b() {
        assert!(InferenceHandler::estimate_vram_requirement("llama-3-70b") >= 40.0);
    }

    #[test]
    fn test_estimate_vram_7b() {
        let vram = InferenceHandler::estimate_vram_requirement("mistral-7b-instruct");
        assert!((4.0..=8.0).contains(&vram));
    }

    #[test]
    fn test_estimate_vram_api_model() {
        assert!(InferenceHandler::estimate_vram_requirement("gpt-4-turbo").abs() < f64::EPSILON,);
        assert!(InferenceHandler::estimate_vram_requirement("claude-3-opus").abs() < f64::EPSILON,);
    }

    #[test]
    fn test_estimate_vram_size_hints() {
        assert!(InferenceHandler::estimate_vram_requirement("large") >= 20.0);
        assert!(InferenceHandler::estimate_vram_requirement("small") <= 6.0);
    }

    #[test]
    fn test_handler_construction() {
        let handler = make_handler();
        assert!(Arc::strong_count(&handler.gate_registry) >= 1);
    }

    #[tokio::test]
    async fn test_gates_includes_local() {
        let handler = make_handler();
        let result = handler.gates(&None).await.unwrap();
        let gates = result["gates"].as_array().unwrap();
        assert!(
            gates.iter().any(|g| g["gate"] == "local"),
            "local gate should always be listed"
        );
    }

    #[tokio::test]
    async fn test_schedule_requires_prompt() {
        let handler = make_handler();
        let params = Some(json!({"model": "test"}));
        let result = handler.schedule(&params).await;
        assert!(result.is_err(), "should fail without prompt");
    }

    #[test]
    fn test_estimate_vram_30b_class() {
        assert!(
            (18.0..=22.0).contains(&InferenceHandler::estimate_vram_requirement(
                "model-30b-chat",
            ))
        );
    }

    #[test]
    fn test_estimate_vram_13b_class() {
        assert!((9.0..=11.0).contains(&InferenceHandler::estimate_vram_requirement("llama-13b",)));
    }

    #[test]
    fn test_estimate_vram_tiny_and_mini_hints() {
        assert!((2.0..=4.0).contains(&InferenceHandler::estimate_vram_requirement("phi-3b")));
        assert!((3.0..=5.0).contains(&InferenceHandler::estimate_vram_requirement("mini-model")));
    }

    #[tokio::test]
    async fn test_schedule_missing_params_none_errors() {
        let handler = make_handler();
        let err = handler.schedule(&None).await.unwrap_err();
        assert!(err.to_string().contains("Missing parameters"));
    }

    #[tokio::test]
    async fn test_schedule_empty_object_missing_prompt_errors() {
        let handler = make_handler();
        let err = handler.schedule(&Some(json!({}))).await.unwrap_err();
        assert!(err.to_string().contains("prompt") || err.to_string().contains("Missing"));
    }

    // --- inference.register_provider tests ---

    #[tokio::test]
    async fn test_register_provider_success() {
        let handler = make_handler();
        let params = Some(json!({
            "name": "neuralSpring",
            "endpoint": "/tmp/neural.sock",
        }));
        let result = handler.register_provider(&params).await.unwrap();
        assert_eq!(result["registered"], true);
        assert_eq!(result["name"], "neuralSpring");
        let caps = result["capabilities"].as_array().unwrap();
        assert!(caps.iter().any(|c| c == "complete"));
        assert!(caps.iter().any(|c| c == "embed"));
        assert!(caps.iter().any(|c| c == "models"));
    }

    #[tokio::test]
    async fn test_register_provider_custom_capabilities() {
        let handler = make_handler();
        let params = Some(json!({
            "name": "custom",
            "endpoint": "/tmp/custom.sock",
            "capabilities": ["complete", "fine_tune"]
        }));
        let result = handler.register_provider(&params).await.unwrap();
        let caps = result["capabilities"].as_array().unwrap();
        assert_eq!(caps.len(), 2);
    }

    #[tokio::test]
    async fn test_register_provider_missing_name_errors() {
        let handler = make_handler();
        let params = Some(json!({ "endpoint": "/tmp/x.sock" }));
        assert!(handler.register_provider(&params).await.is_err());
    }

    #[tokio::test]
    async fn test_register_provider_missing_endpoint_errors() {
        let handler = make_handler();
        let params = Some(json!({ "name": "x" }));
        assert!(handler.register_provider(&params).await.is_err());
    }

    #[tokio::test]
    async fn test_register_provider_missing_params() {
        let handler = make_handler();
        assert!(handler.register_provider(&None).await.is_err());
    }

    #[tokio::test]
    async fn test_register_provider_replaces_existing() {
        let handler = make_handler();
        let params1 = Some(json!({
            "name": "neural",
            "endpoint": "/tmp/neural1.sock",
        }));
        handler.register_provider(&params1).await.unwrap();

        let params2 = Some(json!({
            "name": "neural",
            "endpoint": "/tmp/neural2.sock",
        }));
        handler.register_provider(&params2).await.unwrap();

        let list = handler.list_providers(&None).await.unwrap();
        assert_eq!(list["count"], 1, "re-registration should replace, not duplicate");
    }

    #[tokio::test]
    async fn test_list_providers_empty() {
        let handler = make_handler();
        let result = handler.list_providers(&None).await.unwrap();
        assert_eq!(result["count"], 0);
        assert!(result["providers"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_providers_after_registration() {
        let handler = make_handler();
        handler
            .register_provider(&Some(json!({
                "name": "neuralSpring",
                "endpoint": "/tmp/neural.sock",
            })))
            .await
            .unwrap();

        let result = handler.list_providers(&None).await.unwrap();
        assert_eq!(result["count"], 1);
        let providers = result["providers"].as_array().unwrap();
        assert_eq!(providers[0]["name"], "neuralSpring");
        assert!(providers[0]["healthy"].as_bool().unwrap());
    }
}

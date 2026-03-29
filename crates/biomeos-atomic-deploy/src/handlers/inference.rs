// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Inference scheduling and cross-gate model orchestration.
//!
//! Routes AI inference requests to the best available GPU gate by querying
//! each gate's compute capabilities and selecting the optimal target based
//! on VRAM, availability, and model requirements.
//!
//! # JSON-RPC Methods
//!
//! - `inference.schedule` — schedule an inference job on the best gate
//! - `inference.gates` — list gates with GPU/AI capabilities
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

/// Inference scheduling handler.
#[derive(Clone)]
pub struct InferenceHandler {
    router: Arc<NeuralRouter>,
    gate_registry: Arc<GateRegistry>,
}

impl InferenceHandler {
    /// Create a new inference handler with access to routing and gate registry.
    pub fn new(router: Arc<NeuralRouter>, gate_registry: Arc<GateRegistry>) -> Self {
        Self {
            router,
            gate_registry,
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
    /// - `params` (optional): Additional model parameters (temperature, max_tokens, etc.)
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
mod tests {
    use super::*;
    use crate::gate_registry::GateRegistry;
    use crate::neural_router::NeuralRouter;

    fn make_handler() -> InferenceHandler {
        let router = Arc::new(NeuralRouter::new("test-family"));
        let mut registry = GateRegistry::new();
        registry.register(
            "gate2",
            biomeos_core::TransportEndpoint::parse("tcp://192.168.1.132:9001").unwrap(),
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
}

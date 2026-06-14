// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability call parameter enrichment (JH-2, JH-11, exp111).
//!
//! Injects authentication and resource envelope fields into forwarded
//! capability call params before they reach the `CapabilityHandler`.

use biomeos_core::method_gate::CallerContext;
use serde_json::{Value, json};

use super::NeuralApiServer;

impl NeuralApiServer {
    /// Enrich capability call params with forwarding context.
    ///
    /// Injected fields:
    /// - `_resource_envelope` — downstream primals enforce cpu/mem/timeout_ms/method_allowlist.
    /// - `_bearer_token` — downstream primals in enforced mode need the caller's
    ///   token for their own MethodGate check.
    /// - `_token_verified` — whether biomeOS verified the token against BearDog
    ///   via IPC (JH-11 federation step 1).
    pub(crate) async fn enrich_for_forwarding(
        &self,
        params: &Option<Value>,
        caller: &CallerContext,
    ) -> Option<Value> {
        let mut enriched = params.clone().unwrap_or(json!({}));

        if let Some(obj) = enriched.as_object_mut() {
            if let Some(ref claims) = caller.claims {
                if let Some(ref env) = claims.resources {
                    obj.insert("_resource_envelope".to_string(), env.to_forwarding_value());
                }
            }
            if let Some(ref token) = caller.bearer_token {
                obj.insert("_bearer_token".to_string(), json!(token));

                let verified = if let Some(ref verifier) = self.security_verifier {
                    verifier.verify_async(token).await.is_some()
                } else {
                    false
                };
                obj.insert("_token_verified".to_string(), json!(verified));
            }
        }

        Some(enriched)
    }

    /// Weight health introspection — convergence diagnostics and circuit breaker status.
    pub(crate) async fn handle_weight_health(&self) -> anyhow::Result<serde_json::Value> {
        let weights = self.router.get_routing_weights().await;
        let summary = self.router.get_weight_summary().await;

        let mut open_circuits = Vec::new();
        let mut converging = 0u32;
        let mut cold = 0u32;

        for w in &weights {
            let total = w.success_count + w.failure_count;
            if w.circuit_open {
                open_circuits.push(json!({
                    "provider": w.provider,
                    "capability": w.capability,
                    "consecutive_failures": w.consecutive_failures,
                    "opened_at": w.circuit_opened_at,
                }));
            }
            if total >= 10 {
                converging += 1;
            } else {
                cold += 1;
            }
        }

        let shadow = self.router.shadow_stats();

        Ok(json!({
            "healthy": open_circuits.is_empty(),
            "persistent": self.router.weights_are_persistent().await,
            "summary": summary,
            "convergence": {
                "converging": converging,
                "cold": cold,
                "total_providers": weights.len(),
            },
            "open_circuits": open_circuits,
            "shadow_routing": {
                "total_dispatches": shadow.0,
                "disagreements": shadow.1,
                "divergence_pct": if shadow.0 > 0 { (shadow.1 as f64 / shadow.0 as f64) * 100.0 } else { 0.0 },
                "limit": super::SHADOW_TRAINING_COMPLETION_THRESHOLD,
                "phase": if shadow.0 >= super::SHADOW_TRAINING_COMPLETION_THRESHOLD { "complete" } else { "active" },
            },
            "perceptron": match self.router.perceptron_shadow_stats() {
                Some((total, disagree)) => json!({
                    "phase": format!("{:?}", self.router.perceptron_phase().unwrap_or(
                        crate::neural_router::PerceptronPhase::Shadow
                    )),
                    "remote_infer": self.router.perceptron_has_remote_infer(),
                    "total_dispatches": total,
                    "disagreements": disagree,
                    "divergence_pct": if total > 0 { (disagree as f64 / total as f64) * 100.0 } else { 0.0 },
                }),
                None => json!(null),
            },
            "training_data_buffered": self.router.training_data_count().await,
        }))
    }

    /// Drain buffered training rows for barraCuda `ml.mlp_train` consumption.
    ///
    /// Returns all accumulated dispatch training rows and clears the buffer.
    /// Each row contains the 36-dim feature vectors, chosen provider index,
    /// and the post-dispatch outcome (success, latency).
    pub(crate) async fn handle_training_data_drain(&self) -> anyhow::Result<serde_json::Value> {
        let rows = self.router.drain_training_data().await;
        let count = rows.len();
        Ok(json!({
            "rows": rows,
            "count": count,
            "feature_dim": crate::neural_router::perceptron::FEATURE_DIM,
        }))
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Composition orchestration logic for Neural API routing.
//!
//! Extracted from `routing.rs` to keep the dispatch table focused on routing
//! while composition lifecycle (deploy → gossip → verify) lives here.

use serde_json::{Value, json};

use super::NeuralApiServer;

impl NeuralApiServer {
    /// Orchestrate a full composition — sequences prerequisites then executes the target.
    ///
    /// For `nucleus`, this means: tower → nest + node (parallel) → nucleus_complete.
    /// Each prerequisite is started via `composition.start { execute: true }` only if
    /// its health gate is not already satisfied. Returns results for each step.
    ///
    /// Lifecycle: deploy → register (local) → gossip (advertise) → verify (primalSpring).
    /// Gossip and verify are best-effort — failures are logged but don't block
    /// the orchestration result. This enables graceful degradation when swarmVine
    /// or primalSpring aren't available.
    pub(super) async fn orchestrate_composition(
        &self,
        params: &Option<Value>,
    ) -> anyhow::Result<Value> {
        use anyhow::Context;

        let params = params.as_ref().context("Missing parameters")?;
        let target = params["composition"]
            .as_str()
            .or_else(|| params["name"].as_str())
            .context("Missing 'composition' parameter")?;

        let sequence = match target {
            "tower" => vec!["tower"],
            "nest" => vec!["tower", "nest"],
            "node" => vec!["tower", "node"],
            "nucleus" => vec!["tower", "nest", "node"],
            other => {
                return Ok(json!({
                    "error": format!("Unknown composition: {other}"),
                    "available": ["tower", "nest", "node", "nucleus"],
                }));
            }
        };

        let user_params = params.get("params").cloned().unwrap_or(json!({}));
        let mut steps = Vec::new();
        let mut deployed_tiers: Vec<&str> = Vec::new();

        for composition in &sequence {
            let step_params = Some(json!({
                "composition": composition,
                "execute": true,
                "params": user_params,
            }));

            let health = self.lifecycle_handler.composition_health(&None).await?;
            let already_healthy = health
                .get("subsystems")
                .and_then(|s| s.get(*composition))
                .and_then(|v| v.as_str())
                == Some("ok");

            if already_healthy {
                steps.push(json!({
                    "composition": composition,
                    "action": "skipped",
                    "reason": "already healthy",
                }));
                deployed_tiers.push(composition);
                continue;
            }

            let result = self.lifecycle_handler.composition_start(&step_params).await?;
            let ready = result.get("ready").and_then(|r| r.as_bool()) == Some(true);

            if ready {
                let graph_id = result["graph"].as_str().unwrap_or_default();
                let graph_params = Some(json!({
                    "graph_id": graph_id,
                    "params": user_params,
                }));
                let exec_result = self.graph_handler.execute(&graph_params).await;
                match exec_result {
                    Ok(v) => {
                        steps.push(json!({
                            "composition": composition,
                            "action": "executed",
                            "graph": graph_id,
                            "result": v,
                        }));
                        deployed_tiers.push(composition);
                    }
                    Err(e) => {
                        steps.push(json!({
                            "composition": composition,
                            "action": "failed",
                            "graph": graph_id,
                            "error": e.to_string(),
                        }));
                        return Ok(json!({
                            "target": target,
                            "completed": false,
                            "steps": steps,
                            "error": format!("Composition '{composition}' failed: {e}"),
                        }));
                    }
                }
            } else {
                steps.push(json!({
                    "composition": composition,
                    "action": "blocked",
                    "blocked_by": result.get("blocked_by"),
                }));
                return Ok(json!({
                    "target": target,
                    "completed": false,
                    "steps": steps,
                    "error": format!("Composition '{composition}' blocked by prerequisites"),
                }));
            }
        }

        // Post-deploy: gossip advertise deployed capabilities to swarmVine.
        // Best-effort — swarmVine may not be available during bootstrap.
        let gossip_result = self
            .advertise_composition_to_gossip(target, &deployed_tiers)
            .await;

        // Post-deploy: verify composition via primalSpring (integration primal).
        // Best-effort — primalSpring may not be deployed yet.
        let verify_result = self
            .verify_composition_in_mesh(target, &deployed_tiers)
            .await;

        Ok(json!({
            "target": target,
            "completed": true,
            "steps": steps,
            "gossip": gossip_result,
            "verify": verify_result,
        }))
    }

    /// Advertise deployed composition capabilities to swarmVine gossip mesh.
    ///
    /// Injects capability advertisements for each deployed tier so that
    /// cross-gate discovery can find this gate's available compositions.
    async fn advertise_composition_to_gossip(
        &self,
        target: &str,
        deployed_tiers: &[&str],
    ) -> Value {
        let registry = self.translation_registry.read().await;

        if !registry.has_capability("gossip.advertise") {
            return json!({ "status": "skipped", "reason": "gossip.advertise not available" });
        }

        let gate_id = &self.family_id;
        let advertise_params = json!({
            "topic": "tower",
            "key": format!("composition.available:{}:{}", gate_id, target),
            "value": json!({
                "gate": gate_id,
                "composition": target,
                "tiers": deployed_tiers,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }).to_string(),
            "ttl_secs": 300,
        });

        match registry.call_capability("gossip.advertise", advertise_params).await {
            Ok(v) => json!({ "status": "advertised", "result": v }),
            Err(e) => {
                tracing::debug!("Gossip advertise best-effort failed: {e}");
                json!({ "status": "unavailable", "error": e.to_string() })
            }
        }
    }

    /// Verify composition health via primalSpring integration testing.
    ///
    /// Calls `composition.validate` which routes to primalSpring for
    /// cross-primal IPC verification after deployment.
    async fn verify_composition_in_mesh(
        &self,
        target: &str,
        deployed_tiers: &[&str],
    ) -> Value {
        let registry = self.translation_registry.read().await;

        if !registry.has_capability("composition.validate") {
            return json!({ "status": "skipped", "reason": "composition.validate not available" });
        }

        let validate_params = json!({
            "composition": target,
            "tiers": deployed_tiers,
            "gate": &self.family_id,
            "mode": "post_deploy",
        });

        match registry.call_capability("composition.validate", validate_params).await {
            Ok(v) => json!({ "status": "verified", "result": v }),
            Err(e) => {
                tracing::debug!("Composition verify best-effort failed: {e}");
                json!({ "status": "unavailable", "error": e.to_string() })
            }
        }
    }
}

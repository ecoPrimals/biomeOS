// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Composition orchestration logic for Neural API routing.
//!
//! Extracted from `routing.rs` to keep the dispatch table focused on routing
//! while composition lifecycle (deploy → gossip → verify) lives here.

use serde_json::{Value, json};

use super::NeuralApiServer;

impl NeuralApiServer {
    /// Handle direct `deploy.result` RPC calls from external callers.
    ///
    /// Allows lifecycle events (individual primal starts/stops) to be signaled
    /// to the gossip mesh without going through full orchestration.
    /// Expected params: `{ composition, tiers, success, error? }`
    pub(super) async fn handle_deploy_result_emit(
        &self,
        params: &Option<Value>,
    ) -> anyhow::Result<Value> {
        use anyhow::Context;

        let params = params.as_ref().context("Missing parameters")?;
        let composition = params["composition"]
            .as_str()
            .unwrap_or("unknown");
        let success = params["success"].as_bool().unwrap_or(true);
        let error = params["error"].as_str();

        let tiers: Vec<&str> = params["tiers"]
            .as_array()
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let result = self
            .emit_deploy_result(composition, &tiers, success, error)
            .await;
        Ok(result)
    }

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
                        let error_msg = format!("Composition '{composition}' failed: {e}");
                        self.emit_deploy_result(
                            target,
                            &deployed_tiers,
                            false,
                            Some(&error_msg),
                        )
                        .await;
                        return Ok(json!({
                            "target": target,
                            "completed": false,
                            "steps": steps,
                            "error": error_msg,
                        }));
                    }
                }
            } else {
                steps.push(json!({
                    "composition": composition,
                    "action": "blocked",
                    "blocked_by": result.get("blocked_by"),
                }));
                let error_msg =
                    format!("Composition '{composition}' blocked by prerequisites");
                self.emit_deploy_result(target, &deployed_tiers, false, Some(&error_msg))
                    .await;
                return Ok(json!({
                    "target": target,
                    "completed": false,
                    "steps": steps,
                    "error": error_msg,
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

        // Phase 1: emit deploy.result gossip for fleet convergence tracking.
        let deploy_result_gossip = self
            .emit_deploy_result(target, &deployed_tiers, true, None)
            .await;

        Ok(json!({
            "target": target,
            "completed": true,
            "steps": steps,
            "gossip": gossip_result,
            "verify": verify_result,
            "deploy_result": deploy_result_gossip,
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

    /// Emit a `deploy.result` gossip message for fleet convergence tracking.
    ///
    /// Phase 1 of the deployment signaling evolution. Every orchestration outcome
    /// (success or failure) is broadcast to the gossip mesh so other gates and
    /// primalSpring can track fleet-wide deployment state.
    ///
    /// Best-effort — if swarmVine is unavailable the result is logged but not fatal.
    async fn emit_deploy_result(
        &self,
        composition: &str,
        deployed_tiers: &[&str],
        success: bool,
        error: Option<&str>,
    ) -> Value {
        let registry = self.translation_registry.read().await;

        if !registry.has_capability("gossip.advertise") {
            return json!({ "status": "skipped", "reason": "gossip.advertise not available" });
        }

        let gate_id = &self.family_id;
        let node_id = std::env::var("NODE_ID").unwrap_or_else(|_| gate_id.clone());
        let payload = json!({
            "gate": gate_id,
            "node_id": node_id,
            "composition": composition,
            "tiers_deployed": deployed_tiers,
            "success": success,
            "error": error,
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "version": env!("CARGO_PKG_VERSION"),
        });

        let advertise_params = json!({
            "topic": "deploy.result",
            "key": format!("deploy.result:{}:{}", gate_id, composition),
            "value": payload.to_string(),
            "ttl_secs": 600,
        });

        match registry.call_capability("gossip.advertise", advertise_params).await {
            Ok(v) => {
                tracing::info!(
                    gate = %gate_id,
                    composition = %composition,
                    success = %success,
                    "deploy.result emitted to gossip mesh"
                );
                json!({ "status": "emitted", "result": v })
            }
            Err(e) => {
                tracing::debug!("deploy.result gossip best-effort failed: {e}");
                json!({ "status": "unavailable", "error": e.to_string() })
            }
        }
    }
}

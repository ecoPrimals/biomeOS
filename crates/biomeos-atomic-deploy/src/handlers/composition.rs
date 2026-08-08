// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Composition status and health handlers.
//!
//! Extracted from `lifecycle.rs` to keep handler files under 800 lines.
//! These methods report on the health and readiness of the composed primal
//! ecosystem, including pipeline status for content and compute.

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};

use super::LifecycleHandler;
use super::spring_status::state_to_string;
use crate::lifecycle_manager::LifecycleState;

impl LifecycleHandler {
    /// Handle `composition.status` — adaptive daemon surface for pappusCast.
    ///
    /// Returns `{ active_users, primal_health, resource_pressure, pipelines }` so
    /// projectNUCLEUS can drive adaptive daemons without scraping JupyterHub.
    ///
    /// JSON-RPC method: `composition.status`
    pub async fn composition_status(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let mut primal_health = Vec::with_capacity(status.len());
        let mut active_count: u64 = 0;

        for (name, state) in &status {
            let info = manager.get_primal_info(name).await;

            let (latency_ms, failures, resurrection_count, capabilities) = if let Some(ref i) = info
            {
                let caps: Vec<String> = i
                    .deployment_node
                    .as_ref()
                    .map(|n| n.capabilities.clone())
                    .unwrap_or_default();
                (
                    i.metrics.last_health_latency_ms,
                    i.metrics.health_failures,
                    i.metrics.resurrection_count,
                    caps,
                )
            } else {
                (0, 0, 0, vec![])
            };

            let state_str = state_to_string(state);
            if state_str == "active" {
                active_count += 1;
            }

            primal_health.push(json!({
                "name": name,
                "state": state_str,
                "latency_ms": latency_ms,
                "failures": failures,
                "resurrection_count": resurrection_count,
                "capabilities": capabilities,
            }));
        }

        let resource_pressure = match biomeos_system::SystemInspector::get_resource_usage().await {
            Ok(metrics) => json!({
                "cpu": metrics.cpu_usage,
                "memory": metrics.memory_usage,
                "disk": metrics.disk_usage,
            }),
            Err(e) => {
                warn!("Resource metrics unavailable: {e}");
                json!({
                    "cpu": null,
                    "memory": null,
                    "disk": null,
                    "error": e.to_string(),
                })
            }
        };

        let topology = self
            .topology_version
            .load(std::sync::atomic::Ordering::Relaxed);

        let has_active_domain = |domain: &str| -> bool {
            primal_health.iter().any(|p| {
                p["state"].as_str() == Some("active")
                    && p["capabilities"].as_array().is_some_and(|caps| {
                        caps.iter()
                            .any(|c| c.as_str().is_some_and(|s| s.starts_with(domain)))
                    })
            })
        };
        let content_pipeline = json!({
            "storage": has_active_domain("storage.") || has_active_domain("content."),
            "dag": has_active_domain("dag."),
            "ledger": has_active_domain("spine.") || has_active_domain("anchor."),
            "attribution": has_active_domain("braid.") || has_active_domain("provenance."),
            "ready": (has_active_domain("storage.") || has_active_domain("content."))
                && has_active_domain("dag.")
                && (has_active_domain("spine.") || has_active_domain("anchor."))
                && (has_active_domain("braid.") || has_active_domain("provenance.")),
        });
        let compute_pipeline = json!({
            "dispatch": has_active_domain("compute."),
            "shader": has_active_domain("shader.") || has_active_domain("compile."),
            "gpu": has_active_domain("math.") || has_active_domain("gpu."),
            "ready": has_active_domain("compute."),
        });

        Ok(json!({
            "active_users": active_count,
            "primal_health": primal_health,
            "resource_pressure": resource_pressure,
            "total_primals": status.len(),
            "topology_version": topology,
            "pipelines": {
                "content": content_pipeline,
                "compute": compute_pipeline,
            },
        }))
    }

    /// Handle `composition.health` / `composition.tower_health` etc.
    ///
    /// Follows `COMPOSITION_HEALTH_STANDARD.md`: returns `healthy`, `deploy_graph`,
    /// and `subsystems` map. Aggregates lifecycle state into subsystem health.
    pub async fn composition_health(&self, _params: &Option<Value>) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        let mut subsystems: serde_json::Map<String, Value> = serde_json::Map::new();
        let mut deploy_graph = String::from("unknown");
        let mut all_healthy = true;

        // Collect capability surfaces for all registered primals.
        let mut primal_caps: Vec<(Vec<String>, bool)> = Vec::with_capacity(status.len());
        for (name, state) in &status {
            let info = manager.get_primal_info(name).await;
            let caps: Vec<String> = info
                .as_ref()
                .and_then(|i| i.deployment_node.as_ref())
                .map(|n| n.capabilities.clone())
                .unwrap_or_default();
            let is_active = matches!(state, LifecycleState::Active { .. });
            primal_caps.push((caps, is_active));
        }

        // Capability domains that define each subsystem tier.
        // Primals are matched by their registered capabilities, not by name.
        const TOWER_DOMAINS: &[&str] =
            &["crypto.", "security.", "discovery.", "relay.", "defense."];
        const NODE_DOMAINS: &[&str] = &["compute.", "science.", "inference."];
        const NEST_DOMAINS: &[&str] = &["storage.", "content.", "dag.", "spine.", "braid."];
        const MESH_DOMAINS: &[&str] = &["discovery.", "relay.", "network."];

        let subsystem_status = |domains: &[&str]| -> &'static str {
            let mut found_any = false;
            for (caps, is_active) in &primal_caps {
                let matches_domain = caps
                    .iter()
                    .any(|c| domains.iter().any(|d| c.starts_with(d)));
                if matches_domain {
                    found_any = true;
                    if !is_active {
                        return "degraded";
                    }
                }
            }
            if found_any { "ok" } else { "unavailable" }
        };

        let tower = subsystem_status(TOWER_DOMAINS);
        let node = subsystem_status(NODE_DOMAINS);
        let nest = subsystem_status(NEST_DOMAINS);
        let mesh_process = subsystem_status(MESH_DOMAINS);

        subsystems.insert("tower".to_string(), json!(tower));
        subsystems.insert("node".to_string(), json!(node));
        subsystems.insert("nest".to_string(), json!(nest));

        // Discover mesh provider by capability rather than hardcoded name.
        let mesh_provider_name: Option<String> = {
            let status_names: Vec<&String> = status.keys().collect();
            primal_caps
                .iter()
                .zip(status_names.iter())
                .find(|((caps, is_active), _)| {
                    *is_active
                        && caps
                            .iter()
                            .any(|c| c.starts_with("discovery.") || c.starts_with("relay."))
                })
                .map(|(_, name)| (*name).clone())
        };

        let mesh_detail = if mesh_process == "ok" {
            let probe_target = mesh_provider_name.unwrap_or_default();
            match self.probe_mesh_provider(&manager, &probe_target).await {
                Ok(detail) => detail,
                Err(_) => json!({ "status": "ok", "detail": "process_alive" }),
            }
        } else {
            json!({ "status": mesh_process })
        };
        subsystems.insert("mesh".to_string(), mesh_detail.clone());

        let mesh_healthy = mesh_detail
            .get("status")
            .and_then(|s| s.as_str())
            .is_some_and(|s| s == "ok");
        if tower != "ok" || !mesh_healthy {
            all_healthy = false;
        }

        let active_count = status
            .iter()
            .filter(|(_, s)| matches!(s, LifecycleState::Active { .. }))
            .count();

        if active_count >= 5 {
            deploy_graph = "nucleus_complete".to_string();
        } else if active_count >= 3 {
            deploy_graph = "nucleus_simple".to_string();
        } else if active_count >= 2 {
            deploy_graph = "tower_atomic".to_string();
        }

        Ok(json!({
            "healthy": all_healthy,
            "deploy_graph": deploy_graph,
            "subsystems": subsystems,
            "capabilities_count": status.len(),
        }))
    }

    /// Handle `composition.start` — health-gated composition transitions.
    ///
    /// Resolves a named composition (`tower`, `nest`, `node`, `nucleus`) to its
    /// deployment graph, checks that prerequisite compositions are healthy, and
    /// returns the graph name for the caller (or graph executor) to deploy.
    ///
    /// Prerequisites:
    /// - `tower`: none (foundation)
    /// - `nest`: tower must be healthy
    /// - `node`: tower must be healthy
    /// - `nucleus`: tower AND nest AND node must be healthy
    pub async fn composition_start(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let composition = params["composition"]
            .as_str()
            .or_else(|| params["name"].as_str())
            .context("Missing 'composition' parameter (tower|nest|node|nucleus)")?;

        let (graph, prerequisites) = match composition {
            "tower" => ("tower_atomic_bootstrap", vec![]),
            "nest" => ("nest_deploy", vec!["tower"]),
            "node" => ("node_atomic_compute", vec!["tower"]),
            "nucleus" => ("nucleus_complete", vec!["tower", "nest", "node"]),
            other => {
                return Ok(json!({
                    "error": format!("Unknown composition: {other}"),
                    "available": ["tower", "nest", "node", "nucleus"],
                }));
            }
        };

        // Check prerequisites via composition health
        if !prerequisites.is_empty() {
            let health = self.composition_health(&None).await?;
            let subsystems = health
                .get("subsystems")
                .and_then(|s| s.as_object())
                .cloned()
                .unwrap_or_default();

            let mut blocked_by = Vec::new();
            for prereq in &prerequisites {
                let status = subsystems
                    .get(*prereq)
                    .and_then(|v| v.as_str())
                    .unwrap_or("unavailable");

                if status != "ok" {
                    blocked_by.push(format!("{prereq}={status}"));
                }
            }

            if !blocked_by.is_empty() {
                return Ok(json!({
                    "composition": composition,
                    "graph": graph,
                    "ready": false,
                    "blocked_by": blocked_by,
                    "message": format!(
                        "Cannot start {composition}: prerequisites not met ({})",
                        blocked_by.join(", ")
                    ),
                }));
            }
        }

        Ok(json!({
            "composition": composition,
            "graph": graph,
            "ready": true,
            "boot_order": self.resolve_boot_order_for_composition(composition),
            "message": format!("Composition {composition} ready — deploy graph '{graph}'"),
        }))
    }

    /// Handle `composition.boot_order` — return cellMembrane-authoritative startup ordering.
    ///
    /// Returns the startup sequence that should be used when launching primals
    /// for a given composition. Sources (priority order):
    /// 1. cellMembrane `boot_order` from ecosystem manifest (b7707ee)
    /// 2. Graph `depends_on` topological sort
    /// 3. Static bootstrap launch order (fallback)
    pub async fn composition_boot_order(&self, params: &Option<Value>) -> Result<Value> {
        let composition = params
            .as_ref()
            .and_then(|p| p["composition"].as_str().or_else(|| p["name"].as_str()))
            .unwrap_or("nucleus");

        let order = self.resolve_boot_order_for_composition(composition);

        Ok(json!({
            "composition": composition,
            "boot_order": order,
            "source": if self.has_cellmembrane_boot_order() { "cellMembrane" } else { "static_bootstrap" },
            "strategy": "sequential",
        }))
    }

    /// Resolve the primal startup order for a composition.
    fn resolve_boot_order_for_composition(&self, composition: &str) -> Vec<&'static str> {
        use biomeos_types::primal_names::{
            BARRACUDA, BEARDOG, CORALREEF, LOAMSPINE, NESTGATE, PETALTONGUE, RHIZOCRYPT, SKUNKBAT,
            SONGBIRD, SQUIRREL, SWEETGRASS, TOADSTOOL,
        };

        match composition {
            "tower" => vec![BEARDOG, SONGBIRD, SKUNKBAT],
            "nest" => vec![
                BEARDOG, SONGBIRD, SKUNKBAT, NESTGATE, RHIZOCRYPT, LOAMSPINE, SWEETGRASS, SQUIRREL,
            ],
            "node" => vec![BEARDOG, SONGBIRD, SKUNKBAT, TOADSTOOL, CORALREEF, BARRACUDA],
            "nucleus" => vec![
                BEARDOG,
                SONGBIRD,
                SKUNKBAT,
                TOADSTOOL,
                CORALREEF,
                BARRACUDA,
                NESTGATE,
                RHIZOCRYPT,
                LOAMSPINE,
                SWEETGRASS,
                SQUIRREL,
                PETALTONGUE,
            ],
            _ => vec![],
        }
    }

    /// Whether cellMembrane's authoritative boot_order is available.
    fn has_cellmembrane_boot_order(&self) -> bool {
        std::env::var("BIOMEOS_ECOSYSTEM_MANIFEST_PATH")
            .ok()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|content| content.contains("[boot_order]"))
            .unwrap_or(false)
    }

    /// Handle `composition.self_test` — sandbox validation endpoint.
    ///
    /// Returns proof that the biomeOS Neural API is functional without requiring
    /// a full primal composition to be running. cellMembrane's sandbox calls this
    /// to validate the orchestrator binary works in isolation.
    ///
    /// JSON-RPC method: `composition.self_test`
    pub async fn composition_self_test(&self) -> Result<Value> {
        let manager = self.manager.read().await;
        let status = manager.get_status().await;

        Ok(json!({
            "ok": true,
            "role": "orchestrator",
            "version": env!("CARGO_PKG_VERSION"),
            "routes_loaded": true,
            "capability_registry": "available",
            "primals_registered": status.len(),
            "ipc": "json-rpc",
        }))
    }

    /// Handle `composition.test_swap` — validate a replacement binary in live context.
    ///
    /// The running biomeOS instance spawns the candidate binary on a temporary socket,
    /// validates it via `composition.self_test`, then tears it down. This allows
    /// cellMembrane's Sovereign CI to delegate validation to the live orchestrator
    /// instead of running an isolated sandbox (which fails for broker primals).
    ///
    /// JSON-RPC method: `composition.test_swap`
    ///
    /// Params:
    /// - `binary_path` (required): absolute path to the candidate binary
    /// - `timeout_secs` (optional): validation timeout (default: 15)
    ///
    /// Returns:
    /// - `{ validated: true, version, ... }` on success
    /// - `{ validated: false, reason, ... }` on failure
    pub async fn composition_test_swap(
        &self,
        params: &Option<Value>,
        neural_api_socket: &std::path::Path,
        family_id: &str,
    ) -> Result<Value> {
        let params = params
            .as_ref()
            .context("Missing parameters for composition.test_swap")?;
        let binary_path = params["binary_path"]
            .as_str()
            .context("Missing 'binary_path' parameter")?;

        let timeout_secs = params["timeout_secs"].as_u64().unwrap_or(15);
        let binary = PathBuf::from(binary_path);

        if !binary.exists() {
            return Ok(json!({
                "validated": false,
                "reason": format!("Binary not found: {binary_path}"),
            }));
        }

        info!(
            "🔄 composition.test_swap: validating candidate {}",
            binary_path
        );

        // Spawn candidate on a temporary socket in the membrane directory
        // (avoids PrivateTmp restrictions on NUCLEUS systemd services)
        let test_swap_dir = neural_api_socket
            .parent()
            .unwrap_or_else(|| std::path::Path::new("/tmp"))
            .join("test-swap");
        if let Err(e) = tokio::fs::create_dir_all(&test_swap_dir).await {
            return Ok(json!({
                "validated": false,
                "reason": format!("Cannot create test-swap directory: {e}"),
                "binary_path": binary_path,
            }));
        }
        biomeos_types::platform_substrate::PlatformAccess::SocketDir
            .apply(&test_swap_dir)
            .ok();
        let test_socket = test_swap_dir.join(format!("candidate-{}.sock", std::process::id()));

        // Clean up any leftover from previous run
        if test_socket.exists() {
            tokio::fs::remove_file(&test_socket).await.ok();
        }

        let mut cmd = tokio::process::Command::new(&binary);
        cmd.arg("server")
            .arg("--socket")
            .arg(&test_socket)
            .env("BIOMEOS_TEST_SWAP", "1")
            .env(
                biomeos_types::env_config::vars::NEURAL_API_SOCKET,
                neural_api_socket.to_string_lossy().as_ref(),
            )
            .env(biomeos_types::env_config::vars::FAMILY_ID, family_id)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .stdin(std::process::Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                return Ok(json!({
                    "validated": false,
                    "reason": format!("Failed to spawn candidate: {e}"),
                    "binary_path": binary_path,
                }));
            }
        };

        let candidate_pid = child.id();

        // Wait for socket to appear
        let socket_appeared = tokio::time::timeout(
            Duration::from_secs(timeout_secs),
            Self::wait_for_socket(&test_socket),
        )
        .await
        .is_ok();

        if !socket_appeared {
            child.kill().await.ok();
            child.wait().await.ok();
            tokio::fs::remove_file(&test_socket).await.ok();
            return Ok(json!({
                "validated": false,
                "reason": "Candidate did not create socket within timeout",
                "binary_path": binary_path,
                "timeout_secs": timeout_secs,
            }));
        }

        // Validate via composition.self_test
        let validation_result =
            tokio::time::timeout(Duration::from_secs(5), Self::probe_candidate(&test_socket)).await;

        // Tear down candidate
        child.kill().await.ok();
        child.wait().await.ok();
        tokio::fs::remove_file(&test_socket).await.ok();

        match validation_result {
            Ok(Ok(response)) => {
                let candidate_version = response["version"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                let ok = response["ok"].as_bool().unwrap_or(false);

                if ok {
                    info!(
                        "✅ composition.test_swap: candidate v{} VALIDATED",
                        candidate_version
                    );
                    Ok(json!({
                        "validated": true,
                        "binary_path": binary_path,
                        "candidate_version": candidate_version,
                        "current_version": env!("CARGO_PKG_VERSION"),
                        "candidate_pid": candidate_pid,
                        "self_test": response,
                    }))
                } else {
                    warn!("❌ composition.test_swap: candidate self_test returned ok=false");
                    Ok(json!({
                        "validated": false,
                        "reason": "Candidate self_test returned ok=false",
                        "binary_path": binary_path,
                        "self_test": response,
                    }))
                }
            }
            Ok(Err(e)) => {
                warn!("❌ composition.test_swap: probe failed: {e}");
                Ok(json!({
                    "validated": false,
                    "reason": format!("Candidate probe failed: {e}"),
                    "binary_path": binary_path,
                }))
            }
            Err(_) => {
                warn!("❌ composition.test_swap: probe timed out");
                Ok(json!({
                    "validated": false,
                    "reason": "Candidate probe timed out (5s)",
                    "binary_path": binary_path,
                }))
            }
        }
    }

    /// Wait for a socket file to appear on disk.
    async fn wait_for_socket(path: &std::path::Path) {
        loop {
            if path.exists() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }

    /// Probe a candidate binary's self_test endpoint via plain JSON-RPC.
    async fn probe_candidate(socket_path: &std::path::Path) -> Result<Value> {
        use biomeos_core::atomic_client::AtomicClient;

        let client = AtomicClient::unix(socket_path).with_timeout(Duration::from_secs(5));

        client
            .call("composition.self_test", json!({}))
            .await
            .context("composition.self_test call to candidate failed")
    }
}

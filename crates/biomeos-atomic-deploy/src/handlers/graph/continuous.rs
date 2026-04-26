// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Continuous session lifecycle — start, pause, resume, stop.

use super::{ContinuousSession, GraphHandler};
use anyhow::{Context, Result};
use biomeos_graph::continuous::{ContinuousExecutor, SessionCommand};
use biomeos_graph::events::GraphEventBroadcaster;
use biomeos_graph::graph::DeploymentGraph;
use biomeos_types::SystemPaths;
use serde_json::{Value, json};
use tracing::{debug, info, warn};

impl GraphHandler {
    /// Start a continuous graph execution session.
    ///
    /// JSON-RPC method: `graph.start_continuous`
    ///
    /// Loads the graph from disk as a `DeploymentGraph`, creates a
    /// `ContinuousExecutor`, and runs it in a background task. Returns
    /// a `session_id` that can be used for pause/resume/stop.
    pub async fn start_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let graph_id = params["graph_id"].as_str().context("Missing graph_id")?;

        let graph_path = self.resolve_graph_path(graph_id).with_context(|| {
            format!(
                "Graph '{graph_id}' not found in nucleus ({}) or runtime ({})",
                self.graphs_dir.display(),
                self.runtime_graphs_dir.display()
            )
        })?;

        let toml_str = std::fs::read_to_string(&graph_path)
            .with_context(|| format!("Failed to read: {}", graph_path.display()))?;

        let deployment_graph: DeploymentGraph = toml::from_str(&toml_str)
            .with_context(|| format!("Failed to parse DeploymentGraph: {graph_id}"))?;

        let coordination = &deployment_graph.definition.coordination;
        if *coordination != biomeos_graph::graph::CoordinationPattern::Continuous {
            anyhow::bail!("Graph '{graph_id}' has coordination '{coordination:?}', not Continuous");
        }

        self.load_translations_from_deployment_graph(&deployment_graph)
            .await;

        let session_id = format!("{graph_id}-{}", chrono::Utc::now().timestamp_millis());
        let session_broadcaster = GraphEventBroadcaster::new(16);

        if let Some(ref shared) = self.event_broadcaster {
            let mut rx = session_broadcaster.subscribe();
            let shared = shared.clone();
            let sid = session_id.clone();
            tokio::spawn(async move {
                while let Ok(event) = rx.recv().await {
                    debug!("   [tick relay {}] {:?}", sid, event);
                    let _ = shared.broadcast(event).await;
                }
            });
        }

        let mut executor = ContinuousExecutor::new(deployment_graph, session_broadcaster);

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<SessionCommand>(16);
        let state_rx = executor.state_receiver();
        let session_id_log = session_id.clone();
        let router = self.router.clone();

        tokio::spawn(async move {
            info!("🎮 Starting continuous session: {}", session_id_log);
            executor
                .run(cmd_rx, move |node_id, node, feedback| {
                    let router = router.clone();
                    Box::pin(async move {
                        let capability = match &node.capability {
                            Some(cap) => cap.clone(),
                            None => {
                                debug!("  tick node {} — no capability, passthrough", node_id);
                                return Ok(json!({"node": node_id, "status": "passthrough"}));
                            }
                        };

                        let (domain, operation) = capability
                            .split_once('.')
                            .unwrap_or((&capability, "execute"));

                        let providers = router.get_capability_providers(domain).await;
                        let provider = match providers.and_then(|p| p.into_iter().next()) {
                            Some(p) => p,
                            None => {
                                debug!("  tick node {} — no provider for {}", node_id, domain);
                                return Ok(json!({
                                    "node": node_id,
                                    "status": "skipped",
                                    "reason": format!("no provider for capability '{domain}'")
                                }));
                            }
                        };

                        let rpc_method = format!("{domain}.{operation}");
                        let args = feedback.unwrap_or(json!({}));
                        let rpc_params = json!({
                            "method": rpc_method,
                            "params": args,
                        });

                        match router
                            .forward_request(&provider.endpoint, &rpc_method, &rpc_params)
                            .await
                        {
                            Ok(result) => Ok(json!({
                                "node": node_id,
                                "status": "ok",
                                "primal": provider.primal_name.as_ref(),
                                "result": result,
                            })),
                            Err(e) => {
                                warn!("  tick node {} ({}) failed: {}", node_id, capability, e);
                                if !node.required {
                                    Ok(json!({
                                        "node": node_id,
                                        "status": "degraded",
                                        "error": e.to_string(),
                                    }))
                                } else {
                                    Err(e)
                                }
                            }
                        }
                    })
                })
                .await;
            info!("🛑 Continuous session stopped: {}", session_id_log);
        });

        let started_at = chrono::Utc::now().to_rfc3339();

        self.continuous_sessions.write().await.insert(
            session_id.clone(),
            ContinuousSession {
                graph_id: graph_id.to_string(),
                command_tx: cmd_tx,
                state_rx,
                started_at: started_at.clone(),
            },
        );

        info!(
            "✅ Continuous session started: {} ({})",
            session_id, graph_id
        );

        Ok(json!({
            "session_id": session_id,
            "graph_id": graph_id,
            "started_at": started_at,
        }))
    }

    /// Pause a running continuous session.
    ///
    /// JSON-RPC method: `graph.pause_continuous`
    pub async fn pause_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let session_id = Self::extract_session_id(params)?;
        let sessions = self.continuous_sessions.read().await;
        let session = sessions
            .get(&session_id)
            .with_context(|| format!("Continuous session not found: {session_id}"))?;

        session
            .command_tx
            .send(SessionCommand::Pause)
            .await
            .context("Session command channel closed")?;

        info!("⏸️  Paused continuous session: {}", session_id);
        Ok(json!({"session_id": session_id, "command": "pause"}))
    }

    /// Resume a paused continuous session.
    ///
    /// JSON-RPC method: `graph.resume_continuous`
    pub async fn resume_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let session_id = Self::extract_session_id(params)?;
        let sessions = self.continuous_sessions.read().await;
        let session = sessions
            .get(&session_id)
            .with_context(|| format!("Continuous session not found: {session_id}"))?;

        session
            .command_tx
            .send(SessionCommand::Resume)
            .await
            .context("Session command channel closed")?;

        info!("▶️  Resumed continuous session: {}", session_id);
        Ok(json!({"session_id": session_id, "command": "resume"}))
    }

    /// Stop a continuous session.
    ///
    /// JSON-RPC method: `graph.stop_continuous`
    pub async fn stop_continuous(&self, params: &Option<Value>) -> Result<Value> {
        let session_id = Self::extract_session_id(params)?;

        let session = self
            .continuous_sessions
            .write()
            .await
            .remove(&session_id)
            .with_context(|| format!("Continuous session not found: {session_id}"))?;

        let _ = session.command_tx.send(SessionCommand::Stop).await;

        info!("🛑 Stopped continuous session: {}", session_id);
        Ok(json!({"session_id": session_id, "command": "stop"}))
    }

    /// Register capability translations from a `DeploymentGraph` so that
    /// `capability.call` can route to providers declared in continuous graphs,
    /// matching the parity of `load_translations_from_graph` on the sequential path.
    async fn load_translations_from_deployment_graph(&self, graph: &DeploymentGraph) {
        let runtime_dir = SystemPaths::new()
            .map(|p| p.runtime_dir().to_string_lossy().to_string())
            .unwrap_or_else(|_| {
                std::env::var("BIOMEOS_RUNTIME_DIR")
                    .or_else(|_| std::env::var("TMPDIR"))
                    .unwrap_or_else(|_| "/tmp".to_string())
            });

        let mut registry = self.translation_registry.write().await;

        for node in graph.nodes() {
            if let Some(ref capability) = node.capability {
                let (domain, operation) = capability
                    .split_once('.')
                    .unwrap_or((capability.as_str(), "default"));

                let semantic_name = format!("{domain}.{operation}");
                let method = format!("{domain}.invoke");
                let primal_name = node.name.clone();
                let socket_path =
                    format!("{}/{}-{}.sock", runtime_dir, primal_name, self.family_id);

                debug!(
                    "   Continuous graph: registering {} → {}:{} @ {}",
                    semantic_name, primal_name, method, socket_path
                );

                registry.register_translation(
                    &semantic_name,
                    &primal_name,
                    &method,
                    &socket_path,
                    None,
                );
            }
        }
    }
}

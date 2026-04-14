// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::discovery::discover_ai_socket_path;
use super::types::{AiSuggestion, GraphSnapshot, LearningEvent, SuggestionFeedback};
use crate::events::GraphEvent;
use crate::graph::PrimalGraph;
use anyhow::{Context, Result};
use biomeos_nucleus::client::call_unix_socket_rpc;
use std::collections::HashMap;
use tokio::time::{Duration, timeout};
use tracing::{debug, warn};

impl super::AiGraphAdvisor {
    /// Create a new AI advisor
    pub fn new() -> Self {
        Self {
            squirrel_available: false,
            ai_socket_path: None,
            squirrel_timeout: Duration::from_secs(5),
            local_patterns: super::local::initialize_local_patterns(),
        }
    }

    /// Create advisor with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            squirrel_available: false,
            ai_socket_path: None,
            squirrel_timeout: timeout,
            local_patterns: super::local::initialize_local_patterns(),
        }
    }

    /// Check if the AI provider is available
    pub async fn check_squirrel_availability(&mut self) -> Result<bool> {
        let socket_path = match discover_ai_socket_path().await {
            Ok(p) => p,
            Err(e) => {
                debug!("AI primal discovery failed: {e}");
                self.ai_socket_path = None;
                self.squirrel_available = false;
                return Ok(false);
            }
        };

        self.ai_socket_path = Some(socket_path.clone());

        match call_unix_socket_rpc::<serde_json::Value>(
            socket_path
                .to_str()
                .context("AI socket path is not valid UTF-8")?,
            "health.check",
            serde_json::json!({}),
        )
        .await
        {
            Ok(result) => {
                let healthy = result
                    .get("status")
                    .and_then(|v| v.as_str())
                    .is_some_and(|s| s == "healthy" || s == "ok");
                self.squirrel_available = healthy;
                debug!("AI primal health check: available={}", healthy);
                Ok(healthy)
            }
            Err(e) => {
                debug!("AI primal health check failed: {}", e);
                self.squirrel_available = false;
                Ok(false)
            }
        }
    }

    /// Get AI suggestions for a graph
    pub async fn get_suggestions(&self, graph: &PrimalGraph) -> Result<Vec<AiSuggestion>> {
        if self.squirrel_available {
            self.get_squirrel_suggestions(graph).await
        } else {
            Ok(self.get_local_suggestions(graph))
        }
    }

    async fn get_squirrel_suggestions(&self, graph: &PrimalGraph) -> Result<Vec<AiSuggestion>> {
        let socket_path = self
            .ai_socket_path
            .as_ref()
            .context("AI provider socket not discovered")?;
        let sock_str = socket_path
            .to_str()
            .context("AI socket path is not valid UTF-8")?;
        let graph_snapshot = GraphSnapshot::from_graph(graph);

        let result = timeout(
            self.squirrel_timeout,
            call_unix_socket_rpc::<serde_json::Value>(
                sock_str,
                "ai.analyze_graph",
                serde_json::json!({
                    "graph_id": graph.id.as_str(),
                    "graph_name": graph.name,
                    "snapshot": graph_snapshot,
                    "coordination": format!("{:?}", graph.coordination),
                    "node_count": graph.nodes.len(),
                    "edge_count": graph.edges.len()
                }),
            ),
        )
        .await;

        match result {
            Ok(Ok(response)) => {
                if let Some(suggestions_json) =
                    response.get("suggestions").and_then(|v| v.as_array())
                {
                    let suggestions: Vec<AiSuggestion> = suggestions_json
                        .iter()
                        .filter_map(|v| serde_json::from_value(v.clone()).ok())
                        .collect();

                    if !suggestions.is_empty() {
                        debug!("Received {} suggestions from Squirrel", suggestions.len());
                        return Ok(suggestions);
                    }
                }
                Ok(self.get_local_suggestions(graph))
            }
            Ok(Err(e)) => {
                warn!("Squirrel request failed: {}, using local patterns", e);
                Ok(self.get_local_suggestions(graph))
            }
            Err(_) => {
                warn!("Squirrel request timed out, using local patterns");
                Ok(self.get_local_suggestions(graph))
            }
        }
    }

    /// Send learning event to Squirrel
    pub async fn send_learning_event(&self, event: LearningEvent) -> Result<()> {
        if !self.squirrel_available {
            debug!("Squirrel unavailable, skipping learning event");
            return Ok(());
        }

        let socket_path = self
            .ai_socket_path
            .as_ref()
            .context("AI provider socket not discovered")?;

        match call_unix_socket_rpc::<serde_json::Value>(
            socket_path
                .to_str()
                .context("AI socket path is not valid UTF-8")?,
            "ai.learn_from_event",
            serde_json::json!({
                "event_type": event.event_type,
                "before": event.before,
                "after": event.after,
                "action": event.action,
                "context": event.context
            }),
        )
        .await
        {
            Ok(_) => {
                debug!("Learning event sent to Squirrel: {}", event.event_type);
                Ok(())
            }
            Err(e) => {
                warn!("Failed to send learning event to Squirrel: {}", e);
                Ok(())
            }
        }
    }

    /// Send feedback on a suggestion
    pub async fn send_feedback(&self, feedback: SuggestionFeedback) -> Result<()> {
        if !self.squirrel_available {
            debug!("Squirrel unavailable, skipping feedback");
            return Ok(());
        }

        let socket_path = self
            .ai_socket_path
            .as_ref()
            .context("AI provider socket not discovered")?;

        match call_unix_socket_rpc::<serde_json::Value>(
            socket_path
                .to_str()
                .context("AI socket path is not valid UTF-8")?,
            "ai.record_feedback",
            serde_json::json!({
                "suggestion_id": feedback.suggestion_id,
                "accepted": feedback.accepted,
                "comments": feedback.comments,
                "outcome": feedback.outcome
            }),
        )
        .await
        {
            Ok(_) => {
                debug!(
                    "Feedback sent to Squirrel for suggestion: {}",
                    feedback.suggestion_id
                );
                Ok(())
            }
            Err(e) => {
                warn!("Failed to send feedback to Squirrel: {}", e);
                Ok(())
            }
        }
    }

    /// Learn from graph events by forwarding to Squirrel when available.
    /// Events are logged at debug level regardless for observability.
    pub async fn learn_from_event(&self, event: &GraphEvent) -> Result<()> {
        let (event_type, context) = match event {
            GraphEvent::NodeFailed { node_id, error, .. } => (
                "node_failed",
                HashMap::from([
                    ("node_id".to_string(), node_id.clone()),
                    ("error".to_string(), error.clone()),
                ]),
            ),
            GraphEvent::DecisionMade { reasoning, .. } => (
                "decision_made",
                HashMap::from([("reasoning".to_string(), reasoning.join("; "))]),
            ),
            _ => return Ok(()),
        };

        debug!(
            event_type,
            context = ?context,
            squirrel_available = self.squirrel_available,
            "AI advisor: learned from graph event"
        );

        if let Some(ref socket) = self.ai_socket_path {
            if self.squirrel_available {
                let params = serde_json::json!({
                    "event_type": event_type,
                    "context": context,
                });
                if let Err(e) = timeout(
                    self.squirrel_timeout,
                    call_unix_socket_rpc::<serde_json::Value>(socket, "ai.learn_event", params),
                )
                .await
                {
                    debug!("AI advisor: learn_event RPC did not complete: {e}");
                }
            }
        }

        Ok(())
    }
}

impl Default for super::AiGraphAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

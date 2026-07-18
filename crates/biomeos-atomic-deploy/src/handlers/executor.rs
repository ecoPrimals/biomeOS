// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Executor introspection handlers for the Neural API.
//!
//! Exposes available graph executor types and their runtime state:
//! - `executor.list` — summary of continuous, pipeline, and single-shot executors
//! - `executor.status` — detailed metrics per executor type

use anyhow::{Result, bail};
use biomeos_graph::continuous::SessionState;
use serde_json::{Value, json};
use std::time::Instant;

use super::graph::GraphHandler;

const EXECUTOR_CONTINUOUS: &str = "continuous";
const EXECUTOR_PIPELINE: &str = "pipeline";
const EXECUTOR_SINGLE_SHOT: &str = "single-shot";

/// Introspection handler backed by graph execution state.
pub struct ExecutorHandler<'a> {
    graph: &'a GraphHandler,
    started_at: Instant,
}

impl<'a> ExecutorHandler<'a> {
    /// Create a handler view over the shared graph execution trackers.
    #[must_use]
    pub fn new(graph: &'a GraphHandler, started_at: Instant) -> Self {
        Self { graph, started_at }
    }

    /// Handle `executor.list` — returns available executor types and their state.
    pub async fn list(&self) -> Result<Value> {
        let continuous = self.continuous_summary().await;
        let single_shot = self.single_shot_summary().await;
        let pipeline = self.pipeline_summary();

        let executors = vec![
            executor_list_entry(
                EXECUTOR_CONTINUOUS,
                continuous.active_sessions,
                continuous.status,
            ),
            executor_list_entry(
                EXECUTOR_SINGLE_SHOT,
                single_shot.active_sessions,
                single_shot.status,
            ),
            executor_list_entry(
                EXECUTOR_PIPELINE,
                pipeline.active_sessions,
                pipeline.status,
            ),
        ];

        Ok(json!({
            "executors": executors,
            "count": executors.len(),
        }))
    }

    /// Handle `executor.status` — returns detailed executor metrics.
    ///
    /// When `params.executor_type` is set, returns details for that executor only.
    /// Otherwise returns metrics for all executor types.
    pub async fn status(&self, params: &Option<Value>) -> Result<Value> {
        let filter = params
            .as_ref()
            .and_then(|p| p.get("executor_type"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty());

        if let Some(executor_type) = filter {
            let normalized = normalize_executor_type(executor_type)?;
            let detail = if normalized == EXECUTOR_CONTINUOUS {
                self.continuous_detail().await
            } else if normalized == EXECUTOR_SINGLE_SHOT {
                self.single_shot_detail().await
            } else {
                self.pipeline_detail()
            };
            return Ok(json!({
                "executor_type": executor_type,
                "executor": detail,
            }));
        }

        Ok(json!({
            "executors": {
                EXECUTOR_CONTINUOUS: self.continuous_detail().await,
                EXECUTOR_SINGLE_SHOT: self.single_shot_detail().await,
                EXECUTOR_PIPELINE: self.pipeline_detail(),
            },
            "uptime_s": self.uptime_secs(),
        }))
    }

    fn uptime_secs(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    async fn continuous_summary(&self) -> ExecutorSummary {
        let sessions = self.graph.continuous_sessions().read().await;
        let active_sessions = sessions
            .values()
            .filter(|session| {
                let state = *session.state_rx.borrow();
                !matches!(state, SessionState::Stopped | SessionState::Stopping)
            })
            .count();
        ExecutorSummary {
            active_sessions,
            status: if active_sessions > 0 {
                "active"
            } else {
                "idle"
            },
        }
    }

    async fn single_shot_summary(&self) -> ExecutorSummary {
        let executions = self.graph.executions().read().await;
        let active_sessions = executions
            .values()
            .filter(|exec| exec.state == "running")
            .count();
        ExecutorSummary {
            active_sessions,
            status: if active_sessions > 0 {
                "active"
            } else {
                "idle"
            },
        }
    }

    fn pipeline_summary(&self) -> ExecutorSummary {
        ExecutorSummary {
            active_sessions: 0,
            status: "idle",
        }
    }

    async fn continuous_detail(&self) -> Value {
        let sessions = self.graph.continuous_sessions().read().await;
        let mut active = 0_u64;
        let mut pending = 0_u64;
        let mut completed = 0_u64;
        let failed = 0_u64;
        let mut session_list = Vec::with_capacity(sessions.len());

        for (session_id, session) in sessions.iter() {
            let state = *session.state_rx.borrow();
            match state {
                SessionState::Starting => pending += 1,
                SessionState::Running | SessionState::Paused => active += 1,
                SessionState::Stopping => pending += 1,
                SessionState::Stopped => completed += 1,
            }

            session_list.push(json!({
                "session_id": session_id,
                "graph_id": session.graph_id,
                "state": state.to_string(),
                "started_at": session.started_at,
            }));
        }

        json!({
            "name": EXECUTOR_CONTINUOUS,
            "type": EXECUTOR_CONTINUOUS,
            "active": active,
            "pending": pending,
            "completed": completed,
            "failed": failed,
            "active_sessions": active,
            "status": if active > 0 { "active" } else { "idle" },
            "uptime_s": self.uptime_secs(),
            "sessions": session_list,
        })
    }

    async fn single_shot_detail(&self) -> Value {
        let executions = self.graph.executions().read().await;
        let mut active = 0_u64;
        let mut pending = 0_u64;
        let mut completed = 0_u64;
        let mut failed = 0_u64;

        for exec in executions.values() {
            match exec.state.as_str() {
                "running" => {
                    active += 1;
                    pending += 1;
                }
                "completed" => completed += 1,
                "failed" => failed += 1,
                other => {
                    tracing::debug!(state = other, "unknown single-shot execution state");
                    pending += 1;
                }
            }
        }

        json!({
            "name": EXECUTOR_SINGLE_SHOT,
            "type": EXECUTOR_SINGLE_SHOT,
            "active": active,
            "pending": pending,
            "completed": completed,
            "failed": failed,
            "active_sessions": active,
            "status": if active > 0 { "active" } else { "idle" },
            "uptime_s": self.uptime_secs(),
            "tracked_executions": executions.len(),
        })
    }

    fn pipeline_detail(&self) -> Value {
        json!({
            "name": EXECUTOR_PIPELINE,
            "type": EXECUTOR_PIPELINE,
            "active": 0,
            "pending": 0,
            "completed": 0,
            "failed": 0,
            "active_sessions": 0,
            "status": "idle",
            "uptime_s": self.uptime_secs(),
            "note": "pipeline executions are synchronous and not tracked in execution state",
        })
    }
}

struct ExecutorSummary {
    active_sessions: usize,
    status: &'static str,
}

fn executor_list_entry(name: &str, active_sessions: usize, status: &str) -> Value {
    json!({
        "name": name,
        "type": name,
        "active_sessions": active_sessions,
        "status": status,
    })
}

fn normalize_executor_type(executor_type: &str) -> Result<&'static str> {
    match executor_type {
        EXECUTOR_CONTINUOUS | "continuous_executor" => Ok(EXECUTOR_CONTINUOUS),
        EXECUTOR_PIPELINE | "pipeline_executor" => Ok(EXECUTOR_PIPELINE),
        EXECUTOR_SINGLE_SHOT | "single_shot" | "singleshot" => Ok(EXECUTOR_SINGLE_SHOT),
        other => bail!("Unknown executor_type: {other}"),
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Continuous coordination executor for fixed-timestep graph loops.
//!
//! Enables real-time niches (game engines, dashboards, audio pipelines, surgical VR)
//! by running a graph at a fixed tick rate with:
//! - **TickClock**: Fixed-timestep accumulator with frame-skip protection
//! - **SessionState**: Start / Running / Paused / Stopping / Stopped lifecycle
//! - **Feedback edges**: Node outputs feed back as next-tick inputs
//! - **Budget enforcement**: Per-node time budgets with graceful fallback

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::sync::{watch, RwLock};
use tracing::{debug, info, warn};

use crate::events::{GraphEvent, GraphEventBroadcaster};
use crate::graph::{DeploymentGraph, TickConfig};
use crate::node::GraphNode;

// ---------------------------------------------------------------------------
// TickClock
// ---------------------------------------------------------------------------

/// Fixed-timestep clock with accumulator and frame-skip protection.
///
/// Classic game-loop pattern: accumulate real elapsed time, consume it in
/// fixed-size steps. If real time runs ahead (e.g. the system stuttered),
/// `max_accumulator` caps how many steps we catch up.
#[derive(Debug, Clone)]
pub struct TickClock {
    /// Duration of one tick at the target rate
    tick_duration: Duration,
    /// Maximum accumulated time before we clamp (prevents spiral-of-death)
    max_accumulator: Duration,
    /// Accumulated unprocessed time
    accumulator: Duration,
    /// Wall-clock time of the last `advance()` call
    last_advance: Instant,
    /// Monotonically increasing tick counter
    tick_count: u64,
}

impl TickClock {
    /// Create a clock from a [`TickConfig`].
    pub fn from_config(config: &TickConfig) -> Self {
        let tick_duration = Duration::from_secs_f64(1.0 / config.target_hz);
        let max_accumulator = Duration::from_secs_f64(config.max_accumulator_ms / 1000.0);
        Self {
            tick_duration,
            max_accumulator,
            accumulator: Duration::ZERO,
            last_advance: Instant::now(),
            tick_count: 0,
        }
    }

    /// Create a clock for a given target Hz.
    pub fn new(target_hz: f64) -> Self {
        Self::from_config(&TickConfig {
            target_hz,
            ..TickConfig::default()
        })
    }

    /// Advance the clock by real elapsed time. Returns how many ticks should run.
    pub fn advance(&mut self) -> u32 {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_advance);
        self.last_advance = now;

        self.accumulator += elapsed;
        if self.accumulator > self.max_accumulator {
            let skipped = (self.accumulator - self.max_accumulator).as_secs_f64()
                / self.tick_duration.as_secs_f64();
            if skipped > 1.0 {
                warn!(
                    "TickClock: clamping accumulator, skipping ~{:.0} ticks",
                    skipped
                );
            }
            self.accumulator = self.max_accumulator;
        }

        let mut ticks = 0u32;
        while self.accumulator >= self.tick_duration {
            self.accumulator -= self.tick_duration;
            self.tick_count += 1;
            ticks += 1;
        }

        ticks
    }

    /// Current tick count.
    pub fn tick_count(&self) -> u64 {
        self.tick_count
    }

    /// Duration of one tick.
    pub fn tick_duration(&self) -> Duration {
        self.tick_duration
    }

    /// Target Hz this clock was configured for.
    pub fn target_hz(&self) -> f64 {
        1.0 / self.tick_duration.as_secs_f64()
    }

    /// Reset the clock (e.g. after a pause).
    pub fn reset_accumulator(&mut self) {
        self.accumulator = Duration::ZERO;
        self.last_advance = Instant::now();
    }
}

// ---------------------------------------------------------------------------
// SessionState
// ---------------------------------------------------------------------------

/// Lifecycle state of a continuous session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionState {
    /// Session created, not yet running
    Starting,
    /// Actively ticking
    Running,
    /// Temporarily halted; tick clock paused
    Paused,
    /// Graceful shutdown in progress
    Stopping,
    /// Fully stopped
    Stopped,
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Starting => write!(f, "starting"),
            SessionState::Running => write!(f, "running"),
            SessionState::Paused => write!(f, "paused"),
            SessionState::Stopping => write!(f, "stopping"),
            SessionState::Stopped => write!(f, "stopped"),
        }
    }
}

// ---------------------------------------------------------------------------
// NodeOutput cache
// ---------------------------------------------------------------------------

/// Cached output from a node, used for feedback edges and budget-overrun fallback.
#[derive(Debug, Clone)]
struct CachedOutput {
    value: serde_json::Value,
    /// Tick counter for diagnostics and cache-staleness checks.
    _tick: u64,
}

// ---------------------------------------------------------------------------
// ContinuousExecutor
// ---------------------------------------------------------------------------

/// Executes a deployment graph in a continuous fixed-timestep loop.
///
/// Each tick:
/// 1. Collect feedback inputs from previous tick
/// 2. Execute nodes in dependency order with per-node budget
/// 3. Cache outputs for feedback edges
/// 4. Broadcast tick event
///
/// The executor is controlled via [`SessionCommand`]s sent through a channel.
pub struct ContinuousExecutor {
    graph: DeploymentGraph,
    tick_config: Arc<TickConfig>,
    broadcaster: GraphEventBroadcaster,
    state_tx: watch::Sender<SessionState>,
    state_rx: watch::Receiver<SessionState>,
    output_cache: Arc<RwLock<HashMap<String, CachedOutput>>>,
    feedback_map: HashMap<String, String>,
    node_order: Vec<String>,
}

/// Commands that can be sent to control a running continuous session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    /// Pause the tick loop
    Pause,
    /// Resume from pause
    Resume,
    /// Gracefully stop the session
    Stop,
}

impl ContinuousExecutor {
    /// Create a new continuous executor for the given graph.
    ///
    /// The graph's `tick` config is used for clock parameters.
    /// Falls back to 60 Hz defaults if no tick config is present.
    pub fn new(graph: DeploymentGraph, broadcaster: GraphEventBroadcaster) -> Self {
        let tick_config = Arc::new(graph.definition.tick.clone().unwrap_or_default());

        let (state_tx, state_rx) = watch::channel(SessionState::Starting);

        let feedback_map: HashMap<String, String> = graph
            .nodes()
            .iter()
            .filter_map(|n| {
                n.feedback_to
                    .as_ref()
                    .map(|target| (n.id.as_str().to_string(), target.clone()))
            })
            .collect();

        let node_order: Vec<String> = graph
            .nodes_in_order()
            .iter()
            .map(|n| n.id.as_str().to_string())
            .collect();

        Self {
            graph,
            tick_config,
            broadcaster,
            state_tx,
            state_rx,
            output_cache: Arc::new(RwLock::new(HashMap::new())),
            feedback_map,
            node_order,
        }
    }

    /// Subscribe to session state changes.
    pub fn state_receiver(&self) -> watch::Receiver<SessionState> {
        self.state_rx.clone()
    }

    /// Current session state.
    pub fn state(&self) -> SessionState {
        *self.state_rx.borrow()
    }

    /// Run the continuous loop until stopped.
    ///
    /// `cmd_rx` receives [`SessionCommand`]s for pause/resume/stop.
    /// `node_executor` is called for each node on each tick to do actual work.
    pub async fn run<F, Fut>(
        &mut self,
        mut cmd_rx: tokio::sync::mpsc::Receiver<SessionCommand>,
        node_executor: F,
    ) where
        F: Fn(String, GraphNode, Option<serde_json::Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = anyhow::Result<serde_json::Value>> + Send,
    {
        let graph_id: Arc<str> = Arc::from(self.graph.id().as_str());
        info!(
            "Continuous session starting: {} @ {} Hz",
            graph_id.as_ref(),
            self.tick_config.target_hz
        );

        let _ = self.state_tx.send(SessionState::Running);
        let _ = self
            .broadcaster
            .broadcast(GraphEvent::SessionStarted {
                graph_id: graph_id.as_ref().to_string(),
                target_hz: self.tick_config.target_hz,
                timestamp: Utc::now(),
            })
            .await;

        let mut clock = TickClock::from_config(self.tick_config.as_ref());
        let budget_warning = Duration::from_secs_f64(self.tick_config.budget_warning_ms / 1000.0);

        loop {
            // Check for commands (non-blocking)
            match cmd_rx.try_recv() {
                Ok(SessionCommand::Stop) => {
                    info!("Session stop requested: {}", graph_id.as_ref());
                    let _ = self.state_tx.send(SessionState::Stopping);
                    self.broadcast_state_change(graph_id.as_ref(), "stopping", clock.tick_count())
                        .await;
                    break;
                }
                Ok(SessionCommand::Pause) => {
                    if *self.state_rx.borrow() == SessionState::Running {
                        let _ = self.state_tx.send(SessionState::Paused);
                        self.broadcast_state_change(
                            graph_id.as_ref(),
                            "paused",
                            clock.tick_count(),
                        )
                        .await;
                        info!("Session paused: {}", graph_id.as_ref());
                    }
                }
                Ok(SessionCommand::Resume) => {
                    if *self.state_rx.borrow() == SessionState::Paused {
                        clock.reset_accumulator();
                        let _ = self.state_tx.send(SessionState::Running);
                        self.broadcast_state_change(
                            graph_id.as_ref(),
                            "running",
                            clock.tick_count(),
                        )
                        .await;
                        info!("Session resumed: {}", graph_id.as_ref());
                    }
                }
                Err(_) => {}
            }

            if *self.state_rx.borrow() == SessionState::Paused {
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }

            let ticks_to_run = clock.advance();

            for _ in 0..ticks_to_run {
                let tick_start = Instant::now();
                let tick_num = clock.tick_count();
                let mut budget_overruns = 0usize;

                for node_id in &self.node_order {
                    let node = match self.find_node(node_id) {
                        Some(n) => n.clone(),
                        None => continue,
                    };

                    // ConditionalDag support: skip nodes whose condition is not met.
                    // In continuous graphs this enables optional primals (e.g., AI narration
                    // only when Squirrel is available) by evaluating conditions per tick.
                    let env = &self.graph.definition.env;
                    if node.should_skip(env) || !node.condition_met(env) {
                        debug!("   ⏭️  Skipping node (condition): {}", node_id);
                        continue;
                    }

                    let feedback_input = self.get_feedback_input(node_id).await;

                    let node_budget = node
                        .budget_ms
                        .map(|ms| Duration::from_secs_f64(ms / 1000.0))
                        .unwrap_or(budget_warning);

                    let is_optional = node.is_optional();
                    let node_start = Instant::now();
                    let result = tokio::time::timeout(
                        node_budget * 2,
                        node_executor(node_id.clone(), node, feedback_input),
                    )
                    .await;

                    let node_elapsed = node_start.elapsed();

                    match result {
                        Ok(Ok(output)) => {
                            if node_elapsed > node_budget {
                                budget_overruns += 1;
                                debug!(
                                    "Node {} exceeded budget: {:.2}ms > {:.2}ms",
                                    node_id,
                                    node_elapsed.as_secs_f64() * 1000.0,
                                    node_budget.as_secs_f64() * 1000.0
                                );
                            }
                            let mut cache = self.output_cache.write().await;
                            cache.insert(
                                node_id.clone(),
                                CachedOutput {
                                    value: output,
                                    _tick: tick_num,
                                },
                            );
                        }
                        Ok(Err(e)) => {
                            if is_optional {
                                debug!(
                                    "Optional node {} skipped on tick {}: {}",
                                    node_id, tick_num, e
                                );
                            } else {
                                warn!("Node {} error on tick {}: {}", node_id, tick_num, e);
                            }
                        }
                        Err(_) => {
                            if is_optional {
                                debug!(
                                    "Optional node {} timed out on tick {} — skipped",
                                    node_id, tick_num
                                );
                            } else {
                                budget_overruns += 1;
                                warn!(
                                    "Node {} timed out on tick {} — reusing cached output",
                                    node_id, tick_num
                                );
                            }
                        }
                    }
                }

                let tick_duration_us = tick_start.elapsed().as_micros() as u64;
                let _ = self
                    .broadcaster
                    .broadcast(GraphEvent::TickCompleted {
                        graph_id: graph_id.as_ref().to_string(),
                        tick: tick_num,
                        duration_us: tick_duration_us,
                        budget_overruns,
                        timestamp: Utc::now(),
                    })
                    .await;
            }

            if ticks_to_run == 0 {
                tokio::time::sleep(Duration::from_micros(500)).await;
            }
        }

        let _ = self.state_tx.send(SessionState::Stopped);
        self.broadcast_state_change(graph_id.as_ref(), "stopped", clock.tick_count())
            .await;
        info!(
            "Continuous session stopped: {} (total ticks: {})",
            graph_id.as_ref(),
            clock.tick_count()
        );
    }

    fn find_node(&self, node_id: &str) -> Option<&GraphNode> {
        self.graph.nodes().iter().find(|n| n.id.as_str() == node_id)
    }

    async fn get_feedback_input(&self, node_id: &str) -> Option<serde_json::Value> {
        let cache = self.output_cache.read().await;
        for (source, target) in &self.feedback_map {
            #[allow(clippy::collapsible_if)]
            if target == node_id {
                if let Some(cached) = cache.get(source) {
                    return Some(cached.value.clone());
                }
            }
        }
        None
    }

    async fn broadcast_state_change(&self, graph_id: &str, new_state: &str, tick: u64) {
        let _ = self
            .broadcaster
            .broadcast(GraphEvent::SessionStateChanged {
                graph_id: graph_id.to_string(),
                new_state: new_state.to_string(),
                tick_at_change: tick,
                timestamp: Utc::now(),
            })
            .await;
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_tick_clock_basic() {
        let clock = TickClock::new(60.0);
        assert_eq!(clock.tick_count(), 0);
        assert!((clock.target_hz() - 60.0).abs() < 0.01);

        let tick_dur = clock.tick_duration();
        assert!((tick_dur.as_secs_f64() - 1.0 / 60.0).abs() < 0.0001);
    }

    #[test]
    fn test_tick_clock_advance_zero_elapsed() {
        let mut clock = TickClock::new(60.0);
        let ticks = clock.advance();
        assert_eq!(ticks, 0);
    }

    #[test]
    fn test_tick_clock_advance_after_sleep() {
        // Intentional: testing clock advancement with real elapsed time
        let mut clock = TickClock::new(10.0); // 10 Hz = 100ms per tick
        std::thread::sleep(Duration::from_millis(10));
        let _ticks = clock.advance();
        assert_eq!(clock.tick_count(), 0, "10ms is not enough for a 100ms tick");

        std::thread::sleep(Duration::from_millis(250));
        let ticks = clock.advance();
        assert!(
            ticks >= 1,
            "250ms should produce at least one 100ms tick, got {ticks}"
        );
        assert!(clock.tick_count() >= 1);
    }

    #[test]
    fn test_tick_clock_max_accumulator_clamp() {
        // Intentional: testing accumulator clamping with real elapsed time
        let config = TickConfig {
            target_hz: 10.0,
            max_accumulator_ms: 200.0,
            budget_warning_ms: 4.0,
        };
        let mut clock = TickClock::from_config(&config);
        std::thread::sleep(Duration::from_millis(500));
        let ticks = clock.advance();
        assert!(
            ticks <= 2,
            "Should clamp to max_accumulator worth of ticks, got {ticks}"
        );
    }

    #[test]
    fn test_tick_clock_reset_accumulator() {
        // Intentional: testing reset with real elapsed time
        let mut clock = TickClock::new(60.0);
        std::thread::sleep(Duration::from_millis(50));
        clock.reset_accumulator();
        let ticks = clock.advance();
        assert_eq!(ticks, 0);
    }

    #[test]
    fn test_session_state_display() {
        assert_eq!(SessionState::Starting.to_string(), "starting");
        assert_eq!(SessionState::Running.to_string(), "running");
        assert_eq!(SessionState::Paused.to_string(), "paused");
        assert_eq!(SessionState::Stopping.to_string(), "stopping");
        assert_eq!(SessionState::Stopped.to_string(), "stopped");
    }

    #[test]
    fn test_session_state_serde() {
        let state = SessionState::Running;
        let json = serde_json::to_string(&state).unwrap();
        assert_eq!(json, "\"running\"");
        let deserialized: SessionState = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, SessionState::Running);
    }

    #[test]
    fn test_session_state_all_variants_serde() {
        let states = vec![
            SessionState::Starting,
            SessionState::Running,
            SessionState::Paused,
            SessionState::Stopping,
            SessionState::Stopped,
        ];
        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let rt: SessionState = serde_json::from_str(&json).unwrap();
            assert_eq!(rt, state);
        }
    }

    #[test]
    fn test_tick_config_default() {
        let config = TickConfig::default();
        assert!((config.target_hz - 60.0).abs() < 0.01);
        assert!((config.max_accumulator_ms - 100.0).abs() < 0.01);
        assert!((config.budget_warning_ms - 4.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_continuous_executor_creation() {
        let toml_str = r#"
            [graph]
            id = "test-continuous"
            name = "Test Continuous"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 30.0
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        let broadcaster = GraphEventBroadcaster::new(100);
        let executor = ContinuousExecutor::new(graph, broadcaster);
        assert_eq!(executor.state(), SessionState::Starting);
    }

    #[tokio::test]
    async fn test_continuous_executor_stop() {
        let toml_str = r#"
            [graph]
            id = "stop-test"
            name = "Stop Test"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 100.0

            [[graph.nodes]]
            id = "tick-node"
            name = "Tick Node"
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        let broadcaster = GraphEventBroadcaster::new(100);
        let mut executor = ContinuousExecutor::new(graph, broadcaster);

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(10);
        let mut state_rx = executor.state_receiver();

        let handle = tokio::spawn(async move {
            executor
                .run(cmd_rx, |_node_id, _node, _feedback| async {
                    Ok(serde_json::json!({"ok": true}))
                })
                .await;
        });

        // Wait for session to start
        while *state_rx.borrow() != SessionState::Running {
            state_rx.changed().await.unwrap();
        }

        cmd_tx.send(SessionCommand::Stop).await.unwrap();

        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_continuous_executor_pause_resume() {
        let toml_str = r#"
            [graph]
            id = "pause-test"
            name = "Pause Test"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 100.0

            [[graph.nodes]]
            id = "node-a"
            name = "Node A"
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        let broadcaster = GraphEventBroadcaster::new(100);
        let mut executor = ContinuousExecutor::new(graph, broadcaster);

        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(10);
        let mut state_rx = executor.state_receiver();

        let handle = tokio::spawn(async move {
            executor
                .run(cmd_rx, |_node_id, _node, _feedback| async {
                    Ok(serde_json::json!({"ok": true}))
                })
                .await;
        });

        while *state_rx.borrow() != SessionState::Running {
            state_rx.changed().await.unwrap();
        }

        cmd_tx.send(SessionCommand::Pause).await.unwrap();
        while *state_rx.borrow() != SessionState::Paused {
            state_rx.changed().await.unwrap();
        }

        cmd_tx.send(SessionCommand::Resume).await.unwrap();
        while *state_rx.borrow() != SessionState::Running {
            state_rx.changed().await.unwrap();
        }

        cmd_tx.send(SessionCommand::Stop).await.unwrap();
        handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_feedback_edge_wiring() {
        let toml_str = r#"
            [graph]
            id = "feedback-test"
            name = "Feedback Test"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 100.0

            [[graph.nodes]]
            id = "producer"
            name = "Producer"
            feedback_to = "consumer"

            [[graph.nodes]]
            id = "consumer"
            name = "Consumer"
            depends_on = ["producer"]
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        let broadcaster = GraphEventBroadcaster::new(100);
        let executor = ContinuousExecutor::new(graph, broadcaster);

        assert_eq!(executor.feedback_map.len(), 1);
        assert_eq!(
            executor.feedback_map.get("producer"),
            Some(&"consumer".to_string())
        );
    }

    #[test]
    fn test_coordination_pattern_serde() {
        use crate::graph::CoordinationPattern;

        let patterns = vec![
            (CoordinationPattern::Sequential, "\"sequential\""),
            (CoordinationPattern::Parallel, "\"parallel\""),
            (CoordinationPattern::ConditionalDag, "\"conditionaldag\""),
            (CoordinationPattern::Pipeline, "\"pipeline\""),
            (CoordinationPattern::Continuous, "\"continuous\""),
        ];
        for (pat, expected) in patterns {
            let json = serde_json::to_string(&pat).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_graph_with_tick_config_parsing() {
        let toml_str = r#"
            [graph]
            id = "tick-parse"
            name = "Tick Parse"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 120.0
            max_accumulator_ms = 200.0
            budget_warning_ms = 8.0
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        assert_eq!(
            graph.definition.coordination,
            crate::graph::CoordinationPattern::Continuous
        );
        let tick = graph.definition.tick.as_ref().unwrap();
        assert!((tick.target_hz - 120.0).abs() < 0.01);
        assert!((tick.max_accumulator_ms - 200.0).abs() < 0.01);
        assert!((tick.budget_warning_ms - 8.0).abs() < 0.01);
    }

    #[test]
    fn test_node_budget_ms_parsing() {
        let toml_str = r#"
            [graph]
            id = "budget-test"
            name = "Budget"
            version = "1.0.0"

            [[graph.nodes]]
            id = "fast-node"
            name = "Fast"
            budget_ms = 2.0

            [[graph.nodes]]
            id = "slow-node"
            name = "Slow"
            budget_ms = 8.0
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        let nodes = graph.nodes();
        assert_eq!(nodes[0].budget_ms, Some(2.0));
        assert_eq!(nodes[1].budget_ms, Some(8.0));
    }

    #[test]
    fn test_node_feedback_to_parsing() {
        let toml_str = r#"
            [graph]
            id = "fb-parse"
            name = "Feedback"
            version = "1.0.0"

            [[graph.nodes]]
            id = "physics"
            name = "Physics"
            feedback_to = "game-logic"

            [[graph.nodes]]
            id = "game-logic"
            name = "Logic"
        "#;
        let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
        assert_eq!(graph.nodes()[0].feedback_to.as_deref(), Some("game-logic"));
        assert_eq!(graph.nodes()[1].feedback_to, None);
    }
}

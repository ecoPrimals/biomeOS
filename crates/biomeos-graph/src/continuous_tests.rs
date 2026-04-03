// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::time::Duration;

use crate::GraphEventBroadcaster;
use crate::continuous::*;
use crate::graph::{DeploymentGraph, TickConfig};

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

#[tokio::test(start_paused = true)]
async fn test_tick_clock_advance_after_sleep() {
    let mut clock = TickClock::new(10.0); // 10 Hz = 100ms per tick
    tokio::time::advance(Duration::from_millis(10)).await;
    let _ticks = clock.advance();
    assert_eq!(clock.tick_count(), 0, "10ms is not enough for a 100ms tick");

    tokio::time::advance(Duration::from_millis(250)).await;
    let ticks = clock.advance();
    assert!(
        ticks >= 1,
        "250ms should produce at least one 100ms tick, got {ticks}"
    );
    assert!(clock.tick_count() >= 1);
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_max_accumulator_clamp() {
    let config = TickConfig {
        target_hz: 10.0,
        max_accumulator_ms: 200.0,
        budget_warning_ms: 4.0,
    };
    let mut clock = TickClock::from_config(&config);
    tokio::time::advance(Duration::from_millis(500)).await;
    let ticks = clock.advance();
    assert!(
        ticks <= 2,
        "Should clamp to max_accumulator worth of ticks, got {ticks}"
    );
}

/// When real time far exceeds `max_accumulator`, `skipped > 1.0` and the clock logs a clamp warning.
#[tokio::test(start_paused = true)]
async fn test_tick_clock_clamp_logs_when_skipping_multiple_ticks() {
    let config = TickConfig {
        target_hz: 10.0,
        max_accumulator_ms: 200.0,
        budget_warning_ms: 4.0,
    };
    let mut clock = TickClock::from_config(&config);
    tokio::time::advance(Duration::from_millis(800)).await;
    let ticks = clock.advance();
    assert!(
        (1..=2).contains(&ticks),
        "clamped accumulator should yield 1–2 ticks at 10 Hz, got {ticks}"
    );
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_reset_accumulator() {
    let mut clock = TickClock::new(60.0);
    tokio::time::advance(Duration::from_millis(50)).await;
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

    assert_eq!(executor.feedback_map_for_test().len(), 1);
    assert_eq!(
        executor.feedback_map_for_test().get("producer"),
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

#[tokio::test(start_paused = true)]
async fn test_tick_clock_yields_multiple_ticks_one_advance() {
    let config = TickConfig {
        target_hz: 10.0,
        max_accumulator_ms: 500.0,
        budget_warning_ms: 4.0,
    };
    let mut clock = TickClock::from_config(&config);
    tokio::time::advance(Duration::from_millis(350)).await;
    let ticks = clock.advance();
    assert!(
        ticks >= 2,
        "350ms at 10 Hz (100ms/tick) should yield multiple ticks, got {ticks}"
    );
    assert!(clock.tick_count() >= 2);
}

#[tokio::test(start_paused = true)]
async fn test_tick_clock_target_hz_matches_tick_duration() {
    let c = TickClock::new(50.0);
    assert!((c.target_hz() - 50.0).abs() < 0.001);
    let dur = c.tick_duration();
    assert!((dur.as_secs_f64() - 0.02).abs() < 1e-6);
}

#[tokio::test(start_paused = true)]
async fn test_continuous_executor_optional_node_error_skipped() {
    let toml_str = r#"
            [graph]
            id = "opt-err"
            name = "Optional Err"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 200.0

            [[graph.nodes]]
            id = "fragile"
            name = "Fragile"
            fallback = "skip"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    let broadcaster = GraphEventBroadcaster::new(100);
    let mut executor = ContinuousExecutor::new(graph, broadcaster);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(10);
    let mut state_rx = executor.state_receiver();

    let handle = tokio::spawn(async move {
        executor
            .run(cmd_rx, |_node_id, _node, _feedback| async {
                Err(anyhow::anyhow!("simulated optional failure"))
            })
            .await;
    });

    while *state_rx.borrow() != SessionState::Running {
        state_rx.changed().await.unwrap();
    }

    tokio::time::advance(Duration::from_millis(80)).await;
    cmd_tx.send(SessionCommand::Stop).await.unwrap();
    handle.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn test_continuous_feedback_same_tick_order() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    let toml_str = r#"
            [graph]
            id = "fb-run"
            name = "Feedback Run"
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
    let mut executor = ContinuousExecutor::new(graph, broadcaster);

    let saw_feedback = Arc::new(AtomicBool::new(false));
    let saw_clone = Arc::clone(&saw_feedback);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(10);
    let mut state_rx = executor.state_receiver();

    let handle = tokio::spawn(async move {
        executor
            .run(cmd_rx, move |node_id, _node, feedback| {
                let flag = Arc::clone(&saw_clone);
                async move {
                    if node_id == "consumer" {
                        if feedback
                            .as_ref()
                            .and_then(|v| v.get("from"))
                            .and_then(|x| x.as_str())
                            == Some("producer")
                        {
                            flag.store(true, Ordering::SeqCst);
                        }
                    }
                    if node_id == "producer" {
                        return Ok(serde_json::json!({"from": "producer"}));
                    }
                    Ok(serde_json::json!({"ok": true}))
                }
            })
            .await;
    });

    while *state_rx.borrow() != SessionState::Running {
        state_rx.changed().await.unwrap();
    }

    // Small `advance` steps interleave the spawned executor on the current-thread test
    // runtime; one large advance does not schedule it enough for feedback to be observed.
    for _ in 0..150 {
        tokio::time::advance(Duration::from_millis(1)).await;
    }
    cmd_tx.send(SessionCommand::Stop).await.unwrap();
    handle.await.unwrap();

    assert!(
        saw_feedback.load(Ordering::SeqCst),
        "consumer should receive producer output as feedback in-order on the same tick"
    );
}

#[tokio::test(start_paused = true)]
async fn test_continuous_executor_skip_if_unmet_skips_node() {
    let toml_str = r#"
            [graph]
            id = "skip-if"
            name = "Skip If"
            version = "1.0.0"
            coordination = "continuous"

            [graph.tick]
            target_hz = 120.0

            [graph.env]
            RUN_HEAVY = "false"

            [[graph.nodes]]
            id = "heavy"
            name = "Heavy"

            [graph.nodes.config]
            skip_if = "${RUN_HEAVY} == false"
        "#;
    let graph: DeploymentGraph = toml::from_str(toml_str).unwrap();
    let broadcaster = GraphEventBroadcaster::new(100);
    let mut executor = ContinuousExecutor::new(graph, broadcaster);

    let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel(10);
    let mut state_rx = executor.state_receiver();

    let handle = tokio::spawn(async move {
        executor
            .run(cmd_rx, |_id, _node, _fb| async {
                Ok(serde_json::json!({}))
            })
            .await;
    });

    while *state_rx.borrow() != SessionState::Running {
        state_rx.changed().await.unwrap();
    }

    tokio::time::advance(Duration::from_millis(50)).await;
    cmd_tx.send(SessionCommand::Stop).await.unwrap();
    handle.await.unwrap();
}

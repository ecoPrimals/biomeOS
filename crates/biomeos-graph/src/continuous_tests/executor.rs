// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::time::Duration;

use crate::GraphEventBroadcaster;
use crate::continuous::*;
use crate::graph::DeploymentGraph;

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

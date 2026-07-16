// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::continuous::*;
use crate::graph::{DeploymentGraph, TickConfig};

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

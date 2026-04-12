// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::expect_used, reason = "test assertions")]

use super::*;
use crate::neural_graph::{GraphNode, PrimalSelector};
use serde_json::json;

fn make_node(
    id: &str,
    primal_by_name: Option<&str>,
    primal_by_capability: Option<&str>,
) -> GraphNode {
    let primal = match (primal_by_name, primal_by_capability) {
        (Some(name), cap) => Some(PrimalSelector {
            by_name: Some(name.to_string()),
            by_capability: cap.map(String::from),
        }),
        (None, Some(cap)) => Some(PrimalSelector {
            by_name: None,
            by_capability: Some(cap.to_string()),
        }),
        _ => None,
    };
    GraphNode {
        id: id.to_string(),
        primal,
        ..Default::default()
    }
}

// ── resolve_primal_name (pure logic) ────────────────────────────────────────

#[test]
fn test_resolve_primal_name_from_by_name() {
    let node = make_node("node-id", Some("beardog"), None);
    assert_eq!(GraphHandler::resolve_primal_name(&node), "beardog");
}

#[test]
fn test_resolve_primal_name_fallback_to_node_id() {
    let node = make_node("songbird", None, None);
    assert_eq!(GraphHandler::resolve_primal_name(&node), "songbird");
}

#[test]
fn test_resolve_primal_name_by_capability_only_uses_node_id() {
    let node = make_node("security-node", None, Some("security"));
    assert_eq!(GraphHandler::resolve_primal_name(&node), "security-node");
}

// ── extract_session_id (pure logic) ────────────────────────────────────────

#[test]
fn test_extract_session_id_success() {
    let params = Some(json!({"session_id": "session-abc-123"}));
    let result = GraphHandler::extract_session_id(&params).expect("extract");
    assert_eq!(result, "session-abc-123");
}

#[test]
fn test_extract_session_id_missing_params() {
    let err = GraphHandler::extract_session_id(&None).expect_err("should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[test]
fn test_extract_session_id_missing_session_id() {
    let params = Some(json!({}));
    let err = GraphHandler::extract_session_id(&params).expect_err("should fail");
    assert!(err.to_string().contains("Missing session_id"));
}

#[test]
fn test_extract_session_id_session_id_not_a_string() {
    let params = Some(json!({"session_id": 42}));
    let err = GraphHandler::extract_session_id(&params).expect_err("should fail");
    assert!(err.to_string().contains("Missing session_id"));
}

#[test]
fn test_extract_session_id_empty_string_is_ok() {
    let params = Some(json!({"session_id": ""}));
    let id = GraphHandler::extract_session_id(&params).expect("empty session id allowed");
    assert!(id.is_empty());
}

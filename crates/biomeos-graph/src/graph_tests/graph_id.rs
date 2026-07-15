// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::graph::*;

#[test]
fn test_graph_id_validation() {
    assert!(GraphId::new("livespore-deploy").is_ok());
    assert!(GraphId::new("tower-atomic-bootstrap").is_ok());
    assert!(GraphId::new("tower_atomic_bootstrap").is_ok());
    assert!(GraphId::new("test123").is_ok());

    assert!(GraphId::new("").is_err());
    assert!(GraphId::new("UPPERCASE").is_err());
    assert!(GraphId::new("has spaces").is_err());
}

#[test]
fn test_graph_id_as_str() {
    let id = GraphId::new("my-graph").unwrap();
    assert_eq!(id.as_str(), "my-graph");
}

#[test]
fn test_graph_id_display() {
    let id = GraphId::new("test-graph").unwrap();
    assert_eq!(format!("{id}"), "test-graph");
}

#[test]
fn test_graph_id_try_from_string() {
    let id: Result<GraphId, _> = GraphId::try_from("valid-id".to_string());
    assert!(id.is_ok());

    let id: Result<GraphId, _> = GraphId::try_from("INVALID".to_string());
    assert!(id.is_err());
}

#[test]
fn test_graph_id_into_string() {
    let id = GraphId::new("my-id").unwrap();
    let s: String = id.into();
    assert_eq!(s, "my-id");
}

#[test]
fn test_graph_id_equality() {
    let id1 = GraphId::new("same").unwrap();
    let id2 = GraphId::new("same").unwrap();
    assert_eq!(id1, id2);
}

#[test]
fn test_graph_id_hash() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(GraphId::new("a").unwrap());
    set.insert(GraphId::new("b").unwrap());
    set.insert(GraphId::new("a").unwrap()); // duplicate
    assert_eq!(set.len(), 2);
}

#[test]
fn test_graph_id_serde_roundtrip() {
    let id = GraphId::new("test-serde").unwrap();
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: GraphId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

#[test]
fn test_graph_id_serde_invalid() {
    let json = "\"INVALID_ID\"";
    let result: Result<GraphId, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

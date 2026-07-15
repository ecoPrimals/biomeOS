// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use chrono::Utc;
use std::sync::Arc;

use super::super::*;

#[test]
fn test_subscription_filter_graph_id() {
    let filter = SubscriptionFilter {
        graph_id: Some("test_graph".to_string()),
        event_types: None,
        node_filter: None,
    };

    let event = GraphEvent::GraphStarted {
        graph_id: "test_graph".to_string(),
        graph_name: "Test Graph".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };

    assert!(filter.matches(&event));

    let event2 = GraphEvent::GraphStarted {
        graph_id: "other_graph".to_string(),
        graph_name: "Other Graph".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };

    assert!(!filter.matches(&event2));
}

#[test]
fn test_subscription_filter_node() {
    let filter = SubscriptionFilter {
        graph_id: None,
        event_types: None,
        node_filter: Some("node1".to_string()),
    };

    let event = GraphEvent::NodeStarted {
        graph_id: "test".to_string(),
        node_id: "node1".to_string(),
        primal: "test_primal".to_string(),
        operation: "test_op".to_string(),
        timestamp: Utc::now(),
    };

    assert!(filter.matches(&event));

    let event2 = GraphEvent::NodeStarted {
        graph_id: "test".to_string(),
        node_id: "node2".to_string(),
        primal: "test_primal".to_string(),
        operation: "test_op".to_string(),
        timestamp: Utc::now(),
    };

    assert!(!filter.matches(&event2));
}

#[tokio::test]
async fn test_subscription_filter_empty() {
    let filter = SubscriptionFilter {
        graph_id: None,
        event_types: None,
        node_filter: None,
    };

    // Empty filter matches everything
    let event = GraphEvent::GraphStarted {
        graph_id: "any".to_string(),
        graph_name: "Any Graph".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };

    assert!(filter.matches(&event));
}

#[test]
fn test_subscription_filter_serialization() {
    let filter = SubscriptionFilter {
        graph_id: Some("test-graph".to_string()),
        event_types: Some(vec!["graph_started".to_string()]),
        node_filter: Some("node*".to_string()),
    };

    let json = serde_json::to_string(&filter).expect("serialize");
    assert!(json.contains("test-graph"));
    assert!(json.contains("graph_started"));
    assert!(json.contains("node*"));
}

#[test]
fn test_subscription_filter_deserialization() {
    let json = r#"{"graph_id": "g1", "event_types": ["a", "b"], "node_filter": "n*"}"#;
    let filter: SubscriptionFilter = serde_json::from_str(json).expect("deserialize");

    assert_eq!(filter.graph_id, Some("g1".to_string()));
    assert_eq!(filter.event_types.as_ref().map(Vec::len), Some(2));
    assert_eq!(filter.node_filter, Some("n*".to_string()));
}

#[test]
fn test_graph_event_serialization() {
    let event = GraphEvent::GraphCompleted {
        graph_id: "test".to_string(),
        success: true,
        duration_ms: 1234,
        nodes_executed: 5,
        nodes_failed: 0,
        timestamp: Utc::now(),
    };

    let json = serde_json::to_string(&event).expect("serialize");
    assert!(json.contains("test"));
    assert!(json.contains("1234"));
    assert!(json.contains("true"));
}

#[test]
fn test_subscription_filter_event_types() {
    let filter = SubscriptionFilter {
        graph_id: None,
        event_types: Some(vec!["NodeStarted".to_string(), "NodeCompleted".to_string()]),
        node_filter: None,
    };

    let started_event = GraphEvent::NodeStarted {
        graph_id: "g1".to_string(),
        node_id: "n1".to_string(),
        primal: "p1".to_string(),
        operation: "op1".to_string(),
        timestamp: Utc::now(),
    };
    assert!(filter.matches(&started_event));

    let completed_event = GraphEvent::NodeCompleted {
        graph_id: "g1".to_string(),
        node_id: "n1".to_string(),
        duration_ms: 100,
        output: None,
        timestamp: Utc::now(),
    };
    assert!(filter.matches(&completed_event));

    let graph_started = GraphEvent::GraphStarted {
        graph_id: "g1".to_string(),
        graph_name: "G1".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };
    assert!(!filter.matches(&graph_started));
}

#[test]
fn test_subscription_filter_node_failed() {
    let filter = SubscriptionFilter {
        graph_id: None,
        event_types: None,
        node_filter: Some("target_node".to_string()),
    };

    let matching_event = GraphEvent::NodeFailed {
        graph_id: "g1".to_string(),
        node_id: "target_node".to_string(),
        error: "err".to_string(),
        retry_attempt: 0,
        will_retry: false,
        timestamp: Utc::now(),
    };
    assert!(filter.matches(&matching_event));

    let non_matching = GraphEvent::NodeFailed {
        graph_id: "g1".to_string(),
        node_id: "other_node".to_string(),
        error: "err".to_string(),
        retry_attempt: 0,
        will_retry: false,
        timestamp: Utc::now(),
    };
    assert!(!filter.matches(&non_matching));
}

#[test]
fn test_subscription_filter_combined() {
    let filter = SubscriptionFilter {
        graph_id: Some("my_graph".to_string()),
        event_types: Some(vec!["NodeCompleted".to_string()]),
        node_filter: Some("node_a".to_string()),
    };

    let matching = GraphEvent::NodeCompleted {
        graph_id: "my_graph".to_string(),
        node_id: "node_a".to_string(),
        duration_ms: 50,
        output: None,
        timestamp: Utc::now(),
    };
    assert!(filter.matches(&matching));

    let wrong_graph = GraphEvent::NodeCompleted {
        graph_id: "other_graph".to_string(),
        node_id: "node_a".to_string(),
        duration_ms: 50,
        output: None,
        timestamp: Utc::now(),
    };
    assert!(!filter.matches(&wrong_graph));
}

#[test]
fn test_graph_event_websocket_server_construction() {
    use std::net::SocketAddr;
    use std::str::FromStr;

    let addr = SocketAddr::from_str("127.0.0.1:0").expect("parse addr");
    let broadcaster = Arc::new(GraphEventBroadcaster::new(16));
    let server = GraphEventWebSocketServer::new(addr, broadcaster);
    // Server should be constructible without panicking
    drop(server);
}

#[test]
fn test_subscription_filter_default() {
    let filter = SubscriptionFilter::default();
    assert!(filter.graph_id.is_none());
    assert!(filter.event_types.is_none());
    assert!(filter.node_filter.is_none());

    let event = GraphEvent::GraphStarted {
        graph_id: "any".to_string(),
        graph_name: "Any".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };
    assert!(filter.matches(&event));
}

#[test]
fn test_subscription_filter_non_node_events_pass_node_filter() {
    // GraphStarted has no node_id - node_filter should pass (returns true for non-node events)
    let filter = SubscriptionFilter {
        graph_id: None,
        event_types: None,
        node_filter: Some("some_node".to_string()),
    };
    let event = GraphEvent::GraphStarted {
        graph_id: "g1".to_string(),
        graph_name: "G1".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };
    assert!(filter.matches(&event));
}

#[test]
fn test_subscription_filter_event_type_empty_list() {
    let filter = SubscriptionFilter {
        graph_id: None,
        event_types: Some(vec![]),
        node_filter: None,
    };
    let event = GraphEvent::GraphStarted {
        graph_id: "g1".to_string(),
        graph_name: "G1".to_string(),
        total_nodes: 1,
        coordination: "sequential".to_string(),
        timestamp: Utc::now(),
    };
    assert!(!filter.matches(&event));
}

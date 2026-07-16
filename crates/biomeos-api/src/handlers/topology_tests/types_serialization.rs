// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_topology_node_serialize() {
    let node = TopologyNode {
        id: "test-node".to_string(),
        name: "Test".to_string(),
        primal_type: "security".to_string(),
        health: "healthy".to_string(),
        capabilities: vec!["crypto".to_string()],
        endpoints: None,
        metadata: None,
    };
    let json = serde_json::to_string(&node).unwrap();
    assert!(json.contains("test-node"));
    assert!(json.contains("security"));
}

#[test]
fn test_topology_edge_serialize() {
    let edge = TopologyEdge {
        from: "node-a".to_string(),
        to: "node-b".to_string(),
        edge_type: "capability_invocation".to_string(),
        capability: Some("crypto.encrypt".to_string()),
        metrics: None,
    };
    let json = serde_json::to_string(&edge).unwrap();
    assert!(json.contains("node-a"));
    assert!(json.contains("node-b"));
    assert!(json.contains("capability_invocation"));
}

#[test]
fn test_node_endpoints_serialize() {
    let endpoints = NodeEndpoints {
        unix_socket: Some("/tmp/test.sock".to_string()),
        http: None,
    };
    let json = serde_json::to_string(&endpoints).unwrap();
    assert!(json.contains("/tmp/test.sock"));
    assert!(!json.contains("http")); // Skip serializing None
}

#[test]
fn test_node_metadata_serialize() {
    let metadata = NodeMetadata {
        version: Some("1.0.0".to_string()),
        family_id: Some("1894e909e454".to_string()),
        node_id: Some("desktop".to_string()),
        trust_level: Some(3),
    };
    let json = serde_json::to_string(&metadata).unwrap();
    assert!(json.contains("1.0.0"));
    assert!(json.contains("1894e909e454"));
    assert!(json.contains('3'));
}

#[test]
fn test_edge_metrics_full_serialization() {
    let metrics = EdgeMetrics {
        request_count: Some(100),
        avg_latency_ms: Some(2.5),
        latency_ms: Some(3.0),
        bandwidth_mbps: Some(10.5),
        packet_loss: Some(0.1),
        last_measured: Some("2026-03-11T12:00:00Z".to_string()),
    };
    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("100"));
    assert!(json.contains("2.5"));
    assert!(json.contains("10.5"));
}

#[test]
fn test_node_endpoints_http() {
    let endpoints = NodeEndpoints {
        unix_socket: None,
        http: Some("http://localhost:8080".to_string()),
    };
    let json = serde_json::to_string(&endpoints).unwrap();
    assert!(json.contains("http://localhost:8080"));
}

#[test]
fn test_topology_response_serialization() {
    let response = TopologyResponse {
        primals: vec![TopologyNode {
            id: "test".to_string(),
            name: "Test".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        }],
        connections: vec![],
        health_status: HealthStatus {
            overall: "healthy".to_string(),
            primals_healthy: 1,
            primals_total: 1,
        },
        error: None,
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("primals"));
    assert!(json.contains("connections"));
    assert!(json.contains("health_status"));
}

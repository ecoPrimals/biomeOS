// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use super::*;

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
fn test_health_status_healthy() {
    let nodes = vec![
        TopologyNode {
            id: "a".to_string(),
            name: "A".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
        TopologyNode {
            id: "b".to_string(),
            name: "B".to_string(),
            primal_type: "discovery".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
    ];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "healthy");
    assert_eq!(status.primals_healthy, 2);
    assert_eq!(status.primals_total, 2);
}

#[test]
fn test_health_status_degraded() {
    let nodes = vec![
        TopologyNode {
            id: "a".to_string(),
            name: "A".to_string(),
            primal_type: "security".to_string(),
            health: "healthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
        TopologyNode {
            id: "b".to_string(),
            name: "B".to_string(),
            primal_type: "discovery".to_string(),
            health: "unhealthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
    ];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "degraded");
    assert_eq!(status.primals_healthy, 1);
}

#[test]
fn test_extract_node_id_three_parts() {
    let result = extract_node_id_from_primal("beardog-fam-desktop");
    assert_eq!(result, Some("desktop".to_string()));
}

#[test]
fn test_extract_node_id_two_parts() {
    let result = extract_node_id_from_primal("beardog-desktop");
    assert_eq!(result, Some("desktop".to_string()));
}

#[test]
fn test_extract_node_id_one_part() {
    let result = extract_node_id_from_primal("standalone");
    assert_eq!(result, Some("standalone".to_string()));
}

#[test]
fn test_collect_edge_metrics_security() {
    let metrics = collect_edge_metrics("discovery-node", "security-node");
    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert_eq!(m.latency_ms, Some(5.0)); // Security has overhead
}

#[test]
fn test_collect_edge_metrics_discovery() {
    let metrics = collect_edge_metrics("discovery-node", "storage-node");
    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert_eq!(m.latency_ms, Some(1.5)); // Discovery is fast
}

#[test]
fn test_collect_edge_metrics_default() {
    let metrics = collect_edge_metrics("node-a", "node-b");
    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert_eq!(m.latency_ms, Some(2.0)); // Default latency
}

#[test]
fn test_standalone_topology() {
    let (nodes, edges) = get_standalone_topology();
    assert!(!nodes.is_empty());
    assert!(!edges.is_empty());

    // Check nodes have proper capabilities
    for node in &nodes {
        assert!(!node.capabilities.is_empty());
        assert!(!node.id.is_empty());
    }
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
fn test_extract_node_id_four_parts() {
    let result = extract_node_id_from_primal("beardog-fam-node-desktop");
    assert_eq!(result, Some("desktop".to_string()));
}

#[test]
fn test_extract_node_id_empty_string() {
    let result = extract_node_id_from_primal("");
    assert_eq!(result, Some(String::new()));
}

#[test]
fn test_collect_edge_metrics_crypto() {
    let metrics = collect_edge_metrics("orchestration-node", "crypto-provider");
    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert_eq!(m.latency_ms, Some(5.0));
}

#[test]
fn test_collect_edge_metrics_http() {
    let metrics = collect_edge_metrics("http-client", "storage-node");
    assert!(metrics.is_some());
    let m = metrics.unwrap();
    assert_eq!(m.latency_ms, Some(1.5));
}

#[test]
fn test_health_status_all_unhealthy() {
    let nodes = vec![
        TopologyNode {
            id: "a".to_string(),
            name: "A".to_string(),
            primal_type: "security".to_string(),
            health: "unhealthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
        TopologyNode {
            id: "b".to_string(),
            name: "B".to_string(),
            primal_type: "discovery".to_string(),
            health: "unhealthy".to_string(),
            capabilities: vec![],
            endpoints: None,
            metadata: None,
        },
    ];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "unhealthy");
    assert_eq!(status.primals_healthy, 0);
    assert_eq!(status.primals_total, 2);
}

#[test]
fn test_health_status_empty() {
    let nodes: Vec<TopologyNode> = vec![];
    let status = calculate_health_status(&nodes);
    assert_eq!(status.overall, "healthy");
    assert_eq!(status.primals_healthy, 0);
    assert_eq!(status.primals_total, 0);
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
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("primals"));
    assert!(json.contains("connections"));
    assert!(json.contains("health_status"));
}

#[tokio::test]
async fn test_get_topology_standalone_mode() {
    use crate::AppState;
    use std::sync::Arc;

    let state = Arc::new(
        AppState::builder()
            .config(crate::Config {
                standalone_mode: true,
                ..Default::default()
            })
            .build_with_defaults()
            .expect("should build"),
    );

    let result = get_topology(axum::extract::State(state)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.primals.is_empty());
    assert!(!response.connections.is_empty());
    assert_eq!(response.health_status.overall, "healthy");
}

#[tokio::test]
async fn test_get_topology_live_mode_with_mock() {
    use crate::AppState;
    use biomeos_core::discovery_modern::Capability;
    use biomeos_core::{DiscoveryResult, HealthStatus, PrimalDiscovery, PrimalType};
    use biomeos_types::{Endpoint, FamilyId, PrimalId};
    use semver::Version;
    use std::sync::Arc;

    struct MockDiscovery {
        primals: Vec<biomeos_core::DiscoveredPrimal>,
    }

    #[async_trait::async_trait]
    impl PrimalDiscovery for MockDiscovery {
        async fn discover(
            &self,
            _endpoint: &Endpoint,
        ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
            Err(biomeos_core::DiscoveryError::NotFound {
                endpoint: "mock".to_string(),
            })
        }

        async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
            Ok(self.primals.clone())
        }

        async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
            Ok(HealthStatus::Healthy)
        }
    }

    let primals = vec![
        biomeos_core::DiscoveredPrimal {
            id: PrimalId::new_unchecked("songbird-family-1"),
            name: "Songbird".to_string(),
            primal_type: PrimalType::Orchestration,
            version: Version::parse("1.0.0").expect("valid"),
            health: HealthStatus::Healthy,
            capabilities: vec![Capability::from("orchestration")],
            endpoint: Endpoint::new("unix:///tmp/songbird.sock").expect("valid"),
            metadata: serde_json::json!({}),
            family_id: Some(FamilyId::new("family-1")),
        },
        biomeos_core::DiscoveredPrimal {
            id: PrimalId::new_unchecked("beardog-family-1"),
            name: "BearDog".to_string(),
            primal_type: PrimalType::Security,
            version: Version::parse("1.0.0").expect("valid"),
            health: HealthStatus::Healthy,
            capabilities: vec![Capability::from("security")],
            endpoint: Endpoint::new("unix:///tmp/beardog.sock").expect("valid"),
            metadata: serde_json::json!({}),
            family_id: Some(FamilyId::new("family-1")),
        },
    ];

    let discovery = MockDiscovery { primals };
    let state = Arc::new(
        AppState::builder()
            .discovery(discovery)
            .build_with_defaults()
            .expect("should build"),
    );

    let result = get_topology(axum::extract::State(state)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.primals.len(), 2);
    assert_eq!(response.health_status.primals_total, 2);
    assert_eq!(response.health_status.primals_healthy, 2);
    // Orchestration -> Security connection should exist
    let has_orchestration_security_edge = response
        .connections
        .iter()
        .any(|e| e.from == "songbird-family-1" && e.to == "beardog-family-1");
    assert!(
        has_orchestration_security_edge,
        "Expected orchestration->security edge"
    );
}

#[tokio::test]
async fn test_get_topology_live_mode_discovery_failure_fallback() {
    use crate::AppState;
    use biomeos_core::{DiscoveryError, DiscoveryResult, HealthStatus, PrimalDiscovery};
    use biomeos_types::{Endpoint, PrimalId};
    use std::sync::Arc;

    struct FailingDiscovery;

    #[async_trait::async_trait]
    impl PrimalDiscovery for FailingDiscovery {
        async fn discover(
            &self,
            _endpoint: &Endpoint,
        ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
            Err(DiscoveryError::NotFound {
                endpoint: "mock".to_string(),
            })
        }

        async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
            Err(DiscoveryError::NotFound {
                endpoint: "discovery failed".to_string(),
            })
        }

        async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
            Ok(HealthStatus::Unknown)
        }
    }

    let state = Arc::new(
        AppState::builder()
            .discovery(FailingDiscovery)
            .build_with_defaults()
            .expect("should build"),
    );

    let result = get_topology(axum::extract::State(state)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(
        !response.primals.is_empty(),
        "Should fall back to standalone topology"
    );
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_collect_edge_metrics_unreachable_socket() {
    let metrics = collect_edge_metrics(
        "discovery-node",
        "security-node",
        "unix:///tmp/nonexistent-biomeos-topology-probe.sock",
    )
    .await;
    assert!(metrics.is_none());
}

#[tokio::test]
async fn test_collect_edge_metrics_invalid_endpoint() {
    let metrics = collect_edge_metrics("node-a", "node-b", "not-a-valid-endpoint").await;
    assert!(metrics.is_none());
}

#[tokio::test]
async fn test_collect_edge_metrics_returns_latency_when_present() {
    let metrics = EdgeMetrics {
        request_count: None,
        avg_latency_ms: Some(2.5),
        latency_ms: Some(2.5),
        bandwidth_mbps: None,
        packet_loss: None,
        last_measured: Some(chrono::Utc::now().to_rfc3339()),
    };
    assert_eq!(metrics.latency_ms, Some(2.5));
    assert!(metrics.packet_loss.is_none());
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
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    struct MockDiscovery {
        primals: Vec<biomeos_core::DiscoveredPrimal>,
    }

    impl PrimalDiscovery for MockDiscovery {
        fn discover(
            &self,
            _endpoint: &Endpoint,
        ) -> Pin<
            Box<dyn Future<Output = DiscoveryResult<biomeos_core::DiscoveredPrimal>> + Send + '_>,
        > {
            Box::pin(async move {
                Err(biomeos_core::DiscoveryError::NotFound {
                    endpoint: "mock".to_string(),
                })
            })
        }

        fn discover_all(
            &self,
        ) -> Pin<
            Box<
                dyn Future<Output = DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>>>
                    + Send
                    + '_,
            >,
        > {
            let primals = self.primals.clone();
            Box::pin(async move { Ok(primals) })
        }

        fn check_health(
            &self,
            _id: &PrimalId,
        ) -> Pin<Box<dyn Future<Output = DiscoveryResult<HealthStatus>> + Send + '_>> {
            Box::pin(async move { Ok(HealthStatus::Healthy) })
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
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    struct FailingDiscovery;

    impl PrimalDiscovery for FailingDiscovery {
        fn discover(
            &self,
            _endpoint: &Endpoint,
        ) -> Pin<
            Box<dyn Future<Output = DiscoveryResult<biomeos_core::DiscoveredPrimal>> + Send + '_>,
        > {
            Box::pin(async move {
                Err(DiscoveryError::NotFound {
                    endpoint: "mock".to_string(),
                })
            })
        }

        fn discover_all(
            &self,
        ) -> Pin<
            Box<
                dyn Future<Output = DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>>>
                    + Send
                    + '_,
            >,
        > {
            Box::pin(async move {
                Err(DiscoveryError::NotFound {
                    endpoint: "discovery failed".to_string(),
                })
            })
        }

        fn check_health(
            &self,
            _id: &PrimalId,
        ) -> Pin<Box<dyn Future<Output = DiscoveryResult<HealthStatus>> + Send + '_>> {
            Box::pin(async move { Ok(HealthStatus::Unknown) })
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
    assert!(response.primals.is_empty(), "Should not fabricate primals");
    assert!(response.connections.is_empty());
    assert_eq!(response.health_status.overall, "degraded");
    assert!(
        response
            .error
            .as_deref()
            .is_some_and(|msg| msg.contains("Live discovery failed")),
        "Expected discovery failure context in error field"
    );
}

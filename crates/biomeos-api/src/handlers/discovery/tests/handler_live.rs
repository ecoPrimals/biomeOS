// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_get_discovered_primals_live_mode_success() {
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
            id: PrimalId::new_unchecked("beardog-1"),
            name: "BearDog".to_string(),
            primal_type: PrimalType::Security,
            version: Version::parse("1.0.0").expect("valid version"),
            health: HealthStatus::Healthy,
            capabilities: vec![Capability::from("security")],
            endpoint: Endpoint::new("unix:///tmp/beardog.sock").expect("valid endpoint"),
            metadata: serde_json::json!({}),
            family_id: Some(FamilyId::new("family-1")),
        },
        biomeos_core::DiscoveredPrimal {
            id: PrimalId::new_unchecked("songbird-1"),
            name: "Songbird".to_string(),
            primal_type: PrimalType::Orchestration,
            version: Version::parse("2.0.0").expect("valid version"),
            health: HealthStatus::Degraded,
            capabilities: vec![Capability::from("orchestration")],
            endpoint: Endpoint::new("unix:///tmp/songbird.sock").expect("valid endpoint"),
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

    let result = get_discovered_primals(axum::extract::State(state)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.mode, "live");
    assert_eq!(response.primals.len(), 2);
    assert_eq!(response.count, 2);

    // Verify conversion from core types to API types
    let beardog = response
        .primals
        .iter()
        .find(|p| p.id == "beardog-1")
        .expect("should find BearDog");
    assert_eq!(beardog.name, "BearDog");
    assert_eq!(beardog.health, "healthy");
    assert_eq!(beardog.trust_level, Some(3));

    let songbird = response
        .primals
        .iter()
        .find(|p| p.id == "songbird-1")
        .expect("should find Songbird");
    assert_eq!(songbird.health, "degraded");
}

#[tokio::test]
async fn test_get_discovered_primals_live_mode_failure() {
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

    let discovery = FailingDiscovery;
    let state = Arc::new(
        AppState::builder()
            .discovery(discovery)
            .build_with_defaults()
            .expect("should build"),
    );

    let result = get_discovered_primals(axum::extract::State(state)).await;

    assert!(result.is_ok()); // Should return empty list, not error
    let response = result.unwrap();
    assert_eq!(response.mode, "live_failed");
    assert_eq!(response.primals.len(), 0);
    assert_eq!(response.count, 0);
}

#[tokio::test]
#[expect(clippy::too_many_lines, reason = "comprehensive health status test")]
async fn test_get_discovered_primals_health_status_conversion() {
    use crate::AppState;
    use biomeos_core::{DiscoveryResult, HealthStatus, PrimalDiscovery, PrimalType};
    use biomeos_types::{Endpoint, PrimalId};
    use semver::Version;
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    struct MockDiscovery;

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
            Box::pin(async move {
                Ok(vec![
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("healthy"),
                        name: "Healthy".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Healthy,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/healthy.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("degraded"),
                        name: "Degraded".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Degraded,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/degraded.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("unhealthy"),
                        name: "Unhealthy".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Unhealthy,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/unhealthy.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("unknown"),
                        name: "Unknown".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid version"),
                        health: HealthStatus::Unknown,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/unknown.sock")
                            .expect("valid endpoint"),
                        metadata: serde_json::json!({}),
                        family_id: None,
                    },
                ])
            })
        }

        fn check_health(
            &self,
            _id: &PrimalId,
        ) -> Pin<Box<dyn Future<Output = DiscoveryResult<HealthStatus>> + Send + '_>> {
            Box::pin(async move { Ok(HealthStatus::Healthy) })
        }
    }

    let discovery = MockDiscovery;
    let state = Arc::new(
        AppState::builder()
            .discovery(discovery)
            .build_with_defaults()
            .expect("should build"),
    );

    let result = get_discovered_primals(axum::extract::State(state)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.primals.len(), 4);

    let health_map: std::collections::HashMap<_, _> = response
        .primals
        .iter()
        .map(|p| (p.id.as_str(), p.health.as_str()))
        .collect();

    assert_eq!(health_map.get("healthy"), Some(&"healthy"));
    assert_eq!(health_map.get("degraded"), Some(&"degraded"));
    assert_eq!(health_map.get("unhealthy"), Some(&"unhealthy"));
    assert_eq!(health_map.get("unknown"), Some(&"unknown"));
}

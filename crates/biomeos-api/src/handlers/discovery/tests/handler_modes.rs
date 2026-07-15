// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_get_discovered_primals_standalone_mode() {
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

    let result = get_discovered_primals(axum::extract::State(state)).await;

    assert!(result.is_ok());
    let response = result.unwrap();
    // Standalone mode falls back to socket probing
    // Mode will be "socket_probe" instead of "standalone" (no more fabricated data)
    assert!(
        response.mode == "socket_probe" || response.mode == "live",
        "Mode should be socket_probe or live, got: {}",
        response.mode
    );
    assert_eq!(response.count, response.primals.len());
}

#[tokio::test]
async fn test_get_discovered_primals_standalone_mode_label() {
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

    let result = get_discovered_primals(axum::extract::State(state)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    // Standalone mode: socket_probe or live (if discovery succeeds)
    assert!(
        response.mode == "socket_probe" || response.mode == "live",
        "Mode should be socket_probe or live, got: {}",
        response.mode
    );
}

#[tokio::test]
async fn test_get_discovered_primals_trust_level_with_family() {
    use crate::AppState;
    use biomeos_core::discovery_modern::Capability;
    use biomeos_core::{DiscoveryResult, HealthStatus, PrimalDiscovery, PrimalType};
    use biomeos_types::{Endpoint, FamilyId, PrimalId};
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
                        id: PrimalId::new_unchecked("with-family"),
                        name: "WithFamily".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid"),
                        health: HealthStatus::Healthy,
                        capabilities: vec![Capability::from("security")],
                        endpoint: Endpoint::new("unix:///tmp/with.sock").expect("valid"),
                        metadata: serde_json::json!({}),
                        family_id: Some(FamilyId::new("family-1")),
                    },
                    biomeos_core::DiscoveredPrimal {
                        id: PrimalId::new_unchecked("no-family"),
                        name: "NoFamily".to_string(),
                        primal_type: PrimalType::Security,
                        version: Version::parse("1.0.0").expect("valid"),
                        health: HealthStatus::Healthy,
                        capabilities: vec![],
                        endpoint: Endpoint::new("unix:///tmp/no.sock").expect("valid"),
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

    let state = Arc::new(
        AppState::builder()
            .discovery(MockDiscovery)
            .build_with_defaults()
            .expect("should build"),
    );

    let result = get_discovered_primals(axum::extract::State(state)).await;
    assert!(result.is_ok());
    let response = result.unwrap();
    let with_family = response
        .primals
        .iter()
        .find(|p| p.id == "with-family")
        .expect("with-family primal");
    let no_family = response
        .primals
        .iter()
        .find(|p| p.id == "no-family")
        .expect("no-family primal");
    assert_eq!(with_family.trust_level, Some(3));
    assert_eq!(no_family.trust_level, Some(1));
}

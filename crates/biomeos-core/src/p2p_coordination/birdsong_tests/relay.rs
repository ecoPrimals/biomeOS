// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::helpers::*;
use super::super::*;
use std::sync::Arc;

#[tokio::test]
async fn test_coordinate_relay() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: false,
        success: true,
    });
    let routing = Arc::new(MockRoutingProvider);
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let relay_info = coordinator
        .coordinate_relay("requester-1", "target-1", routing)
        .await
        .expect("coordinate_relay should succeed");
    assert_eq!(relay_info.relay_node, "relay-1");
    assert_eq!(relay_info.requester, "requester-1");
    assert_eq!(relay_info.target, "target-1");
    assert_eq!(relay_info.status, RelayStatus::Active);
}

#[tokio::test]
async fn test_coordinate_relay_fails_when_not_ancestor() {
    let security = Arc::new(MockSecurityProviderNonAncestor);
    let discovery = Arc::new(MockDiscoveryProvider {
        encrypted: false,
        success: true,
    });
    let routing = Arc::new(MockRoutingProvider);
    let coordinator = BirdSongCoordinator::new(security, discovery);
    let err = coordinator
        .coordinate_relay("requester", "target", routing)
        .await
        .expect_err("should fail when target is not ancestor");
    assert!(err.to_string().contains("ancestor") || err.to_string().contains("Lineage"));
}

#[tokio::test]
async fn test_coordinate_relay_verify_lineage_error() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(FailVerifyLineageSecurity),
        Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        }),
    );
    let err = coordinator
        .coordinate_relay("r", "t", Arc::new(MockRoutingProvider))
        .await
        .expect_err("lineage verification error should propagate");
    let chain = format!("{err:#}");
    assert!(chain.contains("lineage-verify-fail"), "got: {chain}");
}

#[tokio::test]
async fn test_coordinate_relay_accept_relay_error() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        }),
    );
    let err = coordinator
        .coordinate_relay("r", "t", Arc::new(FailAcceptRelay))
        .await
        .expect_err("accept_relay error should propagate");
    let chain = format!("{err:#}");
    assert!(chain.contains("accept-relay-fail"), "got: {chain}");
}

#[tokio::test]
async fn test_coordinate_relay_unverified_offer_rejected() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        }),
    );
    let err = coordinator
        .coordinate_relay("r", "t", Arc::new(UnverifiedRelay))
        .await
        .expect_err("unverified relay offer should be rejected");
    assert!(
        err.to_string().contains("lineage not verified"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_coordinate_relay_request_relay_error() {
    let coordinator = BirdSongCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        }),
    );
    let err = coordinator
        .coordinate_relay("r", "t", Arc::new(FailRequestRelay))
        .await
        .expect_err("request_relay error should propagate");
    let chain = format!("{err:#}");
    assert!(chain.contains("request-relay-fail"), "got: {chain}");
}

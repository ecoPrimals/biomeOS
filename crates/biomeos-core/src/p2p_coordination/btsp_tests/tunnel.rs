// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use super::common::{
    FailHealthDiscovery, FailHealthSecurity, FailRegisterDiscovery, GoodDiscovery, GoodSecurity,
    UnhealthySecurity, test_proof,
};

#[tokio::test]
async fn test_create_tunnel_success() {
    let coord = BtspCoordinator::new(Arc::new(GoodSecurity), Arc::new(GoodDiscovery));
    let info = coord
        .create_tunnel("node-a", "node-b", test_proof())
        .await
        .expect("create_tunnel should succeed");
    assert_eq!(info.tunnel_id, "tunnel-node-a-node-b");
    assert_eq!(info.status, TunnelStatus::Active);
    assert_eq!(info.endpoints.len(), 2);
}

#[tokio::test]
async fn test_create_tunnel_register_transport_fails() {
    let coord = BtspCoordinator::new(Arc::new(GoodSecurity), Arc::new(FailRegisterDiscovery));
    let err = coord
        .create_tunnel("a", "b", test_proof())
        .await
        .expect_err("register_transport failure should propagate");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("register-transport-failed"),
        "unexpected: {chain}"
    );
}

#[tokio::test]
async fn test_create_tunnel_security_unhealthy_after_creation() {
    let coord = BtspCoordinator::new(Arc::new(UnhealthySecurity), Arc::new(GoodDiscovery));
    let err = coord
        .create_tunnel("a", "b", test_proof())
        .await
        .expect_err("unhealthy tunnel should fail");
    assert!(
        err.to_string().contains("security health check failed"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_create_tunnel_security_health_error() {
    let coord = BtspCoordinator::new(Arc::new(FailHealthSecurity), Arc::new(GoodDiscovery));
    let err = coord
        .create_tunnel("a", "b", test_proof())
        .await
        .expect_err("health check error should propagate");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("security-health-fail") || chain.contains("verify tunnel health"),
        "unexpected: {chain}"
    );
}

#[tokio::test]
async fn test_create_tunnel_transport_health_error() {
    let coord = BtspCoordinator::new(Arc::new(GoodSecurity), Arc::new(FailHealthDiscovery));
    let err = coord
        .create_tunnel("a", "b", test_proof())
        .await
        .expect_err("transport health error should propagate");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("transport-health-fail") || chain.contains("verify tunnel health"),
        "unexpected: {chain}"
    );
}

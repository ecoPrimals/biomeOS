// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{
    AlwaysDegradedSecurity, DegradedDiscovery, FailHealthDiscovery, FailHealthSecurity,
    GoodDiscovery, GoodSecurity, RecoverableSecurity, UnhealthySecurity,
};
use super::super::*;

#[tokio::test]
async fn test_btsp_monitor_tunnel_success() {
    let coord = BtspCoordinator::new(Arc::new(GoodSecurity), Arc::new(GoodDiscovery));
    let health = coord
        .monitor_tunnel("tun-ok")
        .await
        .expect("monitor should succeed");
    assert_eq!(health.tunnel_id, "tun-ok");
    assert_eq!(health.status, super::super::super::HealthStatus::Healthy);
}

#[tokio::test]
async fn test_btsp_monitor_tunnel_security_error() {
    let coord = BtspCoordinator::new(Arc::new(FailHealthSecurity), Arc::new(GoodDiscovery));
    let err = coord
        .monitor_tunnel("tun")
        .await
        .expect_err("security error should propagate");
    let chain = format!("{err:#}");
    assert!(chain.contains("security-health-fail"), "got: {chain}");
}

#[tokio::test]
async fn test_btsp_monitor_tunnel_discovery_error() {
    let coord = BtspCoordinator::new(Arc::new(GoodSecurity), Arc::new(FailHealthDiscovery));
    let err = coord
        .monitor_tunnel("tun")
        .await
        .expect_err("transport error should propagate");
    let chain = format!("{err:#}");
    assert!(chain.contains("transport-health-fail"), "got: {chain}");
}

#[tokio::test]
async fn test_recover_tunnel_already_healthy() {
    let coord = BtspCoordinator::new(Arc::new(GoodSecurity), Arc::new(GoodDiscovery));
    let info = coord
        .recover_tunnel("tun-ok")
        .await
        .expect("already-healthy tunnel should succeed");
    assert_eq!(info.tunnel_id, "tun-ok");
    assert_eq!(info.status, TunnelStatus::Active);
    assert!(info.endpoints.is_empty());
}

#[tokio::test]
async fn test_recover_tunnel_unhealthy_bails() {
    let coord = BtspCoordinator::new(Arc::new(UnhealthySecurity), Arc::new(GoodDiscovery));
    let err = coord
        .recover_tunnel("tun-bad")
        .await
        .expect_err("unhealthy should require recreation");
    assert!(
        err.to_string().contains("requires recreation"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_recover_tunnel_degraded_succeeds() {
    let coord = BtspCoordinator::new(
        Arc::new(RecoverableSecurity::new()),
        Arc::new(DegradedDiscovery),
    );
    let info = coord
        .recover_tunnel("tun-deg")
        .await
        .expect("degraded recovery should succeed");
    assert_eq!(info.status, TunnelStatus::Active);
}

#[tokio::test]
async fn test_recover_tunnel_degraded_stays_degraded() {
    let coord = BtspCoordinator::new(
        Arc::new(AlwaysDegradedSecurity),
        Arc::new(DegradedDiscovery),
    );
    let err = coord
        .recover_tunnel("tun-deg")
        .await
        .expect_err("still-degraded recovery should fail");
    assert!(
        err.to_string().contains("still degraded"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_recover_tunnel_monitor_error_propagates() {
    let coord = BtspCoordinator::new(Arc::new(FailHealthSecurity), Arc::new(GoodDiscovery));
    let err = coord
        .recover_tunnel("tun")
        .await
        .expect_err("monitor error should propagate");
    let chain = format!("{err:#}");
    assert!(chain.contains("security-health-fail"), "got: {chain}");
}

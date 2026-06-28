// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use super::{MockDiscoveryProvider, MockRoutingProvider, MockSecurityProvider};
use std::sync::Arc;
use std::time::SystemTime;

#[test]
fn test_p2p_coordinator_new_with_explicit_providers() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider);
    let routing = Some(Arc::new(MockRoutingProvider));
    let coordinator = P2PCoordinator::new(security, discovery, routing);
    drop(coordinator);
}

#[test]
fn test_p2p_coordinator_new_without_routing() {
    let security = Arc::new(MockSecurityProvider);
    let discovery = Arc::new(MockDiscoveryProvider);
    let coordinator = P2PCoordinator::new(security, discovery, None::<Arc<MockRoutingProvider>>);
    drop(coordinator);
}

#[tokio::test]
async fn test_create_secure_tunnel() {
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let proof = LineageProof {
        lineage_id: "family-1".to_string(),
        depth: 0,
        proof: bytes::Bytes::new(),
        timestamp: SystemTime::now(),
    };
    let tunnel = coordinator
        .create_secure_tunnel("node-a", "node-b", proof)
        .await
        .expect("create tunnel");
    assert_eq!(tunnel.tunnel_id, "tunnel-node-a-node-b");
    assert_eq!(tunnel.endpoints.len(), 2);
}

#[tokio::test]
async fn test_enable_encrypted_discovery() {
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let mode = coordinator
        .enable_encrypted_discovery("family-123")
        .await
        .expect("enable encrypted discovery");
    assert_eq!(mode, DiscoveryMode::Encrypted);
}

#[tokio::test]
async fn test_coordinate_relay_with_routing() {
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider),
        Some(Arc::new(MockRoutingProvider)),
    );
    let relay = coordinator
        .coordinate_relay("requester", "target")
        .await
        .expect("coordinate relay");
    assert_eq!(relay.relay_node, "relay-node");
    assert_eq!(relay.requester, "requester");
    assert_eq!(relay.target, "target");
    assert_eq!(relay.status, RelayStatus::Active);
}

#[tokio::test]
async fn test_coordinate_relay_without_routing_fails() {
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let result = coordinator.coordinate_relay("requester", "target").await;
    match result {
        Err(e) => assert!(
            e.to_string().contains("No routing provider"),
            "unexpected err: {e}"
        ),
        Ok(_) => panic!("expected coordinate_relay to fail without routing"),
    }
}

#[tokio::test]
async fn test_monitor_tunnel_healthy() {
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let health = coordinator
        .monitor_tunnel("tunnel-1")
        .await
        .expect("monitor tunnel");
    assert_eq!(health.tunnel_id, "tunnel-1");
    assert_eq!(health.status, HealthStatus::Healthy);
}

#[tokio::test]
async fn test_monitor_tunnel_unhealthy() {
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let health = coordinator
        .monitor_tunnel("bad-tunnel")
        .await
        .expect("monitor tunnel");
    assert_eq!(health.status, HealthStatus::Unhealthy);
}
#[tokio::test]
async fn test_monitor_tunnel_security_provider_error_message() {
    struct BadSec;
    impl SecurityProvider for BadSec {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("skip")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            anyhow::bail!("security-down")
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("skip")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("skip")
        }
    }
    let coordinator = P2PCoordinator::new(
        Arc::new(BadSec),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let err = coordinator
        .monitor_tunnel("tid")
        .await
        .expect_err("security should fail");
    let chain = format!("{err:#}");
    assert!(chain.contains("security-down"), "got {chain}");
}

#[tokio::test]
async fn test_monitor_tunnel_discovery_provider_error_message() {
    struct BadDisc;
    impl DiscoveryProvider for BadDisc {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            anyhow::bail!("transport-down")
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("skip")
        }
    }
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(BadDisc),
        None::<Arc<MockRoutingProvider>>,
    );
    let err = coordinator
        .monitor_tunnel("tid")
        .await
        .expect_err("transport should fail");
    let chain = format!("{err:#}");
    assert!(chain.contains("transport-down"), "got {chain}");
}

#[tokio::test]
async fn test_new_from_discovery_strict_without_sockets_fails() {
    let temp = tempfile::tempdir().expect("tempdir");
    let result = P2PCoordinator::new_from_discovery_with_config(&P2pDiscoveryConfig {
        strict_discovery: Some(true),
        xdg_runtime_dir: Some(temp.path().to_path_buf()),
    })
    .await;
    let err = result.err().expect("expected empty socket dir to fail");
    let msg = err.to_string();
    assert!(
        msg.contains("security") || msg.contains("registry"),
        "{msg}"
    );
}
#[tokio::test]
async fn test_create_secure_tunnel_propagates_btsp_error() {
    struct FailSec;
    impl SecurityProvider for FailSec {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("tunnel-request-fail")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            anyhow::bail!("skip")
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("skip")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("skip")
        }
    }
    let coordinator = P2PCoordinator::new(
        Arc::new(FailSec),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let proof = LineageProof {
        lineage_id: "x".to_string(),
        depth: 0,
        proof: bytes::Bytes::new(),
        timestamp: std::time::SystemTime::now(),
    };
    let err = coordinator
        .create_secure_tunnel("a", "b", proof)
        .await
        .expect_err("tunnel");
    let chain = format!("{err:#}");
    assert!(chain.contains("tunnel-request-fail"), "{chain}");
}

#[tokio::test]
async fn test_enable_encrypted_discovery_propagates_error() {
    struct FailKeys;
    impl SecurityProvider for FailKeys {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("skip")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            anyhow::bail!("skip")
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("keys-fail")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("skip")
        }
    }
    let coordinator = P2PCoordinator::new(
        Arc::new(FailKeys),
        Arc::new(MockDiscoveryProvider),
        None::<Arc<MockRoutingProvider>>,
    );
    let err = coordinator
        .enable_encrypted_discovery("fam")
        .await
        .expect_err("enc");
    assert!(format!("{err:#}").contains("keys-fail"));
}

#[tokio::test]
async fn test_new_from_discovery_non_strict_empty_dir_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let result = P2PCoordinator::new_from_discovery_with_config(&P2pDiscoveryConfig {
        strict_discovery: Some(false),
        xdg_runtime_dir: Some(temp.path().to_path_buf()),
    })
    .await;
    let err = result.err().expect("expected empty socket dir");
    assert!(err.to_string().contains("security") || err.to_string().contains("No security"));
}
#[tokio::test]
async fn test_enable_encrypted_discovery_fails_when_discovery_enable_errors() {
    struct FailEncrypted;
    impl DiscoveryProvider for FailEncrypted {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            anyhow::bail!("encrypted-mode-fail")
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            anyhow::bail!("skip")
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("skip")
        }
    }
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(FailEncrypted),
        None::<Arc<MockRoutingProvider>>,
    );
    let err = coordinator
        .enable_encrypted_discovery("fam")
        .await
        .expect_err("discovery enable should fail");
    assert!(
        format!("{err:#}").contains("encrypted-mode-fail"),
        "{err:#}"
    );
}

#[tokio::test]
async fn test_coordinate_relay_propagates_when_routing_request_relay_fails() {
    struct FailRelay;
    impl RoutingProvider for FailRelay {
        async fn request_relay(&self, _: &str, _: &str, _: LineageInfo) -> Result<RelayOffer> {
            anyhow::bail!("relay-offer-fail")
        }
        async fn accept_relay(&self, _: &RelayOffer) -> Result<RelayConnection> {
            anyhow::bail!("skip")
        }
    }
    let coordinator = P2PCoordinator::new(
        Arc::new(MockSecurityProvider),
        Arc::new(MockDiscoveryProvider),
        Some(Arc::new(FailRelay)),
    );
    let err = coordinator
        .coordinate_relay("req", "tgt")
        .await
        .expect_err("relay");
    let chain = format!("{err:#}");
    assert!(
        chain.contains("relay-offer-fail") || chain.contains("relay"),
        "{chain}"
    );
}

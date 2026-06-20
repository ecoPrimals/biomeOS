// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::*;
use std::time::SystemTime;

struct MockSecurityProvider;
impl SecurityProvider for MockSecurityProvider {
    async fn request_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        _proof: &LineageProof,
    ) -> Result<TunnelRequest> {
        Ok(TunnelRequest {
            id: format!("tunnel-{node_a}-{node_b}"),
            endpoint_a: TransportEndpoint {
                node_id: node_a.to_string(),
                address: "127.0.0.1".to_string(),
                port: 9000,
                protocol: "tcp".to_string(),
                secure: true,
            },
            endpoint_b: TransportEndpoint {
                node_id: node_b.to_string(),
                address: "127.0.0.1".to_string(),
                port: 9001,
                protocol: "tcp".to_string(),
                secure: true,
            },
            encryption_key: bytes::Bytes::new(),
            created_at: SystemTime::now(),
        })
    }
    async fn check_tunnel_health(&self, tunnel_id: &str) -> Result<TunnelHealth> {
        Ok(TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: None,
            status: if tunnel_id.contains("bad") {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Healthy
            },
        })
    }
    async fn generate_broadcast_keys(&self, family_id: &str) -> Result<BroadcastKeys> {
        Ok(BroadcastKeys {
            broadcast_key: bytes::Bytes::from(format!("key-{family_id}")),
            lineage_proof: LineageProof {
                lineage_id: family_id.to_string(),
                depth: 0,
                proof: bytes::Bytes::new(),
                timestamp: SystemTime::now(),
            },
            generated_at: SystemTime::now(),
        })
    }
    async fn verify_lineage(&self, requester: &str, target: &str) -> Result<LineageInfo> {
        Ok(LineageInfo {
            is_ancestor: requester != target,
            depth: 1,
            proof: LineageProof {
                lineage_id: requester.to_string(),
                depth: 0,
                proof: bytes::Bytes::new(),
                timestamp: SystemTime::now(),
            },
        })
    }
}

struct MockDiscoveryProvider;
impl DiscoveryProvider for MockDiscoveryProvider {
    async fn register_transport(&self, _endpoint: &TransportEndpoint) -> Result<()> {
        Ok(())
    }
    async fn enable_encrypted_mode(&self, _config: EncryptedDiscoveryConfig) -> Result<()> {
        Ok(())
    }
    async fn check_transport_health(&self, transport_id: &str) -> Result<TransportHealth> {
        Ok(TransportHealth {
            connection_status: if transport_id.contains("bad") {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Healthy
            },
            latency_ms: Some(5),
            packet_loss: None,
            status: if transport_id.contains("bad") {
                HealthStatus::Unhealthy
            } else {
                HealthStatus::Healthy
            },
        })
    }
    async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
        Ok(BroadcastTest {
            encrypted: true,
            timestamp: SystemTime::now(),
            success: true,
        })
    }
}

struct MockRoutingProvider;
impl RoutingProvider for MockRoutingProvider {
    async fn request_relay(
        &self,
        _requester: &str,
        _target: &str,
        lineage: LineageInfo,
    ) -> Result<RelayOffer> {
        Ok(RelayOffer {
            relay_node: "relay-node".to_string(),
            relay_endpoint: TransportEndpoint {
                node_id: "relay".to_string(),
                address: "127.0.0.1".to_string(),
                port: 9002,
                protocol: "tcp".to_string(),
                secure: true,
            },
            expires_at: SystemTime::now() + std::time::Duration::from_secs(300),
            lineage_verified: lineage.is_ancestor,
        })
    }
    async fn accept_relay(&self, offer: &RelayOffer) -> Result<RelayConnection> {
        Ok(RelayConnection {
            connection_id: format!("conn-{}", offer.relay_node),
            relay_node: offer.relay_node.clone(),
            established_at: SystemTime::now(),
            status: RelayStatus::Active,
        })
    }
}

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

#[test]
fn test_compute_status_impl() {
    let healthy = TunnelHealth {
        encryption_status: HealthStatus::Healthy,
        forward_secrecy: true,
        last_key_rotation: None,
        status: HealthStatus::Healthy,
    };
    let transport_healthy = TransportHealth {
        connection_status: HealthStatus::Healthy,
        latency_ms: Some(10),
        packet_loss: None,
        status: HealthStatus::Healthy,
    };
    assert_eq!(
        compute_status_impl(&healthy, &transport_healthy),
        HealthStatus::Healthy
    );

    let degraded = TunnelHealth {
        encryption_status: HealthStatus::Healthy,
        forward_secrecy: true,
        last_key_rotation: None,
        status: HealthStatus::Degraded,
    };
    assert_eq!(
        compute_status_impl(&degraded, &transport_healthy),
        HealthStatus::Degraded
    );

    let unhealthy = TransportHealth {
        connection_status: HealthStatus::Healthy,
        latency_ms: None,
        packet_loss: None,
        status: HealthStatus::Unhealthy,
    };
    assert_eq!(
        compute_status_impl(&healthy, &unhealthy),
        HealthStatus::Unhealthy
    );
}

#[test]
fn test_capability_constants() {
    assert_eq!(CAPABILITY_SECURITY, "security");
    assert_eq!(CAPABILITY_DISCOVERY, "discovery");
    assert_eq!(CAPABILITY_ROUTING, "routing");
}

#[test]
fn strict_discovery_resolved_honors_explicit_config() {
    assert!(super::strict_discovery_resolved(&P2pDiscoveryConfig {
        strict_discovery: Some(true),
        xdg_runtime_dir: None,
    }));
    assert!(!super::strict_discovery_resolved(&P2pDiscoveryConfig {
        strict_discovery: Some(false),
        xdg_runtime_dir: None,
    }));
}

#[test]
fn tunnel_health_all_status_variants_roundtrip_json() {
    for status in [
        HealthStatus::Healthy,
        HealthStatus::Degraded,
        HealthStatus::Unhealthy,
    ] {
        let th = TunnelHealth {
            encryption_status: status,
            forward_secrecy: true,
            last_key_rotation: None,
            status,
        };
        let json = serde_json::to_string(&th).expect("serialize tunnel health");
        let back: TunnelHealth = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.status, status);
        assert_eq!(back.encryption_status, status);
    }
}

#[test]
fn transport_health_status_and_connection_status_paths() {
    let t = TransportHealth {
        connection_status: HealthStatus::Degraded,
        latency_ms: Some(12),
        packet_loss: Some(0.5),
        status: HealthStatus::Healthy,
    };
    let json = serde_json::to_string(&t).expect("serialize");
    assert!(json.contains("connection_status"));
    let back: TransportHealth = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.connection_status, HealthStatus::Degraded);
    assert_eq!(back.status, HealthStatus::Healthy);
}

#[test]
fn tunnel_status_and_relay_status_variants_serialize() {
    for ts in [
        TunnelStatus::Active,
        TunnelStatus::Establishing,
        TunnelStatus::Degraded,
        TunnelStatus::Closed,
    ] {
        let json = serde_json::to_string(&ts).unwrap();
        let u: TunnelStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(u, ts);
    }
    for rs in [
        RelayStatus::Active,
        RelayStatus::Establishing,
        RelayStatus::Failed,
    ] {
        let json = serde_json::to_string(&rs).unwrap();
        let u: RelayStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(u, rs);
    }
}

#[test]
fn discovery_mode_plaintext_encrypted_roundtrip() {
    for m in [DiscoveryMode::Plaintext, DiscoveryMode::Encrypted] {
        let json = serde_json::to_string(&m).unwrap();
        let u: DiscoveryMode = serde_json::from_str(&json).unwrap();
        assert_eq!(u, m);
    }
}

#[test]
fn test_compute_status_both_degraded() {
    let security = TunnelHealth {
        encryption_status: HealthStatus::Degraded,
        forward_secrecy: true,
        last_key_rotation: None,
        status: HealthStatus::Degraded,
    };
    let transport = TransportHealth {
        connection_status: HealthStatus::Healthy,
        latency_ms: Some(100),
        packet_loss: None,
        status: HealthStatus::Healthy,
    };
    assert_eq!(
        compute_status_impl(&security, &transport),
        HealthStatus::Degraded
    );
}

#[test]
fn test_compute_status_both_unhealthy() {
    let security = TunnelHealth {
        encryption_status: HealthStatus::Unhealthy,
        forward_secrecy: false,
        last_key_rotation: None,
        status: HealthStatus::Unhealthy,
    };
    let transport = TransportHealth {
        connection_status: HealthStatus::Unhealthy,
        latency_ms: None,
        packet_loss: Some(50.0),
        status: HealthStatus::Unhealthy,
    };
    assert_eq!(
        compute_status_impl(&security, &transport),
        HealthStatus::Unhealthy
    );
}

#[test]
fn test_compute_status_security_healthy_transport_degraded() {
    let security = TunnelHealth {
        encryption_status: HealthStatus::Healthy,
        forward_secrecy: true,
        last_key_rotation: None,
        status: HealthStatus::Healthy,
    };
    let transport = TransportHealth {
        connection_status: HealthStatus::Degraded,
        latency_ms: Some(500),
        packet_loss: None,
        status: HealthStatus::Degraded,
    };
    assert_eq!(
        compute_status_impl(&security, &transport),
        HealthStatus::Degraded
    );
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

#[test]
fn test_compute_status_security_degraded_transport_unhealthy() {
    let security = TunnelHealth {
        encryption_status: HealthStatus::Degraded,
        forward_secrecy: true,
        last_key_rotation: None,
        status: HealthStatus::Degraded,
    };
    let transport = TransportHealth {
        connection_status: HealthStatus::Unhealthy,
        latency_ms: None,
        packet_loss: None,
        status: HealthStatus::Unhealthy,
    };
    assert_eq!(
        compute_status_impl(&security, &transport),
        HealthStatus::Degraded
    );
}

#[test]
fn test_compute_status_security_unhealthy_transport_degraded() {
    let security = TunnelHealth {
        encryption_status: HealthStatus::Unhealthy,
        forward_secrecy: false,
        last_key_rotation: None,
        status: HealthStatus::Unhealthy,
    };
    let transport = TransportHealth {
        connection_status: HealthStatus::Degraded,
        latency_ms: Some(1),
        packet_loss: None,
        status: HealthStatus::Degraded,
    };
    assert_eq!(
        compute_status_impl(&security, &transport),
        HealthStatus::Degraded
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

#[test]
fn test_p2p_discovery_config_default_clone_debug() {
    let a = P2pDiscoveryConfig::default();
    let b = a.clone();
    assert!(format!("{a:?}").contains("P2pDiscoveryConfig"));
    assert_eq!(a.strict_discovery, b.strict_discovery);
    assert_eq!(a.xdg_runtime_dir, b.xdg_runtime_dir);
}

#[test]
fn test_compute_status_security_healthy_transport_unhealthy() {
    let security = TunnelHealth {
        encryption_status: HealthStatus::Healthy,
        forward_secrecy: true,
        last_key_rotation: None,
        status: HealthStatus::Healthy,
    };
    let transport = TransportHealth {
        connection_status: HealthStatus::Unhealthy,
        latency_ms: None,
        packet_loss: Some(10.0),
        status: HealthStatus::Unhealthy,
    };
    assert_eq!(
        compute_status_impl(&security, &transport),
        HealthStatus::Unhealthy
    );
}

#[test]
fn test_compute_status_security_unhealthy_transport_healthy() {
    let security = TunnelHealth {
        encryption_status: HealthStatus::Unhealthy,
        forward_secrecy: false,
        last_key_rotation: None,
        status: HealthStatus::Unhealthy,
    };
    let transport = TransportHealth {
        connection_status: HealthStatus::Healthy,
        latency_ms: Some(1),
        packet_loss: None,
        status: HealthStatus::Healthy,
    };
    assert_eq!(
        compute_status_impl(&security, &transport),
        HealthStatus::Unhealthy
    );
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

// ========================================================================
// strict_discovery_resolved edge cases
// ========================================================================

#[test]
fn strict_discovery_resolved_none_reads_env() {
    let config = P2pDiscoveryConfig {
        strict_discovery: None,
        xdg_runtime_dir: None,
    };
    let result = super::strict_discovery_resolved(&config);
    let env_set = std::env::var(biomeos_types::env_config::vars::STRICT_DISCOVERY).is_ok();
    assert_eq!(result, env_set);
}

// ========================================================================
// Discovery with None strict_discovery (env var path)
// ========================================================================

#[tokio::test]
async fn test_new_from_discovery_none_strict_empty_dir_fails() {
    let temp = tempfile::tempdir().expect("tempdir");
    let result = P2PCoordinator::new_from_discovery_with_config(&P2pDiscoveryConfig {
        strict_discovery: None,
        xdg_runtime_dir: Some(temp.path().to_path_buf()),
    })
    .await;
    assert!(
        result.is_err(),
        "empty socket dir should fail regardless of strict mode"
    );
}

// ========================================================================
// monitor_tunnel with mixed degraded/unhealthy combinations
// ========================================================================

#[tokio::test]
async fn test_monitor_tunnel_both_degraded() {
    struct DegSec;
    impl SecurityProvider for DegSec {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("skip")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            Ok(TunnelHealth {
                encryption_status: HealthStatus::Degraded,
                forward_secrecy: true,
                last_key_rotation: None,
                status: HealthStatus::Degraded,
            })
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("skip")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("skip")
        }
    }
    struct DegDisc;
    impl DiscoveryProvider for DegDisc {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            Ok(TransportHealth {
                connection_status: HealthStatus::Degraded,
                latency_ms: Some(500),
                packet_loss: None,
                status: HealthStatus::Degraded,
            })
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("skip")
        }
    }
    let coordinator = P2PCoordinator::new(
        Arc::new(DegSec),
        Arc::new(DegDisc),
        None::<Arc<MockRoutingProvider>>,
    );
    let health = coordinator
        .monitor_tunnel("tun-deg")
        .await
        .expect("should succeed");
    assert_eq!(health.status, HealthStatus::Degraded);
    assert_eq!(health.security_health.status, HealthStatus::Degraded);
    assert_eq!(health.transport_health.status, HealthStatus::Degraded);
}

// ========================================================================
// Type serialization coverage for types.rs structs
// ========================================================================

#[test]
fn overall_health_json_roundtrip() {
    let health = OverallHealth {
        tunnel_id: "t1".to_string(),
        security_health: TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Healthy,
        },
        transport_health: TransportHealth {
            connection_status: HealthStatus::Healthy,
            latency_ms: Some(10),
            packet_loss: Some(0.1),
            status: HealthStatus::Healthy,
        },
        status: HealthStatus::Healthy,
    };
    let json = serde_json::to_string(&health).unwrap();
    let back: OverallHealth = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tunnel_id, "t1");
    assert_eq!(back.status, HealthStatus::Healthy);
}

#[test]
fn tunnel_info_json_roundtrip() {
    let info = TunnelInfo {
        tunnel_id: "t2".to_string(),
        status: TunnelStatus::Establishing,
        endpoints: vec![TransportEndpoint {
            node_id: "n1".to_string(),
            address: "10.0.0.1".to_string(),
            port: 9000,
            protocol: "tcp".to_string(),
            secure: true,
        }],
        established_at: SystemTime::now(),
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: TunnelInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tunnel_id, "t2");
    assert_eq!(back.status, TunnelStatus::Establishing);
    assert_eq!(back.endpoints.len(), 1);
}

#[test]
fn relay_info_json_roundtrip() {
    let info = RelayInfo {
        relay_node: "relay".to_string(),
        requester: "req".to_string(),
        target: "tgt".to_string(),
        status: RelayStatus::Active,
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: RelayInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.relay_node, "relay");
    assert_eq!(back.status, RelayStatus::Active);
}

#[test]
fn transport_endpoint_json_roundtrip() {
    let ep = TransportEndpoint {
        node_id: "node-1".to_string(),
        address: "192.168.1.1".to_string(),
        port: 8080,
        protocol: "quic".to_string(),
        secure: false,
    };
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(back.node_id, "node-1");
    assert_eq!(back.port, 8080);
    assert_eq!(back.protocol, "quic");
    assert!(!back.secure);
}

#[test]
fn encrypted_discovery_config_json_roundtrip() {
    let config = EncryptedDiscoveryConfig {
        encryption_key: bytes::Bytes::from_static(b"key"),
        lineage_filter: LineageProof {
            lineage_id: "test".to_string(),
            depth: 0,
            proof: bytes::Bytes::new(),
            timestamp: SystemTime::now(),
        },
        mode: DiscoveryMode::Encrypted,
    };
    let json = serde_json::to_string(&config).unwrap();
    let back: EncryptedDiscoveryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, DiscoveryMode::Encrypted);
}

#[test]
fn broadcast_keys_json_roundtrip() {
    let keys = BroadcastKeys {
        broadcast_key: bytes::Bytes::from_static(b"bkey"),
        lineage_proof: LineageProof {
            lineage_id: "fam".to_string(),
            depth: 2,
            proof: bytes::Bytes::new(),
            timestamp: SystemTime::now(),
        },
        generated_at: SystemTime::now(),
    };
    let json = serde_json::to_string(&keys).unwrap();
    let back: BroadcastKeys = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lineage_proof.lineage_id, "fam");
    assert_eq!(back.lineage_proof.depth, 2);
}

#[test]
fn lineage_info_json_roundtrip() {
    let info = LineageInfo {
        is_ancestor: true,
        depth: 5,
        proof: LineageProof {
            lineage_id: "l".to_string(),
            depth: 0,
            proof: bytes::Bytes::new(),
            timestamp: SystemTime::now(),
        },
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: LineageInfo = serde_json::from_str(&json).unwrap();
    assert!(back.is_ancestor);
    assert_eq!(back.depth, 5);
}

#[test]
fn relay_offer_json_roundtrip() {
    let offer = RelayOffer {
        relay_node: "r".to_string(),
        relay_endpoint: TransportEndpoint {
            node_id: "r".to_string(),
            address: "10.0.0.1".to_string(),
            port: 9000,
            protocol: "tcp".to_string(),
            secure: true,
        },
        expires_at: SystemTime::now() + std::time::Duration::from_secs(60),
        lineage_verified: true,
    };
    let json = serde_json::to_string(&offer).unwrap();
    let back: RelayOffer = serde_json::from_str(&json).unwrap();
    assert_eq!(back.relay_node, "r");
    assert!(back.lineage_verified);
}

#[test]
fn relay_connection_json_roundtrip() {
    let conn = RelayConnection {
        connection_id: "c1".to_string(),
        relay_node: "r".to_string(),
        established_at: SystemTime::now(),
        status: RelayStatus::Establishing,
    };
    let json = serde_json::to_string(&conn).unwrap();
    let back: RelayConnection = serde_json::from_str(&json).unwrap();
    assert_eq!(back.connection_id, "c1");
    assert_eq!(back.status, RelayStatus::Establishing);
}

#[test]
fn tunnel_request_json_roundtrip() {
    let req = TunnelRequest {
        id: "tr1".to_string(),
        endpoint_a: TransportEndpoint {
            node_id: "a".to_string(),
            address: "1.2.3.4".to_string(),
            port: 100,
            protocol: "udp".to_string(),
            secure: false,
        },
        endpoint_b: TransportEndpoint {
            node_id: "b".to_string(),
            address: "5.6.7.8".to_string(),
            port: 200,
            protocol: "tcp".to_string(),
            secure: true,
        },
        encryption_key: bytes::Bytes::from_static(b"enc"),
        created_at: SystemTime::now(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: TunnelRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "tr1");
    assert_eq!(back.endpoint_a.port, 100);
    assert_eq!(back.endpoint_b.protocol, "tcp");
}

#[test]
fn broadcast_test_json_roundtrip() {
    let bt = BroadcastTest {
        encrypted: true,
        timestamp: SystemTime::now(),
        success: false,
    };
    let json = serde_json::to_string(&bt).unwrap();
    let back: BroadcastTest = serde_json::from_str(&json).unwrap();
    assert!(back.encrypted);
    assert!(!back.success);
}

#[test]
fn lineage_proof_json_roundtrip() {
    let proof = LineageProof {
        lineage_id: "fam-1".to_string(),
        depth: 3,
        proof: bytes::Bytes::from_static(b"proof-data"),
        timestamp: SystemTime::now(),
    };
    let json = serde_json::to_string(&proof).unwrap();
    let back: LineageProof = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lineage_id, "fam-1");
    assert_eq!(back.depth, 3);
}

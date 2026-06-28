// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use super::MockRoutingProvider;
use std::sync::Arc;

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

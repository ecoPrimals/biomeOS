// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::expect_used, reason = "test assertions")]

    use super::super::{
        BroadcastKeys, BroadcastTest, EncryptedDiscoveryConfig, HealthStatus, LineageInfo,
        LineageProof, TransportEndpoint, TunnelRequest,
    };
    use super::*;
    use bytes::Bytes;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn healthy_tunnel() -> TunnelHealth {
        TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Healthy,
        }
    }

    fn healthy_transport() -> TransportHealth {
        TransportHealth {
            connection_status: HealthStatus::Healthy,
            latency_ms: Some(10),
            packet_loss: Some(0.0),
            status: HealthStatus::Healthy,
        }
    }

    #[test]
    fn test_compute_overall_status_both_healthy() {
        let security = healthy_tunnel();
        let transport = healthy_transport();
        assert_eq!(
            super::compute_overall_status(&security, &transport),
            HealthStatus::Healthy
        );
    }

    #[test]
    fn test_compute_overall_status_security_degraded() {
        let security = TunnelHealth {
            status: HealthStatus::Degraded,
            ..healthy_tunnel()
        };
        let transport = healthy_transport();
        assert_eq!(
            super::compute_overall_status(&security, &transport),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn test_compute_overall_status_transport_unhealthy() {
        let security = healthy_tunnel();
        let transport = TransportHealth {
            status: HealthStatus::Unhealthy,
            ..healthy_transport()
        };
        assert_eq!(
            super::compute_overall_status(&security, &transport),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_compute_overall_status_security_unhealthy() {
        let security = TunnelHealth {
            status: HealthStatus::Unhealthy,
            ..healthy_tunnel()
        };
        let transport = healthy_transport();
        assert_eq!(
            super::compute_overall_status(&security, &transport),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_compute_overall_status_both_degraded() {
        let security = TunnelHealth {
            status: HealthStatus::Degraded,
            ..healthy_tunnel()
        };
        let transport = TransportHealth {
            status: HealthStatus::Degraded,
            ..healthy_transport()
        };
        assert_eq!(
            super::compute_overall_status(&security, &transport),
            HealthStatus::Degraded
        );
    }

    #[test]
    fn test_compute_overall_status_transport_degraded_security_healthy() {
        let security = healthy_tunnel();
        let transport = TransportHealth {
            status: HealthStatus::Degraded,
            ..healthy_transport()
        };
        assert_eq!(
            super::compute_overall_status(&security, &transport),
            HealthStatus::Degraded
        );
    }

    // ====================================================================
    // Mock providers for BtspCoordinator integration tests
    // ====================================================================

    fn test_proof() -> LineageProof {
        LineageProof {
            lineage_id: "test".to_string(),
            depth: 0,
            proof: Bytes::new(),
            timestamp: SystemTime::now(),
        }
    }

    fn make_tunnel_request(a: &str, b: &str) -> TunnelRequest {
        TunnelRequest {
            id: format!("tunnel-{a}-{b}"),
            endpoint_a: TransportEndpoint {
                node_id: a.to_string(),
                address: "10.0.0.1".to_string(),
                port: 9000,
                protocol: "tcp".to_string(),
                secure: true,
            },
            endpoint_b: TransportEndpoint {
                node_id: b.to_string(),
                address: "10.0.0.2".to_string(),
                port: 9001,
                protocol: "tcp".to_string(),
                secure: true,
            },
            encryption_key: Bytes::new(),
            created_at: SystemTime::now(),
        }
    }

    struct GoodSecurity;
    impl SecurityProvider for GoodSecurity {
        async fn request_tunnel(
            &self,
            a: &str,
            b: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            Ok(make_tunnel_request(a, b))
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            Ok(healthy_tunnel())
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("unused in btsp tests")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("unused in btsp tests")
        }
    }

    struct GoodDiscovery;
    impl DiscoveryProvider for GoodDiscovery {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            Ok(healthy_transport())
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("unused in btsp tests")
        }
    }

    struct FailRegisterDiscovery;
    impl DiscoveryProvider for FailRegisterDiscovery {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            anyhow::bail!("register-transport-failed")
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            Ok(healthy_transport())
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("unused")
        }
    }

    struct UnhealthySecurity;
    impl SecurityProvider for UnhealthySecurity {
        async fn request_tunnel(
            &self,
            a: &str,
            b: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            Ok(make_tunnel_request(a, b))
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            Ok(TunnelHealth {
                status: HealthStatus::Unhealthy,
                encryption_status: HealthStatus::Unhealthy,
                ..healthy_tunnel()
            })
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("unused")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("unused")
        }
    }

    struct FailHealthSecurity;
    impl SecurityProvider for FailHealthSecurity {
        async fn request_tunnel(
            &self,
            a: &str,
            b: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            Ok(make_tunnel_request(a, b))
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            anyhow::bail!("security-health-fail")
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("unused")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("unused")
        }
    }

    struct FailHealthDiscovery;
    impl DiscoveryProvider for FailHealthDiscovery {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            anyhow::bail!("transport-health-fail")
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("unused")
        }
    }

    /// Security that returns Degraded on first `check_tunnel_health`, then Healthy.
    struct RecoverableSecurity {
        calls: AtomicUsize,
    }
    impl RecoverableSecurity {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }
    impl SecurityProvider for RecoverableSecurity {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("unused")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            let call = self.calls.fetch_add(1, Ordering::SeqCst);
            let status = if call == 0 {
                HealthStatus::Degraded
            } else {
                HealthStatus::Healthy
            };
            Ok(TunnelHealth {
                encryption_status: status,
                forward_secrecy: true,
                last_key_rotation: None,
                status,
            })
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("unused")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("unused")
        }
    }

    struct AlwaysDegradedSecurity;
    impl SecurityProvider for AlwaysDegradedSecurity {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("unused")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            Ok(TunnelHealth {
                status: HealthStatus::Degraded,
                encryption_status: HealthStatus::Degraded,
                ..healthy_tunnel()
            })
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("unused")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("unused")
        }
    }

    struct DegradedDiscovery;
    impl DiscoveryProvider for DegradedDiscovery {
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
            anyhow::bail!("unused")
        }
    }

    // ====================================================================
    // BtspCoordinator::create_tunnel tests
    // ====================================================================

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

    // ====================================================================
    // BtspCoordinator::monitor_tunnel tests
    // ====================================================================

    #[tokio::test]
    async fn test_btsp_monitor_tunnel_success() {
        let coord = BtspCoordinator::new(Arc::new(GoodSecurity), Arc::new(GoodDiscovery));
        let health = coord
            .monitor_tunnel("tun-ok")
            .await
            .expect("monitor should succeed");
        assert_eq!(health.tunnel_id, "tun-ok");
        assert_eq!(health.status, HealthStatus::Healthy);
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

    // ====================================================================
    // BtspCoordinator::recover_tunnel tests
    // ====================================================================

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

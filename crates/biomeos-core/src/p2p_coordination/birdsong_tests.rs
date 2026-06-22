// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::expect_used, reason = "test assertions")]

    use super::*;
    use crate::p2p_coordination::{
        BroadcastKeys, BroadcastTest, LineageInfo, LineageProof, RelayConnection, RelayOffer,
        RelayStatus, TransportEndpoint, TransportHealth, TunnelHealth, TunnelRequest,
    };
    use biomeos_types::constants::ports;
    use std::time::SystemTime;

    struct MockSecurityProvider;

    impl SecurityProvider for MockSecurityProvider {
        async fn request_tunnel(
            &self,
            _node_a: &str,
            _node_b: &str,
            _proof: &LineageProof,
        ) -> Result<TunnelRequest> {
            Err(anyhow::anyhow!("not used in birdsong tests"))
        }

        async fn check_tunnel_health(&self, _tunnel_id: &str) -> Result<TunnelHealth> {
            Err(anyhow::anyhow!("not used in birdsong tests"))
        }

        async fn generate_broadcast_keys(&self, _family_id: &str) -> Result<BroadcastKeys> {
            Ok(BroadcastKeys {
                broadcast_key: Bytes::from_static(&[1, 2, 3]),
                lineage_proof: LineageProof {
                    lineage_id: "test".to_string(),
                    depth: 0,
                    proof: Bytes::new(),
                    timestamp: SystemTime::now(),
                },
                generated_at: SystemTime::now(),
            })
        }

        async fn verify_lineage(&self, _requester: &str, _target: &str) -> Result<LineageInfo> {
            Ok(LineageInfo {
                is_ancestor: true,
                depth: 1,
                proof: LineageProof {
                    lineage_id: "test".to_string(),
                    depth: 1,
                    proof: Bytes::new(),
                    timestamp: SystemTime::now(),
                },
            })
        }
    }

    struct MockDiscoveryProvider {
        encrypted: bool,
        success: bool,
    }

    impl DiscoveryProvider for MockDiscoveryProvider {
        async fn register_transport(&self, _endpoint: &TransportEndpoint) -> Result<()> {
            Ok(())
        }

        async fn enable_encrypted_mode(&self, _config: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }

        async fn check_transport_health(&self, _transport_id: &str) -> Result<TransportHealth> {
            Err(anyhow::anyhow!("not used in birdsong tests"))
        }

        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            Ok(BroadcastTest {
                encrypted: self.encrypted,
                timestamp: SystemTime::now(),
                success: self.success,
            })
        }
    }

    #[test]
    fn test_discovery_mode() {
        assert_eq!(DiscoveryMode::Plaintext, DiscoveryMode::Plaintext);
        assert_ne!(DiscoveryMode::Plaintext, DiscoveryMode::Encrypted);
    }

    #[test]
    fn test_birdsong_coordinator_new() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let _coordinator = BirdSongCoordinator::new(security, discovery);
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: true,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .enable_encrypted_discovery("family-1")
            .await
            .expect("enable_encrypted_discovery should succeed");
        assert_eq!(mode, DiscoveryMode::Encrypted);
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery_fails_when_not_encrypted() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let err = coordinator
            .enable_encrypted_discovery("family-1")
            .await
            .expect_err("should fail when broadcast not encrypted");
        assert!(err.to_string().contains("encrypted"));
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery_fails_when_test_unsuccessful() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: true,
            success: false,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let err = coordinator
            .enable_encrypted_discovery("family-1")
            .await
            .expect_err("should fail when test unsuccessful");
        assert!(
            err.to_string().contains("unsuccessful") || err.to_string().contains("verification")
        );
    }

    #[tokio::test]
    async fn test_disable_encrypted_discovery() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .disable_encrypted_discovery()
            .await
            .expect("disable_encrypted_discovery should succeed");
        assert_eq!(mode, DiscoveryMode::Plaintext);
    }

    #[tokio::test]
    async fn test_get_discovery_mode_encrypted() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: true,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .get_discovery_mode()
            .await
            .expect("get_discovery_mode should succeed");
        assert_eq!(mode, DiscoveryMode::Encrypted);
    }

    #[tokio::test]
    async fn test_get_discovery_mode_plaintext() {
        let security = Arc::new(MockSecurityProvider);
        let discovery = Arc::new(MockDiscoveryProvider {
            encrypted: false,
            success: true,
        });
        let coordinator = BirdSongCoordinator::new(security, discovery);
        let mode = coordinator
            .get_discovery_mode()
            .await
            .expect("get_discovery_mode should succeed");
        assert_eq!(mode, DiscoveryMode::Plaintext);
    }

    struct MockRoutingProvider;

    impl super::RoutingProvider for MockRoutingProvider {
        async fn request_relay(
            &self,
            _requester: &str,
            _target: &str,
            _lineage: LineageInfo,
        ) -> Result<RelayOffer> {
            Ok(RelayOffer {
                relay_node: "relay-1".to_string(),
                relay_endpoint: TransportEndpoint {
                    node_id: "relay-1".to_string(),
                    address: "127.0.0.1".to_string(),
                    port: ports::NEURAL_API,
                    protocol: "tcp".to_string(),
                    secure: true,
                },
                expires_at: SystemTime::now(),
                lineage_verified: true,
            })
        }

        async fn accept_relay(&self, _offer: &RelayOffer) -> Result<RelayConnection> {
            Ok(RelayConnection {
                connection_id: "conn-1".to_string(),
                relay_node: "relay-1".to_string(),
                established_at: SystemTime::now(),
                status: RelayStatus::Active,
            })
        }
    }

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

    struct MockSecurityProviderNonAncestor;

    impl SecurityProvider for MockSecurityProviderNonAncestor {
        async fn request_tunnel(
            &self,
            _node_a: &str,
            _node_b: &str,
            _proof: &LineageProof,
        ) -> Result<TunnelRequest> {
            Err(anyhow::anyhow!("not used"))
        }

        async fn check_tunnel_health(&self, _tunnel_id: &str) -> Result<TunnelHealth> {
            Err(anyhow::anyhow!("not used"))
        }

        async fn generate_broadcast_keys(&self, _family_id: &str) -> Result<BroadcastKeys> {
            Err(anyhow::anyhow!("not used"))
        }

        async fn verify_lineage(&self, _requester: &str, _target: &str) -> Result<LineageInfo> {
            Ok(LineageInfo {
                is_ancestor: false,
                depth: 0,
                proof: LineageProof {
                    lineage_id: "test".to_string(),
                    depth: 0,
                    proof: Bytes::new(),
                    timestamp: SystemTime::now(),
                },
            })
        }
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

    // ====================================================================
    // Additional error-path tests
    // ====================================================================

    struct FailEnableDiscovery;
    impl DiscoveryProvider for FailEnableDiscovery {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            anyhow::bail!("enable-mode-fail")
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            anyhow::bail!("unused")
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("unused")
        }
    }

    #[tokio::test]
    async fn test_disable_encrypted_discovery_fails_when_provider_errors() {
        let coordinator = BirdSongCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(FailEnableDiscovery),
        );
        let err = coordinator
            .disable_encrypted_discovery()
            .await
            .expect_err("should fail when discovery provider errors");
        let chain = format!("{err:#}");
        assert!(chain.contains("enable-mode-fail"), "got: {chain}");
    }

    struct FailBroadcastDiscovery;
    impl DiscoveryProvider for FailBroadcastDiscovery {
        async fn register_transport(&self, _: &TransportEndpoint) -> Result<()> {
            Ok(())
        }
        async fn enable_encrypted_mode(&self, _: EncryptedDiscoveryConfig) -> Result<()> {
            Ok(())
        }
        async fn check_transport_health(&self, _: &str) -> Result<TransportHealth> {
            anyhow::bail!("unused")
        }
        async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
            anyhow::bail!("broadcast-test-fail")
        }
    }

    #[tokio::test]
    async fn test_get_discovery_mode_error_propagates() {
        let coordinator = BirdSongCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(FailBroadcastDiscovery),
        );
        let err = coordinator
            .get_discovery_mode()
            .await
            .expect_err("should propagate broadcast test error");
        let chain = format!("{err:#}");
        assert!(chain.contains("broadcast-test-fail"), "got: {chain}");
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery_broadcast_test_error_propagates() {
        let coordinator = BirdSongCoordinator::new(
            Arc::new(MockSecurityProvider),
            Arc::new(FailBroadcastDiscovery),
        );
        let err = coordinator
            .enable_encrypted_discovery("fam")
            .await
            .expect_err("broadcast test error should propagate");
        let chain = format!("{err:#}");
        assert!(
            chain.contains("broadcast-test-fail") || chain.contains("test encrypted"),
            "got: {chain}"
        );
    }

    struct FailVerifyLineageSecurity;
    impl SecurityProvider for FailVerifyLineageSecurity {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("unused")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            anyhow::bail!("unused")
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("unused")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("lineage-verify-fail")
        }
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

    struct FailAcceptRelay;
    impl super::RoutingProvider for FailAcceptRelay {
        async fn request_relay(&self, _: &str, _: &str, _: LineageInfo) -> Result<RelayOffer> {
            Ok(RelayOffer {
                relay_node: "relay-1".to_string(),
                relay_endpoint: TransportEndpoint {
                    node_id: "relay-1".to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 9999,
                    protocol: "tcp".to_string(),
                    secure: true,
                },
                expires_at: SystemTime::now(),
                lineage_verified: true,
            })
        }
        async fn accept_relay(&self, _: &RelayOffer) -> Result<RelayConnection> {
            anyhow::bail!("accept-relay-fail")
        }
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

    struct UnverifiedRelay;
    impl super::RoutingProvider for UnverifiedRelay {
        async fn request_relay(&self, _: &str, _: &str, _: LineageInfo) -> Result<RelayOffer> {
            Ok(RelayOffer {
                relay_node: "relay-1".to_string(),
                relay_endpoint: TransportEndpoint {
                    node_id: "relay-1".to_string(),
                    address: "127.0.0.1".to_string(),
                    port: 9999,
                    protocol: "tcp".to_string(),
                    secure: true,
                },
                expires_at: SystemTime::now(),
                lineage_verified: false,
            })
        }
        async fn accept_relay(&self, _: &RelayOffer) -> Result<RelayConnection> {
            anyhow::bail!("should not be called")
        }
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

    struct FailRequestRelay;
    impl super::RoutingProvider for FailRequestRelay {
        async fn request_relay(&self, _: &str, _: &str, _: LineageInfo) -> Result<RelayOffer> {
            anyhow::bail!("request-relay-fail")
        }
        async fn accept_relay(&self, _: &RelayOffer) -> Result<RelayConnection> {
            anyhow::bail!("should not be called")
        }
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

    struct FailGenerateKeysSecurity;
    impl SecurityProvider for FailGenerateKeysSecurity {
        async fn request_tunnel(
            &self,
            _: &str,
            _: &str,
            _: &LineageProof,
        ) -> Result<TunnelRequest> {
            anyhow::bail!("unused")
        }
        async fn check_tunnel_health(&self, _: &str) -> Result<TunnelHealth> {
            anyhow::bail!("unused")
        }
        async fn generate_broadcast_keys(&self, _: &str) -> Result<BroadcastKeys> {
            anyhow::bail!("generate-keys-fail")
        }
        async fn verify_lineage(&self, _: &str, _: &str) -> Result<LineageInfo> {
            anyhow::bail!("unused")
        }
    }

    #[tokio::test]
    async fn test_enable_encrypted_discovery_generate_keys_error() {
        let coordinator = BirdSongCoordinator::new(
            Arc::new(FailGenerateKeysSecurity),
            Arc::new(MockDiscoveryProvider {
                encrypted: true,
                success: true,
            }),
        );
        let err = coordinator
            .enable_encrypted_discovery("fam")
            .await
            .expect_err("key generation error should propagate");
        let chain = format!("{err:#}");
        assert!(chain.contains("generate-keys-fail"), "got: {chain}");
    }

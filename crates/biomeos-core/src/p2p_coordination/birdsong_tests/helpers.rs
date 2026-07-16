// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use crate::p2p_coordination::{
    BroadcastKeys, BroadcastTest, LineageInfo, LineageProof, RelayConnection, RelayOffer,
    RelayStatus, TransportEndpoint, TransportHealth, TunnelHealth, TunnelRequest,
};
use biomeos_types::constants::ports;
use std::time::SystemTime;

pub(super) struct MockSecurityProvider;

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

pub(super) struct MockDiscoveryProvider {
    pub encrypted: bool,
    pub success: bool,
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

pub(super) struct MockRoutingProvider;

impl super::super::RoutingProvider for MockRoutingProvider {
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

pub(super) struct MockSecurityProviderNonAncestor;

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

pub(super) struct FailEnableDiscovery;

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

pub(super) struct FailBroadcastDiscovery;

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

pub(super) struct FailVerifyLineageSecurity;

impl SecurityProvider for FailVerifyLineageSecurity {
    async fn request_tunnel(&self, _: &str, _: &str, _: &LineageProof) -> Result<TunnelRequest> {
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

pub(super) struct FailAcceptRelay;

impl super::super::RoutingProvider for FailAcceptRelay {
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

pub(super) struct UnverifiedRelay;

impl super::super::RoutingProvider for UnverifiedRelay {
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

pub(super) struct FailRequestRelay;

impl super::super::RoutingProvider for FailRequestRelay {
    async fn request_relay(&self, _: &str, _: &str, _: LineageInfo) -> Result<RelayOffer> {
        anyhow::bail!("request-relay-fail")
    }
    async fn accept_relay(&self, _: &RelayOffer) -> Result<RelayConnection> {
        anyhow::bail!("should not be called")
    }
}

pub(super) struct FailGenerateKeysSecurity;

impl SecurityProvider for FailGenerateKeysSecurity {
    async fn request_tunnel(&self, _: &str, _: &str, _: &LineageProof) -> Result<TunnelRequest> {
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

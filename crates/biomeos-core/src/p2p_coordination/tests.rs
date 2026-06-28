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

#[path = "tests_coordinator.rs"]
mod coordinator;

#[path = "tests_status.rs"]
mod status;

#[path = "tests_types.rs"]
mod types;

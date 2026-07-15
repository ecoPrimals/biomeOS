// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::super::{
    BroadcastKeys, BroadcastTest, EncryptedDiscoveryConfig, HealthStatus, LineageInfo,
    LineageProof, TransportEndpoint, TunnelRequest,
};
use super::super::*;
use bytes::Bytes;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::SystemTime;

pub(super) fn healthy_tunnel() -> TunnelHealth {
    TunnelHealth {
        encryption_status: HealthStatus::Healthy,
        forward_secrecy: true,
        last_key_rotation: None,
        status: HealthStatus::Healthy,
    }
}

pub(super) fn healthy_transport() -> TransportHealth {
    TransportHealth {
        connection_status: HealthStatus::Healthy,
        latency_ms: Some(10),
        packet_loss: Some(0.0),
        status: HealthStatus::Healthy,
    }
}

pub(super) fn test_proof() -> LineageProof {
    LineageProof {
        lineage_id: "test".to_string(),
        depth: 0,
        proof: Bytes::new(),
        timestamp: SystemTime::now(),
    }
}

pub(super) fn make_tunnel_request(a: &str, b: &str) -> TunnelRequest {
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

pub(super) struct GoodSecurity;
impl SecurityProvider for GoodSecurity {
    async fn request_tunnel(&self, a: &str, b: &str, _: &LineageProof) -> Result<TunnelRequest> {
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

pub(super) struct GoodDiscovery;
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

pub(super) struct FailRegisterDiscovery;
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

pub(super) struct UnhealthySecurity;
impl SecurityProvider for UnhealthySecurity {
    async fn request_tunnel(&self, a: &str, b: &str, _: &LineageProof) -> Result<TunnelRequest> {
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

pub(super) struct FailHealthSecurity;
impl SecurityProvider for FailHealthSecurity {
    async fn request_tunnel(&self, a: &str, b: &str, _: &LineageProof) -> Result<TunnelRequest> {
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

pub(super) struct FailHealthDiscovery;
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
pub(super) struct RecoverableSecurity {
    calls: AtomicUsize,
}
impl RecoverableSecurity {
    pub(super) fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
        }
    }
}
impl SecurityProvider for RecoverableSecurity {
    async fn request_tunnel(&self, _: &str, _: &str, _: &LineageProof) -> Result<TunnelRequest> {
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

pub(super) struct AlwaysDegradedSecurity;
impl SecurityProvider for AlwaysDegradedSecurity {
    async fn request_tunnel(&self, _: &str, _: &str, _: &LineageProof) -> Result<TunnelRequest> {
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

pub(super) struct DegradedDiscovery;
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

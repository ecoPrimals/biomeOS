// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Real Primal Adapters for P2P Coordination
//!
//! These adapters connect BiomeOS's P2P coordination logic to primals discovered
//! by capability. No hardcoded primal names — capability-based discovery.
//!
//! # Architecture
//!
//! ```text
//! BiomeOS P2P Coordinator
//!         │
//!         ├─► SecurityProvider trait (capability: crypto)
//!         │   └─► CryptoSecurityAdapter (this file)
//!         │       └─► CLI-based crypto primal (discovered at runtime)
//!         │
//!         └─► DiscoveryProvider trait (capability: discovery)
//!             └─► MeshDiscoveryAdapter (this file)
//!                 └─► HTTP-based discovery primal (discovered at runtime)
//! ```
//!
//! # Key Features
//!
//! - **Capability-Based**: Adapters work with any primal providing the capability
//! - **Type-Safe**: Rust types for all interactions
//! - **Error Handling**: Proper error propagation
//! - **Health Monitoring**: Real-time health checks

use super::types::{BroadcastKeys, EncryptedDiscoveryConfig, TransportHealth, TunnelHealth};
use super::{DiscoveryProvider, SecurityProvider};
use biomeos_types::constants::ports;
use crate::api_adapter::cli_adapter::CliAdapter;
use crate::atomic_client::AtomicClient;
use bytes::Bytes;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;
use std::time::SystemTime;
use tracing::{debug, info};

/// Crypto/security adapter for security primal operations
///
/// Implements SecurityProvider for any primal providing crypto capability.
/// Uses CLI interface when the primal is CLI-based (discovered at runtime).
pub struct CryptoSecurityAdapter {
    cli: CliAdapter,
}

impl CryptoSecurityAdapter {
    pub fn new(binary_path: String) -> Self {
        let cli = CliAdapter::new(binary_path);
        Self { cli }
    }
}

#[async_trait]
impl SecurityProvider for CryptoSecurityAdapter {
    async fn request_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        proof: &super::LineageProof,
    ) -> Result<super::TunnelRequest> {
        info!(
            "CryptoSecurityAdapter: Requesting BTSP tunnel between {} and {}",
            node_a, node_b
        );

        let lineage_id = &proof.lineage_id;

        // Security-primal CLI: `btsp create-tunnel --node-a … --node-b … --lineage …`
        let args = vec![
            "btsp",
            "create-tunnel",
            "--node-a",
            node_a,
            "--node-b",
            node_b,
            "--lineage",
            lineage_id,
        ];

        let result = self
            .cli
            .execute(&args)
            .await
            .context("Failed to execute crypto primal BTSP create-tunnel command")?;

        if !result.is_success() {
            anyhow::bail!("Crypto primal command failed: {}", result.stderr());
        }

        let output = result.stdout();

        let tunnel_id = Self::parse_tunnel_id(output)?;
        let encryption_key = Self::parse_field_bytes(output, "encryption_key")
            .unwrap_or_default();
        let endpoint_a = super::TransportEndpoint {
            node_id: node_a.to_string(),
            address: format!("btsp://{node_a}"),
            port: ports::NEURAL_API,
            protocol: "btsp".to_string(),
            secure: true,
        };
        let endpoint_b = super::TransportEndpoint {
            node_id: node_b.to_string(),
            address: format!("btsp://{node_b}"),
            port: ports::NEURAL_API,
            protocol: "btsp".to_string(),
            secure: true,
        };

        Ok(super::TunnelRequest {
            id: tunnel_id,
            endpoint_a,
            endpoint_b,
            encryption_key,
            created_at: SystemTime::now(),
        })
    }

    async fn check_tunnel_health(&self, tunnel_id: &str) -> Result<TunnelHealth> {
        debug!("CryptoSecurityAdapter: Checking tunnel health for {}", tunnel_id);

        let args = vec!["btsp", "status", "--tunnel-id", tunnel_id];

        let result = self
            .cli
            .execute(&args)
            .await
            .context("Failed to execute crypto primal BTSP status command")?;

        if !result.is_success() {
            anyhow::bail!("Crypto primal command failed: {}", result.stderr());
        }

        let output = result.stdout();

        let status = if output.contains("healthy") {
            super::HealthStatus::Healthy
        } else if output.contains("degraded") {
            super::HealthStatus::Degraded
        } else {
            super::HealthStatus::Unhealthy
        };

        let forward_secrecy = output.contains("forward_secrecy: true")
            || output.contains("pfs: enabled");

        Ok(TunnelHealth {
            encryption_status: status,
            forward_secrecy,
            last_key_rotation: None,
            status,
        })
    }

    async fn generate_broadcast_keys(&self, family_id: &str) -> Result<BroadcastKeys> {
        info!(
            "CryptoSecurityAdapter: Generating broadcast keys for family {}",
            family_id
        );

        let args = vec!["birdsong", "generate-keys", "--family-id", family_id];

        let result = self
            .cli
            .execute(&args)
            .await
            .context("Failed to execute crypto primal BirdSong generate-keys command")?;

        if !result.is_success() {
            anyhow::bail!("Crypto primal command failed: {}", result.stderr());
        }

        let output = result.stdout();

        // Parse keys from output
        let broadcast_key = Self::parse_broadcast_key(output)?;

        Ok(BroadcastKeys {
            broadcast_key,
            lineage_proof: super::LineageProof {
                lineage_id: family_id.to_string(),
                depth: 0,
                proof: Bytes::new(),
                timestamp: SystemTime::now(),
            },
            generated_at: SystemTime::now(),
        })
    }

    async fn verify_lineage(&self, requester: &str, target: &str) -> Result<super::LineageInfo> {
        debug!(
            "CryptoSecurityAdapter: Verifying lineage between {} and {}",
            requester, target
        );

        let args = vec![
            "lineage",
            "verify",
            "--requester",
            requester,
            "--target",
            target,
        ];

        let result = self
            .cli
            .execute(&args)
            .await
            .context("Failed to execute crypto primal lineage verify command")?;

        if !result.is_success() {
            anyhow::bail!("Crypto primal command failed: {}", result.stderr());
        }

        let output = result.stdout();

        let is_ancestor = output.contains("valid") || output.contains("verified");
        let depth = Self::parse_field_u32(output, "depth").unwrap_or(1);

        Ok(super::LineageInfo {
            is_ancestor,
            depth,
            proof: super::LineageProof {
                lineage_id: requester.to_string(),
                depth,
                proof: Self::parse_field_bytes(output, "proof").unwrap_or_default(),
                timestamp: SystemTime::now(),
            },
        })
    }
}

impl CryptoSecurityAdapter {
    fn parse_tunnel_id(output: &str) -> Result<String> {
        Self::parse_field_str(output, "tunnel_id")
            .context("Failed to parse tunnel_id from crypto primal output")
    }

    fn parse_broadcast_key(output: &str) -> Result<Bytes> {
        let key_str = Self::parse_field_str(output, "broadcast_key")
            .context("Failed to parse broadcast_key from crypto primal output")?;
        Ok(Bytes::copy_from_slice(key_str.as_bytes()))
    }

    fn parse_field_str(output: &str, field: &str) -> Option<String> {
        output
            .lines()
            .find(|line| line.contains(field))
            .and_then(|line| line.split(':').nth(1))
            .map(|s| s.trim().to_string())
    }

    fn parse_field_u32(output: &str, field: &str) -> Option<u32> {
        Self::parse_field_str(output, field)
            .and_then(|s| s.parse().ok())
    }

    fn parse_field_bytes(output: &str, field: &str) -> Option<Bytes> {
        Self::parse_field_str(output, field)
            .map(|s| Bytes::copy_from_slice(s.as_bytes()))
    }
}

/// JSON-RPC client for the discovery capability (transport registration, health).
///
/// Discovers a primal providing the `discovery` capability at runtime, then
/// communicates via JSON-RPC. Zero hardcoded endpoints or primal names.
#[derive(Debug)]
pub struct DiscoveryRegistryClient {
    client: AtomicClient,
}

impl DiscoveryRegistryClient {
    /// Discover a discovery-capable primal for the given family via capability lookup.
    pub async fn discover(family_id: &str) -> anyhow::Result<Self> {
        let client = AtomicClient::discover_by_capability_with_opts(
            super::CAPABILITY_DISCOVERY,
            crate::atomic_client::DiscoverByCapabilityOpts {
                family_id: Some(family_id),
                ..Default::default()
            },
        )
        .await
        .context("No primal with 'discovery' capability found")?;
        Ok(Self { client })
    }

    async fn register_service(
        &self,
        service_name: &str,
        capabilities: &[&str],
        endpoint: &str,
        version: &str,
    ) -> anyhow::Result<()> {
        self.client
            .call(
                "transport.register",
                json!({
                    "service_name": service_name,
                    "capabilities": capabilities,
                    "endpoint": endpoint,
                    "version": version,
                }),
            )
            .await
            .context("transport.register RPC failed")?;
        Ok(())
    }

    async fn enable_encrypted_mode(
        &self,
        mode: &str,
        key_len: usize,
    ) -> anyhow::Result<()> {
        self.client
            .call(
                "discovery.encrypted_mode",
                json!({ "mode": mode, "key_length": key_len }),
            )
            .await
            .context("discovery.encrypted_mode RPC failed")?;
        Ok(())
    }

    async fn check_health(&self, transport_id: &str) -> anyhow::Result<serde_json::Value> {
        self.client
            .call("transport.health", json!({ "transport_id": transport_id }))
            .await
            .context("transport.health RPC failed")
    }

    async fn test_broadcast(&self) -> anyhow::Result<serde_json::Value> {
        self.client
            .call("discovery.test_broadcast", json!({}))
            .await
            .context("discovery.test_broadcast RPC failed")
    }
}

/// Mesh/discovery adapter for discovery primal operations
///
/// Implements `DiscoveryProvider` for any primal providing the `discovery` capability.
/// Uses JSON-RPC via `AtomicClient` discovered at runtime — zero hardcoded endpoints.
pub struct MeshDiscoveryAdapter {
    discovery_client: DiscoveryRegistryClient,
}

impl MeshDiscoveryAdapter {
    /// Create adapter by discovering a primal with the `discovery` capability at runtime.
    pub async fn from_discovery(family_id: &str) -> anyhow::Result<Self> {
        let discovery_client = DiscoveryRegistryClient::discover(family_id).await?;
        Ok(Self { discovery_client })
    }
}

#[async_trait]
impl DiscoveryProvider for MeshDiscoveryAdapter {
    async fn register_transport(&self, endpoint: &super::TransportEndpoint) -> Result<()> {
        info!(
            "MeshDiscoveryAdapter: Registering transport for node {}",
            endpoint.node_id
        );

        let service_name = format!("transport-{}", endpoint.node_id);
        let address = format!("{}:{}", endpoint.address, endpoint.port);

        self.discovery_client
            .register_service(&service_name, &["transport"], &address, "1.0.0")
            .await
            .context("Failed to register transport with discovery primal")?;

        info!("MeshDiscoveryAdapter: Transport registered successfully");
        Ok(())
    }

    async fn enable_encrypted_mode(&self, config: EncryptedDiscoveryConfig) -> Result<()> {
        info!(
            "MeshDiscoveryAdapter: Enabling encrypted discovery mode: {:?}",
            config.mode
        );

        let mode_str = format!("{:?}", config.mode);
        self.discovery_client
            .enable_encrypted_mode(&mode_str, config.encryption_key.len())
            .await
            .context("Failed to enable encrypted mode on discovery primal")?;

        info!("MeshDiscoveryAdapter: Encrypted discovery mode enabled");
        Ok(())
    }

    async fn check_transport_health(&self, transport_id: &str) -> Result<TransportHealth> {
        debug!(
            "MeshDiscoveryAdapter: Checking transport health for {}",
            transport_id
        );

        let response = self
            .discovery_client
            .check_health(transport_id)
            .await
            .context("Failed to query transport health from discovery primal")?;

        let status_str = response
            .get("status")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");

        let status = match status_str {
            "healthy" => super::HealthStatus::Healthy,
            "degraded" => super::HealthStatus::Degraded,
            _ => super::HealthStatus::Unhealthy,
        };

        Ok(TransportHealth {
            connection_status: status,
            latency_ms: response
                .get("latency_ms")
                .and_then(serde_json::Value::as_u64)
                .and_then(|v| u32::try_from(v).ok()),
            packet_loss: response
                .get("packet_loss")
                .and_then(serde_json::Value::as_f64)
                .map(|v| v as f32),
            status,
        })
    }

    async fn test_encrypted_broadcast(&self) -> Result<super::BroadcastTest> {
        info!("MeshDiscoveryAdapter: Testing encrypted broadcast");

        let response = self
            .discovery_client
            .test_broadcast()
            .await
            .context("Failed to test encrypted broadcast via discovery primal")?;

        let success = response
            .get("success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let encrypted = response
            .get("encrypted")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Ok(super::BroadcastTest {
            encrypted,
            timestamp: SystemTime::now(),
            success,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tunnel_id() {
        let output = "tunnel_id: btsp-123-456\nendpoints: btsp://a:9000,btsp://b:9000";
        let id = CryptoSecurityAdapter::parse_tunnel_id(output).unwrap();
        assert_eq!(id, "btsp-123-456");
    }
}

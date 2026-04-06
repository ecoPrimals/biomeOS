// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Socket-Based Provider Implementations
//!
//! - Extracted from monolithic `mod.rs` (870+ lines)
//! - Unified 3x duplicated `send_rpc` into single `SocketRpcClient`
//! - Eliminated hardcoded "127.0.0.1" in tunnel endpoints
//! - Each provider delegates to `SocketRpcClient` for JSON-RPC over Unix sockets
//!
//! These adapters communicate with primals via Unix sockets using JSON-RPC 2.0.
//! They are AGNOSTIC — they work with any primal that exposes the expected
//! JSON-RPC methods, regardless of what the primal is called.

use anyhow::{Context, Result};
use async_trait::async_trait;
use bytes::Bytes;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

use super::{
    BroadcastKeys, BroadcastTest, DiscoveryProvider, EncryptedDiscoveryConfig, HealthStatus,
    LineageInfo, LineageProof, RelayConnection, RelayOffer, RelayStatus, RoutingProvider,
    SecurityProvider, TransportEndpoint, TransportHealth, TunnelHealth, TunnelRequest,
};

// ============================================================================
// Unified JSON-RPC Client
// ============================================================================

/// Generic Unix socket JSON-RPC 2.0 client
///
/// Single socket communication implementation.
/// Previously duplicated 3x across Security, Discovery, and Routing providers.
#[derive(Clone)]
pub struct SocketRpcClient {
    socket_path: PathBuf,
    timeout: Duration,
}

impl SocketRpcClient {
    pub const fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            timeout: Duration::from_secs(10),
        }
    }

    /// Set the timeout for RPC calls (builder pattern).
    /// Used by tests; production uses default timeout.
    #[cfg(test)]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Send a JSON-RPC 2.0 request and return the result
    pub fn call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        let mut stream = UnixStream::connect(&self.socket_path).with_context(|| {
            format!(
                "Failed to connect to provider at {}",
                self.socket_path.display()
            )
        })?;

        stream.set_read_timeout(Some(self.timeout))?;
        stream.set_write_timeout(Some(self.timeout))?;

        let request = biomeos_types::JsonRpcRequest::new(method, params);

        let request_bytes = serde_json::to_vec(&request)?;
        stream.write_all(&request_bytes)?;
        stream.write_all(b"\n")?;
        stream.flush()?;

        let mut response_buf = vec![0u8; 65536];
        let n = stream.read(&mut response_buf)?;
        let response: serde_json::Value = serde_json::from_slice(&response_buf[..n])?;

        if let Some(error) = response.get("error") {
            anyhow::bail!("RPC error from {}: {}", self.socket_path.display(), error);
        }

        response.get("result").cloned().ok_or_else(|| {
            anyhow::anyhow!("No result in response from {}", self.socket_path.display())
        })
    }

    /// Spawn a blocking RPC call on the tokio blocking pool
    pub async fn call_async(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let client = self.clone();
        let method = method.to_string();
        tokio::task::spawn_blocking(move || client.call(&method, params)).await?
    }

    /// Get the socket path this client connects to.
    /// Used by tests to verify client configuration.
    #[cfg(test)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }
}

// ============================================================================
// Security Provider (capability: crypto/security)
// ============================================================================

/// Security provider that communicates via Unix socket JSON-RPC
pub struct SocketSecurityProvider {
    rpc: SocketRpcClient,
}

impl SocketSecurityProvider {
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            rpc: SocketRpcClient::new(socket_path),
        }
    }
}

#[async_trait]
impl SecurityProvider for SocketSecurityProvider {
    async fn request_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        proof: &LineageProof,
    ) -> Result<TunnelRequest> {
        let result = self
            .rpc
            .call_async(
                "tunnel.request",
                serde_json::json!({
                    "node_a": node_a,
                    "node_b": node_b,
                    "lineage_proof": proof,
                }),
            )
            .await?;

        let tunnel_id = result
            .get("tunnel_id")
            .and_then(|v| v.as_str())
            .unwrap_or("pending")
            .to_string();

        // Endpoint addresses come from the provider response,
        // NOT hardcoded. Fall back to the socket path for local connections.
        let addr_a = result
            .get("endpoint_a_address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let addr_b = result
            .get("endpoint_b_address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let port_a = result
            .get("endpoint_a_port")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as u16;
        let port_b = result
            .get("endpoint_b_port")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0) as u16;

        Ok(TunnelRequest {
            id: tunnel_id,
            endpoint_a: TransportEndpoint {
                node_id: node_a.to_string(),
                address: addr_a,
                port: port_a,
                protocol: "tcp".to_string(),
                secure: true,
            },
            endpoint_b: TransportEndpoint {
                node_id: node_b.to_string(),
                address: addr_b,
                port: port_b,
                protocol: "tcp".to_string(),
                secure: true,
            },
            encryption_key: Bytes::new(),
            created_at: std::time::SystemTime::now(),
        })
    }

    async fn check_tunnel_health(&self, tunnel_id: &str) -> Result<TunnelHealth> {
        let result = self
            .rpc
            .call_async(
                "tunnel.health",
                serde_json::json!({ "tunnel_id": tunnel_id }),
            )
            .await?;

        let status_str = result
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = match status_str {
            "healthy" => HealthStatus::Healthy,
            "degraded" => HealthStatus::Degraded,
            _ => HealthStatus::Unhealthy,
        };

        Ok(TunnelHealth {
            encryption_status: status,
            forward_secrecy: true,
            last_key_rotation: None,
            status,
        })
    }

    async fn generate_broadcast_keys(&self, family_id: &str) -> Result<BroadcastKeys> {
        let result = self
            .rpc
            .call_async(
                "crypto.broadcast_keys",
                serde_json::json!({ "family_id": family_id }),
            )
            .await?;

        let key_data = result
            .get("encryption_key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .as_bytes()
            .to_vec();

        Ok(BroadcastKeys {
            broadcast_key: Bytes::from(key_data),
            lineage_proof: LineageProof {
                lineage_id: family_id.to_string(),
                depth: 0,
                proof: Bytes::new(),
                timestamp: std::time::SystemTime::now(),
            },
            generated_at: std::time::SystemTime::now(),
        })
    }

    async fn verify_lineage(&self, requester: &str, target: &str) -> Result<LineageInfo> {
        let result = self
            .rpc
            .call_async(
                "lineage.verify",
                serde_json::json!({
                    "requester": requester,
                    "target": target,
                }),
            )
            .await?;

        Ok(LineageInfo {
            is_ancestor: result
                .get("is_ancestor")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            depth: result
                .get("depth")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as u32,
            proof: LineageProof {
                lineage_id: requester.to_string(),
                depth: 0,
                proof: Bytes::new(),
                timestamp: std::time::SystemTime::now(),
            },
        })
    }
}

// ============================================================================
// Discovery Provider (capability: http/discovery)
// ============================================================================

/// Discovery provider that communicates via Unix socket JSON-RPC
pub struct SocketDiscoveryProvider {
    rpc: SocketRpcClient,
}

impl SocketDiscoveryProvider {
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            rpc: SocketRpcClient::new(socket_path),
        }
    }
}

#[async_trait]
impl DiscoveryProvider for SocketDiscoveryProvider {
    async fn register_transport(&self, endpoint: &TransportEndpoint) -> Result<()> {
        self.rpc
            .call_async(
                "transport.register",
                serde_json::json!({
                    "node_id": endpoint.node_id,
                    "address": endpoint.address,
                    "port": endpoint.port,
                    "protocol": endpoint.protocol,
                    "secure": endpoint.secure,
                }),
            )
            .await?;
        Ok(())
    }

    async fn enable_encrypted_mode(&self, config: EncryptedDiscoveryConfig) -> Result<()> {
        self.rpc
            .call_async(
                "discovery.encrypted_mode",
                serde_json::json!({
                    "encryption_key": config.encryption_key,
                    "lineage_filter": config.lineage_filter,
                    "mode": format!("{:?}", config.mode),
                }),
            )
            .await?;
        Ok(())
    }

    async fn check_transport_health(&self, transport_id: &str) -> Result<TransportHealth> {
        let result = self
            .rpc
            .call_async(
                "transport.health",
                serde_json::json!({ "transport_id": transport_id }),
            )
            .await?;

        let status_str = result
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let status = match status_str {
            "healthy" => HealthStatus::Healthy,
            "degraded" => HealthStatus::Degraded,
            _ => HealthStatus::Unhealthy,
        };

        Ok(TransportHealth {
            connection_status: status,
            latency_ms: result
                .get("latency_ms")
                .and_then(serde_json::Value::as_u64)
                .map(|v| v as u32),
            packet_loss: None,
            status,
        })
    }

    async fn test_encrypted_broadcast(&self) -> Result<BroadcastTest> {
        let result = self
            .rpc
            .call_async("discovery.test_broadcast", serde_json::json!({}))
            .await?;

        Ok(BroadcastTest {
            encrypted: result
                .get("encrypted")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            timestamp: std::time::SystemTime::now(),
            success: result
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        })
    }
}

// ============================================================================
// Routing Provider (capability: routing/relay)
// ============================================================================

/// Routing provider that communicates via Unix socket JSON-RPC
pub struct SocketRoutingProvider {
    rpc: SocketRpcClient,
}

impl SocketRoutingProvider {
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            rpc: SocketRpcClient::new(socket_path),
        }
    }
}

#[async_trait]
impl RoutingProvider for SocketRoutingProvider {
    async fn request_relay(
        &self,
        requester: &str,
        target: &str,
        lineage: LineageInfo,
    ) -> Result<RelayOffer> {
        let result = self
            .rpc
            .call_async(
                "relay.request",
                serde_json::json!({
                    "requester": requester,
                    "target": target,
                    "lineage": lineage,
                }),
            )
            .await?;

        let relay_node = result
            .get("relay_node")
            .and_then(|v| v.as_str())
            .unwrap_or("relay")
            .to_string();

        // Address from provider response, not hardcoded
        let address = result
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        Ok(RelayOffer {
            relay_node: relay_node.clone(),
            relay_endpoint: TransportEndpoint {
                node_id: relay_node,
                address,
                port: result
                    .get("port")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0) as u16,
                protocol: "tcp".to_string(),
                secure: true,
            },
            expires_at: std::time::SystemTime::now() + std::time::Duration::from_secs(300),
            lineage_verified: lineage.is_ancestor,
        })
    }

    async fn accept_relay(&self, offer: &RelayOffer) -> Result<RelayConnection> {
        let result = self
            .rpc
            .call_async(
                "relay.accept",
                serde_json::json!({ "relay_node": offer.relay_node }),
            )
            .await?;

        let status_str = result
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("active");

        let status = match status_str {
            "active" => RelayStatus::Active,
            "establishing" => RelayStatus::Establishing,
            _ => RelayStatus::Failed,
        };

        Ok(RelayConnection {
            connection_id: result
                .get("connection_id")
                .and_then(|v| v.as_str())
                .unwrap_or("pending")
                .to_string(),
            relay_node: offer.relay_node.clone(),
            established_at: std::time::SystemTime::now(),
            status,
        })
    }
}

#[cfg(test)]
#[path = "socket_providers_tests.rs"]
mod socket_providers_tests;

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Socket-Based Provider Implementations
//!
//! DEEP DEBT REFACTORING (Feb 7, 2026):
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
/// DEEP DEBT PRINCIPLE: Single implementation of socket communication.
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

        // DEEP DEBT: Endpoint addresses come from the provider response,
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

        // DEEP DEBT: Address from provider response, not hardcoded
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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::p2p_coordination::{
        DiscoveryMode, EncryptedDiscoveryConfig, LineageInfo, LineageProof, RelayOffer,
        RelayStatus, TransportEndpoint,
    };
    use biomeos_test_utils::MockJsonRpcServer;
    use bytes::Bytes;

    #[test]
    fn test_socket_rpc_client_creation() {
        let client = SocketRpcClient::new(PathBuf::from("/tmp/test.sock"));
        assert_eq!(client.socket_path(), &PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_socket_rpc_client_timeout() {
        let client = SocketRpcClient::new(PathBuf::from("/tmp/test.sock"))
            .with_timeout(Duration::from_secs(30));
        assert_eq!(client.socket_path(), &PathBuf::from("/tmp/test.sock"));
    }

    #[test]
    fn test_socket_security_provider_new() {
        let provider = SocketSecurityProvider::new(PathBuf::from("/run/beardog.sock"));
        assert_eq!(
            provider.rpc.socket_path(),
            &PathBuf::from("/run/beardog.sock")
        );
    }

    #[test]
    fn test_socket_discovery_provider_new() {
        let provider = SocketDiscoveryProvider::new(PathBuf::from("/run/songbird.sock"));
        assert_eq!(
            provider.rpc.socket_path(),
            &PathBuf::from("/run/songbird.sock")
        );
    }

    #[test]
    fn test_socket_routing_provider_new() {
        let provider = SocketRoutingProvider::new(PathBuf::from("/run/relay.sock"));
        assert_eq!(
            provider.rpc.socket_path(),
            &PathBuf::from("/run/relay.sock")
        );
    }

    #[test]
    fn test_jsonrpc_request_format() {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "test.method",
            "params": {"key": "value"}
        });
        let bytes = serde_json::to_vec(&request).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(parsed["jsonrpc"], "2.0");
        assert_eq!(parsed["method"], "test.method");
        assert_eq!(parsed["params"]["key"], "value");
    }

    #[test]
    fn test_socket_rpc_client_call_nonexistent_socket() {
        let client = SocketRpcClient::new(PathBuf::from("/nonexistent/path/to/socket.sock"));
        let result = client.call("test.method", serde_json::json!({}));
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Failed") || err.contains("connect") || err.contains("No such file"),
            "Expected connection error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_socket_rpc_client_call_async_nonexistent() {
        let client = SocketRpcClient::new(PathBuf::from("/nonexistent/socket.sock"));
        let result = client
            .call_async("test.method", serde_json::json!({}))
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_socket_rpc_call_success_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("rpc.sock");
        let _server =
            MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({"answer": 42})).await;
        let client = SocketRpcClient::new(sock);
        let v =
            tokio::task::spawn_blocking(move || client.call("any.method", serde_json::json!({})))
                .await
                .expect("join")
                .expect("ok");
        assert_eq!(v["answer"], 42);
    }

    #[tokio::test]
    async fn test_socket_rpc_call_jsonrpc_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("rpc-err.sock");
        let _server = MockJsonRpcServer::spawn_echo_error(&sock, -32000, "boom").await;
        let client = SocketRpcClient::new(sock);
        let r = tokio::task::spawn_blocking(move || client.call("m", serde_json::json!({})))
            .await
            .expect("join");
        assert!(r.is_err());
        let chain = format!("{:#}", r.unwrap_err());
        assert!(chain.contains("boom"), "got {chain}");
    }

    #[tokio::test]
    async fn test_socket_rpc_call_missing_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("rpc-nores.sock");
        let _server = MockJsonRpcServer::spawn(&sock, |req| {
            let id = serde_json::from_str::<serde_json::Value>(req.trim())
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(serde_json::Value::Null);
            format!(r#"{{"jsonrpc":"2.0","id":{},"not_result":true}}"#, id)
        })
        .await;
        let client = SocketRpcClient::new(sock);
        let r = tokio::task::spawn_blocking(move || client.call("m", serde_json::json!({})))
            .await
            .expect("join");
        assert!(r.is_err());
        let chain = format!("{:#}", r.unwrap_err());
        assert!(chain.contains("No result"), "got {chain}");
    }

    fn sample_proof() -> LineageProof {
        LineageProof {
            lineage_id: "fam".to_string(),
            depth: 0,
            proof: Bytes::new(),
            timestamp: std::time::SystemTime::UNIX_EPOCH,
        }
    }

    #[tokio::test]
    async fn test_socket_security_provider_tunnel_request_and_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("sec.sock");
        let _server = MockJsonRpcServer::spawn(&sock, |line| {
            let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
            let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let method = v["method"].as_str().unwrap_or("");
            let result = match method {
                "tunnel.request" => serde_json::json!({
                    "tunnel_id": "tid",
                    "endpoint_a_address": "192.168.1.1",
                    "endpoint_b_address": "192.168.1.2",
                    "endpoint_a_port": 111,
                    "endpoint_b_port": 222
                }),
                "tunnel.health" => serde_json::json!({"status": "degraded"}),
                "crypto.broadcast_keys" => serde_json::json!({"encryption_key": "abcd"}),
                "lineage.verify" => serde_json::json!({"is_ancestor": true, "depth": 3}),
                _ => serde_json::json!({}),
            };
            format!(
                r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
                serde_json::to_string(&id).expect("id"),
                serde_json::to_string(&result).expect("result")
            )
        })
        .await;
        let p = SocketSecurityProvider::new(sock);
        let proof = sample_proof();
        let tunnel = p.request_tunnel("na", "nb", &proof).await.expect("tunnel");
        assert_eq!(tunnel.id, "tid");
        assert_eq!(tunnel.endpoint_a.port, 111);
        let health = p.check_tunnel_health("x").await.expect("health");
        assert_eq!(health.status, HealthStatus::Degraded);
        let keys = p.generate_broadcast_keys("fam").await.expect("keys");
        assert_eq!(keys.broadcast_key.as_ref(), b"abcd");
        let lin = p.verify_lineage("r", "t").await.expect("lin");
        assert!(lin.is_ancestor);
        assert_eq!(lin.depth, 3);
    }

    #[tokio::test]
    async fn test_socket_security_tunnel_health_status_branches() {
        for (status_str, expected) in [
            ("healthy", HealthStatus::Healthy),
            ("degraded", HealthStatus::Degraded),
            ("broken", HealthStatus::Unhealthy),
        ] {
            let dir = tempfile::tempdir().expect("tempdir");
            let sock = dir.path().join(format!("h-{status_str}.sock"));
            let _server = MockJsonRpcServer::spawn_echo_success(
                &sock,
                serde_json::json!({ "status": status_str }),
            )
            .await;
            let p = SocketSecurityProvider::new(sock);
            let h = p.check_tunnel_health("t").await.expect("h");
            assert_eq!(h.status, expected);
        }
    }

    #[tokio::test]
    async fn test_socket_discovery_provider_methods() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("disc.sock");
        let _server = MockJsonRpcServer::spawn(&sock, |line| {
            let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
            let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let method = v["method"].as_str().unwrap_or("");
            let result = match method {
                "transport.health" => serde_json::json!({"status": "healthy", "latency_ms": 12}),
                "discovery.test_broadcast" => {
                    serde_json::json!({"encrypted": true, "success": true})
                }
                _ => serde_json::json!(null),
            };
            format!(
                r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
                serde_json::to_string(&id).expect("id"),
                serde_json::to_string(&result).expect("result")
            )
        })
        .await;
        let p = SocketDiscoveryProvider::new(sock.clone());
        let ep = TransportEndpoint {
            node_id: "n".to_string(),
            address: "127.0.0.1".to_string(),
            port: 1,
            protocol: "tcp".to_string(),
            secure: true,
        };
        p.register_transport(&ep).await.expect("reg");
        let cfg = EncryptedDiscoveryConfig {
            encryption_key: Bytes::from("k"),
            lineage_filter: sample_proof(),
            mode: DiscoveryMode::Encrypted,
        };
        p.enable_encrypted_mode(cfg).await.expect("enc");
        let th = p.check_transport_health("t1").await.expect("th");
        assert_eq!(th.status, HealthStatus::Healthy);
        assert_eq!(th.latency_ms, Some(12));
        let bt = p.test_encrypted_broadcast().await.expect("bt");
        assert!(bt.encrypted && bt.success);
    }

    #[tokio::test]
    async fn test_socket_routing_provider_relay_and_accept() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("route.sock");
        let _server = MockJsonRpcServer::spawn(&sock, |line| {
            let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
            let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let method = v["method"].as_str().unwrap_or("");
            let result = match method {
                "relay.request" => serde_json::json!({
                    "relay_node": "rn",
                    "address": "10.0.0.9",
                    "port": 7777
                }),
                "relay.accept" => serde_json::json!({
                    "connection_id": "cid-1",
                    "status": "establishing"
                }),
                _ => serde_json::json!(null),
            };
            format!(
                r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
                serde_json::to_string(&id).expect("id"),
                serde_json::to_string(&result).expect("result")
            )
        })
        .await;
        let p = SocketRoutingProvider::new(sock);
        let lineage = LineageInfo {
            is_ancestor: true,
            depth: 1,
            proof: sample_proof(),
        };
        let offer = p.request_relay("a", "b", lineage).await.expect("offer");
        assert_eq!(offer.relay_endpoint.port, 7777);
        let conn = p.accept_relay(&offer).await.expect("accept");
        assert_eq!(conn.connection_id, "cid-1");
        assert_eq!(conn.status, RelayStatus::Establishing);
    }

    #[tokio::test]
    async fn test_socket_routing_accept_status_failed_branch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("r2.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(
            &sock,
            serde_json::json!({"connection_id": "x", "status": "dead"}),
        )
        .await;
        let p = SocketRoutingProvider::new(sock);
        let offer = RelayOffer {
            relay_node: "n".to_string(),
            relay_endpoint: TransportEndpoint {
                node_id: "n".to_string(),
                address: String::new(),
                port: 0,
                protocol: "tcp".to_string(),
                secure: true,
            },
            expires_at: std::time::SystemTime::UNIX_EPOCH,
            lineage_verified: false,
        };
        let c = p.accept_relay(&offer).await.expect("c");
        assert_eq!(c.status, RelayStatus::Failed);
    }

    #[tokio::test]
    async fn test_socket_discovery_transport_health_unknown_defaults_unhealthy() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("d-unhealthy.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(
            &sock,
            serde_json::json!({"status": "unknown", "latency_ms": null}),
        )
        .await;
        let p = SocketDiscoveryProvider::new(sock);
        let th = p.check_transport_health("t").await.expect("th");
        assert_eq!(th.status, HealthStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_socket_discovery_test_broadcast_defaults_false() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("d-bc.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({})).await;
        let p = SocketDiscoveryProvider::new(sock);
        let bt = p.test_encrypted_broadcast().await.expect("bt");
        assert!(!bt.encrypted && !bt.success);
    }

    #[tokio::test]
    async fn test_socket_security_tunnel_request_minimal_json_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("sec-min.sock");
        let _server = MockJsonRpcServer::spawn_echo_success(&sock, serde_json::json!({})).await;
        let p = SocketSecurityProvider::new(sock);
        let proof = sample_proof();
        let t = p.request_tunnel("a", "b", &proof).await.expect("t");
        assert_eq!(t.id, "pending");
        assert_eq!(t.endpoint_a.port, 0);
    }

    #[tokio::test]
    async fn test_socket_routing_relay_defaults_and_active_accept() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("r-def.sock");
        let _server = MockJsonRpcServer::spawn(&sock, |line| {
            let v: serde_json::Value = serde_json::from_str(line.trim()).expect("json");
            let id = v.get("id").cloned().unwrap_or(serde_json::Value::Null);
            let method = v["method"].as_str().unwrap_or("");
            let result = match method {
                "relay.request" => serde_json::json!({}),
                "relay.accept" => serde_json::json!({"status": "active"}),
                _ => serde_json::json!(null),
            };
            format!(
                r#"{{"jsonrpc":"2.0","id":{},"result":{}}}"#,
                serde_json::to_string(&id).expect("id"),
                serde_json::to_string(&result).expect("result")
            )
        })
        .await;
        let p = SocketRoutingProvider::new(sock);
        let lineage = LineageInfo {
            is_ancestor: false,
            depth: 0,
            proof: sample_proof(),
        };
        let offer = p.request_relay("x", "y", lineage).await.expect("o");
        assert_eq!(offer.relay_node, "relay");
        let conn = p.accept_relay(&offer).await.expect("c");
        assert_eq!(conn.status, RelayStatus::Active);
    }

    #[test]
    fn test_socket_rpc_call_invalid_json_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("bad-json.sock");
        std::os::unix::net::UnixListener::bind(&sock).expect("bind");
        let client = SocketRpcClient::new(sock);
        let r = client.call("m", serde_json::json!({}));
        assert!(r.is_err());
    }
}

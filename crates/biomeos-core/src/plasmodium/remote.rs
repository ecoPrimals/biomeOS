// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Remote gate queries via HTTP JSON-RPC (Songbird gateway).

use anyhow::Result;
use biomeos_types::primal_names;
use serde_json::{Value, json};

use crate::atomic_client::AtomicClient;

use super::types::{BondType, ComputeInfo, GateInfo, PrimalStatus};

/// Parse `host:port` from a mesh peer address; uses `default_port` when no port is present or when
/// the port segment fails to parse as `u16`.
#[must_use]
pub(crate) fn parse_mesh_peer_address(address: &str, default_port: u16) -> (String, u16) {
    if let Some(idx) = address.rfind(':') {
        let h = &address[..idx];
        let p = address[idx + 1..].parse::<u16>().unwrap_or(default_port);
        (h.to_string(), p)
    } else {
        (address.to_string(), default_port)
    }
}

impl super::Plasmodium {
    /// Query a remote gate's NUCLEUS status via HTTP JSON-RPC gateway
    ///
    /// Uses HTTP POST to `/jsonrpc` on the remote discovery provider.
    /// The port is runtime-discovered from the `mesh.peers` response
    /// (beacon exchange), with env var and constants as fallbacks.
    pub(super) async fn query_remote_gate(&self, address: &str, node_id: &str) -> Result<GateInfo> {
        let default_port: u16 = std::env::var("SONGBIRD_MESH_PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(biomeos_types::constants::network::DEFAULT_HTTP_PORT);

        let (host, port) = parse_mesh_peer_address(address, default_port);

        // Use HTTP JSON-RPC gateway (covalent bond transport)
        let client = AtomicClient::http(&host, port);

        // Query health
        let health_result: Result<Value> = client.call("health", json!({})).await;
        let reachable = health_result.is_ok();

        if !reachable {
            anyhow::bail!("Gate {node_id} not reachable at {host}:{port}");
        }

        // Query remote primals
        let primals = Self::query_remote_primals(&client).await;

        Ok(GateInfo {
            gate_id: node_id.to_string(),
            address: address.to_string(),
            is_local: false,
            primals,
            compute: ComputeInfo::default(),
            models: vec![],
            load: 0.0,
            reachable: true,
            bond_type: BondType::Covalent,
        })
    }

    /// Query remote primals via Songbird TCP
    pub(super) async fn query_remote_primals(client: &AtomicClient) -> Vec<PrimalStatus> {
        let mut primals = Vec::new();

        // Try lifecycle.status first (if neural API is running)
        if let Ok(result) = client.call("lifecycle.status", json!({})).await
            && let Some(services) = result.get("services").and_then(|s| s.as_object())
        {
            for (name, status) in services {
                primals.push(PrimalStatus {
                    name: name.clone(),
                    healthy: status
                        .get("status")
                        .and_then(|s| s.as_str())
                        .is_some_and(|s| s == "healthy"),
                    version: status
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(std::string::ToString::to_string),
                });
            }
        }

        if primals.is_empty() {
            let discovery_provider = std::env::var("DISCOVERY_PROVIDER")
                .unwrap_or_else(|_| primal_names::SONGBIRD.to_string());
            primals.push(PrimalStatus {
                name: discovery_provider,
                healthy: true,
                version: None,
            });
        }

        primals
    }
}

#[cfg(test)]
mod tests {
    use super::parse_mesh_peer_address;

    #[test]
    fn parse_mesh_peer_host_and_port() {
        let (h, p) = parse_mesh_peer_address("10.0.0.1:9443", 8080);
        assert_eq!(h, "10.0.0.1");
        assert_eq!(p, 9443);
    }

    #[test]
    fn parse_mesh_peer_no_port_uses_default() {
        let (h, p) = parse_mesh_peer_address("gateway.local", 8080);
        assert_eq!(h, "gateway.local");
        assert_eq!(p, 8080);
    }

    #[test]
    fn parse_mesh_peer_invalid_port_uses_default() {
        let (h, p) = parse_mesh_peer_address("host:999999", 3000);
        assert_eq!(h, "host");
        assert_eq!(p, 3000);
    }

    #[test]
    fn parse_mesh_peer_last_colon_splits_port() {
        let (h, p) = parse_mesh_peer_address("a:b:c:9000", 1);
        assert_eq!(h, "a:b:c");
        assert_eq!(p, 9000);
    }
}

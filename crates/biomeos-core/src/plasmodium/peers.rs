// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Songbird mesh peer discovery and `PLASMODIUM_PEERS` env merge.

use crate::atomic_client::AtomicClient;

use serde_json::json;
use tracing::debug;

use super::types::PeerInfo;

impl super::Plasmodium {
    /// Discover peers via Songbird mesh + `PLASMODIUM_PEERS` env var
    pub(crate) async fn discover_peers(&self) -> Vec<PeerInfo> {
        let mut peers = Vec::new();

        let discovery_provider = std::env::var("DISCOVERY_PROVIDER")
            .unwrap_or_else(|_| biomeos_types::primal_names::SONGBIRD.to_string());
        if let Ok(client) = AtomicClient::discover(&discovery_provider).await {
            if let Ok(result) = client.call("mesh.peers", json!({})).await {
                let peers_array = result
                    .get("peers")
                    .and_then(|p| p.as_array())
                    .cloned()
                    .unwrap_or_default();

                for peer_val in peers_array {
                    if let (Some(node_id), Some(address)) = (
                        peer_val.get("node_id").and_then(|n| n.as_str()),
                        peer_val.get("address").and_then(|a| a.as_str()),
                    ) {
                        peers.push(PeerInfo {
                            node_id: node_id.to_string(),
                            address: address.to_string(),
                        });
                    }
                }
            } else {
                debug!("mesh.peers call failed, falling back to env var");
            }
        } else {
            debug!(
                "Discovery provider '{discovery_provider}' not available, \
                 using PLASMODIUM_PEERS for peer discovery"
            );
        }

        // Always merge PLASMODIUM_PEERS env var (supplements Songbird mesh)
        // Format: node_id@host:port  or  node_id@ssh:user@host
        let peer_list_opt = self
            .peers_override
            .clone()
            .or_else(|| std::env::var("PLASMODIUM_PEERS").ok());

        if let Some(ref peer_list) = peer_list_opt {
            for peer_str in peer_list.split(',') {
                let peer_str = peer_str.trim();
                if peer_str.is_empty() {
                    continue;
                }

                let parts: Vec<&str> = peer_str.splitn(2, '@').collect();
                if parts.len() == 2 {
                    let node_id = parts[0].to_string();
                    let address = parts[1].to_string();
                    // Don't add duplicates
                    if !peers.iter().any(|p| p.node_id == node_id) {
                        peers.push(PeerInfo { node_id, address });
                    }
                } else {
                    // Just an IP/hostname
                    if !peers.iter().any(|p| p.node_id == parts[0]) {
                        peers.push(PeerInfo {
                            node_id: parts[0].to_string(),
                            address: parts[0].to_string(),
                        });
                    }
                }
            }
        }

        peers
    }
}

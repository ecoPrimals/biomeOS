// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery Operations
//!
//! Handles all service discovery operations including registry discovery,
//! network scanning, capability-based discovery, and orchestration services.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;

use super::core::PrimalInfo;
use biomeos_primal_sdk::PrimalCapability;
use biomeos_types::{BiomeOSConfig, Health, PrimalType};

use crate::family_discovery::get_family_id;
use crate::socket_discovery::SocketDiscovery;

/// Primal Discovery Service for ecosystem-wide primal discovery
#[derive(Debug, Clone)]
pub struct PrimalDiscoveryService {
    /// Discovery configuration (filters, timeouts) — reserved for future use
    _config: Arc<BiomeOSConfig>,
}

/// Discovery result from primal scanning
#[derive(Debug, Clone)]
pub struct DiscoveryResult {
    /// Primal identifier
    pub id: String,
    /// Communication endpoint
    pub endpoint: String,
    /// Primal type classification
    pub primal_type: PrimalType,
    /// Capabilities discovered
    pub capabilities: Vec<PrimalCapability>,
    /// Health status at discovery time
    pub health: Health,
    /// When this primal was discovered
    pub discovered_at: chrono::DateTime<chrono::Utc>,
}

/// Probe result from endpoint probing
#[derive(Debug, Clone)]
pub struct ProbeResult {
    /// Primal name reported by the endpoint
    pub name: String,
    /// Primal version reported by the endpoint
    pub version: String,
    /// Capabilities reported by the endpoint
    pub capabilities: Vec<PrimalCapability>,
    /// Health status reported by the endpoint
    pub health: Health,
}

impl PrimalDiscoveryService {
    /// Create new discovery service
    #[must_use]
    pub const fn new(config: Arc<BiomeOSConfig>) -> Self {
        Self { _config: config }
    }

    /// Initialize the discovery service
    pub fn initialize(&self) -> Result<()> {
        tracing::info!("🚀 Initializing Primal Discovery Service");
        Ok(())
    }
}

use super::core::UniversalBiomeOSManager;

impl UniversalBiomeOSManager {
    /// General discovery method using the 5-tier socket discovery protocol.
    ///
    /// Scans all known primal names via `SocketDiscovery::discover_with_fallback`,
    /// registering each discovered primal and returning their endpoints.
    pub async fn discover(&self) -> Result<Vec<String>> {
        tracing::info!("🌐 Starting comprehensive primal discovery via SocketDiscovery");

        let family_id = get_family_id();
        let discovery = SocketDiscovery::new(&family_id);
        let mut all_endpoints = Vec::new();

        for &primal_name in biomeos_types::primal_names::CORE_PRIMALS {
            if let Some(endpoint) = discovery.discover_with_fallback(primal_name).await {
                let ep_str = endpoint.to_string();
                tracing::debug!("Discovered {primal_name} at {ep_str}");

                let primal_info = PrimalInfo {
                    id: primal_name.to_string(),
                    name: primal_name.to_string(),
                    primal_type: PrimalType::from_discovered("core", primal_name, "1.0"),
                    endpoint: ep_str.clone(),
                    capabilities: vec![],
                    health: Health::unknown("Discovered, not yet probed"),
                    last_seen: chrono::Utc::now(),
                    discovered_at: chrono::Utc::now(),
                    metadata: HashMap::new(),
                };

                let _ = self.register_primal(primal_info).await;
                all_endpoints.push(ep_str);
            }
        }

        tracing::info!("Discovery complete: {} primals found", all_endpoints.len());

        Ok(all_endpoints)
    }

    /// Discover primals by capabilities using the 5-tier socket discovery protocol.
    ///
    /// First checks already-registered primals, then falls back to live
    /// `SocketDiscovery::discover_capability` for each requested capability.
    pub async fn discover_by_capability(
        &self,
        capabilities: &[PrimalCapability],
    ) -> Result<Vec<String>> {
        tracing::info!("🔍 Discovering primals by capabilities: {:?}", capabilities);

        let primals = self.registered_primals.read().await;
        let mut matching_ids = Vec::new();

        for (id, primal) in primals.iter() {
            let has_required_capabilities = capabilities.iter().any(|required_cap| {
                primal.capabilities.iter().any(|primal_cap| {
                    primal_cap.category == required_cap.category
                        && primal_cap.name == required_cap.name
                })
            });

            if has_required_capabilities {
                matching_ids.push(id.clone());
            }
        }
        drop(primals);

        if matching_ids.is_empty() {
            let family_id = get_family_id();
            let discovery = SocketDiscovery::new(&family_id);

            for cap in capabilities {
                if let Some(socket) = discovery.discover_capability(&cap.category).await {
                    let ep_str = socket.endpoint.to_string();
                    tracing::debug!(
                        "Socket discovery found {} for capability {}",
                        ep_str,
                        cap.category
                    );
                    matching_ids.push(ep_str);
                }
            }
        }

        tracing::info!(
            "Capability-based discovery found {} matching primals",
            matching_ids.len()
        );
        Ok(matching_ids)
    }

    /// Discover all services using all available methods
    pub async fn discover_all_services(&self) -> Result<HashMap<String, serde_json::Value>> {
        let endpoints = self.discover().await?;
        let mut services = HashMap::new();

        let primals = self.registered_primals.read().await;
        for (id, primal) in primals.iter() {
            if endpoints.contains(&primal.endpoint) {
                services.insert(
                    id.clone(),
                    serde_json::json!({
                        "name": primal.name,
                        "type": primal.primal_type,
                        "endpoint": primal.endpoint,
                        "health": primal.health,
                        "capabilities": primal.capabilities,
                        "last_seen": primal.last_seen,
                    }),
                );
            }
        }

        Ok(services)
    }

    /// Discover services via DNS
    ///
    /// DNS-SD over mDNS (`_biomeos._tcp.local`), with bounded LAN TCP fallback and
    /// `health.liveness` verification (newline-delimited JSON-RPC).
    pub async fn discover_via_dns(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🌐 DNS-SD discovery (mDNS _biomeos._tcp.local)");
        let map = dns_sd::discover_dns_sd_services().await;
        Ok(map)
    }

    /// Discover services by capabilities
    pub async fn discover_by_capabilities(
        &self,
        capabilities: &[PrimalCapability],
    ) -> Result<HashMap<String, serde_json::Value>> {
        let endpoints = self.discover_by_capability(capabilities).await?;
        let mut services = HashMap::new();

        let primals = self.registered_primals.read().await;
        for (id, primal) in primals.iter() {
            if endpoints.contains(&primal.endpoint) {
                services.insert(
                    id.clone(),
                    serde_json::json!({
                        "name": primal.name,
                        "type": primal.primal_type,
                        "endpoint": primal.endpoint,
                        "health": primal.health,
                        "capabilities": primal.capabilities,
                        "matches_criteria": true,
                    }),
                );
            }
        }

        Ok(services)
    }
}

/// DNS-SD over mDNS (RFC 6762) plus bounded LAN TCP fallback.
mod dns_sd;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests;

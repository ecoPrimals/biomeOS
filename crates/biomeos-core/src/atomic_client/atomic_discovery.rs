// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal endpoint discovery by name (socket discovery + env overrides).

use anyhow::Result;
use tracing::{debug, trace};

use crate::socket_discovery::{SocketDiscovery, TransportEndpoint};

use super::{DiscoverByCapabilityOpts, DiscoverOpts};

pub(crate) fn family_id_for_discovery(family_id_override: Option<&str>) -> String {
    if let Some(id) = family_id_override {
        return id.to_string();
    }
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("NODE_FAMILY_ID"))
        .unwrap_or_else(|_| {
            trace!("No FAMILY_ID set, using 'default' for discovery");
            "default".to_string()
        })
}

pub(crate) fn strict_discovery_from_env_or_override(strict_override: Option<bool>) -> bool {
    strict_override.unwrap_or_else(|| std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok())
}

/// Resolve a primal by name to a transport endpoint (Tier 1 / Tier 2 fallback).
pub(crate) async fn discover_named_endpoint(
    primal_name: &str,
    opts: DiscoverOpts<'_>,
) -> Result<TransportEndpoint> {
    debug!("Discovering primal with fallback: {}", primal_name);

    let family_id = family_id_for_discovery(opts.family_id);

    let discovery = SocketDiscovery::new(&family_id);

    match discovery
        .discover_with_fallback_with_env_overrides(
            primal_name,
            opts.env_overrides,
            opts.tcp_tier2_override,
        )
        .await
    {
        Some(endpoint) => {
            debug!(
                "Discovered {} via {}: {}",
                primal_name,
                if endpoint.is_native() {
                    "Tier 1"
                } else {
                    "Tier 2"
                },
                endpoint
            );

            Ok(endpoint)
        }
        None => {
            anyhow::bail!(
                "Primal '{}' not found via any transport. Try:\n\
                 1. Set {}_SOCKET=/path/to/{}.sock (Unix)\n\
                 2. Set {}_TCP=host:port (TCP)\n\
                 3. Ensure primal is running in family: {}",
                primal_name,
                primal_name.to_uppercase(),
                primal_name,
                primal_name.to_uppercase(),
                family_id
            )
        }
    }
}

/// Registry lookup for capability-based discovery (no taxonomy).
pub(crate) async fn discover_capability_registry_endpoint(
    capability: &str,
    family_id_override: Option<&str>,
) -> Option<TransportEndpoint> {
    let family_id = family_id_for_discovery(family_id_override);
    let discovery = SocketDiscovery::new(&family_id);
    if let Some(socket) = discovery.discover_capability(capability).await {
        debug!(
            "Discovered capability {} via registry: {}",
            capability,
            socket.endpoint.display_string()
        );
        Some(socket.endpoint)
    } else {
        None
    }
}

/// Taxonomy bootstrap path: strict flag and optional primal name from capability string.
pub(crate) fn taxonomy_primal_for_capability(
    opts: &DiscoverByCapabilityOpts<'_>,
    capability: &str,
) -> Option<String> {
    if strict_discovery_from_env_or_override(opts.strict_discovery) {
        return None;
    }
    biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability).map(String::from)
}

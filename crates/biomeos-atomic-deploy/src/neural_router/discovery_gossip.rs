// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Cross-gate capability discovery via swarmVine gossip table.
//!
//! When local discovery fails, queries the local swarmVine instance's gossip
//! table for `capability.advertise:{gate}:{primal}` entries that match the
//! requested capability. Returns gate+primal info for targeted mesh routing.

use biomeos_core::TransportEndpoint;
use serde_json::{Value, json};
use tracing::{debug, trace};

use super::NeuralRouter;
use crate::capability_translation::resolve_primal_socket;

/// Result of a gossip table lookup for cross-gate capability resolution.
#[derive(Debug, Clone)]
pub struct GossipCapabilityHint {
    /// The remote gate that advertises this capability.
    pub gate: String,
    /// The primal on that gate that provides it.
    pub primal: String,
    /// The original capability name that was queried (used in trace logging and assertions).
    #[expect(dead_code, reason = "semantic context for trace logging; used in test assertions")]
    pub capability: String,
}

/// Parse a gossip.query response JSON into a capability hint.
///
/// Extracted as a free function for testability without requiring a `NeuralRouter`.
fn parse_gossip_entries(capability: &str, response: &Value) -> Option<GossipCapabilityHint> {
    let entries = response.get("entries")?.as_array()?;

    for entry in entries {
        let key = entry.get("key")?.as_str()?;
        let value = entry.get("value")?.as_str().unwrap_or("");

        let cap_domain = capability.split('.').next().unwrap_or(capability);
        if !value.contains(capability) && !value.contains(cap_domain) {
            continue;
        }

        // Parse key: "capability.advertise:{gate}:{primal}"
        let suffix = key.strip_prefix("capability.advertise:")?;
        let (gate, primal) = suffix.split_once(':')?;

        debug!("Gossip hit: {capability} → {primal} @ {gate}");

        return Some(GossipCapabilityHint {
            gate: gate.to_string(),
            primal: primal.to_string(),
            capability: capability.to_string(),
        });
    }

    None
}

impl NeuralRouter {
    /// Query the local swarmVine gossip table for cross-gate capability providers.
    ///
    /// Looks for gossip entries under the `tower` topic with keys matching
    /// `capability.advertise:{gate}:{primal}` whose values contain the queried
    /// capability name. Returns `None` if swarmVine is unavailable or no match.
    pub(crate) async fn try_gossip_capability_lookup(
        &self,
        capability: &str,
    ) -> Option<GossipCapabilityHint> {
        let family_id = biomeos_types::env_config::family_id().unwrap_or_default();
        let swarmvine_socket = resolve_primal_socket("swarmvine", &family_id);

        if swarmvine_socket.is_empty() {
            trace!("swarmVine socket not resolved, skipping gossip lookup");
            return None;
        }

        let socket_path = std::path::PathBuf::from(&swarmvine_socket);
        if !socket_path.exists() {
            trace!("swarmVine socket not present at {swarmvine_socket}, skipping gossip lookup");
            return None;
        }

        let endpoint = TransportEndpoint::UnixSocket { path: socket_path };

        let query_params = json!({
            "topic": "tower",
            "key_prefix": "capability.advertise:",
            "value_contains": capability,
        });

        let result = self
            .forward_request_with_timeout(
                &endpoint,
                "gossip.query",
                &query_params,
                Some(std::time::Duration::from_secs(2)),
            )
            .await;

        match result {
            Ok(response) => parse_gossip_entries(capability, &response),
            Err(e) => {
                debug!("swarmVine gossip.query failed: {e}");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_single_entry() {
        let response = json!({
            "entries": [{
                "key": "capability.advertise:ironGate:beardog",
                "value": "crypto,security,beacon"
            }]
        });

        let hint = parse_gossip_entries("crypto", &response).unwrap();
        assert_eq!(hint.gate, "ironGate");
        assert_eq!(hint.primal, "beardog");
        assert_eq!(hint.capability, "crypto");
    }

    #[test]
    fn parses_dotted_capability() {
        let response = json!({
            "entries": [{
                "key": "capability.advertise:westGate:sweetgrass",
                "value": "braid.list,braid.query,braid.head"
            }]
        });

        let hint = parse_gossip_entries("braid.query", &response).unwrap();
        assert_eq!(hint.gate, "westGate");
        assert_eq!(hint.primal, "sweetgrass");
    }

    #[test]
    fn domain_prefix_match() {
        let response = json!({
            "entries": [{
                "key": "capability.advertise:strandGate:coralreef",
                "value": "compute,workload,orchestration"
            }]
        });

        let hint = parse_gossip_entries("compute.dispatch", &response).unwrap();
        assert_eq!(hint.gate, "strandGate");
        assert_eq!(hint.primal, "coralreef");
    }

    #[test]
    fn no_match_returns_none() {
        let response = json!({
            "entries": [{
                "key": "capability.advertise:ironGate:beardog",
                "value": "crypto,security"
            }]
        });

        assert!(parse_gossip_entries("storage", &response).is_none());
    }

    #[test]
    fn empty_entries_returns_none() {
        let response = json!({ "entries": [] });
        assert!(parse_gossip_entries("anything", &response).is_none());
    }

    #[test]
    fn missing_entries_field_returns_none() {
        let response = json!({ "status": "ok" });
        assert!(parse_gossip_entries("crypto", &response).is_none());
    }

    #[test]
    fn multiple_entries_returns_first_match() {
        let response = json!({
            "entries": [
                {
                    "key": "capability.advertise:ironGate:beardog",
                    "value": "crypto,security"
                },
                {
                    "key": "capability.advertise:southGate:beardog",
                    "value": "crypto,tls"
                }
            ]
        });

        let hint = parse_gossip_entries("crypto", &response).unwrap();
        assert_eq!(hint.gate, "ironGate");
    }
}

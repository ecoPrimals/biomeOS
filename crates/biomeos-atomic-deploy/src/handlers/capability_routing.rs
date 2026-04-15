// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Routing phase markers for `capability.call` (contract + observability).

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Routing phase outcome for observability and contract compliance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoutingPhase {
    /// Phase 1: Translation lookup found a semantic mapping (or direct-route decision).
    RouteResolved {
        /// Capability domain (e.g. `crypto`).
        capability: String,
        /// Provider primal id from translation, gate label, or relay name.
        provider: String,
        /// JSON-RPC method name to invoke on the provider.
        method: String,
    },
    /// Phase 2: Endpoint resolved for the provider / capability domain.
    EndpointResolved {
        /// Primal name tied to the resolved endpoint.
        provider: String,
        /// Transport URI / display form (e.g. `unix:///...`).
        endpoint: String,
    },
    /// Phase 3: Request forwarded and response received.
    Forwarded {
        /// Wall-clock milliseconds for the full `capability.call` after routing started.
        elapsed_ms: u64,
    },
}

/// Build the `_routing_trace` JSON object for successful calls (phases + summary fields).
#[must_use]
pub fn routing_trace_value(phases: &[RoutingPhase], domain_capability: &str) -> Value {
    let mut phase_names: Vec<&'static str> = Vec::with_capacity(phases.len());
    let mut capability = domain_capability.to_string();
    let mut provider: Option<String> = None;
    let mut method: Option<String> = None;
    let mut endpoint: Option<String> = None;
    let mut elapsed_ms = 0u64;

    for p in phases {
        match p {
            RoutingPhase::RouteResolved {
                capability: cap,
                provider: pr,
                method: meth,
            } => {
                phase_names.push("route_resolved");
                capability = cap.clone();
                provider = Some(pr.clone());
                method = Some(meth.clone());
            }
            RoutingPhase::EndpointResolved {
                provider: pr,
                endpoint: ep,
            } => {
                phase_names.push("endpoint_resolved");
                provider = Some(pr.clone());
                endpoint = Some(ep.clone());
            }
            RoutingPhase::Forwarded { elapsed_ms: ms } => {
                phase_names.push("forwarded");
                elapsed_ms = *ms;
            }
        }
    }

    let mut out = serde_json::Map::new();
    out.insert(
        "phases".to_string(),
        Value::Array(phase_names.iter().map(|s| json!(s)).collect()),
    );
    out.insert("capability".to_string(), json!(capability));
    if let Some(pr) = provider {
        out.insert("provider".to_string(), json!(pr));
    }
    if let Some(m) = method {
        out.insert("method".to_string(), json!(m));
    }
    if let Some(ep) = endpoint {
        out.insert("endpoint".to_string(), json!(ep));
    }
    out.insert("elapsed_ms".to_string(), json!(elapsed_ms));
    Value::Object(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routing_phase_serde_roundtrip() {
        let samples = vec![
            RoutingPhase::RouteResolved {
                capability: "crypto".to_string(),
                provider: "beardog".to_string(),
                method: "chacha20_poly1305_encrypt".to_string(),
            },
            RoutingPhase::EndpointResolved {
                provider: "beardog".to_string(),
                endpoint: "unix:///run/x.sock".to_string(),
            },
            RoutingPhase::Forwarded { elapsed_ms: 12 },
        ];
        for p in samples {
            let s = serde_json::to_string(&p).expect("serialize");
            let back: RoutingPhase = serde_json::from_str(&s).expect("deserialize");
            assert_eq!(p, back);
        }
    }

    #[test]
    fn routing_trace_value_matches_contract_shape() {
        let v = routing_trace_value(
            &[
                RoutingPhase::RouteResolved {
                    capability: "crypto".to_string(),
                    provider: "beardog".to_string(),
                    method: "x".to_string(),
                },
                RoutingPhase::EndpointResolved {
                    provider: "beardog".to_string(),
                    endpoint: "unix:///a.sock".to_string(),
                },
                RoutingPhase::Forwarded { elapsed_ms: 9 },
            ],
            "crypto",
        );
        assert_eq!(
            v["phases"],
            json!(["route_resolved", "endpoint_resolved", "forwarded"])
        );
        assert_eq!(v["capability"], json!("crypto"));
        assert_eq!(v["elapsed_ms"], json!(9));
    }

    #[test]
    fn routing_contract_spec_readable() {
        const SPEC: &str = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../specs/CAPABILITY_CALL_ROUTING_CONTRACT.md"
        ));
        assert!(
            SPEC.contains("## Pipeline (three phases)"),
            "spec must document the pipeline"
        );
        assert!(
            SPEC.contains("## Error semantics"),
            "spec must document errors"
        );
        assert!(
            SPEC.contains("## Observability: `_routing_trace`"),
            "spec must document _routing_trace"
        );
        assert!(
            SPEC.contains("CapabilityTranslationRegistry"),
            "spec must mention translation registry"
        );
    }
}

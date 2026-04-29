// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC `capability.call` and translation listing helpers (Route→Resolve→Forward).

use super::CapabilityCallOutcome;
use super::CapabilityHandler;
use super::elapsed_ms_since;
use crate::handlers::capability_routing::{RoutingPhase, routing_trace_value};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use tracing::{debug, trace};

impl CapabilityHandler {
    /// Semantic capability call with automatic translation.
    ///
    /// JSON-RPC method: `capability.call`
    ///
    /// This is the main entry point for TRUE PRIMAL communication.
    /// Consumers use semantic names; we translate and route.
    ///
    /// # Parameters (canonical format)
    /// - `capability`: Target capability domain (e.g., "crypto")
    /// - `operation`: Semantic operation (e.g., "sha256")
    /// - `args`: Arguments for the operation
    ///
    /// # Backward-compatible formats
    /// - Dotted capability: `{ "capability": "crypto.sha256", "args": {...} }`
    ///   splits on first dot into domain + operation.
    /// - `params` alias: `{ "capability": "crypto", "operation": "sha256", "params": {...} }`
    ///   treated as `args`.
    ///
    /// # Routing trace
    /// Set `"_routing_trace": true` to receive a serialized trace in
    /// [`CapabilityCallOutcome::routing_trace`] (wired as JSON-RPC `_routing_trace` by the
    /// Neural API router). See `specs/CAPABILITY_CALL_ROUTING_CONTRACT.md`.
    pub async fn call(&self, params: &Option<Value>) -> Result<CapabilityCallOutcome> {
        let start = std::time::Instant::now();
        let params = params.as_ref().context("Missing parameters")?;
        let want_trace = params
            .get("_routing_trace")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let raw_capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;

        // Support dotted capability names: "crypto.sha256" → domain="crypto", op="sha256"
        let (capability, operation) = if let Some(explicit_op) = params["operation"].as_str() {
            (raw_capability, explicit_op.to_string())
        } else if let Some(dot_pos) = raw_capability.find('.') {
            (
                &raw_capability[..dot_pos],
                raw_capability[dot_pos + 1..].to_string(),
            )
        } else {
            anyhow::bail!(
                "Missing 'operation' field and capability '{raw_capability}' has no dotted operation"
            );
        };

        // Accept both "args" and "params" (backward compat for older callers)
        let args = params
            .get("args")
            .or_else(|| params.get("params"))
            .cloned()
            .unwrap_or(json!({}));

        // Cross-gate routing: if `gate` is specified, forward to that gate's
        // biomeOS Neural API. Fail explicitly if the gate is not registered —
        // silent fallback to local routing would break multi-gate compositions.
        if let Some(gate_name) = params["gate"].as_str() {
            if gate_name == "local" {
                trace!("capability.call: gate='local', routing locally");
            } else if let Some(remote_endpoint) = self.gate_registry.resolve(gate_name) {
                let semantic_name = format!("{capability}.{operation}");
                debug!(
                    "   Cross-gate routing: {semantic_name} → gate '{gate_name}' @ {}",
                    remote_endpoint.display_string()
                );

                let remote_call = json!({
                    "capability": capability,
                    "operation": operation,
                    "args": args,
                });

                let result = self
                    .router
                    .forward_request(remote_endpoint, "capability.call", &remote_call)
                    .await?;

                let elapsed_ms = elapsed_ms_since(start);
                trace!("   {semantic_name} completed in {elapsed_ms}ms via gate '{gate_name}'");
                let routing_trace = want_trace.then(|| {
                    routing_trace_value(
                        &[
                            RoutingPhase::RouteResolved {
                                capability: capability.to_string(),
                                provider: gate_name.to_string(),
                                method: "capability.call".to_string(),
                            },
                            RoutingPhase::EndpointResolved {
                                provider: gate_name.to_string(),
                                endpoint: remote_endpoint.display_string(),
                            },
                            RoutingPhase::Forwarded { elapsed_ms },
                        ],
                        capability,
                    )
                });
                return Ok(CapabilityCallOutcome {
                    result,
                    routing_trace,
                });
            } else {
                anyhow::bail!(
                    "Gate '{gate_name}' is not registered. \
                     Register it via graph env or route.register before targeting it. \
                     Known gates: {:?}",
                    self.gate_registry.gate_names()
                );
            }
        }

        trace!("capability.call: {}.{}", capability, &operation);

        let semantic_name = format!("{}.{}", capability, &operation);

        // Tower Atomic relay: prefer routing through Songbird when available.
        // Songbird handles BTSP handshake, correct socket resolution, and method
        // translation — resolving composition gaps from primalSpring benchScale
        // validation (BTSP rejection, socket path mismatch, method prefix).
        if let Ok(tower) = self.router.discover_tower_atomic().await {
            match self
                .router
                .forward_request(&tower.primary_endpoint, &semantic_name, &args)
                .await
            {
                Ok(value) => {
                    let elapsed_ms = elapsed_ms_since(start);
                    trace!(
                        "   ✓ {} completed in {}ms via Tower Atomic relay",
                        semantic_name, elapsed_ms
                    );
                    let routing_trace = want_trace.then(|| {
                        let relay_provider = tower
                            .primals
                            .iter()
                            .find(|p| p.endpoint == tower.primary_endpoint)
                            .map(|p| p.name.to_string())
                            .or_else(|| tower.primals.last().map(|p| p.name.to_string()))
                            .unwrap_or_else(|| "tower_atomic".to_string());
                        routing_trace_value(
                            &[
                                RoutingPhase::RouteResolved {
                                    capability: capability.to_string(),
                                    provider: "tower_atomic".to_string(),
                                    method: semantic_name.clone(),
                                },
                                RoutingPhase::EndpointResolved {
                                    provider: relay_provider,
                                    endpoint: tower.primary_endpoint.display_string(),
                                },
                                RoutingPhase::Forwarded { elapsed_ms },
                            ],
                            capability,
                        )
                    });
                    return Ok(CapabilityCallOutcome {
                        result: value,
                        routing_trace,
                    });
                }
                Err(e) => {
                    debug!(
                        "Tower Atomic relay failed for {}: {}, falling back to direct routing",
                        semantic_name, e
                    );
                }
            }
        }

        // Direct routing fallback: look up translation
        let registry = self.translation_registry.read().await;
        let translation = registry.get_translation(&semantic_name);

        match translation {
            Some(trans) => {
                debug!(
                    "   Translated: {} → {}:{}",
                    semantic_name, trans.provider, trans.actual_method
                );

                let forward_method = trans.actual_method.clone();
                let provider_from_trans = trans.provider.clone();

                drop(registry);

                let atomic = self.router.discover_capability(capability).await?;

                // Prefer the provider declared in the translation registry.
                // Without this, `providers[0]` (discovery order) wins and a
                // domain like "storage" can route to ToadStool instead of
                // NestGate when ToadStool also advertises storage capabilities.
                let (endpoint, primary_name) = if let Some(preferred) = atomic
                    .primals
                    .iter()
                    .find(|p| p.name.eq_ignore_ascii_case(&provider_from_trans))
                {
                    (preferred.endpoint.clone(), preferred.name.to_string())
                } else {
                    (
                        atomic.primary_endpoint.clone(),
                        atomic
                            .primals
                            .first()
                            .map(|p| p.name.to_string())
                            .unwrap_or_else(|| provider_from_trans.clone()),
                    )
                };

                let result = self
                    .router
                    .forward_request(&endpoint, &forward_method, &args)
                    .await?;

                let elapsed_ms = elapsed_ms_since(start);
                trace!(
                    "   ✓ {} completed in {}ms via {}",
                    semantic_name, elapsed_ms, provider_from_trans
                );

                let routing_trace = want_trace.then(|| {
                    routing_trace_value(
                        &[
                            RoutingPhase::RouteResolved {
                                capability: capability.to_string(),
                                provider: provider_from_trans,
                                method: forward_method,
                            },
                            RoutingPhase::EndpointResolved {
                                provider: primary_name,
                                endpoint: atomic.primary_endpoint.display_string(),
                            },
                            RoutingPhase::Forwarded { elapsed_ms },
                        ],
                        capability,
                    )
                });

                Ok(CapabilityCallOutcome {
                    result,
                    routing_trace,
                })
            }
            None => {
                drop(registry);
                debug!(
                    "No translation for {}, attempting direct route",
                    semantic_name
                );

                let atomic = self.router.discover_capability(capability).await?;

                // Forward just the operation: the target primal already knows
                // its own domain. Sending the full semantic_name ({domain}.{op})
                // causes method-not-found on primals that register only {op}.
                // Primals needing a specific method name register translations.
                let forward_method = operation.clone();

                let primary_name = atomic
                    .primals
                    .first()
                    .map(|p| p.name.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let result = self
                    .router
                    .forward_request(&atomic.primary_endpoint, &forward_method, &args)
                    .await?;

                let elapsed_ms = elapsed_ms_since(start);

                let routing_trace = want_trace.then(|| {
                    routing_trace_value(
                        &[
                            RoutingPhase::RouteResolved {
                                capability: capability.to_string(),
                                provider: String::new(),
                                method: forward_method,
                            },
                            RoutingPhase::EndpointResolved {
                                provider: primary_name,
                                endpoint: atomic.primary_endpoint.display_string(),
                            },
                            RoutingPhase::Forwarded { elapsed_ms },
                        ],
                        capability,
                    )
                });

                Ok(CapabilityCallOutcome {
                    result,
                    routing_trace,
                })
            }
        }
    }

    /// Discover available translations for a capability.
    ///
    /// JSON-RPC method: `capability.discover_translations`
    pub async fn discover_translations(&self, params: &Option<Value>) -> Result<Value> {
        let params = params.as_ref().context("Missing parameters")?;
        let capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;

        let registry = self.translation_registry.read().await;
        let translations = registry.provider_capabilities(capability);

        Ok(json!({
            "capability": capability,
            "translations": translations,
            "count": translations.len()
        }))
    }

    /// List all registered translations.
    ///
    /// JSON-RPC method: `capability.list_translations`
    pub async fn list_translations(&self) -> Result<Value> {
        let registry = self.translation_registry.read().await;
        let all_translations = registry.list_all();

        Ok(json!({
            "translations": all_translations.iter().map(|t| {
                json!({
                    "semantic": t.semantic,
                    "provider": t.provider,
                    "method": t.actual_method
                })
            }).collect::<Vec<_>>(),
            "count": all_translations.len()
        }))
    }
}

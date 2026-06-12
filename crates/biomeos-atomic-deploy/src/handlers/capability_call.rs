// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC `capability.call` and translation listing helpers (Route→Resolve→Forward).

use super::CapabilityCallOutcome;
use super::CapabilityHandler;
use super::elapsed_ms_since;
use crate::handlers::capability_routing::{RoutingPhase, routing_trace_value};
use crate::handlers::signal as signal_handler;
use anyhow::{Context, Result};
use serde_json::{Value, json};
use tracing::{debug, info, trace};

/// Trace label for dispatches routed through the Songbird mesh gateway.
const MESH_PROVIDER_LABEL: &str = "songbird_mesh";

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

        // JH-2: extract resource envelope from enriched params.
        // The routing layer injects `_resource_envelope` when the caller's
        // ionic token carries resource constraints.
        let envelope = params.get("_resource_envelope");

        let timeout_cap = envelope
            .and_then(|e| e.get("timeout_ms"))
            .and_then(|v| v.as_u64())
            .map(std::time::Duration::from_millis);

        // JH-2: enforce cpu/mem caps at orchestrator level. If the caller's
        // args request specific resources (cpu_cores, mem_bytes) that exceed
        // the envelope limits, reject before forwarding to the downstream
        // primal. This is a pre-dispatch guard; downstream primals (ToadStool)
        // also enforce at their compute dispatch level.
        if let Some(env) = envelope {
            let args_preview = params.get("args").or_else(|| params.get("params"));
            if let Some(args_val) = args_preview {
                if let Some(requested_mem) = args_val.get("mem_bytes").and_then(|v| v.as_u64()) {
                    if let Some(limit) = env.get("mem").and_then(|v| v.as_u64()) {
                        if requested_mem > limit {
                            anyhow::bail!(
                                "Resource envelope violation: requested mem_bytes ({requested_mem}) \
                                 exceeds token limit ({limit})"
                            );
                        }
                    }
                }
                if let Some(requested_cpu) = args_val.get("cpu_cores").and_then(|v| v.as_f64()) {
                    if let Some(limit) = env.get("cpu").and_then(|v| v.as_f64()) {
                        if requested_cpu > limit {
                            anyhow::bail!(
                                "Resource envelope violation: requested cpu_cores ({requested_cpu}) \
                                 exceeds token limit ({limit})"
                            );
                        }
                    }
                }
            }
        }

        // Accept both "args" and "params" (backward compat for older callers)
        let mut args = params
            .get("args")
            .or_else(|| params.get("params"))
            .cloned()
            .unwrap_or(json!({}));

        // JH-2: forward the resource envelope inside args so downstream
        // primals (e.g. ToadStool) can enforce cpu/mem/timeout_ms at their
        // compute dispatch level.
        if let Some(envelope) = params.get("_resource_envelope") {
            if let Some(obj) = args.as_object_mut() {
                obj.insert("_resource_envelope".to_string(), envelope.clone());
            }
        }

        // exp111: forward bearer token inside args so downstream primals in
        // enforced mode can perform their own MethodGate authorization check.
        // Without this, any primal running with BIOMEOS_AUTH_MODE=enforced
        // rejects forwarded calls with -32001 PERMISSION_DENIED.
        if let Some(token) = params.get("_bearer_token") {
            if let Some(obj) = args.as_object_mut() {
                obj.insert("_bearer_token".to_string(), token.clone());
            }
        }

        // Cross-gate routing: if `gate` is specified, forward to that gate's
        // biomeOS Neural API. Fail explicitly if the gate is not registered —
        // silent fallback to local routing would break multi-gate compositions.
        if let Some(gate_name) = params["gate"].as_str() {
            if gate_name == "local" {
                trace!("capability.call: gate='local', routing locally");
            } else if let Some(remote_endpoint) = {
                let registry = self.gate_registry.read().await;
                registry.resolve(gate_name).cloned()
            } {
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
                    .forward_request_with_timeout(
                        &remote_endpoint,
                        "capability.call",
                        &remote_call,
                        timeout_cap,
                    )
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
                // CG-8: Songbird mesh fallback for unregistered gates.
                // Forward capability.call to Songbird with routing:"any" — Songbird
                // handles local UDS + remote mesh TCP + TURN relay transparently.
                if let Some(relay_result) = self
                    .try_songbird_mesh_dispatch(capability, &operation, &args, timeout_cap)
                    .await
                {
                    let elapsed_ms = elapsed_ms_since(start);
                    let routing_trace = want_trace.then(|| {
                        routing_trace_value(
                            &[
                                RoutingPhase::RouteResolved {
                                    capability: capability.to_string(),
                                    provider: format!("relay:{gate_name}"),
                                    method: "capability.call".to_string(),
                                },
                                RoutingPhase::Forwarded { elapsed_ms },
                            ],
                            capability,
                        )
                    });
                    return Ok(CapabilityCallOutcome {
                        result: relay_result?,
                        routing_trace,
                    });
                }

                anyhow::bail!(
                    "Gate '{gate_name}' is not registered and relay fallback unavailable. \
                     Register it via graph env or route.register before targeting it. \
                     Known gates: {:?}",
                    self.gate_registry.read().await.gate_names()
                );
            }
        }

        trace!("capability.call: {}.{}", capability, &operation);

        // Layer 5 prep: record utilization for hot/cold method analysis
        let semantic_name_early = format!("{capability}.{operation}");
        self.router.record_utilization(&semantic_name_early).await;

        // Atomic signal interception: if the capability is a signal tier
        // (tower/node/nest/meta), check for a signal graph and execute it
        // instead of trying to discover a primal named "tower".
        if signal_handler::is_signal_tier(capability) {
            if let (Some(graphs_dir), Some(graph_handler)) = (&self.graphs_dir, &self.graph_handler)
            {
                let graph_path =
                    signal_handler::signal_graph_path(graphs_dir, capability, &operation);
                if graph_path.exists() {
                    info!(
                        "Signal intercept: {}.{} -> graph execution",
                        capability, operation
                    );
                    let signal_params = json!({
                        "signal": format!("{capability}.{operation}"),
                        "params": args,
                    });
                    let result = signal_handler::dispatch(
                        graphs_dir,
                        &self.family_id,
                        graph_handler,
                        &Some(signal_params),
                    )
                    .await?;
                    let elapsed_ms = elapsed_ms_since(start);
                    let routing_trace = want_trace.then(|| {
                        routing_trace_value(
                            &[
                                RoutingPhase::RouteResolved {
                                    capability: capability.to_string(),
                                    provider: "signal_graph".to_string(),
                                    method: format!("{capability}.{operation}"),
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
                }
            }
            // No signal graph found or signal dispatch not configured —
            // fall through to normal routing.
        }

        let semantic_name = format!("{}.{}", capability, &operation);

        // Tower Atomic relay: prefer routing through Songbird when available.
        // Songbird handles BTSP handshake, correct socket resolution, and method
        // translation — resolving composition gaps from primalSpring benchScale
        // validation (BTSP rejection, socket path mismatch, method prefix).
        if let Ok(tower) = self.router.discover_tower_atomic().await {
            match self
                .router
                .forward_request_with_timeout(
                    &tower.primary_endpoint,
                    &semantic_name,
                    &args,
                    timeout_cap,
                )
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

                let atomic = match self.router.discover_capability(capability).await {
                    Ok(a) => a,
                    Err(_local_err) => {
                        // Translation exists but no local provider — try Songbird mesh
                        if let Some(mesh_result) = self
                            .try_songbird_mesh_dispatch(capability, &operation, &args, timeout_cap)
                            .await
                        {
                            let elapsed_ms = elapsed_ms_since(start);
                            let routing_trace = want_trace.then(|| {
                                routing_trace_value(
                                    &[
                                        RoutingPhase::RouteResolved {
                                            capability: capability.to_string(),
                                            provider: MESH_PROVIDER_LABEL.to_string(),
                                            method: forward_method.clone(),
                                        },
                                        RoutingPhase::Forwarded { elapsed_ms },
                                    ],
                                    capability,
                                )
                            });
                            return Ok(CapabilityCallOutcome {
                                result: mesh_result?,
                                routing_trace,
                            });
                        }
                        return Err(_local_err);
                    }
                };

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
                    .forward_request_with_timeout(&endpoint, &forward_method, &args, timeout_cap)
                    .await;

                let elapsed_ms = elapsed_ms_since(start);

                // Layer 4: record dispatch outcome for adaptive routing weights
                self.router
                    .record_dispatch_outcome(capability, &primary_name, result.is_ok(), elapsed_ms)
                    .await;

                let result = result?;

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

                match self.router.discover_capability(capability).await {
                    Ok(atomic) => {
                        let forward_method = operation.clone();

                        let primary_name = atomic
                            .primals
                            .iter()
                            .find(|p| p.endpoint == atomic.primary_endpoint)
                            .or_else(|| atomic.primals.first())
                            .map(|p| p.name.to_string())
                            .unwrap_or_else(|| "unknown".to_string());

                        let result = self
                            .router
                            .forward_request_with_timeout(
                                &atomic.primary_endpoint,
                                &forward_method,
                                &args,
                                timeout_cap,
                            )
                            .await;

                        let elapsed_ms = elapsed_ms_since(start);

                        self.router
                            .record_dispatch_outcome(
                                capability,
                                &primary_name,
                                result.is_ok(),
                                elapsed_ms,
                            )
                            .await;

                        match result {
                            Ok(result) => {
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
                                                endpoint: atomic
                                                    .primary_endpoint
                                                    .display_string(),
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
                            Err(forward_err) => {
                                // Forward failed (partition / unreachable) — try
                                // mesh before returning the error.
                                debug!(
                                    "Forward to {} failed ({forward_err:#}), trying mesh fallback",
                                    atomic.primary_endpoint.display_string()
                                );
                                if let Some(mesh_result) = self
                                    .try_songbird_mesh_dispatch(
                                        capability,
                                        &operation,
                                        &args,
                                        timeout_cap,
                                    )
                                    .await
                                {
                                    let elapsed_ms = elapsed_ms_since(start);
                                    let routing_trace = want_trace.then(|| {
                                        routing_trace_value(
                                            &[
                                                RoutingPhase::RouteResolved {
                                                    capability: capability.to_string(),
                                                    provider: MESH_PROVIDER_LABEL.to_string(),
                                                    method: semantic_name.clone(),
                                                },
                                                RoutingPhase::Forwarded { elapsed_ms },
                                            ],
                                            capability,
                                        )
                                    });
                                    return Ok(CapabilityCallOutcome {
                                        result: mesh_result?,
                                        routing_trace,
                                    });
                                }
                                Err(forward_err)
                            }
                        }
                    }
                    Err(local_err) => {
                        // No local provider found — try Songbird mesh dispatch.
                        // Songbird handles local UDS + remote mesh TCP + TURN relay,
                        // reaching primals on other gates transparently.
                        if let Some(mesh_result) = self
                            .try_songbird_mesh_dispatch(capability, &operation, &args, timeout_cap)
                            .await
                        {
                            let elapsed_ms = elapsed_ms_since(start);
                            let routing_trace = want_trace.then(|| {
                                routing_trace_value(
                                    &[
                                        RoutingPhase::RouteResolved {
                                            capability: capability.to_string(),
                                            provider: MESH_PROVIDER_LABEL.to_string(),
                                            method: semantic_name.clone(),
                                        },
                                        RoutingPhase::Forwarded { elapsed_ms },
                                    ],
                                    capability,
                                )
                            });
                            return Ok(CapabilityCallOutcome {
                                result: mesh_result?,
                                routing_trace,
                            });
                        }
                        Err(local_err)
                    }
                }
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

    /// CG-8: Attempt cross-gate dispatch through Songbird mesh.
    ///
    /// Forwards `capability.call` to Songbird with `routing: "any"`, which lets
    /// Songbird resolve locally first, then transparently forward over mesh TCP
    /// to remote peers (with TURN relay fallback for NAT). Returns `None` if
    /// Songbird is unavailable or capability is not found on any reachable gate.
    async fn try_songbird_mesh_dispatch(
        &self,
        capability: &str,
        operation: &str,
        args: &Value,
        timeout: Option<std::time::Duration>,
    ) -> Option<Result<Value>> {
        let relay_endpoint = self.router.find_primal_by_capability("relay").await.ok()?;

        debug!(
            "Songbird mesh dispatch: {capability}.{operation} via {}",
            relay_endpoint.endpoint.display_string()
        );

        let songbird_params = json!({
            "capability": capability,
            "operation": operation,
            "params": args,
            "routing": "any",
        });

        let mesh_timeout = timeout.or(Some(std::time::Duration::from_secs(15)));
        let result = self
            .router
            .forward_request_with_timeout(
                &relay_endpoint.endpoint,
                "capability.call",
                &songbird_params,
                mesh_timeout,
            )
            .await;

        match &result {
            Ok(response) => {
                let gate = response
                    .get("gate")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let provider = response
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                debug!("Songbird mesh resolved: {capability}.{operation} → {provider} @ {gate}");
                // Unwrap inner result: Songbird wraps as { provider, gate, result }
                let inner = response
                    .get("result")
                    .cloned()
                    .unwrap_or_else(|| response.clone());
                Some(Ok(inner))
            }
            Err(e) => {
                debug!("Songbird mesh dispatch failed for {capability}.{operation}: {e}");
                None
            }
        }
    }
}

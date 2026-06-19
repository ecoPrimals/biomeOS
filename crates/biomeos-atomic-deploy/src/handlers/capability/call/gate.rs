// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Cross-gate routing for `capability.call`.

use super::super::helpers::elapsed_ms_since;
use super::super::{CapabilityCallOutcome, CapabilityHandler};
use super::preamble::CallContext;
use crate::handlers::capability_routing::{RoutingPhase, routing_trace_value};
use anyhow::Result;
use serde_json::json;
use tracing::{debug, trace};

impl CapabilityHandler {
    /// Route via an explicit remote gate when `gate` is set on the request.
    ///
    /// Returns `Ok(None)` when no gate was specified (caller should continue local routing).
    pub(super) async fn try_gate_routing(
        &self,
        ctx: &CallContext,
        start: std::time::Instant,
    ) -> Result<Option<CapabilityCallOutcome>> {
        let Some(gate_name) = ctx.gate_name.as_deref() else {
            return Ok(None);
        };

        if gate_name == "local" {
            trace!("capability.call: gate='local', routing locally");
            return Ok(None);
        }

        if let Some(remote_endpoint) = {
            let registry = self.gate_registry.read().await;
            registry.resolve(gate_name).cloned()
        } {
            let semantic_name = format!("{}.{}", ctx.capability, ctx.operation);
            debug!(
                "   Cross-gate routing: {semantic_name} → gate '{gate_name}' @ {}",
                remote_endpoint.display_string()
            );

            let remote_call = json!({
                "capability": ctx.capability,
                "operation": ctx.operation,
                "args": ctx.args,
            });

            let result = self
                .router
                .forward_request_with_timeout(
                    &remote_endpoint,
                    "capability.call",
                    &remote_call,
                    ctx.timeout_cap,
                )
                .await?;

            let elapsed_ms = elapsed_ms_since(start);
            trace!("   {semantic_name} completed in {elapsed_ms}ms via gate '{gate_name}'");
            let routing_trace = ctx.want_trace.then(|| {
                routing_trace_value(
                    &[
                        RoutingPhase::RouteResolved {
                            capability: ctx.capability.clone(),
                            provider: gate_name.to_string(),
                            method: "capability.call".to_string(),
                        },
                        RoutingPhase::EndpointResolved {
                            provider: gate_name.to_string(),
                            endpoint: remote_endpoint.display_string(),
                        },
                        RoutingPhase::Forwarded { elapsed_ms },
                    ],
                    &ctx.capability,
                )
            });
            return Ok(Some(CapabilityCallOutcome {
                result,
                routing_trace,
            }));
        }

        // CG-8: Songbird mesh fallback for unregistered gates.
        // Forward capability.call to Songbird with routing:"any" — Songbird
        // handles local UDS + remote mesh TCP + TURN relay transparently.
        if let Some(relay_result) = self
            .try_songbird_mesh_dispatch(&ctx.capability, &ctx.operation, &ctx.args, ctx.timeout_cap)
            .await
        {
            let elapsed_ms = elapsed_ms_since(start);
            let routing_trace = ctx.want_trace.then(|| {
                routing_trace_value(
                    &[
                        RoutingPhase::RouteResolved {
                            capability: ctx.capability.clone(),
                            provider: format!("relay:{gate_name}"),
                            method: "capability.call".to_string(),
                        },
                        RoutingPhase::Forwarded { elapsed_ms },
                    ],
                    &ctx.capability,
                )
            });
            return Ok(Some(CapabilityCallOutcome {
                result: relay_result?,
                routing_trace,
            }));
        }

        anyhow::bail!(
            "Gate '{gate_name}' is not registered and relay fallback unavailable. \
             Register it via graph env or route.register before targeting it. \
             Known gates: {:?}",
            self.gate_registry.read().await.gate_names()
        );
    }
}

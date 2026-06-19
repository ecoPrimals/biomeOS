// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Local routing orchestration: signal graphs, Tower Atomic relay, then translation/direct paths.

mod direct;
mod translation;

use super::super::helpers::elapsed_ms_since;
use super::super::{CapabilityCallOutcome, CapabilityHandler};
use super::preamble::CallContext;
use crate::handlers::capability_routing::{RoutingPhase, routing_trace_value};
use crate::handlers::signal as signal_handler;
use anyhow::Result;
use serde_json::json;
use tracing::{debug, info, trace};

impl CapabilityHandler {
    pub(super) async fn dispatch_local(
        &self,
        ctx: &CallContext,
        start: std::time::Instant,
    ) -> Result<CapabilityCallOutcome> {
        trace!("capability.call: {}.{}", ctx.capability, ctx.operation);

        // Layer 5 prep: record utilization for hot/cold method analysis
        let semantic_name_early = format!("{}.{}", ctx.capability, ctx.operation);
        self.router.record_utilization(&semantic_name_early).await;

        // Atomic signal interception: if the capability is a signal tier
        // (tower/node/nest/meta), check for a signal graph and execute it
        // instead of trying to discover a primal named "tower".
        if signal_handler::is_signal_tier(&ctx.capability) {
            if let (Some(graphs_dir), Some(graph_handler)) = (&self.graphs_dir, &self.graph_handler)
            {
                let graph_path =
                    signal_handler::signal_graph_path(graphs_dir, &ctx.capability, &ctx.operation);
                if graph_path.exists() {
                    info!(
                        "Signal intercept: {}.{} -> graph execution",
                        ctx.capability, ctx.operation
                    );
                    let signal_params = json!({
                        "signal": format!("{}.{}" , ctx.capability, ctx.operation),
                        "params": ctx.args,
                    });
                    let result = signal_handler::dispatch(
                        graphs_dir,
                        &self.family_id,
                        graph_handler,
                        &Some(signal_params),
                    )
                    .await?;
                    let elapsed_ms = elapsed_ms_since(start);
                    let routing_trace = ctx.want_trace.then(|| {
                        routing_trace_value(
                            &[
                                RoutingPhase::RouteResolved {
                                    capability: ctx.capability.clone(),
                                    provider: "signal_graph".to_string(),
                                    method: format!("{}.{}", ctx.capability, ctx.operation),
                                },
                                RoutingPhase::Forwarded { elapsed_ms },
                            ],
                            &ctx.capability,
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

        let semantic_name = format!("{}.{}", ctx.capability, ctx.operation);

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
                    &ctx.args,
                    ctx.timeout_cap,
                )
                .await
            {
                Ok(value) => {
                    let elapsed_ms = elapsed_ms_since(start);
                    trace!(
                        "   ✓ {} completed in {}ms via Tower Atomic relay",
                        semantic_name, elapsed_ms
                    );
                    let routing_trace = ctx.want_trace.then(|| {
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
                                    capability: ctx.capability.clone(),
                                    provider: "tower_atomic".to_string(),
                                    method: semantic_name.clone(),
                                },
                                RoutingPhase::EndpointResolved {
                                    provider: relay_provider,
                                    endpoint: tower.primary_endpoint.display_string(),
                                },
                                RoutingPhase::Forwarded { elapsed_ms },
                            ],
                            &ctx.capability,
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
                let trans = trans.clone();
                drop(registry);
                self.dispatch_with_translation(ctx, start, &semantic_name, trans)
                    .await
            }
            None => {
                drop(registry);
                self.dispatch_without_translation(ctx, start, &semantic_name)
                    .await
            }
        }
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Translation-registry routing for `capability.call`.
//!
//! Resolution order:
//! 1. Try dynamic capability discovery (router registry) for fresh endpoints
//! 2. Fall back to the translation's own socket (TOML-resolved at load time)
//! 3. Try Songbird mesh relay as last resort
//!
//! This prevents category registry gaps from shadowing explicit TOML translations.
//! The translation registry is self-sufficient for routing — if it resolved a socket
//! at load time, that socket is authoritative even when the capability registry
//! doesn't have a matching category entry.

use super::super::super::helpers::elapsed_ms_since;
use super::super::super::{CapabilityCallOutcome, CapabilityHandler};
use super::super::preamble::CallContext;
use crate::capability_translation::CapabilityTranslation;
use crate::handlers::capability_routing::{RoutingPhase, routing_trace_value};
use crate::neural_router::{DiscoveredAtomic, DiscoveredPrimal};
use anyhow::Result;
use biomeos_core::TransportEndpoint;
use std::sync::Arc;
use tracing::{debug, trace};

/// Trace label for dispatches routed through the mesh gateway.
const MESH_PROVIDER_LABEL: &str = "mesh_relay";

impl CapabilityHandler {
    pub(super) async fn dispatch_with_translation(
        &self,
        ctx: &CallContext,
        start: std::time::Instant,
        semantic_name: &str,
        trans: CapabilityTranslation,
    ) -> Result<CapabilityCallOutcome> {
        debug!(
            "   Translated: {} → {}:{}",
            semantic_name, trans.provider, trans.actual_method
        );

        let forward_method = trans.actual_method.clone();
        let provider_from_trans = trans.provider.clone();

        let atomic = match self.router.discover_capability(&ctx.capability).await {
            Ok(a) => a,
            Err(_discovery_err) => {
                // Discovery failed — the capability registry doesn't have a category
                // entry for this domain. Use the translation's own socket endpoint
                // which was resolved at TOML load time. This is the fix for the
                // "category shadow" bug where explicit TOML translations (braid.verify,
                // braid.list, etc.) become unroutable when no graph registers their
                // domain as a category in the capability registry.
                if let Some(endpoint) = TransportEndpoint::parse(&trans.socket) {
                    debug!(
                        "   Discovery miss for '{}', using translation socket: {}",
                        ctx.capability,
                        endpoint.display_string()
                    );
                    DiscoveredAtomic {
                        capability: Arc::from(ctx.capability.as_str()),
                        primals: vec![DiscoveredPrimal {
                            name: Arc::from(provider_from_trans.as_str()),
                            endpoint: endpoint.clone(),
                            capabilities: vec![semantic_name.to_string()],
                            healthy: true,
                            last_check: chrono::Utc::now(),
                        }],
                        atomic_type: None,
                        primary_endpoint: endpoint,
                    }
                } else if let Some(mesh_result) = self
                    .try_songbird_mesh_dispatch(
                        &ctx.capability,
                        &ctx.operation,
                        &ctx.args,
                        ctx.timeout_cap,
                    )
                    .await
                {
                    let elapsed_ms = elapsed_ms_since(start);
                    let routing_trace = ctx.want_trace.then(|| {
                        routing_trace_value(
                            &[
                                RoutingPhase::RouteResolved {
                                    capability: ctx.capability.clone(),
                                    provider: MESH_PROVIDER_LABEL.to_string(),
                                    method: forward_method.clone(),
                                },
                                RoutingPhase::Forwarded { elapsed_ms },
                            ],
                            &ctx.capability,
                        )
                    });
                    return Ok(CapabilityCallOutcome {
                        result: mesh_result?,
                        routing_trace,
                    });
                } else {
                    return Err(_discovery_err);
                }
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

        let result = if trans.ribocipher {
            self.router
                .forward_request_ribocipher(&endpoint, &forward_method, &ctx.args, ctx.timeout_cap)
                .await
        } else {
            self.router
                .forward_request_with_timeout(
                    &endpoint,
                    &forward_method,
                    &ctx.args,
                    ctx.timeout_cap,
                )
                .await
        };

        // Self-healing: if forward failed, the endpoint may be stale (e.g., graph-bootstrap
        // registered {primal}-{family}.sock but the primal bound to {primal}.sock).
        // Attempt a targeted endpoint refresh and retry once.
        let result = match result {
            Err(ref _e) => {
                if let Some(refreshed) = self.router.refresh_stale_endpoint(&primary_name).await {
                    debug!(
                        "   🔄 Retrying {} via refreshed endpoint: {}",
                        semantic_name,
                        refreshed.display_string()
                    );
                    if trans.ribocipher {
                        self.router
                            .forward_request_ribocipher(
                                &refreshed,
                                &forward_method,
                                &ctx.args,
                                ctx.timeout_cap,
                            )
                            .await
                    } else {
                        self.router
                            .forward_request_with_timeout(
                                &refreshed,
                                &forward_method,
                                &ctx.args,
                                ctx.timeout_cap,
                            )
                            .await
                    }
                } else {
                    result
                }
            }
            ok => ok,
        };

        let elapsed_ms = elapsed_ms_since(start);

        // Layer 4: record dispatch outcome for adaptive routing weights
        self.router
            .record_dispatch_outcome(&ctx.capability, &primary_name, result.is_ok(), elapsed_ms)
            .await;

        let result = result?;

        trace!(
            "   ✓ {} completed in {}ms via {}",
            semantic_name, elapsed_ms, provider_from_trans
        );

        let routing_trace = ctx.want_trace.then(|| {
            routing_trace_value(
                &[
                    RoutingPhase::RouteResolved {
                        capability: ctx.capability.clone(),
                        provider: provider_from_trans,
                        method: forward_method,
                    },
                    RoutingPhase::EndpointResolved {
                        provider: primary_name,
                        endpoint: atomic.primary_endpoint.display_string(),
                    },
                    RoutingPhase::Forwarded { elapsed_ms },
                ],
                &ctx.capability,
            )
        });

        Ok(CapabilityCallOutcome {
            result,
            routing_trace,
        })
    }
}

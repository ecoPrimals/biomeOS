// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Translation-registry routing for `capability.call`.

use super::super::super::helpers::elapsed_ms_since;
use super::super::super::{CapabilityCallOutcome, CapabilityHandler};
use super::super::preamble::CallContext;
use crate::capability_translation::CapabilityTranslation;
use crate::handlers::capability_routing::{RoutingPhase, routing_trace_value};
use anyhow::Result;
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
            Err(_local_err) => {
                // Translation exists but no local provider — try Songbird mesh
                if let Some(mesh_result) = self
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
            .forward_request_with_timeout(&endpoint, &forward_method, &ctx.args, ctx.timeout_cap)
            .await;

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

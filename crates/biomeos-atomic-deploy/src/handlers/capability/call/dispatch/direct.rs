// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Direct capability discovery routing when no translation exists.

use super::super::super::helpers::elapsed_ms_since;
use super::super::super::{CapabilityCallOutcome, CapabilityHandler};
use super::super::preamble::CallContext;
use crate::handlers::capability_routing::{RoutingPhase, routing_trace_value};
use anyhow::Result;
use tracing::debug;

/// Trace label for dispatches routed through the mesh gateway.
const MESH_PROVIDER_LABEL: &str = "mesh_relay";

impl CapabilityHandler {
    pub(super) async fn dispatch_without_translation(
        &self,
        ctx: &CallContext,
        start: std::time::Instant,
        semantic_name: &str,
    ) -> Result<CapabilityCallOutcome> {
        debug!(
            "No translation for {}, attempting direct route",
            semantic_name
        );

        match self.router.discover_capability(&ctx.capability).await {
            Ok(atomic) => {
                let forward_method = ctx.operation.clone();

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
                        &ctx.args,
                        ctx.timeout_cap,
                    )
                    .await;

                let elapsed_ms = elapsed_ms_since(start);

                self.router
                    .record_dispatch_outcome(
                        &ctx.capability,
                        &primary_name,
                        result.is_ok(),
                        elapsed_ms,
                    )
                    .await;

                match result {
                    Ok(result) => {
                        let routing_trace = ctx.want_trace.then(|| {
                            routing_trace_value(
                                &[
                                    RoutingPhase::RouteResolved {
                                        capability: ctx.capability.clone(),
                                        provider: String::new(),
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
                    Err(forward_err) => {
                        // Forward failed (partition / unreachable) — try
                        // mesh before returning the error.
                        debug!(
                            "Forward to {} failed ({forward_err:#}), trying mesh fallback",
                            atomic.primary_endpoint.display_string()
                        );
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
                                            method: semantic_name.to_string(),
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
                        Err(forward_err)
                    }
                }
            }
            Err(local_err) => {
                // No local provider found — try Songbird mesh dispatch.
                // Songbird handles local UDS + remote mesh TCP + TURN relay,
                // reaching primals on other gates transparently.
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
                                    method: semantic_name.to_string(),
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
                Err(local_err)
            }
        }
    }
}

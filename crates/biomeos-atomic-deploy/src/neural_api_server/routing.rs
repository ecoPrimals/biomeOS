// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Request routing for Neural API Server
//!
//! Routes JSON-RPC requests to appropriate handlers based on method name.
//! Uses a table-driven handler registry for O(1) lookup.

#[path = "route_table.rs"]
mod route_table;

use biomeos_core::method_gate::CallerContext;
use route_table::{Route, lookup_route};
use serde_json::{Value, json};
use tracing::{debug, trace};

use super::NeuralApiServer;
use super::rpc::{DispatchOutcome, JsonRpcRequest};
use crate::handlers::capability::CapabilityCallOutcome;

fn dispatch(result: Result<Value, anyhow::Error>, id: Value) -> DispatchOutcome {
    match result {
        Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id)),
        Err(e) => {
            // Preserve JSON-RPC error codes from primals (GAP-MATRIX-07b).
            // Without this, a primal returning -32601 "method not found" would be
            // swallowed into a generic -32603 "Internal error", making the caller
            // unable to distinguish "primal rejected request" from "primal is down".
            if let Some(biomeos_types::IpcError::JsonRpcError { code, message, .. }) =
                e.downcast_ref::<biomeos_types::IpcError>()
            {
                return DispatchOutcome::ApplicationError {
                    code: *code,
                    message: message.clone(),
                    id,
                };
            }
            DispatchOutcome::ApplicationError {
                code: -32603,
                message: format!("Internal error: {e}"),
                id,
            }
        }
    }
}

fn dispatch_capability_call(
    result: Result<CapabilityCallOutcome, anyhow::Error>,
    id: Value,
) -> DispatchOutcome {
    match result {
        Ok(outcome) => {
            let mut map = serde_json::Map::new();
            map.insert("jsonrpc".to_string(), json!("2.0"));
            map.insert("result".to_string(), outcome.result);
            if let Some(trace) = outcome.routing_trace {
                map.insert("_routing_trace".to_string(), trace);
            }
            map.insert("id".to_string(), id);
            DispatchOutcome::Success(Value::Object(map))
        }
        Err(e) => {
            if let Some(biomeos_types::IpcError::JsonRpcError { code, message, .. }) =
                e.downcast_ref::<biomeos_types::IpcError>()
            {
                return DispatchOutcome::ApplicationError {
                    code: *code,
                    message: message.clone(),
                    id,
                };
            }
            DispatchOutcome::ApplicationError {
                code: -32603,
                message: format!("Internal error: {e}"),
                id,
            }
        }
    }
}

impl NeuralApiServer {
    /// Build semantic capability call params from a domain.operation method,
    /// injecting resource envelope and bearer token verification.
    async fn build_semantic_params(
        &self,
        domain: &str,
        operation: &str,
        params: &Option<Value>,
        caller: &CallerContext,
    ) -> Value {
        let mut args_obj = params.clone().unwrap_or(json!({}));
        let trace_flag = args_obj
            .as_object()
            .and_then(|o| o.get("_routing_trace"))
            .cloned();
        if let Some(o) = args_obj.as_object_mut() {
            o.remove("_routing_trace");
            o.remove("_bearer_token");
        }
        let mut cap_params = json!({
            "capability": domain,
            "operation": operation,
            "args": args_obj
        });
        if let Some(t) = trace_flag {
            cap_params["_routing_trace"] = t;
        }
        if let Some(ref claims) = caller.claims {
            if let Some(ref env) = claims.resources {
                cap_params["_resource_envelope"] = env.to_forwarding_value();
            }
        }
        if let Some(ref token) = caller.bearer_token {
            cap_params["_bearer_token"] = json!(token);
            let verified = if let Some(ref v) = self.beardog_verifier {
                v.verify_async(token).await.is_some()
            } else {
                false
            };
            cap_params["_token_verified"] = json!(verified);
        }
        cap_params
    }

    /// Handle a JSON-RPC request, returning a structured dispatch outcome.
    ///
    /// Separates protocol errors (method not found, parse error) from application
    /// results. Use `handle_request_json` for backward compatibility (returns `Value`).
    ///
    /// Delegates to focused handlers for each domain:
    /// - Graph operations → `GraphHandler`
    /// - Capability routing → `CapabilityHandler`
    /// - Topology/metrics → `TopologyHandler`
    /// - Niche templates → `NicheHandler`
    /// - Lifecycle management → `LifecycleHandler`
    /// - Protocol escalation → `ProtocolHandler`
    pub async fn handle_request(&self, request_line: &str) -> DispatchOutcome {
        let request = match JsonRpcRequest::parse(request_line) {
            Ok(r) => r,
            Err(e) => {
                return DispatchOutcome::ParseError {
                    message: e.to_string(),
                };
            }
        };

        let id = request.id.clone().unwrap_or(serde_json::Value::Null);
        debug!("📥 Request: {} (id: {})", request.method, id);
        trace!("📥 Full request: {}", request_line.trim());

        let caller = request
            .params
            .as_ref()
            .and_then(|p| p.get("_bearer_token"))
            .and_then(|t| t.as_str())
            .map_or_else(CallerContext::loopback, |tok| {
                CallerContext::loopback().with_bearer_token(tok.to_owned())
            });

        if let Err(gate_err) = self.method_gate.check(request.method.as_ref(), &caller) {
            return DispatchOutcome::ApplicationError {
                code: gate_err.code as i32,
                message: gate_err.message,
                id,
            };
        }

        let params = &request.params;

        let route = match lookup_route(request.method.as_ref()) {
            Some(r) => r,
            None => {
                if let Some((domain, operation)) = request.method.as_ref().split_once('.') {
                    if !domain.is_empty() && !operation.is_empty() {
                        debug!("📡 Semantic fallback: {domain}.{operation} → capability.call");
                        let cap_params =
                            self.build_semantic_params(domain, operation, params, &caller).await;
                        return dispatch_capability_call(
                            self.capability_handler.call(&Some(cap_params)).await,
                            id,
                        );
                    }
                }
                return DispatchOutcome::MethodNotFound {
                    method: request.method.as_ref().to_string(),
                    id,
                };
            }
        };
        let outcome = match route {
            // Graph
            Route::GraphList => dispatch(self.graph_handler.list().await, id),
            Route::GraphGet => dispatch(self.graph_handler.get(params).await, id),
            Route::GraphSave => dispatch(self.graph_handler.save(params).await, id),
            Route::GraphExecute => dispatch(self.graph_handler.execute(params).await, id),
            Route::GraphExecutePipeline => {
                dispatch(self.graph_handler.execute_pipeline(params).await, id)
            }
            Route::GraphStatus => dispatch(self.graph_handler.get_status(params).await, id),
            Route::GraphStartContinuous => {
                dispatch(self.graph_handler.start_continuous(params).await, id)
            }
            Route::GraphPauseContinuous => {
                dispatch(self.graph_handler.pause_continuous(params).await, id)
            }
            Route::GraphResumeContinuous => {
                dispatch(self.graph_handler.resume_continuous(params).await, id)
            }
            Route::GraphStopContinuous => {
                dispatch(self.graph_handler.stop_continuous(params).await, id)
            }
            Route::GraphTickStatus => dispatch(self.graph_handler.tick_status().await, id),
            Route::GraphVerify => {
                dispatch(self.graph_handler.verify_graph(&request.params).await, id)
            }
            Route::GraphSuggestOptimizations => {
                dispatch(self.graph_handler.suggest_optimizations(params).await, id)
            }
            // Topology
            Route::TopologyGet => dispatch(self.topology_handler.get().await, id),
            Route::TopologyPrimals => dispatch(self.topology_handler.get_primals().await, id),
            Route::TopologyProprioception => {
                dispatch(self.topology_handler.get_proprioception().await, id)
            }
            Route::TopologyMetrics => dispatch(self.topology_handler.get_metrics().await, id),
            Route::TopologyRescan => dispatch(self.rescan_primals().await, id),
            // Niche
            Route::NicheList => dispatch(self.niche_handler.list().await, id),
            Route::NicheDeploy => dispatch(self.niche_handler.deploy(params).await, id),
            // Lifecycle
            Route::LifecycleStatus => dispatch(self.lifecycle_handler.status().await, id),
            Route::LifecycleGet => dispatch(self.lifecycle_handler.get(params).await, id),
            Route::LifecycleRegister => dispatch(self.lifecycle_handler.register(params).await, id),
            Route::LifecycleResurrect => {
                dispatch(self.lifecycle_handler.resurrect(params).await, id)
            }
            Route::LifecycleApoptosis => {
                dispatch(self.lifecycle_handler.apoptosis(params).await, id)
            }
            Route::LifecycleShutdownAll => {
                dispatch(self.lifecycle_handler.shutdown_all().await, id)
            }
            Route::LifecycleComposition => dispatch(self.lifecycle_handler.composition().await, id),
            Route::CompositionHealth => {
                dispatch(self.lifecycle_handler.composition_health(params).await, id)
            }
            // Protocol
            Route::ProtocolStatus => dispatch(self.protocol_handler.status().await, id),
            Route::ProtocolEscalate => dispatch(self.protocol_handler.escalate(params).await, id),
            Route::ProtocolFallback => dispatch(self.protocol_handler.fallback(params).await, id),
            Route::ProtocolMetrics => dispatch(self.protocol_handler.metrics(params).await, id),
            Route::ProtocolRegisterPrimal => {
                dispatch(self.protocol_handler.register_primal(params).await, id)
            }
            Route::ProtocolRegisterConnection => {
                dispatch(self.protocol_handler.register_connection(params).await, id)
            }
            Route::ProtocolRecordRequest => {
                dispatch(self.protocol_handler.record_request(params).await, id)
            }
            Route::ProtocolStartMonitoring => {
                dispatch(self.protocol_handler.start_monitoring().await, id)
            }
            Route::ProtocolStopMonitoring => {
                dispatch(self.protocol_handler.stop_monitoring().await, id)
            }
            Route::GraphProtocolMap => dispatch(self.protocol_handler.protocol_map().await, id),
            // Route (batch capability registration)
            Route::BatchRouteRegister => {
                dispatch(self.capability_handler.register_route(params).await, id)
            }
            // Capability
            Route::CapabilityRegister => {
                dispatch(self.capability_handler.register(params).await, id)
            }
            Route::CapabilityDiscover => {
                dispatch(self.capability_handler.discover(params).await, id)
            }
            Route::CapabilityList => dispatch(self.capability_handler.list().await, id),
            Route::CapabilityProviders => {
                dispatch(self.capability_handler.providers(params).await, id)
            }
            Route::CapabilityResolve => dispatch(self.capability_handler.route(params).await, id),
            Route::CapabilityResolveSingle => {
                dispatch(self.capability_handler.resolve(params).await, id)
            }
            Route::CapabilityMetrics => dispatch(self.capability_handler.get_metrics().await, id),
            Route::RoutingWeights => dispatch(self.handle_routing_weights().await, id),
            Route::RoutingExplain => dispatch(self.handle_routing_explain(params).await, id),
            Route::CompositionPatterns => {
                dispatch(Ok(self.router.composition_patterns_json().await), id)
            }
            Route::CompositionPlanTier => dispatch(self.handle_plan_tier(params).await, id),
            Route::CapabilityUtilization => dispatch(Ok(self.router.utilization_json().await), id),
            Route::WeightHealth => dispatch(self.handle_weight_health().await, id),
            Route::CapabilityCall => {
                let enriched = self.enrich_for_forwarding(params, &caller).await;
                dispatch_capability_call(self.capability_handler.call(&enriched).await, id)
            }
            Route::CapabilityDiscoverTranslations => dispatch(
                self.capability_handler.discover_translations(params).await,
                id,
            ),
            Route::CapabilityListTranslations => {
                dispatch(self.capability_handler.list_translations().await, id)
            }
            // MCP
            Route::McpToolsList => dispatch(self.capability_handler.mcp_tools_list().await, id),
            // Agent
            Route::Agent => dispatch(
                super::agents::handle_agent_request(
                    &self.agent_registry,
                    request.method.as_ref(),
                    params,
                )
                .await,
                id,
            ),
            // Inference (canonical namespace)
            Route::InferenceSchedule => dispatch(self.inference_handler.schedule(params).await, id),
            Route::InferenceGates => dispatch(self.inference_handler.gates(params).await, id),
            Route::InferenceRegisterProvider => {
                dispatch(self.inference_handler.register_provider(params).await, id)
            }
            Route::InferenceProviders => {
                dispatch(self.inference_handler.list_providers(params).await, id)
            }
            Route::InferenceComplete => dispatch(self.inference_handler.complete(params).await, id),
            Route::InferenceEmbed => dispatch(self.inference_handler.embed(params).await, id),
            Route::InferenceModels => dispatch(self.inference_handler.models(params).await, id),
            // Health probes (SEMANTIC_METHOD_NAMING_STANDARD.md compliance)
            Route::HealthCheck => dispatch(self.health_check().await, id),
            Route::HealthLiveness => dispatch(self.health_liveness(), id),
            Route::HealthReadiness => dispatch(self.health_readiness().await, id),
            Route::BtspEscalate => dispatch(self.btsp_escalate(), id),
            Route::BtspStatus => dispatch(self.btsp_status().await, id),
            Route::BtspNegotiate => dispatch(
                super::btsp_negotiate::handle_negotiate(
                    &self.btsp_sessions,
                    &params.clone().unwrap_or(serde_json::json!({})),
                )
                .await,
                id,
            ),
            // Auth introspection (JH-0 method gate)
            Route::AuthCheck => {
                let result = self.method_gate.handle_auth_check(&caller);
                DispatchOutcome::Success(super::rpc::success_response(result, id))
            }
            Route::AuthMode => {
                let result = self.method_gate.handle_auth_mode();
                DispatchOutcome::Success(super::rpc::success_response(result, id))
            }
            Route::AuthPeerInfo => {
                let result = self.method_gate.handle_auth_peer_info(&caller);
                DispatchOutcome::Success(super::rpc::success_response(result, id))
            }
            // Composition hot-reload (JH-3)
            Route::CompositionReload => dispatch(self.lifecycle_handler.reload(params).await, id),
            // Composition status (pappusCast adaptive daemons)
            Route::CompositionStatus => {
                dispatch(self.lifecycle_handler.composition_status().await, id)
            }
            // Composition deploy (alias for graph.execute — primalSpring contract)
            Route::CompositionDeploy => dispatch(self.graph_handler.execute(params).await, id),
            // Composition deploy shadow (dry-run validation)
            Route::CompositionDeployShadow => {
                dispatch(self.graph_handler.shadow_deploy(params).await, id)
            }
            Route::IdentityGet => dispatch(Ok(self.identity_response()), id),
            Route::PrimalAnnounce => dispatch(self.handle_primal_announce(params).await, id),
            Route::SignalDispatch => {
                dispatch(self.dispatch_nucleus_signal_raw(params).await, id)
            }
            Route::SignalList => {
                dispatch(crate::handlers::signal::list(&self.graphs_dir).await, id)
            }
            Route::SignalSchema => {
                dispatch(crate::handlers::signal::schema(&self.graphs_dir).await, id)
            }
            // Spring status (Tier 2 notebook integration)
            Route::SpringStatus => dispatch(self.lifecycle_handler.spring_status().await, id),
            // Spore lifecycle — blocked on lithoSpore Tier 3 VM provisioning.
            // Returns a structured deferred response instead of executing a graph
            // with an unread _deferred flag.
            Route::SporeInstantiate => dispatch(
                Ok(json!({
                    "status": "deferred",
                    "reason": "lithoSpore Tier 3 VM provisioning not yet available",
                    "spore_params": params.clone().unwrap_or(json!({})),
                })),
                id,
            ),
            Route::NucleusIngestSpore => {
                dispatch(self.dispatch_nucleus_signal("ingest_spore", params).await, id)
            }
            Route::NucleusEmitSpore => {
                let emit_params = params.clone().unwrap_or(json!({}));
                let spore_id = emit_params
                    .get("spore_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                if spore_id.is_empty() {
                    dispatch(
                        Err::<Value, _>(anyhow::anyhow!(
                            "nucleus.emit_spore requires a spore_id parameter"
                        )),
                        id,
                    )
                } else {
                    dispatch(
                        self.dispatch_nucleus_signal(
                            "emit_spore",
                            &Some(json!({"spore_id": spore_id, "family_id": self.family_id})),
                        )
                        .await,
                        id,
                    )
                }
            }
            // Spring method registration (GAP-09)
            Route::MethodRegister => {
                dispatch(self.capability_handler.register_methods(params).await, id)
            }
            // Legacy
            Route::ProxyHttp => dispatch(self.proxy_http(params).await, id),
            Route::SemanticCapabilityCall => {
                if let Some((domain, operation)) = request.method.as_ref().split_once('.') {
                    let cap_params =
                        self.build_semantic_params(domain, operation, params, &caller).await;
                    dispatch_capability_call(
                        self.capability_handler.call(&Some(cap_params)).await,
                        id,
                    )
                } else {
                    return DispatchOutcome::MethodNotFound {
                        method: request.method.as_ref().to_string(),
                        id,
                    };
                }
            }
        };

        outcome
    }

    async fn handle_primal_announce(
        &self,
        params: &Option<Value>,
    ) -> Result<Value, anyhow::Error> {
        crate::handlers::announce::handle_announce(
            &self.router,
            &self.translation_registry,
            &self.lifecycle_handler,
            &self.beardog_verifier,
            params,
        )
        .await
    }

    /// Raw signal dispatch (signal.dispatch method — caller provides signal name in params).
    async fn dispatch_nucleus_signal_raw(
        &self,
        params: &Option<Value>,
    ) -> Result<Value, anyhow::Error> {
        crate::handlers::signal::dispatch(
            &self.graphs_dir,
            &self.family_id,
            &self.graph_handler,
            params,
        )
        .await
    }

    fn identity_response(&self) -> Value {
        json!({
            "primal": "biomeos",
            "role": "orchestrator",
            "version": env!("CARGO_PKG_VERSION"),
            "capabilities": ["orchestration", "composition", "graph", "topology", "lifecycle", "signal"],
            "is_orchestrator": true,
            "transport": ["uds", "tcp", "http"],
        })
    }

    async fn handle_routing_weights(&self) -> Result<Value, anyhow::Error> {
        let weights = self.router.get_routing_weights().await;
        let summary = self.router.get_weight_summary().await;
        Ok(json!({"weights": weights, "summary": summary}))
    }

    async fn handle_routing_explain(
        &self,
        params: &Option<Value>,
    ) -> Result<Value, anyhow::Error> {
        crate::handlers::capability_routing::explain_route(
            &self.router,
            &*self.translation_registry.read().await,
            params,
        )
        .await
    }

    async fn handle_plan_tier(&self, params: &Option<Value>) -> Result<Value, anyhow::Error> {
        let tier_name = params
            .as_ref()
            .and_then(|p| p.get("tier"))
            .and_then(|v| v.as_str())
            .unwrap_or("tower");
        let tier = match tier_name {
            "tower" => crate::neural_router::CompositionTier::Tower,
            "node" => crate::neural_router::CompositionTier::Node,
            "nest" => crate::neural_router::CompositionTier::Nest,
            "nucleus" => crate::neural_router::CompositionTier::Nucleus,
            "meta" => crate::neural_router::CompositionTier::Meta,
            "orchestration" => crate::neural_router::CompositionTier::Orchestration,
            _ => crate::neural_router::CompositionTier::Standalone,
        };
        let plan = self.router.plan_tier(tier).await;
        serde_json::to_value(&plan).map_err(anyhow::Error::from)
    }

    /// Dispatch a NUCLEUS signal graph (nest.ingest_spore, nest.emit_spore).
    async fn dispatch_nucleus_signal(
        &self,
        signal_name: &str,
        params: &Option<Value>,
    ) -> Result<Value, anyhow::Error> {
        let signal_params = Some(json!({
            "signal": format!("nest.{signal_name}"),
            "params": params.clone().unwrap_or(json!({})),
        }));
        crate::handlers::signal::dispatch(
            &self.graphs_dir,
            &self.family_id,
            &self.graph_handler,
            &signal_params,
        )
        .await
    }

    /// Handle a JSON-RPC request and return a JSON-RPC response value.
    ///
    /// Backward-compatible wrapper that converts `DispatchOutcome` to `Value`.
    pub async fn handle_request_json(&self, request_line: &str) -> Value {
        self.handle_request(request_line).await.into_response()
    }
}

#[cfg(test)]
#[path = "routing_tests.rs"]
mod tests;

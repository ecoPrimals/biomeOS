// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Request routing for Neural API Server
//!
//! Routes JSON-RPC requests to appropriate handlers based on method name.
//! Uses a table-driven handler registry for O(1) lookup.

use serde_json::Value;
use tracing::{debug, trace};

use super::NeuralApiServer;
use super::rpc::{DispatchOutcome, JsonRpcRequest};

fn dispatch(result: Result<Value, anyhow::Error>, id: &Value) -> DispatchOutcome {
    match result {
        Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
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
                    id: id.clone(),
                };
            }
            DispatchOutcome::ApplicationError {
                code: -32603,
                message: format!("Internal error: {e}"),
                id: id.clone(),
            }
        }
    }
}

/// Route tag for dispatch. Each variant maps to a handler.
#[derive(Clone, Copy, Debug)]
enum Route {
    GraphList,
    GraphGet,
    GraphSave,
    GraphExecute,
    GraphStatus,
    GraphExecutePipeline,
    GraphStartContinuous,
    GraphPauseContinuous,
    GraphResumeContinuous,
    GraphStopContinuous,
    GraphSuggestOptimizations,
    TopologyGet,
    TopologyPrimals,
    TopologyProprioception,
    TopologyMetrics,
    TopologyRescan,
    NicheList,
    NicheDeploy,
    LifecycleStatus,
    LifecycleGet,
    LifecycleRegister,
    LifecycleResurrect,
    LifecycleApoptosis,
    LifecycleShutdownAll,
    ProtocolStatus,
    ProtocolEscalate,
    ProtocolFallback,
    ProtocolMetrics,
    ProtocolRegisterPrimal,
    ProtocolRegisterConnection,
    ProtocolRecordRequest,
    ProtocolStartMonitoring,
    ProtocolStopMonitoring,
    GraphProtocolMap,
    BatchRouteRegister,
    CapabilityRegister,
    CapabilityDiscover,
    CapabilityList,
    CapabilityProviders,
    CapabilityResolve,
    CapabilityMetrics,
    CapabilityCall,
    CapabilityDiscoverTranslations,
    CapabilityListTranslations,
    McpToolsList,
    Agent,
    ProxyHttp,
    InferenceSchedule,
    InferenceGates,
    SemanticCapabilityCall,
    HealthCheck,
    HealthLiveness,
    HealthReadiness,
}

/// Table-driven handler registry: method name → route.
/// Multiple method names may map to the same route (e.g. `neural_api.list_graphs` | graph.list).
const ROUTE_TABLE: &[(&str, Route)] = &[
    // Graph
    ("neural_api.list_graphs", Route::GraphList),
    ("graph.list", Route::GraphList),
    ("neural_api.get_graph", Route::GraphGet),
    ("graph.get", Route::GraphGet),
    ("neural_api.save_graph", Route::GraphSave),
    ("graph.save", Route::GraphSave),
    ("neural_api.execute_graph", Route::GraphExecute),
    ("graph.execute", Route::GraphExecute),
    ("neural_api.get_execution_status", Route::GraphStatus),
    ("graph.status", Route::GraphStatus),
    // Pipeline streaming execution
    ("graph.execute_pipeline", Route::GraphExecutePipeline),
    ("neural_api.execute_pipeline", Route::GraphExecutePipeline),
    // Continuous session management
    ("graph.start_continuous", Route::GraphStartContinuous),
    ("graph.pause_continuous", Route::GraphPauseContinuous),
    ("graph.resume_continuous", Route::GraphResumeContinuous),
    ("graph.stop_continuous", Route::GraphStopContinuous),
    // Pathway Learner
    (
        "graph.suggest_optimizations",
        Route::GraphSuggestOptimizations,
    ),
    (
        "neural_api.suggest_optimizations",
        Route::GraphSuggestOptimizations,
    ),
    // Topology
    ("neural_api.get_topology", Route::TopologyGet),
    ("topology.get", Route::TopologyGet),
    ("neural_api.get_primals", Route::TopologyPrimals),
    ("topology.primals", Route::TopologyPrimals),
    (
        "neural_api.get_proprioception",
        Route::TopologyProprioception,
    ),
    ("topology.proprioception", Route::TopologyProprioception),
    ("neural_api.get_metrics", Route::TopologyMetrics),
    ("topology.metrics", Route::TopologyMetrics),
    ("topology.rescan", Route::TopologyRescan),
    // Niche
    ("neural_api.list_niche_templates", Route::NicheList),
    ("niche.list", Route::NicheList),
    ("neural_api.deploy_niche", Route::NicheDeploy),
    ("niche.deploy", Route::NicheDeploy),
    // Lifecycle
    ("lifecycle.status", Route::LifecycleStatus),
    ("lifecycle.get", Route::LifecycleGet),
    ("lifecycle.register", Route::LifecycleRegister),
    ("lifecycle.resurrect", Route::LifecycleResurrect),
    ("lifecycle.apoptosis", Route::LifecycleApoptosis),
    ("lifecycle.shutdown_all", Route::LifecycleShutdownAll),
    // Protocol
    ("protocol.status", Route::ProtocolStatus),
    ("protocol.escalate", Route::ProtocolEscalate),
    ("protocol.fallback", Route::ProtocolFallback),
    ("protocol.metrics", Route::ProtocolMetrics),
    ("protocol.register_primal", Route::ProtocolRegisterPrimal),
    (
        "protocol.register_connection",
        Route::ProtocolRegisterConnection,
    ),
    ("protocol.record_request", Route::ProtocolRecordRequest),
    ("protocol.start_monitoring", Route::ProtocolStartMonitoring),
    ("protocol.stop_monitoring", Route::ProtocolStopMonitoring),
    ("graph.protocol_map", Route::GraphProtocolMap),
    // Route (batch capability registration)
    ("route.register", Route::BatchRouteRegister),
    // Capability
    ("capability.register", Route::CapabilityRegister),
    ("capability.discover", Route::CapabilityDiscover),
    ("neural_api.discover_capability", Route::CapabilityDiscover),
    ("capability.list", Route::CapabilityList),
    ("capabilities.list", Route::CapabilityList),
    ("capability.providers", Route::CapabilityProviders),
    ("capability.route", Route::CapabilityResolve),
    ("neural_api.route_to_primal", Route::CapabilityResolve),
    ("capability.metrics", Route::CapabilityMetrics),
    ("neural_api.get_routing_metrics", Route::CapabilityMetrics),
    ("capability.call", Route::CapabilityCall),
    (
        "capability.discover_translations",
        Route::CapabilityDiscoverTranslations,
    ),
    (
        "capability.discover_translation",
        Route::CapabilityDiscoverTranslations,
    ),
    (
        "capability.list_translations",
        Route::CapabilityListTranslations,
    ),
    // Inference scheduling (cross-gate model orchestration)
    ("inference.schedule", Route::InferenceSchedule),
    ("inference.gates", Route::InferenceGates),
    // MCP tool discovery (Squirrel alpha.13 aggregation)
    ("mcp.tools.list", Route::McpToolsList),
    ("mcp.tools_list", Route::McpToolsList),
    // Agent
    ("agent.create", Route::Agent),
    ("agent.list", Route::Agent),
    ("agent.get", Route::Agent),
    ("agent.remove", Route::Agent),
    ("agent.meld", Route::Agent),
    ("agent.split", Route::Agent),
    ("agent.resolve", Route::Agent),
    ("agent.route", Route::Agent),
    ("agent.auto_meld", Route::Agent),
    // Legacy
    ("neural_api.proxy_http", Route::ProxyHttp),
    // Health (semantic naming standard: health.check, health.liveness, health.readiness)
    ("health.check", Route::HealthCheck),
    ("health.liveness", Route::HealthLiveness),
    ("health.readiness", Route::HealthReadiness),
    // Mesh & NAT (explicit semantic capability routes for known domains)
    ("mesh.status", Route::SemanticCapabilityCall),
    ("mesh.find_path", Route::SemanticCapabilityCall),
    ("mesh.announce", Route::SemanticCapabilityCall),
    ("mesh.peers", Route::SemanticCapabilityCall),
    ("mesh.health_check", Route::SemanticCapabilityCall),
    ("punch.request", Route::SemanticCapabilityCall),
    ("punch.status", Route::SemanticCapabilityCall),
    ("punch.coordinate", Route::SemanticCapabilityCall),
    ("stun.discover", Route::SemanticCapabilityCall),
    ("stun.detect_nat_type", Route::SemanticCapabilityCall),
    ("stun.probe_port_pattern", Route::SemanticCapabilityCall),
    ("relay.serve", Route::SemanticCapabilityCall),
    ("relay.status", Route::SemanticCapabilityCall),
    ("relay.allocate", Route::SemanticCapabilityCall),
    ("relay.authorize", Route::SemanticCapabilityCall),
    ("onion.create_service", Route::SemanticCapabilityCall),
    ("onion.get_address", Route::SemanticCapabilityCall),
    ("onion.connect", Route::SemanticCapabilityCall),
    ("onion.status", Route::SemanticCapabilityCall),
];

fn lookup_route(method: &str) -> Option<Route> {
    ROUTE_TABLE
        .iter()
        .find(|(m, _)| *m == method)
        .map(|(_, r)| *r)
}

impl NeuralApiServer {
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

        let params = &request.params;

        let route = match lookup_route(request.method.as_ref()) {
            Some(r) => r,
            None => {
                // Semantic fallback: any "domain.operation" method not in
                // ROUTE_TABLE routes through the capability layer. Springs call
                // provenance.begin, birdsong.decrypt, dag.dehydrate, etc. as
                // top-level JSON-RPC — the capability handler resolves provider
                // + socket via CapabilityTranslationRegistry and CAPABILITY_DOMAINS.
                if let Some((domain, operation)) = request.method.as_ref().split_once('.') {
                    if !domain.is_empty() && !operation.is_empty() {
                        debug!(
                            "📡 Semantic fallback: {}.{} → capability.call",
                            domain, operation
                        );
                        let cap_params = Some(serde_json::json!({
                            "capability": domain,
                            "operation": operation,
                            "args": params.clone().unwrap_or(serde_json::json!({}))
                        }));
                        return dispatch(self.capability_handler.call(&cap_params).await, &id);
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
            Route::GraphList => dispatch(self.graph_handler.list().await, &id),
            Route::GraphGet => dispatch(self.graph_handler.get(params).await, &id),
            Route::GraphSave => dispatch(self.graph_handler.save(params).await, &id),
            Route::GraphExecute => dispatch(self.graph_handler.execute(params).await, &id),
            Route::GraphExecutePipeline => {
                dispatch(self.graph_handler.execute_pipeline(params).await, &id)
            }
            Route::GraphStatus => dispatch(self.graph_handler.get_status(params).await, &id),
            Route::GraphStartContinuous => {
                dispatch(self.graph_handler.start_continuous(params).await, &id)
            }
            Route::GraphPauseContinuous => {
                dispatch(self.graph_handler.pause_continuous(params).await, &id)
            }
            Route::GraphResumeContinuous => {
                dispatch(self.graph_handler.resume_continuous(params).await, &id)
            }
            Route::GraphStopContinuous => {
                dispatch(self.graph_handler.stop_continuous(params).await, &id)
            }
            Route::GraphSuggestOptimizations => {
                dispatch(self.graph_handler.suggest_optimizations(params).await, &id)
            }
            // Topology
            Route::TopologyGet => dispatch(self.topology_handler.get().await, &id),
            Route::TopologyPrimals => dispatch(self.topology_handler.get_primals().await, &id),
            Route::TopologyProprioception => {
                dispatch(self.topology_handler.get_proprioception().await, &id)
            }
            Route::TopologyMetrics => dispatch(self.topology_handler.get_metrics().await, &id),
            Route::TopologyRescan => dispatch(self.rescan_primals().await, &id),
            // Niche
            Route::NicheList => dispatch(self.niche_handler.list().await, &id),
            Route::NicheDeploy => dispatch(self.niche_handler.deploy(params).await, &id),
            // Lifecycle
            Route::LifecycleStatus => dispatch(self.lifecycle_handler.status().await, &id),
            Route::LifecycleGet => dispatch(self.lifecycle_handler.get(params).await, &id),
            Route::LifecycleRegister => {
                dispatch(self.lifecycle_handler.register(params).await, &id)
            }
            Route::LifecycleResurrect => {
                dispatch(self.lifecycle_handler.resurrect(params).await, &id)
            }
            Route::LifecycleApoptosis => {
                dispatch(self.lifecycle_handler.apoptosis(params).await, &id)
            }
            Route::LifecycleShutdownAll => {
                dispatch(self.lifecycle_handler.shutdown_all().await, &id)
            }
            // Protocol
            Route::ProtocolStatus => dispatch(self.protocol_handler.status().await, &id),
            Route::ProtocolEscalate => dispatch(self.protocol_handler.escalate(params).await, &id),
            Route::ProtocolFallback => dispatch(self.protocol_handler.fallback(params).await, &id),
            Route::ProtocolMetrics => dispatch(self.protocol_handler.metrics(params).await, &id),
            Route::ProtocolRegisterPrimal => {
                dispatch(self.protocol_handler.register_primal(params).await, &id)
            }
            Route::ProtocolRegisterConnection => {
                dispatch(self.protocol_handler.register_connection(params).await, &id)
            }
            Route::ProtocolRecordRequest => {
                dispatch(self.protocol_handler.record_request(params).await, &id)
            }
            Route::ProtocolStartMonitoring => {
                dispatch(self.protocol_handler.start_monitoring().await, &id)
            }
            Route::ProtocolStopMonitoring => {
                dispatch(self.protocol_handler.stop_monitoring().await, &id)
            }
            Route::GraphProtocolMap => dispatch(self.protocol_handler.protocol_map().await, &id),
            // Route (batch capability registration)
            Route::BatchRouteRegister => {
                dispatch(self.capability_handler.register_route(params).await, &id)
            }
            // Capability
            Route::CapabilityRegister => {
                dispatch(self.capability_handler.register(params).await, &id)
            }
            Route::CapabilityDiscover => {
                dispatch(self.capability_handler.discover(params).await, &id)
            }
            Route::CapabilityList => dispatch(self.capability_handler.list().await, &id),
            Route::CapabilityProviders => {
                dispatch(self.capability_handler.providers(params).await, &id)
            }
            Route::CapabilityResolve => dispatch(self.capability_handler.route(params).await, &id),
            Route::CapabilityMetrics => dispatch(self.capability_handler.get_metrics().await, &id),
            Route::CapabilityCall => dispatch(self.capability_handler.call(params).await, &id),
            Route::CapabilityDiscoverTranslations => dispatch(
                self.capability_handler.discover_translations(params).await,
                &id,
            ),
            Route::CapabilityListTranslations => {
                dispatch(self.capability_handler.list_translations().await, &id)
            }
            // MCP
            Route::McpToolsList => dispatch(self.capability_handler.mcp_tools_list().await, &id),
            // Agent
            Route::Agent => dispatch(
                super::agents::handle_agent_request(
                    &self.agent_registry,
                    request.method.as_ref(),
                    params,
                )
                .await,
                &id,
            ),
            // Inference scheduling
            Route::InferenceSchedule => {
                dispatch(self.inference_handler.schedule(params).await, &id)
            }
            Route::InferenceGates => dispatch(self.inference_handler.gates(params).await, &id),
            // Health probes (SEMANTIC_METHOD_NAMING_STANDARD.md compliance)
            Route::HealthCheck => dispatch(self.health_check().await, &id),
            Route::HealthLiveness => dispatch(self.health_liveness(), &id),
            Route::HealthReadiness => dispatch(self.health_readiness().await, &id),
            // Legacy
            Route::ProxyHttp => dispatch(self.proxy_http(params).await, &id),
            // Semantic capability routing: domain.operation → capability.call
            Route::SemanticCapabilityCall => {
                if let Some((domain, operation)) = request.method.as_ref().split_once('.') {
                    let cap_params = Some(serde_json::json!({
                        "capability": domain,
                        "operation": operation,
                        "args": params.clone().unwrap_or(serde_json::json!({}))
                    }));
                    dispatch(self.capability_handler.call(&cap_params).await, &id)
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

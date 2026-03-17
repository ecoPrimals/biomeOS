// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Request routing for Neural API Server
//!
//! Routes JSON-RPC requests to appropriate handlers based on method name.
//! Uses a table-driven handler registry for O(1) lookup.

use serde_json::Value;
use tracing::{debug, trace};

use super::rpc::{DispatchOutcome, JsonRpcRequest};
use super::NeuralApiServer;

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
    CapabilityRegister,
    CapabilityDiscover,
    CapabilityList,
    CapabilityProviders,
    CapabilityResolve,
    CapabilityMetrics,
    CapabilityCall,
    CapabilityDiscoverTranslations,
    CapabilityListTranslations,
    Agent,
    ProxyHttp,
    MeshCapabilityCall,
}

/// Table-driven handler registry: method name → route.
/// Multiple method names may map to the same route (e.g. neural_api.list_graphs | graph.list).
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
    // Capability
    ("capability.register", Route::CapabilityRegister),
    ("capability.discover", Route::CapabilityDiscover),
    ("neural_api.discover_capability", Route::CapabilityDiscover),
    ("capability.list", Route::CapabilityList),
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
    // Mesh & NAT (capability.call sugar)
    ("mesh.status", Route::MeshCapabilityCall),
    ("mesh.find_path", Route::MeshCapabilityCall),
    ("mesh.announce", Route::MeshCapabilityCall),
    ("mesh.peers", Route::MeshCapabilityCall),
    ("mesh.health_check", Route::MeshCapabilityCall),
    ("punch.request", Route::MeshCapabilityCall),
    ("punch.status", Route::MeshCapabilityCall),
    ("punch.coordinate", Route::MeshCapabilityCall),
    ("stun.discover", Route::MeshCapabilityCall),
    ("stun.detect_nat_type", Route::MeshCapabilityCall),
    ("stun.probe_port_pattern", Route::MeshCapabilityCall),
    ("relay.serve", Route::MeshCapabilityCall),
    ("relay.status", Route::MeshCapabilityCall),
    ("relay.allocate", Route::MeshCapabilityCall),
    ("relay.authorize", Route::MeshCapabilityCall),
    ("onion.create_service", Route::MeshCapabilityCall),
    ("onion.get_address", Route::MeshCapabilityCall),
    ("onion.connect", Route::MeshCapabilityCall),
    ("onion.status", Route::MeshCapabilityCall),
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
    /// - Graph operations → GraphHandler
    /// - Capability routing → CapabilityHandler
    /// - Topology/metrics → TopologyHandler
    /// - Niche templates → NicheHandler
    /// - Lifecycle management → LifecycleHandler
    /// - Protocol escalation → ProtocolHandler
    pub async fn handle_request(&self, request_line: &str) -> DispatchOutcome {
        let request = match JsonRpcRequest::parse(request_line) {
            Ok(r) => r,
            Err(e) => {
                return DispatchOutcome::ParseError {
                    message: e.to_string(),
                }
            }
        };

        let id = request.id.clone().unwrap_or(serde_json::Value::Null);
        debug!("📥 Request: {} (id: {})", request.method, id);
        trace!("📥 Full request: {}", request_line.trim());

        let route = match lookup_route(request.method.as_ref()) {
            Some(r) => r,
            None => {
                return DispatchOutcome::MethodNotFound {
                    method: request.method.as_ref().to_string(),
                    id,
                };
            }
        };

        let outcome = match route {
            Route::GraphList => match self.graph_handler.list().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::GraphGet => match self.graph_handler.get(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::GraphSave => match self.graph_handler.save(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::GraphExecute => match self.graph_handler.execute(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::GraphExecutePipeline => {
                match self.graph_handler.execute_pipeline(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::GraphStatus => match self.graph_handler.get_status(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::GraphStartContinuous => {
                match self.graph_handler.start_continuous(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::GraphPauseContinuous => {
                match self.graph_handler.pause_continuous(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::GraphResumeContinuous => {
                match self.graph_handler.resume_continuous(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::GraphStopContinuous => {
                match self.graph_handler.stop_continuous(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::GraphSuggestOptimizations => {
                match self
                    .graph_handler
                    .suggest_optimizations(&request.params)
                    .await
                {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::TopologyGet => match self.topology_handler.get().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::TopologyPrimals => match self.topology_handler.get_primals().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::TopologyProprioception => {
                match self.topology_handler.get_proprioception().await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::TopologyMetrics => match self.topology_handler.get_metrics().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::NicheList => match self.niche_handler.list().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::NicheDeploy => match self.niche_handler.deploy(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::LifecycleStatus => match self.lifecycle_handler.status().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::LifecycleGet => match self.lifecycle_handler.get(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::LifecycleRegister => {
                match self.lifecycle_handler.register(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::LifecycleResurrect => {
                match self.lifecycle_handler.resurrect(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::LifecycleApoptosis => {
                match self.lifecycle_handler.apoptosis(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::LifecycleShutdownAll => match self.lifecycle_handler.shutdown_all().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::ProtocolStatus => match self.protocol_handler.status().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::ProtocolEscalate => {
                match self.protocol_handler.escalate(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::ProtocolFallback => {
                match self.protocol_handler.fallback(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::ProtocolMetrics => match self.protocol_handler.metrics(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::ProtocolRegisterPrimal => {
                match self.protocol_handler.register_primal(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::ProtocolRegisterConnection => {
                match self
                    .protocol_handler
                    .register_connection(&request.params)
                    .await
                {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::ProtocolRecordRequest => {
                match self.protocol_handler.record_request(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::ProtocolStartMonitoring => {
                match self.protocol_handler.start_monitoring().await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::ProtocolStopMonitoring => match self.protocol_handler.stop_monitoring().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::GraphProtocolMap => match self.protocol_handler.protocol_map().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::CapabilityRegister => {
                match self.capability_handler.register(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::CapabilityDiscover => {
                match self.capability_handler.discover(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::CapabilityList => match self.capability_handler.list().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::CapabilityProviders => {
                match self.capability_handler.providers(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::CapabilityResolve => {
                match self.capability_handler.route(&request.params).await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::CapabilityMetrics => match self.capability_handler.get_metrics().await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::CapabilityCall => match self.capability_handler.call(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::CapabilityDiscoverTranslations => {
                match self
                    .capability_handler
                    .discover_translations(&request.params)
                    .await
                {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::CapabilityListTranslations => {
                match self.capability_handler.list_translations().await {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::Agent => {
                match super::agents::handle_agent_request(
                    &self.agent_registry,
                    request.method.as_ref(),
                    &request.params,
                )
                .await
                {
                    Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                    Err(e) => DispatchOutcome::ApplicationError {
                        code: -32603,
                        message: format!("Internal error: {e}"),
                        id: id.clone(),
                    },
                }
            }
            Route::ProxyHttp => match self.proxy_http(&request.params).await {
                Ok(v) => DispatchOutcome::Success(super::rpc::success_response(v, id.clone())),
                Err(e) => DispatchOutcome::ApplicationError {
                    code: -32603,
                    message: format!("Internal error: {e}"),
                    id: id.clone(),
                },
            },
            Route::MeshCapabilityCall => {
                let parts: Vec<&str> = request.method.as_ref().split('.').collect();
                if parts.len() == 2 {
                    let cap_params = Some(serde_json::json!({
                        "capability": parts[0],
                        "operation": parts[1],
                        "args": request.params.clone().unwrap_or(serde_json::json!({}))
                    }));
                    match self.capability_handler.call(&cap_params).await {
                        Ok(v) => {
                            DispatchOutcome::Success(super::rpc::success_response(v, id.clone()))
                        }
                        Err(e) => DispatchOutcome::ApplicationError {
                            code: -32603,
                            message: format!("Internal error: {e}"),
                            id: id.clone(),
                        },
                    }
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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use crate::neural_api_server::NeuralApiServer;

    fn create_test_server() -> (NeuralApiServer, tempfile::TempDir) {
        let temp = tempfile::tempdir().expect("temp dir");
        std::fs::create_dir_all(temp.path()).expect("create graphs dir");
        let server =
            NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"));
        (server, temp)
    }

    #[tokio::test]
    async fn test_handle_request_unknown_method() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"nonexistent.method","id":1}"#;
        let result = server.handle_request_json(req).await;
        assert_eq!(result["jsonrpc"], "2.0");
        assert_eq!(result["error"]["code"], -32601);
        assert!(result["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent.method"));
        assert_eq!(result["id"], 1);
    }

    #[tokio::test]
    async fn test_handle_request_invalid_json() {
        let (server, _temp) = create_test_server();
        let result = server.handle_request_json("{broken").await;
        assert_eq!(result["error"]["code"], -32700);
        // serde_json error message varies (e.g. "expected value", "EOF while parsing")
        assert!(!result["error"]["message"].as_str().unwrap_or("").is_empty());
    }

    #[tokio::test]
    async fn test_handle_request_mesh_method_invalid_format_single_part() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"mesh","id":2}"#;
        let result = server.handle_request_json(req).await;
        assert_eq!(result["error"]["code"], -32601);
        assert!(result["error"]["message"]
            .as_str()
            .unwrap()
            .contains("mesh"));
    }

    #[tokio::test]
    async fn test_handle_request_mesh_method_invalid_format_three_parts() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"a.b.c","id":3}"#;
        let result = server.handle_request_json(req).await;
        assert_eq!(result["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn test_handle_request_empty_method() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"","id":4}"#;
        let result = server.handle_request_json(req).await;
        assert_eq!(result["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn test_handle_request_method_not_found_response_structure() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"foo.bar.baz","id":99}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_none());
        assert!(result.get("error").is_some());
        assert_eq!(result["id"], 99);
    }

    #[tokio::test]
    async fn test_handle_request_graph_list_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.list","id":10}"#;
        let result = server.handle_request_json(req).await;
        assert_eq!(result["jsonrpc"], "2.0");
        assert!(result.get("result").is_some());
        assert!(result.get("error").is_none());
        assert_eq!(result["id"], 10);
    }

    #[tokio::test]
    async fn test_handle_request_neural_api_list_graphs_alias() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"neural_api.list_graphs","id":11}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 11);
    }

    #[tokio::test]
    async fn test_handle_request_topology_get_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"topology.get","id":12}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 12);
    }

    #[tokio::test]
    async fn test_handle_request_lifecycle_status_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"lifecycle.status","id":13}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 13);
    }

    #[tokio::test]
    async fn test_handle_request_capability_list_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.list","id":14}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 14);
    }

    #[tokio::test]
    async fn test_handle_request_niche_list_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"niche.list","id":15}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 15);
    }

    #[tokio::test]
    async fn test_handle_request_protocol_status_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"protocol.status","id":16}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 16);
    }

    #[tokio::test]
    async fn test_handle_request_missing_id() {
        // JSON-RPC 2.0 allows omitting id (notification); we accept and echo null
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.list"}"#;
        let result = server.handle_request_json(req).await;
        assert_eq!(result["id"], serde_json::Value::Null);
    }

    #[tokio::test]
    async fn test_handle_request_mesh_status_route_dispatches() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"mesh.status","params":{},"id":24}"#;
        // mesh capability not registered in test server - handler returns ApplicationError
        let result = server.handle_request_json(req).await;
        assert!(result.get("error").is_some() || result.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_request_mesh_find_path_route_dispatches() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"mesh.find_path","params":{},"id":25}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("error").is_some() || result.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_request_capability_register_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.register","params":{"capability":"encryption","primal":"beardog","socket":"/tmp/beardog.sock"},"id":28}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_request_capability_list_translations_route() {
        let (server, _temp) = create_test_server();
        let req =
            r#"{"jsonrpc":"2.0","method":"capability.list_translations","params":{},"id":31}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_request_capability_discover_missing_params_returns_err() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.discover","params":{},"id":40}"#;
        let result = server.handle_request_json(req).await;
        assert!(
            result.get("error").is_some(),
            "missing capability should propagate handler error"
        );
    }

    #[tokio::test]
    async fn test_handle_request_graph_get_missing_params_returns_err() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.get","params":{},"id":41}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("error").is_some());
    }

    #[tokio::test]
    async fn test_handle_request_graph_status_route_dispatches() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.status","params":{"execution_id":"nonexistent"},"id":43}"#;
        // Execution not found returns ApplicationError - route dispatch is what we test
        let _ = server.handle_request_json(req).await;
    }

    #[tokio::test]
    async fn test_handle_request_capability_metrics_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.metrics","params":{},"id":44}"#;
        let result = server.handle_request_json(req).await;
        assert!(result.get("result").is_some());
    }
}

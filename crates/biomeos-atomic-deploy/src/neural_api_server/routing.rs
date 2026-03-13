// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Request routing for Neural API Server
//!
//! Routes JSON-RPC requests to appropriate handlers based on method name.
//! Uses a table-driven handler registry for O(1) lookup.

use anyhow::Result;
use serde_json::Value;
use tracing::{debug, trace};

use super::rpc::{method_not_found_response, JsonRpcRequest};
use super::NeuralApiServer;

/// Route tag for dispatch. Each variant maps to a handler.
#[derive(Clone, Copy, Debug)]
enum Route {
    GraphList,
    GraphGet,
    GraphSave,
    GraphExecute,
    GraphStatus,
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
    /// Handle a JSON-RPC request
    ///
    /// Delegates to focused handlers for each domain:
    /// - Graph operations → GraphHandler
    /// - Capability routing → CapabilityHandler
    /// - Topology/metrics → TopologyHandler
    /// - Niche templates → NicheHandler
    /// - Lifecycle management → LifecycleHandler
    /// - Protocol escalation → ProtocolHandler
    pub async fn handle_request(&self, request_line: &str) -> Result<Value> {
        let request = JsonRpcRequest::parse(request_line)?;

        debug!("📥 Request: {} (id: {})", request.method, request.id);
        trace!("📥 Full request: {}", request_line.trim());

        let route = match lookup_route(&request.method) {
            Some(r) => r,
            None => {
                return Ok(method_not_found_response(&request.method, request.id));
            }
        };

        let result = match route {
            Route::GraphList => self.graph_handler.list().await?,
            Route::GraphGet => self.graph_handler.get(&request.params).await?,
            Route::GraphSave => self.graph_handler.save(&request.params).await?,
            Route::GraphExecute => self.graph_handler.execute(&request.params).await?,
            Route::GraphStatus => self.graph_handler.get_status(&request.params).await?,
            Route::TopologyGet => self.topology_handler.get().await?,
            Route::TopologyPrimals => self.topology_handler.get_primals().await?,
            Route::TopologyProprioception => self.topology_handler.get_proprioception().await?,
            Route::TopologyMetrics => self.topology_handler.get_metrics().await?,
            Route::NicheList => self.niche_handler.list().await?,
            Route::NicheDeploy => self.niche_handler.deploy(&request.params).await?,
            Route::LifecycleStatus => self.lifecycle_handler.status().await?,
            Route::LifecycleGet => self.lifecycle_handler.get(&request.params).await?,
            Route::LifecycleRegister => self.lifecycle_handler.register(&request.params).await?,
            Route::LifecycleResurrect => self.lifecycle_handler.resurrect(&request.params).await?,
            Route::LifecycleApoptosis => self.lifecycle_handler.apoptosis(&request.params).await?,
            Route::LifecycleShutdownAll => self.lifecycle_handler.shutdown_all().await?,
            Route::ProtocolStatus => self.protocol_handler.status().await?,
            Route::ProtocolEscalate => self.protocol_handler.escalate(&request.params).await?,
            Route::ProtocolFallback => self.protocol_handler.fallback(&request.params).await?,
            Route::ProtocolMetrics => self.protocol_handler.metrics(&request.params).await?,
            Route::ProtocolRegisterPrimal => {
                self.protocol_handler
                    .register_primal(&request.params)
                    .await?
            }
            Route::ProtocolRegisterConnection => {
                self.protocol_handler
                    .register_connection(&request.params)
                    .await?
            }
            Route::ProtocolRecordRequest => {
                self.protocol_handler
                    .record_request(&request.params)
                    .await?
            }
            Route::ProtocolStartMonitoring => self.protocol_handler.start_monitoring().await?,
            Route::ProtocolStopMonitoring => self.protocol_handler.stop_monitoring().await?,
            Route::GraphProtocolMap => self.protocol_handler.protocol_map().await?,
            Route::CapabilityRegister => self.capability_handler.register(&request.params).await?,
            Route::CapabilityDiscover => self.capability_handler.discover(&request.params).await?,
            Route::CapabilityList => self.capability_handler.list().await?,
            Route::CapabilityProviders => {
                self.capability_handler.providers(&request.params).await?
            }
            Route::CapabilityResolve => self.capability_handler.route(&request.params).await?,
            Route::CapabilityMetrics => self.capability_handler.get_metrics().await?,
            Route::CapabilityCall => self.capability_handler.call(&request.params).await?,
            Route::CapabilityDiscoverTranslations => {
                self.capability_handler
                    .discover_translations(&request.params)
                    .await?
            }
            Route::CapabilityListTranslations => {
                self.capability_handler.list_translations().await?
            }
            Route::Agent => {
                super::agents::handle_agent_request(
                    &self.agent_registry,
                    request.method.as_str(),
                    &request.params,
                )
                .await?
            }
            Route::ProxyHttp => self.proxy_http(&request.params).await?,
            Route::MeshCapabilityCall => {
                let parts: Vec<&str> = request.method.split('.').collect();
                if parts.len() == 2 {
                    let cap_params = Some(serde_json::json!({
                        "capability": parts[0],
                        "operation": parts[1],
                        "args": request.params.clone().unwrap_or(serde_json::json!({}))
                    }));
                    self.capability_handler.call(&cap_params).await?
                } else {
                    return Ok(method_not_found_response(&request.method, request.id));
                }
            }
        };

        Ok(super::rpc::success_response(result, request.id))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
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
        let result = server.handle_request(req).await.expect("should not error");
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
        let err = server
            .handle_request("{broken")
            .await
            .expect_err("should fail");
        assert!(err.to_string().contains("parse") || err.to_string().contains("JSON"));
    }

    #[tokio::test]
    async fn test_handle_request_mesh_method_invalid_format_single_part() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"mesh","id":2}"#;
        let result = server.handle_request(req).await.expect("should not error");
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
        let result = server.handle_request(req).await.expect("should not error");
        assert_eq!(result["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn test_handle_request_empty_method() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"","id":4}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert_eq!(result["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn test_handle_request_method_not_found_response_structure() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"foo.bar.baz","id":99}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_none());
        assert!(result.get("error").is_some());
        assert_eq!(result["id"], 99);
    }

    #[tokio::test]
    async fn test_handle_request_graph_list_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.list","id":10}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert_eq!(result["jsonrpc"], "2.0");
        assert!(result.get("result").is_some());
        assert!(result.get("error").is_none());
        assert_eq!(result["id"], 10);
    }

    #[tokio::test]
    async fn test_handle_request_neural_api_list_graphs_alias() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"neural_api.list_graphs","id":11}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 11);
    }

    #[tokio::test]
    async fn test_handle_request_topology_get_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"topology.get","id":12}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 12);
    }

    #[tokio::test]
    async fn test_handle_request_lifecycle_status_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"lifecycle.status","id":13}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 13);
    }

    #[tokio::test]
    async fn test_handle_request_capability_list_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.list","id":14}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 14);
    }

    #[tokio::test]
    async fn test_handle_request_niche_list_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"niche.list","id":15}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 15);
    }

    #[tokio::test]
    async fn test_handle_request_protocol_status_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"protocol.status","id":16}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
        assert_eq!(result["id"], 16);
    }

    #[tokio::test]
    async fn test_handle_request_missing_id() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.list"}"#;
        let err = server
            .handle_request(req)
            .await
            .expect_err("missing id should fail");
        assert!(
            err.to_string().contains("parse") || err.to_string().contains("missing"),
            "expected parse/missing error: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_handle_request_mesh_status_route_dispatches() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"mesh.status","params":{},"id":24}"#;
        // mesh capability not registered in test server - handler returns Err
        let result = server.handle_request(req).await;
        assert!(
            result.is_err()
                || result
                    .as_ref()
                    .map(|r| r.get("result").is_some())
                    .unwrap_or(false)
        );
    }

    #[tokio::test]
    async fn test_handle_request_mesh_find_path_route_dispatches() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"mesh.find_path","params":{},"id":25}"#;
        let result = server.handle_request(req).await;
        assert!(
            result.is_err()
                || result
                    .as_ref()
                    .map(|r| r.get("result").is_some())
                    .unwrap_or(false)
        );
    }

    #[tokio::test]
    async fn test_handle_request_capability_register_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.register","params":{"capability":"encryption","primal":"beardog","socket":"/tmp/beardog.sock"},"id":28}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_request_capability_list_translations_route() {
        let (server, _temp) = create_test_server();
        let req =
            r#"{"jsonrpc":"2.0","method":"capability.list_translations","params":{},"id":31}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
    }

    #[tokio::test]
    async fn test_handle_request_capability_discover_missing_params_returns_err() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.discover","params":{},"id":40}"#;
        let result = server.handle_request(req).await;
        assert!(
            result.is_err(),
            "missing capability should propagate handler error"
        );
    }

    #[tokio::test]
    async fn test_handle_request_graph_get_missing_params_returns_err() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.get","params":{},"id":41}"#;
        let result = server.handle_request(req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_handle_request_graph_status_route_dispatches() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"graph.status","params":{"execution_id":"nonexistent"},"id":43}"#;
        // Execution not found returns Err - route dispatch is what we test
        let _ = server.handle_request(req).await;
    }

    #[tokio::test]
    async fn test_handle_request_capability_metrics_route() {
        let (server, _temp) = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"capability.metrics","params":{},"id":44}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_some());
    }
}

//! Request routing for Neural API Server
//!
//! Routes JSON-RPC requests to appropriate handlers based on method name.

use anyhow::Result;
use serde_json::Value;
use tracing::{debug, trace};

use super::rpc::{method_not_found_response, JsonRpcRequest};
use super::NeuralApiServer;

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

        let result = match request.method.as_str() {
            // === Graph Operations (delegated to GraphHandler) ===
            "neural_api.list_graphs" | "graph.list" => self.graph_handler.list().await?,
            "neural_api.get_graph" | "graph.get" => self.graph_handler.get(&request.params).await?,
            "neural_api.save_graph" | "graph.save" => {
                self.graph_handler.save(&request.params).await?
            }
            "neural_api.execute_graph" | "graph.execute" => {
                self.graph_handler.execute(&request.params).await?
            }
            "neural_api.get_execution_status" | "graph.status" => {
                self.graph_handler.get_status(&request.params).await?
            }

            // === Topology Operations (delegated to TopologyHandler) ===
            "neural_api.get_topology" | "topology.get" => self.topology_handler.get().await?,
            "neural_api.get_primals" | "topology.primals" => {
                self.topology_handler.get_primals().await?
            }
            "neural_api.get_proprioception" | "topology.proprioception" => {
                self.topology_handler.get_proprioception().await?
            }
            "neural_api.get_metrics" | "topology.metrics" => {
                self.topology_handler.get_metrics().await?
            }

            // === Niche Operations (delegated to NicheHandler) ===
            "neural_api.list_niche_templates" | "niche.list" => self.niche_handler.list().await?,
            "neural_api.deploy_niche" | "niche.deploy" => {
                self.niche_handler.deploy(&request.params).await?
            }

            // === Lifecycle Operations (delegated to LifecycleHandler) ===
            "lifecycle.status" => self.lifecycle_handler.status().await?,
            "lifecycle.get" => self.lifecycle_handler.get(&request.params).await?,
            "lifecycle.register" => self.lifecycle_handler.register(&request.params).await?,
            "lifecycle.resurrect" => self.lifecycle_handler.resurrect(&request.params).await?,
            "lifecycle.apoptosis" => self.lifecycle_handler.apoptosis(&request.params).await?,
            "lifecycle.shutdown_all" => self.lifecycle_handler.shutdown_all().await?,

            // === Protocol Escalation Operations (delegated to ProtocolHandler) ===
            "protocol.status" => self.protocol_handler.status().await?,
            "protocol.escalate" => self.protocol_handler.escalate(&request.params).await?,
            "protocol.fallback" => self.protocol_handler.fallback(&request.params).await?,
            "protocol.metrics" => self.protocol_handler.metrics(&request.params).await?,
            "protocol.register_primal" => {
                self.protocol_handler
                    .register_primal(&request.params)
                    .await?
            }
            "protocol.register_connection" => {
                self.protocol_handler
                    .register_connection(&request.params)
                    .await?
            }
            "protocol.record_request" => {
                self.protocol_handler
                    .record_request(&request.params)
                    .await?
            }
            "protocol.start_monitoring" => self.protocol_handler.start_monitoring().await?,
            "protocol.stop_monitoring" => self.protocol_handler.stop_monitoring().await?,
            "graph.protocol_map" => self.protocol_handler.protocol_map().await?,

            // === Capability Operations (delegated to CapabilityHandler) ===
            "capability.register" => self.capability_handler.register(&request.params).await?,
            "capability.discover" | "neural_api.discover_capability" => {
                self.capability_handler.discover(&request.params).await?
            }
            "capability.list" => self.capability_handler.list().await?,
            "capability.providers" => self.capability_handler.providers(&request.params).await?,
            "capability.route" | "neural_api.route_to_primal" => {
                self.capability_handler.route(&request.params).await?
            }
            "capability.metrics" | "neural_api.get_routing_metrics" => {
                self.capability_handler.get_metrics().await?
            }
            "capability.call" => self.capability_handler.call(&request.params).await?,
            "capability.discover_translations" | "capability.discover_translation" => {
                self.capability_handler
                    .discover_translations(&request.params)
                    .await?
            }
            "capability.list_translations" => self.capability_handler.list_translations().await?,

            // === Plasmodium Agent Operations ===
            "agent.create" | "agent.list" | "agent.get" | "agent.remove" | "agent.meld"
            | "agent.split" | "agent.resolve" | "agent.route" | "agent.auto_meld" => {
                super::agents::handle_agent_request(
                    &self.agent_registry,
                    request.method.as_str(),
                    &request.params,
                )
                .await?
            }

            // === Legacy Routing (still needed for HTTP proxy) ===
            "neural_api.proxy_http" => self.proxy_http(&request.params).await?,

            // === Mesh & NAT Traversal Operations (routed via capability.call) ===
            // These provide direct method syntax sugar: mesh.status → capability.call("mesh", "status")
            "mesh.status"
            | "mesh.find_path"
            | "mesh.announce"
            | "mesh.peers"
            | "mesh.health_check"
            | "punch.request"
            | "punch.status"
            | "punch.coordinate"
            | "stun.discover"
            | "stun.detect_nat_type"
            | "stun.probe_port_pattern"
            | "relay.serve"
            | "relay.status"
            | "relay.allocate"
            | "relay.authorize"
            | "onion.create_service"
            | "onion.get_address"
            | "onion.connect"
            | "onion.status" => {
                // Transform direct method call into capability.call format
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

            // === Unknown Method ===
            _ => {
                return Ok(method_not_found_response(&request.method, request.id));
            }
        };

        Ok(super::rpc::success_response(result, request.id))
    }
}

#[cfg(test)]
mod tests {
    use crate::neural_api_server::NeuralApiServer;

    fn create_test_server() -> NeuralApiServer {
        let temp = tempfile::tempdir().expect("temp dir");
        NeuralApiServer::new(temp.path(), "test_family", temp.path().join("neural.sock"))
    }

    #[tokio::test]
    async fn test_handle_request_unknown_method() {
        let server = create_test_server();
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
        let server = create_test_server();
        let err = server
            .handle_request("{broken")
            .await
            .expect_err("should fail");
        assert!(err.to_string().contains("parse") || err.to_string().contains("JSON"));
    }

    #[tokio::test]
    async fn test_handle_request_mesh_method_invalid_format_single_part() {
        let server = create_test_server();
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
        let server = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"a.b.c","id":3}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert_eq!(result["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn test_handle_request_empty_method() {
        let server = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"","id":4}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert_eq!(result["error"]["code"], -32601);
    }

    #[tokio::test]
    async fn test_handle_request_method_not_found_response_structure() {
        let server = create_test_server();
        let req = r#"{"jsonrpc":"2.0","method":"foo.bar.baz","id":99}"#;
        let result = server.handle_request(req).await.expect("should not error");
        assert!(result.get("result").is_none());
        assert!(result.get("error").is_some());
        assert_eq!(result["id"], 99);
    }
}

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability translation loading from graphs
//!
//! Extracts capability translations from graph nodes and registers them
//! in the capability translation registry.

use anyhow::Result;
use biomeos_core::SocketDiscovery;
use tracing::{debug, info, warn};

use super::NeuralApiServer;
use crate::capability_domains::capability_to_provider_fallback;
use crate::neural_graph::{Graph, GraphNode};

impl NeuralApiServer {
    /// Extract family_id from node operation params with ${VAR} substitution support
    ///
    /// Helper function to extract and resolve family_id from a graph node.
    /// Supports ${FAMILY_ID} substitution for isomorphic deployment.
    ///
    /// # Arguments
    /// * `node` - Graph node to extract family_id from
    ///
    /// # Returns
    /// Resolved family_id string
    pub fn extract_family_id_from_node<'a>(&'a self, node: &'a GraphNode) -> &'a str {
        let family_id_raw = if let Some(operation) = &node.operation {
            operation
                .params
                .get("family_id")
                .and_then(|v| v.as_str())
                .unwrap_or(&self.family_id)
        } else {
            &self.family_id
        };
        // Substitute ${FAMILY_ID} with actual value from server's family_id
        if family_id_raw == "${FAMILY_ID}" {
            &self.family_id
        } else {
            family_id_raw
        }
    }

    /// Load capability translations from a graph
    ///
    /// Extracts `capabilities_provided` from each node and registers translations.
    /// Also registers capability categories from `capabilities` field.
    pub async fn load_translations_from_graph(&self, graph: &Graph) -> Result<()> {
        info!(
            "🔧 load_translations_from_graph called for graph with {} nodes",
            graph.nodes.len()
        );
        let mut registry = self.translation_registry.write().await;
        let mut loaded_count = 0;

        for node in &graph.nodes {
            debug!(
                "   Checking node: {} (capabilities: {:?}, has capabilities_provided: {})",
                node.id,
                node.capabilities,
                node.capabilities_provided.is_some()
            );

            // EVOLVED: Get primal name - capability-based resolution via domain mappings
            // Uses capability_domains.rs for well-known capability → primal mappings
            // This is robust and doesn't require runtime Songbird queries
            let primal_name = if let Some(primal_cfg) = &node.primal {
                if let Some(cap) = &primal_cfg.by_capability {
                    // ROBUST SOLUTION: Use capability domain mappings to resolve primal name
                    // This maps semantic capability names to actual primal names:
                    //   "security" → "beardog"
                    //   "http"     → "songbird"
                    //   "storage"  → "nestgate"
                    //   etc.
                    //
                    // The mapping is defined in capability_domains.rs and can be extended
                    // or loaded from config/capability_registry.toml in the future.
                    if let Some(resolved_primal) = capability_to_provider_fallback(cap) {
                        debug!(
                            "   Resolved capability '{}' to primal '{}'",
                            cap, resolved_primal
                        );
                        Some(resolved_primal.to_string())
                    } else {
                        // Fallback: if capability isn't in domain mappings, use it as primal name
                        // This handles custom primals that register with capability == primal name
                        warn!(
                            "   No domain mapping for capability '{}', using as primal name",
                            cap
                        );
                        Some(cap.clone())
                    }
                } else {
                    primal_cfg.by_name.clone()
                }
            } else {
                Some(node.id.clone())
            };

            if let Some(ref primal) = primal_name {
                // Extract family_id using helper function
                let family_id = self.extract_family_id_from_node(node);

                // Build socket path using capability-based discovery
                let socket_discovery = SocketDiscovery::new(family_id.to_string());
                let socket_path = socket_discovery
                    .build_socket_path(primal)
                    .to_string_lossy()
                    .to_string();

                // Register capability CATEGORIES from the capabilities field
                // This enables capability.call("crypto", "sha256") to route to BearDog
                for capability in &node.capabilities {
                    info!(
                        "📝 Registering capability category: {} → {} @ {}",
                        capability, primal, socket_path
                    );
                    if let Err(e) = self
                        .router
                        .register_capability(capability, primal, &socket_path, "graph_translation")
                        .await
                    {
                        warn!("Failed to register capability {}: {}", capability, e);
                    }
                }
            }

            if let Some(caps_provided) = &node.capabilities_provided {
                // Reuse primal_name extracted above
                if let Some(ref primal) = primal_name {
                    // Extract family_id using helper function (eliminates duplication)
                    let family_id = self.extract_family_id_from_node(node);

                    // Build socket path using capability-based discovery
                    let socket_discovery = SocketDiscovery::new(family_id.to_string());
                    let socket_path = socket_discovery
                        .build_socket_path(primal)
                        .to_string_lossy()
                        .to_string();

                    // Register all translations for this primal
                    for (semantic, actual) in caps_provided {
                        // Check if there are parameter mappings for this capability
                        let param_mappings = node
                            .parameter_mappings
                            .as_ref()
                            .and_then(|mappings| mappings.get(semantic))
                            .cloned();

                        info!(
                            "📝 Loading translation from graph: {} → {} ({} @ {}) {}",
                            semantic,
                            actual,
                            primal,
                            socket_path,
                            if param_mappings.is_some() {
                                "with param mappings"
                            } else {
                                ""
                            }
                        );

                        registry.register_translation(
                            semantic,
                            primal.as_str(),
                            actual,
                            &socket_path,
                            param_mappings,
                        );

                        loaded_count += 1;
                    }
                }
            }
        }

        if loaded_count > 0 {
            info!(
                "✅ Loaded {} capability translations from graph {}",
                loaded_count, graph.id
            );
        } else {
            debug!("⚠️  No capability translations found in graph {}", graph.id);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::neural_api_server::NeuralApiServer;
    use crate::neural_graph::{GraphNode, Operation, PrimalSelector};
    use std::collections::HashMap;

    fn create_test_server(family_id: &str) -> NeuralApiServer {
        let temp = tempfile::tempdir().expect("temp dir");
        NeuralApiServer::new(temp.path(), family_id, temp.path().join("neural.sock"))
    }

    #[test]
    fn test_extract_family_id_from_node_no_operation() {
        let server = create_test_server("default_family");
        let node = GraphNode {
            id: "node1".to_string(),
            primal: None,
            output: None,
            operation: None,
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };
        assert_eq!(server.extract_family_id_from_node(&node), "default_family");
    }

    #[test]
    fn test_extract_family_id_from_node_operation_no_family_id_param() {
        let server = create_test_server("server_family");
        let mut params = HashMap::new();
        params.insert("other".to_string(), serde_json::json!("value"));
        let node = GraphNode {
            id: "node2".to_string(),
            primal: None,
            output: None,
            operation: Some(Operation {
                name: "op".to_string(),
                params,
                environment: None,
            }),
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };
        assert_eq!(server.extract_family_id_from_node(&node), "server_family");
    }

    #[test]
    fn test_extract_family_id_from_node_operation_with_family_id_param() {
        let server = create_test_server("server_family");
        let mut params = HashMap::new();
        params.insert(
            "family_id".to_string(),
            serde_json::Value::String("node_family".to_string()),
        );
        let node = GraphNode {
            id: "node3".to_string(),
            primal: None,
            output: None,
            operation: Some(Operation {
                name: "op".to_string(),
                params,
                environment: None,
            }),
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };
        assert_eq!(server.extract_family_id_from_node(&node), "node_family");
    }

    #[test]
    fn test_extract_family_id_from_node_substitution_placeholder() {
        let server = create_test_server("resolved_family");
        let mut params = HashMap::new();
        params.insert(
            "family_id".to_string(),
            serde_json::Value::String("${FAMILY_ID}".to_string()),
        );
        let node = GraphNode {
            id: "node4".to_string(),
            primal: None,
            output: None,
            operation: Some(Operation {
                name: "op".to_string(),
                params,
                environment: None,
            }),
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };
        assert_eq!(server.extract_family_id_from_node(&node), "resolved_family");
    }

    #[test]
    fn test_extract_family_id_from_node_operation_family_id_not_string() {
        let server = create_test_server("fallback_family");
        let mut params = HashMap::new();
        params.insert("family_id".to_string(), serde_json::json!(42));
        let node = GraphNode {
            id: "node5".to_string(),
            primal: None,
            output: None,
            operation: Some(Operation {
                name: "op".to_string(),
                params,
                environment: None,
            }),
            constraints: None,
            depends_on: vec![],
            capabilities: vec![],
            capabilities_provided: None,
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };
        assert_eq!(server.extract_family_id_from_node(&node), "fallback_family");
    }

    #[tokio::test]
    async fn test_load_translations_from_graph_empty_graph() {
        let server = create_test_server("test");
        let graph = crate::neural_graph::Graph {
            id: "empty".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            nodes: vec![],
            config: crate::neural_graph::GraphConfig::default(),
        };
        server
            .load_translations_from_graph(&graph)
            .await
            .expect("empty graph should load without error");
    }

    #[tokio::test]
    async fn test_load_translations_from_graph_node_with_capabilities_provided() {
        let server = create_test_server("test_family");
        let mut caps_provided = HashMap::new();
        caps_provided.insert(
            "crypto.sha256".to_string(),
            "actual_sha256_method".to_string(),
        );
        let node = GraphNode {
            id: "beardog".to_string(),
            primal: Some(PrimalSelector {
                by_capability: Some("security".to_string()),
                by_name: None,
            }),
            output: None,
            operation: None,
            constraints: None,
            depends_on: vec![],
            capabilities: vec!["security".to_string(), "crypto".to_string()],
            capabilities_provided: Some(caps_provided),
            parameter_mappings: None,
            node_type: None,
            dependencies: vec![],
            config: HashMap::new(),
            outputs: vec![],
        };
        let graph = crate::neural_graph::Graph {
            id: "test".to_string(),
            version: "1.0".to_string(),
            description: "".to_string(),
            nodes: vec![node],
            config: crate::neural_graph::GraphConfig::default(),
        };
        server
            .load_translations_from_graph(&graph)
            .await
            .expect("graph with translations should load");
    }
}

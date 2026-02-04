//! Capability translation loading from graphs
//!
//! Extracts capability translations from graph nodes and registers them
//! in the capability translation registry.

use anyhow::Result;
use biomeos_core::SocketDiscovery;
use tracing::{debug, info, warn};

use super::NeuralApiServer;
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

            // EVOLVED: Get primal name - capability-agnostic, runtime discovery
            // No hardcoded capability-to-primal mapping!
            // Primals self-register their capabilities with Songbird
            let primal_name = if let Some(primal_cfg) = &node.primal {
                if let Some(cap) = &primal_cfg.by_capability {
                    // DEEP DEBT PRINCIPLE: Query Songbird at runtime for capability providers
                    // This allows ecosystem evolution without hardcoding
                    //
                    // IMPLEMENTATION NOTE: Async Songbird resolution would be:
                    // 1. Connect to Songbird via SocketDiscovery
                    // 2. Query: songbird.discover_capability(capability_name)
                    // 3. Receive: primal name providing that capability
                    // 4. Use returned primal name for deployment
                    //
                    // For now, use capability name directly - works for standard primals
                    // where capability name == primal name (security → beardog, etc.)
                    // This maintains zero hardcoding while deferring full async resolution
                    Some(cap.clone())
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

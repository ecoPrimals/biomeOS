// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Neural API JSON-RPC Server
//!
//! Exposes the Neural API graph orchestration engine via JSON-RPC 2.0 over Unix socket.
//! This enables Squirrel and petalTongue to discover, execute, and monitor graph deployments.
//!
//! # Architecture
//!
//! The server delegates to focused handlers for each domain:
//! - `GraphHandler` - Graph CRUD and execution
//! - `CapabilityHandler` - Capability routing and discovery
//! - `TopologyHandler` - System topology and metrics
//! - `NicheHandler` - Niche template deployment
//! - `LifecycleHandler` - Primal lifecycle management
//! - `ProtocolHandler` - Protocol escalation (JSON-RPC → tarpc)
//!
//! This decomposition keeps each handler under 500 lines while the server
//! focuses on connection handling and request routing.

pub mod agents;
mod connection;
mod proxy;
mod routing;
mod rpc;
mod server_lifecycle;
mod translation_loader;

// Re-export types that may be needed externally
pub use rpc::{
    JsonRpcRequest, error_response, internal_error_response, method_not_found_response,
    success_response,
};

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::handlers::{
    CapabilityHandler, GraphHandler, InferenceHandler, LifecycleHandler, NicheHandler,
    ProtocolHandler, TopologyHandler,
};
use crate::living_graph::LivingGraph;
use crate::mode::BiomeOsMode;
use crate::neural_router::NeuralRouter;
use crate::nucleation::SocketNucleation;
use crate::protocol_escalation::{EscalationConfig, ProtocolEscalationManager};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Neural API server state
#[derive(Clone)]
pub struct NeuralApiServer {
    /// Path to graphs directory
    pub(super) graphs_dir: PathBuf,

    /// Family ID for this server
    pub(super) family_id: String,

    /// Socket path
    pub(super) socket_path: PathBuf,

    /// Neural Router for capability-based routing
    pub(super) router: Arc<NeuralRouter>,

    /// Operating mode (Bootstrap or Coordinated)
    pub(super) mode: Arc<RwLock<BiomeOsMode>>,

    /// Socket nucleation (deterministic assignment)
    pub(super) nucleation: Arc<RwLock<SocketNucleation>>,

    /// Capability Translation Registry
    pub(super) translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,

    // === Handlers (delegated logic) ===
    /// Graph operations handler
    pub(super) graph_handler: GraphHandler,

    /// Capability routing handler
    pub(super) capability_handler: CapabilityHandler,

    /// Topology and metrics handler
    pub(super) topology_handler: TopologyHandler,

    /// Niche deployment handler
    pub(super) niche_handler: NicheHandler,

    /// Lifecycle management handler (resurrection, apoptosis)
    pub(super) lifecycle_handler: LifecycleHandler,

    /// Protocol escalation handler (JSON-RPC → tarpc)
    pub(super) protocol_handler: ProtocolHandler,

    /// Inference scheduling handler (cross-gate model routing)
    pub(super) inference_handler: InferenceHandler,

    /// Plasmodium agent registry (meld/split/mix routing contexts)
    pub(super) agent_registry: agents::AgentRegistry,
}

impl NeuralApiServer {
    /// Create a new Neural API server
    ///
    /// Mode detection happens on first serve() call
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        socket_path: impl Into<PathBuf>,
    ) -> Self {
        use crate::nucleation::SocketStrategy;

        let graphs_dir = graphs_dir.into();
        let family_id_str = family_id.into();
        let router = Arc::new(NeuralRouter::new(&family_id_str));
        let executions = Arc::new(RwLock::new(HashMap::new()));
        let translation_registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));

        // Create handlers with shared state
        let graph_handler = GraphHandler::new(
            graphs_dir.clone(),
            family_id_str.clone(),
            executions.clone(),
            router.clone(),
            translation_registry.clone(),
        );

        let capability_handler =
            CapabilityHandler::new(router.clone(), translation_registry.clone());

        let topology_handler = TopologyHandler::new(
            family_id_str.clone(),
            router.clone(),
            executions.clone(),
            graphs_dir.clone(),
        );

        let niche_handler = NicheHandler::new(
            graphs_dir.clone(),
            family_id_str.clone(),
            router.clone(),
            executions.clone(),
        );

        let lifecycle_handler = LifecycleHandler::new(&family_id_str);

        // Living Graph for protocol state tracking
        let living_graph = Arc::new(LivingGraph::new(&family_id_str));

        // Protocol Escalation Manager (JSON-RPC → tarpc)
        let escalation_manager = Arc::new(RwLock::new(ProtocolEscalationManager::new(
            living_graph.clone(),
            EscalationConfig::default(),
        )));

        let protocol_handler = ProtocolHandler::new(living_graph.clone(), escalation_manager);

        let gate_registry = Arc::new(crate::gate_registry::GateRegistry::new());
        let inference_handler =
            InferenceHandler::new(router.clone(), gate_registry);

        Self {
            graphs_dir,
            family_id: family_id_str,
            socket_path: socket_path.into(),
            router,
            mode: Arc::new(RwLock::new(BiomeOsMode::Bootstrap)),
            nucleation: Arc::new(RwLock::new(SocketNucleation::new(
                SocketStrategy::XdgRuntime,
            ))),
            translation_registry,
            graph_handler,
            capability_handler,
            topology_handler,
            niche_handler,
            lifecycle_handler,
            protocol_handler,
            inference_handler,
            agent_registry: agents::AgentRegistry::new(),
        }
    }
}

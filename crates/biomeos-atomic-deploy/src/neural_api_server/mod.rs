// SPDX-License-Identifier: AGPL-3.0-or-later
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
mod bootstrap;
pub(crate) mod btsp_negotiate;
mod connection;
pub(crate) mod discovery_init;
mod listeners;
mod proxy;
mod routing;
mod rpc;
mod server_lifecycle;
mod translation_loader;
pub(crate) mod translation_startup;

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
use std::sync::atomic::AtomicBool;
use tokio::sync::RwLock;

/// Neural API server state
#[derive(Clone)]
pub struct NeuralApiServer {
    /// Path to graphs directory
    pub(super) graphs_dir: PathBuf,

    /// Family ID for this server
    pub(super) family_id: String,

    /// Socket path (UDS)
    pub(super) socket_path: PathBuf,

    /// Optional TCP bind port for mobile/cross-gate orchestration.
    /// When set, the server accepts newline-delimited JSON-RPC over TCP
    /// in addition to (or instead of, if `tcp_only` is true) the Unix socket.
    pub(super) tcp_port: Option<u16>,

    /// When true, skip UDS binding entirely (mobile substrates where
    /// SELinux denies `sock_file create`).
    pub(super) tcp_only: bool,

    /// Optional TCP bind host override (e.g. `"127.0.0.1"`).
    /// Defaults to `0.0.0.0` when `None`.
    pub(super) bind_address: Option<String>,

    /// Neural Router for capability-based routing
    pub(super) router: Arc<NeuralRouter>,

    /// Operating mode (Bootstrap or Coordinated)
    pub(super) mode: Arc<RwLock<BiomeOsMode>>,

    /// Runtime BTSP enforcement flag.  Starts `false` (cleartext bootstrap).
    /// Set to `true` by the `btsp.escalate` RPC or automatically after
    /// Tower health is confirmed. Once set, new connections are BTSP-enforced.
    pub(super) btsp_escalated: Arc<AtomicBool>,

    /// When `true`, BTSP enforcement is disabled regardless of `FAMILY_ID`
    /// or `btsp_escalated`. Set via `--btsp-optional` CLI flag.
    pub(super) btsp_optional: bool,

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

    /// Cached `coordination` purpose key (hex-encoded public key).
    ///
    /// Derived from the security provider via `crypto.derive_public_key` after Tower health
    /// is confirmed. Used for graph signing and verification without
    /// re-deriving on every operation.
    pub(super) coordination_pubkey: Arc<RwLock<Option<String>>>,

    /// BTSP Phase 3 session store: tracks authenticated sessions for
    /// cipher negotiation. Sessions are registered after Phase 2 handshake
    /// and upgraded via `btsp.negotiate`.
    pub(super) btsp_sessions: btsp_negotiate::BtspSessionStore,
}

impl NeuralApiServer {
    /// Create a new Neural API server
    ///
    /// Mode detection happens on first `serve()` call
    pub fn new(
        graphs_dir: impl Into<PathBuf>,
        family_id: impl Into<String>,
        socket_path: impl Into<PathBuf>,
    ) -> Self {
        use crate::nucleation::SocketStrategy;

        let graphs_dir: PathBuf = graphs_dir.into();
        let graphs_dir = if graphs_dir.is_relative() {
            std::env::current_dir()
                .map(|cwd| cwd.join(&graphs_dir))
                .unwrap_or(graphs_dir)
        } else {
            graphs_dir
        };
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

        let gate_registry = Arc::new(crate::gate_registry::GateRegistry::new());

        let capability_handler =
            CapabilityHandler::new(router.clone(), translation_registry.clone())
                .with_gate_registry(Arc::clone(&gate_registry));

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

        let living_graph = Arc::new(LivingGraph::new(&family_id_str));

        let escalation_manager = Arc::new(RwLock::new(ProtocolEscalationManager::new(
            living_graph.clone(),
            EscalationConfig::default(),
        )));

        let protocol_handler = ProtocolHandler::new(living_graph.clone(), escalation_manager);

        let inference_handler = InferenceHandler::new(router.clone(), gate_registry);

        Self {
            graphs_dir,
            family_id: family_id_str,
            socket_path: socket_path.into(),
            tcp_port: None,
            tcp_only: false,
            bind_address: None,
            router,
            mode: Arc::new(RwLock::new(BiomeOsMode::Bootstrap)),
            btsp_escalated: Arc::new(AtomicBool::new(false)),
            btsp_optional: false,
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
            coordination_pubkey: Arc::new(RwLock::new(None)),
            btsp_sessions: btsp_negotiate::new_session_store(),
        }
    }

    /// Enable TCP listener on the given port (alongside UDS).
    #[must_use]
    pub fn with_tcp_port(mut self, port: u16) -> Self {
        self.tcp_port = Some(port);
        self
    }

    /// Enable TCP-only mode (skip UDS binding, mobile/cross-gate).
    #[must_use]
    pub fn with_tcp_only(mut self, port: u16) -> Self {
        self.tcp_port = Some(port);
        self.tcp_only = true;
        self
    }

    /// Override TCP bind address (default: `0.0.0.0`).
    ///
    /// Use `"127.0.0.1"` to restrict to localhost only.
    #[must_use]
    pub fn with_bind_address(mut self, addr: String) -> Self {
        self.bind_address = Some(addr);
        self
    }

    /// Disable BTSP enforcement (accept unauthenticated JSON-RPC).
    #[must_use]
    pub fn with_btsp_optional(mut self) -> Self {
        self.btsp_optional = true;
        self
    }
}

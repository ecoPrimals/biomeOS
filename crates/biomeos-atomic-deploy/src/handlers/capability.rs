// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability routing and discovery handlers.
//!
//! This module handles all capability-related JSON-RPC methods:
//! - `capability.resolve` - Single-step "DNS" resolution for a capability (returns one endpoint)
//! - `capability.discover` - Find primals for a capability (returns a list)
//! - `capability.route` - Route requests to capability providers
//! - `capability.register` - Register new capability providers
//! - `capability.list` - List all known capabilities
//! - `capability.providers` - Get providers for a capability
//! - `capability.call` - Semantic capability invocation
//! - `route.register` - Batch-register all capabilities for a remote primal
//! - `method.register` - Spring method registration into semantic routing (GAP-09)
//!
//! # Architecture
//!
//! ```text
//! Consumer → capability.call({ capability: "crypto", operation: "sha256", args: {...} })
//!              │
//!              ▼
//! CapabilityHandler → Translation Registry → NeuralRouter → Primal
//! ```
//!
//! # Canonical Parameter Format
//!
//! ```json
//! { "capability": "domain", "operation": "method", "args": {...} }
//! ```
//!
//! Backward-compatible: dotted capability names (`"crypto.sha256"`) split on
//! first dot; `"params"` accepted as alias for `"args"`.

mod call;
mod helpers;
mod introspection;
mod registration;
mod routing;

use super::graph::GraphHandler;
use crate::capability_translation::CapabilityTranslationRegistry;
use crate::gate_registry::GateRegistry;
use crate::neural_router::NeuralRouter;
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock as TokioRwLock;
use tokio::sync::RwLock;

/// Result of [`CapabilityHandler::call`] — JSON-RPC `result` plus optional routing trace payload.
#[derive(Debug, Clone)]
pub struct CapabilityCallOutcome {
    /// Provider JSON-RPC result (forwarded primal payload).
    pub result: Value,
    /// When `_routing_trace` was requested, serialized trace for `_routing_trace` on the wire.
    pub routing_trace: Option<Value>,
}

/// Capability handler with all capability-related operations.
#[derive(Clone)]
pub struct CapabilityHandler {
    /// Neural Router for capability-based routing
    pub(super) router: Arc<NeuralRouter>,

    /// Capability Translation Registry
    pub(crate) translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,

    /// Gate registry for cross-gate capability forwarding (mutable at runtime via gate.register / route.register)
    pub(super) gate_registry: Arc<TokioRwLock<GateRegistry>>,

    /// Graphs directory for signal dispatch (composition collapse layer).
    /// When a capability.call targets a signal tier (tower/node/nest/meta),
    /// the handler loads the corresponding signal graph instead of routing
    /// to a nonexistent "tower" primal.
    pub(crate) graphs_dir: Option<PathBuf>,

    /// Family ID for signal graph execution context.
    pub(crate) family_id: String,

    /// Graph handler reference for delegating signal execution.
    pub(crate) graph_handler: Option<Arc<GraphHandler>>,
}

impl CapabilityHandler {
    /// Create a new capability handler.
    pub fn new(
        router: Arc<NeuralRouter>,
        translation_registry: Arc<RwLock<CapabilityTranslationRegistry>>,
    ) -> Self {
        Self {
            router,
            translation_registry,
            gate_registry: Arc::new(TokioRwLock::new(GateRegistry::new())),
            graphs_dir: None,
            family_id: String::new(),
            graph_handler: None,
        }
    }

    /// Create a capability handler with a gate registry for cross-gate routing.
    pub fn with_gate_registry(mut self, registry: Arc<TokioRwLock<GateRegistry>>) -> Self {
        self.gate_registry = registry;
        self
    }

    /// Configure signal dispatch (composition collapse layer).
    pub fn with_signal_dispatch(
        mut self,
        graphs_dir: PathBuf,
        family_id: String,
        graph_handler: Arc<GraphHandler>,
    ) -> Self {
        self.graphs_dir = Some(graphs_dir);
        self.family_id = family_id;
        self.graph_handler = Some(graph_handler);
        self
    }
}

// Tests are in capability_tests.rs to keep this file under 800 lines

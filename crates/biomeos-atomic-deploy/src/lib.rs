// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Atomic deployment orchestration with genetic lineage
//!
//! This module provides modern, idiomatic Rust for deploying Tower, Node, and Nest
//! atomics from a USB seed with genetic lineage mixing.
//!
//! # Design Principles
//!
//! - **No Jelly Strings**: Pure Rust, no bash scripts
//! - **Neural API Integration**: Deterministic orchestration
//! - **Genetic Lineage**: Cryptographic family trust
//! - **Verifiable Deployment**: Every step logged and checkpointed

// Crate-level lint configuration
#![warn(missing_docs)]
#![allow(clippy::doc_markdown)] // Allow technical terms without backticks
#![allow(clippy::too_many_lines)] // Complex orchestration logic
#![allow(clippy::redundant_closure_for_method_calls)] // Method references in chains
#![allow(clippy::map_unwrap_or)] // map_or/map_or_else style preference
#![allow(clippy::single_match_else)] // match vs if let style
#![allow(clippy::implicit_hasher)] // HashMap with default hasher
#![allow(clippy::implicit_clone)] // Clone style
#![allow(clippy::used_underscore_binding)] // Debug logging
#![allow(clippy::needless_pass_by_ref_mut)] // Trait method signatures
#![allow(clippy::future_not_send)] // Async with Serialize refs
#![allow(clippy::unused_self)] // Trait method signatures
#![allow(clippy::manual_let_else)] // Style preference
#![allow(clippy::if_not_else)] // Style preference
#![allow(clippy::or_fun_call)] // unwrap_or with const
#![allow(clippy::redundant_clone)] // Arc/clone patterns
#![allow(clippy::ref_option)] // &Option vs Option<&>
#![allow(clippy::case_sensitive_file_extension_comparisons)] // .sock convention
#![allow(clippy::no_effect_underscore_binding)] // Intentional discard
#![allow(clippy::suboptimal_flops)] // mul_add clarity
#![allow(clippy::bool_to_int_with_if)] // Style
#![allow(clippy::let_and_return)] // Explicit return
#![allow(clippy::needless_pass_by_value)] // API signatures
#![allow(clippy::ignored_unit_patterns)] // Ok(()) matching
#![allow(clippy::match_same_arms)] // Grouped match arms
#![forbid(unsafe_code)] // No unsafe code in deployment

//! # Architecture
//!
//! ```text
//! DeploymentOrchestrator
//!     ↓
//!     ├─→ USB Seed Reader (FamilySeed)
//!     ├─→ Child Seed Derivation (per atomic)
//!     ├─→ Primal Launcher (tokio process management)
//!     ├─→ Health Checker (socket + JSON-RPC)
//!     └─→ Neural API Reporter (graph execution)
//! ```

pub mod beardog_jwt_client; // NEW: BearDog JWT integration for orchestrator
pub mod bootstrap; // NEW: Bootstrap sequence execution and mode transitions
pub mod deployment_graph;
pub mod executor; // Shared executor module (types, context, handlers, spawner)
pub mod health_check;
pub mod orchestrator;
pub mod primal_communication;
pub mod primal_coordinator; // NEW: Discovery-based coordination (TRUE PRIMAL)
pub mod primal_discovery; // NEW: Socket scanning discovery
pub mod primal_launcher; // EVOLVING: Legacy launcher → coordinator // NEW: Primal health verification and BTSP tunnel establishment

// Neural API graph execution (TOML-based deterministic deployment)
mod capability_domains; // Capability domain mappings for fallback resolution
pub mod capability_handlers; // Extracted capability-based primal handlers
pub mod capability_translation;
#[cfg(test)]
mod capability_translation_integration_tests; // Sovereign Onion translation tests
#[cfg(test)]
mod capability_translation_tests;
pub mod handlers; // NEW: Smart decomposition of neural_api_server
pub mod http_client;
pub mod lifecycle_manager; // NEW: Primal lifecycle management (resurrection, apoptosis)
pub mod living_graph; // NEW: Runtime protocol state tracking
pub mod mode;
pub mod neural_api_server; // Refactored into submodules: rpc, connection, routing, proxy, translation_loader, server_lifecycle
pub mod neural_executor;
#[cfg(test)]
mod neural_executor_tests;
pub mod neural_graph;
#[cfg(test)]
mod neural_graph_tests;
pub mod neural_router;
#[cfg(test)]
mod neural_router_tests;
pub mod nucleation; // NEW: Capability-based routing layer
mod proc_metrics;
pub mod protocol_escalation;
mod tarpc_client; // NEW: JSON-RPC → tarpc escalation // Pure Rust /proc metrics (ecoBin v3)

pub use deployment_graph::{AtomicDeploymentGraph, DeploymentResult};
pub use health_check::{HealthChecker, HealthStatus};
pub use orchestrator::{AtomicType, DeploymentConfig, DeploymentOrchestrator};
pub use primal_coordinator::{CoordinationStatus, DeploymentGuide, PrimalCoordinator}; // NEW
pub use primal_discovery::{DiscoveredPrimal, PrimalDiscovery}; // NEW
pub use primal_launcher::{PrimalInstance, PrimalLauncher}; // LEGACY

// Neural API exports - types from executor module, re-exported via neural_executor
pub use executor::{
    ExecutionContext, ExecutionReport, NodeStatus, PhaseResult, PhaseResultSummary,
};
pub use neural_executor::GraphExecutor as NeuralGraphExecutor;
// Backwards compatibility alias
pub use executor::ExecutionContext as NeuralExecutionContext;
pub use neural_graph::{Graph as NeuralGraph, GraphConfig, GraphNode as NeuralGraphNode};
pub use neural_router::{
    AtomicType as RouterAtomicType, DiscoveredAtomic, DiscoveredPrimal as RouterDiscoveredPrimal,
    NeuralRouter, RoutingMetrics,
};

// Lifecycle management exports
pub use lifecycle_manager::{
    ApoptosisReason, HealthConfig, LifecycleManager, LifecycleState, ManagedPrimal, PrimalMetrics,
    ResurrectionConfig,
};

// Living Graph / Protocol Escalation exports
pub use living_graph::{
    ConnectionId, ConnectionMetrics, ConnectionState, LivingGraph, PrimalHealth,
    PrimalProtocolState, ProtocolMode, ProtocolSummary,
};
pub use protocol_escalation::{EscalationConfig, EscalationResult, ProtocolEscalationManager};

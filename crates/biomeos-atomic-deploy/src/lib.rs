// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

#![warn(missing_docs)]
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]
// Crate-specific lint expectations — orchestration crate with complex async,
// Arc-heavy coordination, and trait-method signatures that trigger pedantic/nursery.
// Using #[expect] so clippy reports when a suppression becomes unnecessary.
#![expect(
    clippy::too_many_lines,
    reason = "orchestration functions coordinate multi-step deployment"
)]
#![expect(
    clippy::redundant_closure_for_method_calls,
    reason = "closures in async chains are clearer than method refs"
)]
#![expect(
    clippy::future_not_send,
    reason = "async handlers hold Serialize refs across awaits"
)]
#![expect(
    clippy::needless_pass_by_ref_mut,
    reason = "trait method signatures require &mut self"
)]
#![expect(clippy::unused_self, reason = "trait method signatures")]
#![expect(
    clippy::redundant_clone,
    reason = "Arc/clone patterns in concurrent coordinator code"
)]
#![expect(
    clippy::needless_pass_by_value,
    reason = "public API stability — callers pass owned types"
)]
#![expect(
    clippy::map_unwrap_or,
    reason = "map().unwrap_or() chains read left-to-right in Option/Result pipelines"
)]
#![expect(
    clippy::single_match_else,
    reason = "match is clearer than if-let for enum destructuring"
)]
#![expect(
    clippy::manual_let_else,
    reason = "explicit match preferred over let-else in multi-arm contexts"
)]
#![expect(
    clippy::match_same_arms,
    reason = "grouped match arms for readability when variants share handling"
)]
#![expect(
    clippy::case_sensitive_file_extension_comparisons,
    reason = ".sock convention is case-exact on Unix"
)]
#![expect(
    clippy::no_effect_underscore_binding,
    reason = "intentional binding for debug/trace inspection"
)]
#![expect(
    clippy::used_underscore_binding,
    reason = "underscore-prefixed bindings used in tracing/debug"
)]
#![expect(
    clippy::ignored_unit_patterns,
    reason = "Ok(()) matching in async result chains"
)]
#![expect(
    clippy::implicit_clone,
    reason = "to_string() on &str is idiomatic clone"
)]
#![expect(clippy::or_fun_call, reason = "unwrap_or with short expressions")]
#![expect(
    clippy::if_not_else,
    reason = "negated conditions are sometimes clearer for early-return"
)]
#![expect(
    clippy::let_and_return,
    reason = "explicit let binding aids debugger breakpoints"
)]
#![expect(
    clippy::suboptimal_flops,
    reason = "a * b + c reads clearer than mul_add for metrics"
)]
#![expect(
    clippy::ref_option,
    reason = "&Option<T> in public API for ergonomic call sites"
)]
#![expect(
    clippy::bool_to_int_with_if,
    reason = "if cond { 1 } else { 0 } is self-documenting"
)]

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
pub mod gate_registry; // Cross-gate deployment: gate name → remote biomeOS endpoint
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
mod neural_executor_async_tests;
#[cfg(test)]
mod neural_executor_async_tests2;
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

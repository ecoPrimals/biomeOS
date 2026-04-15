// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Handler modules for Neural API Server.
//!
//! Smart decomposition - each handler focuses on one domain:
//! - `graph`: Graph CRUD and execution
//! - `capability`: Capability routing and discovery
//! - `topology`: System topology and metrics
//! - `niche`: Niche template deployment
//! - `lifecycle`: Primal lifecycle management (resurrection, apoptosis)
//! - `protocol`: Protocol escalation (JSON-RPC → tarpc)
//! - `inference`: Cross-gate model scheduling and GPU routing
//!
//! # Architecture
//!
//! ```text
//! NeuralApiServer
//!   ├── GraphHandler      (CRUD, execute)
//!   ├── CapabilityHandler (routing, discovery)
//!   ├── TopologyHandler   (primals, metrics)
//!   ├── NicheHandler      (templates)
//!   ├── LifecycleHandler  (resurrection, apoptosis)
//!   ├── ProtocolHandler   (escalation, fallback, metrics)
//!   └── InferenceHandler  (model scheduling, GPU gate routing)
//! ```
//!
//! # Capability-Based Design
//!
//! All handlers use capability-based discovery instead of hardcoded primal names.
//! This enables TRUE PRIMAL architecture where primals are discovered by what
//! they can do, not what they're named.

/// Capability routing (`capability_call` submodule: `capability.call`, translation lists).
pub mod capability;
#[cfg(test)]
mod capability_call_tests;
pub(crate) mod capability_heuristics;
#[cfg(test)]
mod capability_list_tests;
mod capability_mcp;
pub mod capability_routing;
#[cfg(test)]
mod capability_tests;
pub mod graph;
#[cfg(test)]
mod graph_tests;
pub mod inference;
pub mod lifecycle;
#[cfg(test)]
mod lifecycle_tests;
pub mod niche;
pub mod protocol;
#[cfg(test)]
mod protocol_tests;
pub mod topology;

pub use capability::{CapabilityCallOutcome, CapabilityHandler};
pub use capability_routing::RoutingPhase;
pub use graph::{ExecutionStatus, GraphHandler};
pub use inference::InferenceHandler;
pub use lifecycle::LifecycleHandler;
pub use niche::NicheHandler;
pub use protocol::ProtocolHandler;
pub use topology::TopologyHandler;

// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Plasmodium Agent Routing
//!
//! Agents are lightweight routing contexts that compose capabilities from
//! multiple gates into a single view. They are NOT processes -- they are
//! routing tables maintained by the Neural API.
//!
//! ## Architecture
//!
//! ```text
//! Agent = { name, family_id, routing_table }
//!
//! routing_table maps capability domains to gate:socket targets:
//!   "crypto.*"  -> local:beardog.sock
//!   "compute.*" -> remote:gate2/toadstool.sock (via Songbird mesh)
//!   "storage.*" -> remote:gate2/nestgate.sock (via Songbird mesh)
//! ```
//!
//! ## Meld/Split/Mix
//!
//! - **Meld**: Combine capabilities from multiple gates into one agent
//! - **Split**: Decompose a melded agent when a gate goes offline
//! - **Mix**: Compose a custom agent from selective capabilities

mod collective;
mod registry;
mod rpc;
mod types;

pub use collective::agents_from_collective;
pub use registry::AgentRegistry;
pub use rpc::handle_agent_request;
pub use types::{AgentState, CapabilityRoute, PlasmodiumAgent};

#[cfg(test)]
mod agents_tests;

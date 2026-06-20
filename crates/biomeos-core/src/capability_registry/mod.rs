// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! # biomeOS Capability Registry
//!
//! Central registry for primal capabilities. Enables O(N) scaling by providing
//! a single lookup point for "who provides what?" queries.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │    biomeOS Capability Registry          │
//! ├─────────────────────────────────────────┤
//! │                                         │
//! │  ┌──────────────────────────────────┐  │
//! │  │   Registry Core                   │  │
//! │  │  • Primal registration            │  │
//! │  │  • Capability lookup              │  │
//! │  │  • Health tracking                │  │
//! │  └──────────────────────────────────┘  │
//! │                                         │
//! │  ┌──────────────────────────────────┐  │
//! │  │   Unix Socket IPC Server          │  │
//! │  │  • XDG runtime dir/registry.sock  │ │
//! │  │  • JSON-RPC protocol              │  │
//! │  │  • Async connection handling      │  │
//! │  └──────────────────────────────────┘  │
//! │                                         │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::capability_registry::CapabilityRegistry;
//! use biomeos_core::family_discovery::get_family_id;
//! use biomeos_types::Capability;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create registry with dynamic family discovery
//!     let family_id = get_family_id(); // Discovers from .family.seed or env
//!     let registry = CapabilityRegistry::new(family_id);
//!     
//!     // Start Unix socket server
//!     registry.serve().await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Module layout
//!
//! - [`types`]: JSON-RPC wire protocol types (`PrimalInfo`, `RegistryRequest`, etc.)
//! - [`registry`]: In-memory registry core (register, lookup, heartbeat)
//! - [`server`]: Unix socket IPC server and request dispatch

mod registry;
mod server;
mod types;

pub use registry::CapabilityRegistry;
pub use types::{PrimalInfo, RegisterParams, RegistryRequest, RegistryResponse, ResponseStatus};

// Tests are in capability_registry_tests.rs to keep this file under 1000 lines

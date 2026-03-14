// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Sub-federation management
//!
//! Hierarchical federation on top of genetic lineage baseline.
//! Sub-federations allow granular access control and isolation.

mod beardog;
mod manager;
mod types;

#[cfg(test)]
mod tests;

pub use manager::SubFederationManager;
pub use types::{IsolationLevel, NodeId, SubFederation};

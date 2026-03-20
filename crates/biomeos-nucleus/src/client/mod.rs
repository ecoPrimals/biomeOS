// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unix socket JSON-RPC client and NUCLEUS coordinator
//!
//! Shared utility for talking to primals via Unix sockets.
//! Used by all NUCLEUS layers to avoid code duplication.
//!
//! Also provides the high-level `NucleusClient` that coordinates all 5 layers.

mod coordinator;
mod family_seed;
mod transport;

#[cfg(test)]
mod tests;

pub use coordinator::{NucleusClient, NucleusClientBuilder};
pub use transport::call_unix_socket_rpc;

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Platform-agnostic IPC transport layer (mirrors `biomeos_core::ipc`).
//!
//! Kept local to avoid a dependency cycle: `biomeos-core` depends on this crate.

mod btsp_handshake;
mod connect;
mod endpoint;
mod jsonrpc;
mod stream;

pub use connect::{connect_transport, connect_transport_timed};
pub use endpoint::TransportEndpoint;
pub use jsonrpc::{send_jsonrpc_over_stream, send_jsonrpc_request};
pub use stream::TransportStream;

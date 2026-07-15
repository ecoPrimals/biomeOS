// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of AtomicClient tests (split from `atomic_client_tests.rs`).
//!
//! Domain-focused modules:
//! - [`discovery`] — endpoint and capability discovery (failures and env success)
//! - [`constructors`] — client constructors, opts, and `ExecutionResult` unit tests
//! - [`connection_errors`] — connection refused and timeout without mock servers
//! - [`primal_client`] — `AtomicPrimalClient` error paths
//! - [`unix_rpc`] — Unix socket mock-server JSON-RPC and `try_call` paths
//! - [`http_transport`] — HTTP JSON-RPC transport
//! - [`streaming`] — `call_stream` behavior
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod common;
mod connection_errors;
mod constructors;
mod discovery;
mod http_transport;
mod primal_client;
mod streaming;
mod unix_rpc;

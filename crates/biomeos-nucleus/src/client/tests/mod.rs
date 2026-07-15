// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Nucleus client tests (split from `tests.rs`).
//!
//! Domain-focused modules:
//! - [`common`] — shared mocks and test client factory
//! - [`discovery`] — `NucleusClient::discover` coordinator paths
//! - [`builder`] — `NucleusClientBuilder` smoke tests
//! - [`jsonrpc`] — JSON-RPC request/response serialization
//! - [`unix_rpc`] — Unix socket RPC happy path and basic errors
//! - [`unix_rpc_edge_cases`] — malformed responses and early close
//! - [`unix_rpc_timeouts`] — socket read timeout paths
//! - [`family_seed`] — family seed loading from env/XDG
#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

mod builder;
mod common;
mod discovery;
mod family_seed;
mod jsonrpc;
mod unix_rpc;
mod unix_rpc_edge_cases;
mod unix_rpc_timeouts;

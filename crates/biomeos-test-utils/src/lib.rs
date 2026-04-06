// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` Test Utilities
//!
//! Shared test infrastructure for `BiomeOS` crates, including:
//! - Mock primal servers
//! - VM test harnesses
//! - Network test utilities
//! - Fixture management
//!
//! This crate is only for testing - it should never be used in production code.

#![warn(missing_docs)]
#![forbid(unsafe_code)]
// test-only crate — expect/unwrap are idiomatic in test infrastructure.
// #![allow] rather than #![expect] because the non-test production surface
// of this crate has zero unwrap/expect calls; the submodule test blocks
// carry their own #[expect] attributes.
#![allow(clippy::expect_used, clippy::unwrap_used)]

pub mod assertions;
pub mod fixtures;
pub mod mock_jsonrpc_server;
pub mod mock_primal;
pub mod ready_signal;

pub use fixtures::{create_test_config, create_test_manifest};
pub use mock_jsonrpc_server::MockJsonRpcServer;
pub use mock_primal::{MockPrimal, MockPrimalBuilder};
pub use ready_signal::{ReadyReceiver, ReadySender, ready_signal};

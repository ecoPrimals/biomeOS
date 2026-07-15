// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of CapabilityRegistry tests (split from `capability_registry_tests.rs`).
//!
//! Domain-focused modules:
//! - [`serialization`] — request/response JSON serialization
//! - [`socket_registration`] — socket server register and validation paths
//! - [`socket_queries`] — list, unregister, lookup, and resilience over the socket
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod common;
mod serialization;
mod socket_queries;
mod socket_registration;

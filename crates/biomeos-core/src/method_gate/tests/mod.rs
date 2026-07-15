// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Method gate tests (split from `tests.rs`).

#![expect(clippy::unwrap_used, reason = "Tests use unwrap for clarity")]

mod auth_contract;
mod auth_introspection;
mod caller_context;
mod classify;
mod common;
mod ionic_claims;
mod method_gate_check;
mod resource_envelope;
mod scope;
mod verifier;

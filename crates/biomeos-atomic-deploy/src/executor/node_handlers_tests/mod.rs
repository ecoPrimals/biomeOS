// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Node handler tests (split from `node_handlers_tests.rs`).
//!
//! Domain-focused modules:
//! - [`substitute_env`] — `${VAR}` template substitution
//! - [`logging`] — log.info/warn/error handlers
//! - [`deployment_report`] — deployment report aggregation
//! - [`filesystem`] — filesystem.check_exists handler
//! - [`discovery`] — capability provider discovery
//! - [`crypto_lineage`] — crypto/lineage fallback paths
//! - [`register_capabilities`] — capability registration
//! - [`rpc_integration`] — mock JSON-RPC socket integration tests

#![expect(clippy::unwrap_used, reason = "test assertions")]

mod common;
mod crypto_lineage;
mod deployment_report;
mod discovery;
mod filesystem;
mod logging;
mod register_capabilities;
mod rpc_integration;
mod substitute_env;

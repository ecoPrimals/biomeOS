// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! AtomicClient Tests
//!
//! Extracted from atomic_client.rs to maintain files under 1000 lines.
//! Tests cover JSON-RPC requests/responses, client constructors, configuration,
//! transport endpoints, and edge cases.
//! Additional tests: `atomic_client_tests2/` (domain-focused modules).

use super::atomic_client::*;
use crate::TransportEndpoint;
use crate::atomic_primal_client::AtomicPrimalClient;
use biomeos_types::JsonRpcVersion;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

mod atomic_client;
mod integration;
mod jsonrpc;
mod primal_client;

// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC `capability.call` orchestration (Route→Resolve→Forward).

mod dispatch;
mod gate;
mod mesh;
mod preamble;
mod translations;

use super::{CapabilityCallOutcome, CapabilityHandler};
use anyhow::{Context, Result};
use serde_json::Value;

impl CapabilityHandler {
    /// Semantic capability call with automatic translation.
    ///
    /// JSON-RPC method: `capability.call`
    ///
    /// This is the main entry point for TRUE PRIMAL communication.
    /// Consumers use semantic names; we translate and route.
    ///
    /// # Parameters (canonical format)
    /// - `capability`: Target capability domain (e.g., "crypto")
    /// - `operation`: Semantic operation (e.g., "sha256")
    /// - `args`: Arguments for the operation
    ///
    /// # Backward-compatible formats
    /// - Dotted capability: `{ "capability": "crypto.sha256", "args": {...} }`
    ///   splits on first dot into domain + operation.
    /// - `params` alias: `{ "capability": "crypto", "operation": "sha256", "params": {...} }`
    ///   treated as `args`.
    ///
    /// # Routing trace
    /// Set `"_routing_trace": true` to receive a serialized trace in
    /// [`CapabilityCallOutcome::routing_trace`] (wired as JSON-RPC `_routing_trace` by the
    /// Neural API router). See `specs/CAPABILITY_CALL_ROUTING_CONTRACT.md`.
    pub async fn call(&self, params: &Option<Value>) -> Result<CapabilityCallOutcome> {
        let start = std::time::Instant::now();
        let params = params.as_ref().context("Missing parameters")?;
        let ctx = preamble::CallContext::from_params(params)?;

        if let Some(outcome) = self.try_gate_routing(&ctx, start).await? {
            return Ok(outcome);
        }

        self.dispatch_local(&ctx, start).await
    }
}

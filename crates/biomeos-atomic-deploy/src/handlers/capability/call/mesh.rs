// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Songbird mesh relay dispatch for cross-gate capability calls.

use super::super::CapabilityHandler;
use anyhow::Result;
use serde_json::{Value, json};
use tracing::debug;

impl CapabilityHandler {
    /// CG-8: Attempt cross-gate dispatch through Songbird mesh.
    ///
    /// Forwards `capability.call` to Songbird with `routing: "any"`, which lets
    /// Songbird resolve locally first, then transparently forward over mesh TCP
    /// to remote peers (with TURN relay fallback for NAT). Returns `None` if
    /// Songbird is unavailable or capability is not found on any reachable gate.
    pub(super) async fn try_songbird_mesh_dispatch(
        &self,
        capability: &str,
        operation: &str,
        args: &Value,
        timeout: Option<std::time::Duration>,
    ) -> Option<Result<Value>> {
        let relay_endpoint = self.router.find_primal_by_capability("relay").await.ok()?;

        debug!(
            "Songbird mesh dispatch: {capability}.{operation} via {}",
            relay_endpoint.endpoint.display_string()
        );

        let songbird_params = json!({
            "capability": capability,
            "operation": operation,
            "params": args,
            "routing": "any",
        });

        let mesh_timeout = timeout.or(Some(std::time::Duration::from_secs(15)));
        let result = self
            .router
            .forward_request_with_timeout(
                &relay_endpoint.endpoint,
                "capability.call",
                &songbird_params,
                mesh_timeout,
            )
            .await;

        match &result {
            Ok(response) => {
                let gate = response
                    .get("gate")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let provider = response
                    .get("provider")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                debug!("Songbird mesh resolved: {capability}.{operation} → {provider} @ {gate}");
                // Unwrap inner result: Songbird wraps as { provider, gate, result }
                let inner = response
                    .get("result")
                    .cloned()
                    .unwrap_or_else(|| response.clone());
                Some(Ok(inner))
            }
            Err(e) => {
                debug!("Songbird mesh dispatch failed for {capability}.{operation}: {e}");
                None
            }
        }
    }
}

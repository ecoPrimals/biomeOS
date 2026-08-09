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
        self.songbird_dispatch_inner(capability, operation, args, timeout, "any", None)
            .await
    }

    /// Targeted mesh dispatch using swarmVine gossip intelligence.
    ///
    /// Unlike broadcast `routing: "any"`, this sends the request directly to a
    /// specific gate that advertised the capability via gossip. Faster because
    /// songBird can skip peer-by-peer probing.
    pub(super) async fn try_songbird_mesh_dispatch_targeted(
        &self,
        capability: &str,
        operation: &str,
        args: &Value,
        timeout: Option<std::time::Duration>,
        target_gate: &str,
        target_primal: &str,
    ) -> Option<Result<Value>> {
        debug!(
            "Targeted mesh dispatch: {capability}.{operation} → {target_primal} @ {target_gate}"
        );
        self.songbird_dispatch_inner(
            capability,
            operation,
            args,
            timeout,
            "targeted",
            Some((target_gate, target_primal)),
        )
        .await
    }

    async fn songbird_dispatch_inner(
        &self,
        capability: &str,
        operation: &str,
        args: &Value,
        timeout: Option<std::time::Duration>,
        routing: &str,
        target: Option<(&str, &str)>,
    ) -> Option<Result<Value>> {
        let relay_endpoint = self.router.find_primal_by_capability("relay").await.ok()?;

        debug!(
            "Songbird mesh dispatch ({routing}): {capability}.{operation} via {}",
            relay_endpoint.endpoint.display_string()
        );

        let mut songbird_params = json!({
            "capability": capability,
            "operation": operation,
            "params": args,
            "routing": routing,
        });

        if let Some((gate, primal)) = target {
            songbird_params["target_gate"] = json!(gate);
            songbird_params["target_primal"] = json!(primal);
        }

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

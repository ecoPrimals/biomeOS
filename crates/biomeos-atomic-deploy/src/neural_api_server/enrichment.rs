// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability call parameter enrichment (JH-2, JH-11, exp111).
//!
//! Injects authentication and resource envelope fields into forwarded
//! capability call params before they reach the `CapabilityHandler`.

use biomeos_core::method_gate::CallerContext;
use serde_json::{Value, json};

use super::NeuralApiServer;

impl NeuralApiServer {
    /// Enrich capability call params with forwarding context.
    ///
    /// Injected fields:
    /// - `_resource_envelope` — downstream primals enforce cpu/mem/timeout_ms/method_allowlist.
    /// - `_bearer_token` — downstream primals in enforced mode need the caller's
    ///   token for their own MethodGate check.
    /// - `_token_verified` — whether biomeOS verified the token against BearDog
    ///   via IPC (JH-11 federation step 1).
    pub(crate) async fn enrich_for_forwarding(
        &self,
        params: &Option<Value>,
        caller: &CallerContext,
    ) -> Option<Value> {
        let mut enriched = params.clone().unwrap_or(json!({}));

        if let Some(obj) = enriched.as_object_mut() {
            if let Some(ref claims) = caller.claims {
                if let Some(ref env) = claims.resources {
                    obj.insert("_resource_envelope".to_string(), env.to_forwarding_value());
                }
            }
            if let Some(ref token) = caller.bearer_token {
                obj.insert("_bearer_token".to_string(), json!(token));

                let verified = if let Some(ref verifier) = self.beardog_verifier {
                    verifier.verify_async(token).await.is_some()
                } else {
                    false
                };
                obj.insert("_token_verified".to_string(), json!(verified));
            }
        }

        Some(enriched)
    }
}

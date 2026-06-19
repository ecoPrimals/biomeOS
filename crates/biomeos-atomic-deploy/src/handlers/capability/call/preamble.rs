// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Parameter parsing and resource-envelope validation for `capability.call`.

use anyhow::{Context, Result};
use serde_json::{Value, json};

/// Parsed, validated inputs for a semantic capability invocation.
pub(super) struct CallContext {
    pub(super) capability: String,
    pub(super) operation: String,
    pub(super) args: Value,
    pub(super) want_trace: bool,
    pub(super) timeout_cap: Option<std::time::Duration>,
    pub(super) gate_name: Option<String>,
}

impl CallContext {
    pub(super) fn from_params(params: &Value) -> Result<Self> {
        let want_trace = params
            .get("_routing_trace")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let raw_capability = params["capability"]
            .as_str()
            .context("Missing 'capability' field")?;

        // Support dotted capability names: "crypto.sha256" → domain="crypto", op="sha256"
        let (capability, operation) = if let Some(explicit_op) = params["operation"].as_str() {
            (raw_capability.to_string(), explicit_op.to_string())
        } else if let Some(dot_pos) = raw_capability.find('.') {
            (
                raw_capability[..dot_pos].to_string(),
                raw_capability[dot_pos + 1..].to_string(),
            )
        } else {
            anyhow::bail!(
                "Missing 'operation' field and capability '{raw_capability}' has no dotted operation"
            );
        };

        // JH-2: extract resource envelope from enriched params.
        // The routing layer injects `_resource_envelope` when the caller's
        // ionic token carries resource constraints.
        let envelope = params.get("_resource_envelope");

        let timeout_cap = envelope
            .and_then(|e| e.get("timeout_ms"))
            .and_then(|v| v.as_u64())
            .map(std::time::Duration::from_millis);

        // JH-2: enforce cpu/mem caps at orchestrator level. If the caller's
        // args request specific resources (cpu_cores, mem_bytes) that exceed
        // the envelope limits, reject before forwarding to the downstream
        // primal. This is a pre-dispatch guard; downstream primals (ToadStool)
        // also enforce at their compute dispatch level.
        if let Some(env) = envelope {
            let args_preview = params.get("args").or_else(|| params.get("params"));
            if let Some(args_val) = args_preview {
                if let Some(requested_mem) = args_val.get("mem_bytes").and_then(|v| v.as_u64()) {
                    if let Some(limit) = env.get("mem").and_then(|v| v.as_u64()) {
                        if requested_mem > limit {
                            anyhow::bail!(
                                "Resource envelope violation: requested mem_bytes ({requested_mem}) \
                                 exceeds token limit ({limit})"
                            );
                        }
                    }
                }
                if let Some(requested_cpu) = args_val.get("cpu_cores").and_then(|v| v.as_f64()) {
                    if let Some(limit) = env.get("cpu").and_then(|v| v.as_f64()) {
                        if requested_cpu > limit {
                            anyhow::bail!(
                                "Resource envelope violation: requested cpu_cores ({requested_cpu}) \
                                 exceeds token limit ({limit})"
                            );
                        }
                    }
                }
            }
        }

        // Accept both "args" and "params" (backward compat for older callers)
        let mut args = params
            .get("args")
            .or_else(|| params.get("params"))
            .cloned()
            .unwrap_or(json!({}));

        // JH-2: forward the resource envelope inside args so downstream
        // primals (e.g. ToadStool) can enforce cpu/mem/timeout_ms at their
        // compute dispatch level.
        if let Some(envelope) = params.get("_resource_envelope") {
            if let Some(obj) = args.as_object_mut() {
                obj.insert("_resource_envelope".to_string(), envelope.clone());
            }
        }

        // exp111: forward bearer token inside args so downstream primals in
        // enforced mode can perform their own MethodGate authorization check.
        // Without this, any primal running with BIOMEOS_AUTH_MODE=enforced
        // rejects forwarded calls with -32001 PERMISSION_DENIED.
        if let Some(token) = params.get("_bearer_token") {
            if let Some(obj) = args.as_object_mut() {
                obj.insert("_bearer_token".to_string(), token.clone());
            }
        }

        let gate_name = params["gate"].as_str().map(str::to_string);

        Ok(Self {
            capability,
            operation,
            args,
            want_trace,
            timeout_cap,
            gate_name,
        })
    }
}
